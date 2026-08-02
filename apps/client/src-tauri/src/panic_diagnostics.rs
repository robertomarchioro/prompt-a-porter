//! Diagnostica dei panic — issue #586.
//!
//! `ort` 2.0.0-rc.13 chiama `.expect("Failed to load ONNX Runtime dylib")`
//! internamente in `load_dynamic::init` per qualunque fallimento di `dlopen`
//! della libreria nativa onnxruntime: quel fallimento diventa un **panic**
//! dentro la crate, non un `Result` che possiamo intercettare. Con
//! `panic = "abort"` (Cargo.toml, profilo release) il processo termina con
//! SIGABRT; l'hook di panic di default scrive SOLO su stderr, che un'app GUI
//! non ha — nessuna traccia arrivava mai nel file di log dell'app (#586).
//!
//! Questo modulo installa un hook di panic globale, il PRIMA POSSIBILE in
//! `run()` (prima di qualunque plugin/setup Tauri): l'hook **viene invocato
//! anche quando `panic = "abort"`** — l'abort segue l'esecuzione dell'hook,
//! non la sostituisce (vedi documentazione di `std::panic::set_hook`). Scrive
//! sia via `log::error!` (utile se il logger è già inizializzato e il
//! processo sopravvive abbastanza da svuotare il buffer) sia — SEMPRE, con
//! una scrittura sincrona e flush esplicito — su un file di crash dedicato
//! nella stessa directory dei log applicativi: in caso di abort immediato il
//! buffer del logger potrebbe non essere mai svuotato.
//!
//! ONESTÀ: questo hook rende il crash **diagnosticabile** (payload,
//! posizione file:riga e backtrace finiscono su disco), NON lo elimina. La
//! causa profonda del fallimento di `dlopen` (#586) resta ignota finché non
//! viene osservata dal vivo dall'utente.

use std::io::Write;
use std::panic::PanicHookInfo;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Directory dei log applicativi, popolata appena disponibile (dentro
/// `.setup()`, quando `app.path().app_log_dir()` è risolvibile). Prima di
/// quel momento — o se la risoluzione fallisce — si usa `std::env::temp_dir()`
/// come fallback, così un panic verificatosi PRIMA del setup (es. durante la
/// registrazione dei plugin) lascia comunque una traccia su disco.
static CRASH_LOG_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Nome del file di crash dedicato, accanto a `pap.log` nella stessa
/// directory (append: ogni crash aggiunge una sezione, non sovrascrive le
/// precedenti).
const NOME_CRASH_LOG: &str = "pap-crash.log";

/// Comunica al modulo la directory dei log applicativi non appena è nota.
/// No-op se già impostata (il primo chiamante vince — coerente con
/// `OnceLock`, e comunque nel processo reale viene chiamata una sola volta).
pub fn imposta_directory_crash_log(dir: PathBuf) {
    let _ = CRASH_LOG_DIR.set(dir);
}

fn percorso_crash_log() -> PathBuf {
    CRASH_LOG_DIR
        .get()
        .cloned()
        .unwrap_or_else(std::env::temp_dir)
        .join(NOME_CRASH_LOG)
}

/// Estrae dal payload del panic un testo leggibile, gestendo sia `&str`
/// (caso comune, es. `panic!("...")` / `.expect("...")`) sia `String`.
fn estrai_payload(info: &PanicHookInfo<'_>) -> String {
    info.payload()
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "<payload panic non testuale>".to_string())
}

/// Formatta una sezione di crash completa: timestamp unix, messaggio,
/// posizione (file:riga:colonna) e backtrace best-effort.
fn formatta_crash(info: &PanicHookInfo<'_>) -> String {
    let payload = estrai_payload(info);
    let location = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "<posizione sconosciuta>".to_string());

    // Cattura SEMPRE il backtrace, indipendentemente da `RUST_BACKTRACE`: in
    // un crash lato utente quella env var non è mai impostata.
    let backtrace = std::backtrace::Backtrace::force_capture();

    let ora_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    format!(
        "\n=== PANIC (unix {ora_unix}) ===\n\
         messaggio: {payload}\n\
         posizione: {location}\n\
         backtrace:\n{backtrace}\n\
         === fine panic ===\n"
    )
}

/// Installa l'hook globale di panic, concatenando quello precedente (mai
/// sostituito: sempre invocato DOPO il nostro logging, così eventuale
/// comportamento di default — es. stampa su stderr nei build di sviluppo —
/// resta intatto).
///
/// Va chiamata il prima possibile in `run()`, prima di registrare qualunque
/// plugin Tauri o eseguire `.setup()`.
pub fn installa() {
    let hook_precedente = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let riga = formatta_crash(info);

        // 1. Via `log::error!` — raggiunge `pap.log` (con redazione/target
        //    coerenti col resto dell'app) se il logger è già inizializzato e
        //    il processo sopravvive abbastanza da svuotare il buffer.
        log::error!("{riga}");

        // 2. Scrittura sincrona diretta sul file di crash dedicato, con
        //    flush esplicito: unica garanzia affidabile quando `panic =
        //    "abort"` segue immediatamente l'esecuzione dell'hook — il
        //    buffer del logger potrebbe non essere mai svuotato.
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(percorso_crash_log())
        {
            let _ = file.write_all(riga.as_bytes());
            let _ = file.flush();
        }

        hook_precedente(info);
    }));
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Mutex;

    /// `std::panic::set_hook`/`take_hook` sono globali per l'intero processo:
    /// i due test sotto che li manipolano DEVONO essere serializzati fra
    /// loro (altrimenti `take_hook` concorrenti possono perdersi a vicenda
    /// l'hook salvato). Nessun altro test nel crate tocca l'hook di panic
    /// (vedi grep in fase di sviluppo), quindi questo mutex locale basta.
    static TEST_HOOK_LOCK: Mutex<()> = Mutex::new(());

    /// Verifica che il percorso del crash log sia sempre costruito con il
    /// nome file atteso, sia che la directory sia già stata impostata sia
    /// col fallback su `temp_dir()` (la `OnceLock` è statica e condivisa fra
    /// i test dello stesso binario: non asseriamo un valore assoluto, solo
    /// la forma del percorso).
    #[test]
    fn percorso_crash_log_termina_con_nome_atteso() {
        let percorso = percorso_crash_log();
        assert_eq!(percorso.file_name().unwrap(), NOME_CRASH_LOG);
    }

    /// `formatta_crash` deve includere payload testuale, posizione e la
    /// sezione di backtrace, indipendentemente da `RUST_BACKTRACE`.
    #[test]
    fn formatta_crash_include_payload_posizione_e_backtrace() {
        let _guard = TEST_HOOK_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // Cattura temporaneamente l'hook per estrarre il `PanicHookInfo`
        // formattato, senza scrivere su disco/log globali. Ripristina
        // SEMPRE l'hook precedente al termine, per non inquinare altri
        // test del processo.
        let catturato = std::sync::Arc::new(Mutex::new(String::new()));
        let catturato_clone = catturato.clone();
        let precedente = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            *catturato_clone.lock().unwrap() = formatta_crash(info);
        }));
        let risultato = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            panic!("messaggio di test diagnostico");
        }));
        std::panic::set_hook(precedente);
        assert!(risultato.is_err());

        let testo = catturato.lock().unwrap().clone();
        assert!(testo.contains("messaggio di test diagnostico"), "{testo}");
        assert!(testo.contains("panic_diagnostics.rs"), "{testo}");
        assert!(testo.contains("backtrace:"), "{testo}");
    }

    /// L'hook installato da `installa()` scrive davvero sul file di crash
    /// con flush sincrono, e concatena l'hook precedente invece di
    /// sostituirlo (deve venire invocato, non scartato).
    #[test]
    fn installa_scrive_sul_file_di_crash_e_concatena_hook_precedente() {
        let _guard = TEST_HOOK_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let tmp = tempfile::tempdir().unwrap();
        imposta_directory_crash_log(tmp.path().to_path_buf());

        let hook_precedente_chiamato =
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = hook_precedente_chiamato.clone();

        // Simula l'hook "già presente" prima della nostra installazione, e
        // ricorda l'hook realmente originale del processo per ripristinarlo
        // a fine test.
        let hook_originale_processo = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |_info| {
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
        }));

        installa();

        let risultato = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            panic!("crash di prova per il test del file dedicato");
        }));

        std::panic::set_hook(hook_originale_processo);
        assert!(risultato.is_err());

        assert!(
            hook_precedente_chiamato.load(std::sync::atomic::Ordering::SeqCst),
            "l'hook precedente deve essere invocato (concatenamento)"
        );

        let contenuto = std::fs::read_to_string(tmp.path().join(NOME_CRASH_LOG))
            .expect("il file di crash deve esistere");
        assert!(contenuto.contains("crash di prova per il test del file dedicato"));
    }
}
