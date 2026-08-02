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
//! `write_all` + `flush()` sincroni — su un file di crash dedicato nella
//! stessa directory dei log applicativi: in caso di abort immediato il
//! buffer del logger potrebbe non essere mai svuotato. Da NON leggere come
//! una garanzia più forte di quella che offre: su un `File` grezzo `flush()`
//! è di fatto un no-op (nessun user-space buffer da svuotare, a differenza
//! di un `BufWriter`) — la scrittura sopravvive comunque a un `abort()` del
//! processo (il kernel ha già i byte), ma NON a un crash del kernel stesso o
//! a una perdita di alimentazione prima che il blocco arrivi su disco.
//!
//! ONESTÀ: questo hook rende il crash **diagnosticabile** (payload,
//! posizione file:riga e backtrace finiscono su disco), NON lo elimina. La
//! causa profonda del fallimento di `dlopen` (#586) resta ignota finché non
//! viene osservata dal vivo dall'utente.
//!
//! ## Redazione dei segreti (CWE-532, review post-merge PR #589)
//!
//! Il payload di un panic è testo arbitrario proveniente da QUALUNQUE punto
//! dell'albero delle dipendenze — non solo `ort`/`dlopen`: un
//! `.unwrap()`/`.expect()` su codice HTTP, header o risposte dei provider AI
//! può incorporare nel messaggio dati che in condizioni normali finirebbero
//! nel record `ureq` redatto da `log_redazione` (vedi doc di quel modulo).
//! La redazione esistente è ancorata al **target** del record di log
//! (`ureq`/`ureq::…`): qui il target è sempre `pap_lib::panic_diagnostics`,
//! quindi quel meccanismo non si applicherebbe MAI a un panic, per
//! costruzione. Questo modulo tratta perciò l'intero blocco di panic come
//! **sospetto per contenuto**, non per target: applica
//! `log_redazione::redigi_valori_header` (content-based, cerca header
//! sensibili a inizio riga indipendentemente dal target) al testo formattato
//! PRIMA di scriverlo sia via `log::error!` sia sul file dedicato — così
//! `pap-crash.log`, che l'app invita ad allegare a una segnalazione
//! pubblica, non può contenere una chiave API in chiaro.

// `PanicInfo` (non il più recente `PanicHookInfo`, stesso tipo rinominato in
// Rust 1.81): `Cargo.toml` dichiara `rust-version = "1.77"` (promesso anche
// in `docs/contribuire/setup-sviluppo.md`) — `PanicHookInfo` non esiste
// prima di 1.81 e romperebbe la build su quel toolchain minimo dichiarato
// (errore di compilazione reale, non un lint). `PanicInfo` resta l'alias
// deprecato ma pienamente funzionante dello stesso tipo su tutte le
// versioni, quindi non richiede di alzare l'MSRV del progetto solo per
// questo modulo diagnostico (review post-merge PR #589). `allow(deprecated)`
// mirato solo qui: su un toolchain ≥1.81 il compilatore segnala l'alias
// come deprecato, ma è esattamente la scelta intenzionale di questo modulo.
use std::io::Write;
#[allow(deprecated)]
use std::panic::PanicInfo;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::log_redazione;

/// Directory dei log applicativi, popolata appena disponibile (dentro
/// `.setup()`, quando `app.path().app_log_dir()` è risolvibile). Prima di
/// quel momento — o se la risoluzione fallisce — si usa `std::env::temp_dir()`
/// come fallback, così un panic verificatosi PRIMA del setup (es. durante la
/// registrazione dei plugin) lascia comunque una traccia su disco.
static CRASH_LOG_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Nome del file di crash dedicato, accanto a `pap.log` nella stessa
/// directory (append: ogni crash aggiunge una sezione, non sovrascrive le
/// precedenti). `pub(crate)` così `debug_log` può riconoscerlo per nome ed
/// evitargli la redazione a freddo per-prefisso (inapplicabile: le sezioni
/// `=== PANIC ... ===` non hanno il prefisso `[data][ora][target][LIVELLO]`
/// da cui quella redazione ricava il target).
pub(crate) const NOME_CRASH_LOG: &str = "pap-crash.log";

/// Comunica al modulo la directory dei log applicativi non appena è nota.
/// No-op se già impostata (il primo chiamante vince — coerente con
/// `OnceLock`, e comunque nel processo reale viene chiamata una sola volta).
/// `OnceLock::set` è `#[must_use]`: la seconda chiamata ignorata non è un
/// errore atteso nel processo reale, ma una riga di debug rende visibile
/// l'anomalia se mai succedesse (es. `.setup()` invocato più volte).
pub fn imposta_directory_crash_log(dir: PathBuf) {
    if CRASH_LOG_DIR.set(dir).is_err() {
        log::debug!("imposta_directory_crash_log: ignorata, directory già impostata in precedenza");
    }
}

fn percorso_crash_log() -> PathBuf {
    CRASH_LOG_DIR
        .get()
        .cloned()
        .unwrap_or_else(std::env::temp_dir)
        .join(NOME_CRASH_LOG)
}

/// Applica permessi 0600 (solo owner) su Unix al file già aperto. No-op su
/// altre piattaforme. Best-effort: un fallimento resta silenzioso (l'hook di
/// panic non ha un canale affidabile per segnalarlo — è già nel percorso più
/// critico del processo).
fn imposta_permessi_owner(file: &std::fs::File) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = file.set_permissions(std::fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    {
        let _ = file;
    }
}

/// Apre il file di crash in scrittura, resistendo a un symlink pre-piazzato
/// sul percorso (review post-merge PR #589, H2 — stesso rischio già chiuso
/// altrove in questo crate dalla #569, `embeddings::apri_tmp_symlink_safe`):
/// `percorso_crash_log()` può cadere su `std::env::temp_dir()` (su Linux
/// `/tmp`, condivisa e scrivibile da tutti gli utenti locali) finché
/// `imposta_directory_crash_log` non è stata chiamata da `.setup()` — un
/// nome file prevedibile in quella directory è esattamente lo scenario da
/// cui `create_new`/O_EXCL protegge.
///
/// A differenza dei temporanei one-shot della #569, questo file va aperto in
/// **append** attraverso più crash della stessa sessione (o di sessioni
/// precedenti): un `create_new` puro fallirebbe sempre dal secondo crash in
/// poi. Strategia in due passi:
/// 1. `create_new` con permessi 0600 fin dalla creazione (niente finestra
///    TOCTOU tra creazione e `set_permissions`): se il path è libero, il
///    file nasce già sicuro. Questo copre lo scenario reale della issue —
///    un symlink piazzato PRIMA che l'app scriva mai su quel percorso: la
///    `create_new` fallisce con `AlreadyExists` e NON lo segue.
/// 2. Solo se il path esiste già, `symlink_metadata` (lstat, non segue)
///    verifica che sia un file regolare prima di riaprirlo in append.
///    Residuo noto: fra questo controllo e la `open` sottostante resta una
///    finestra TOCTOU stretta, non chiudibile senza `O_NOFOLLOW` a basso
///    livello (nessuna dipendenza `libc`/`rustix` in questo crate); mitigata
///    in pratica dallo sticky bit di `/tmp` (un utente diverso dal
///    proprietario non può rimpiazzare un file creato da noi con
///    `create_new`) e dal fatto che lo scenario descritto dalla issue — il
///    symlink preesistente al primo avvio — è già bloccato dal passo 1.
fn apri_crash_log_safe(path: &std::path::Path) -> Option<std::fs::File> {
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    match opts.open(path) {
        Ok(file) => return Some(file),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => return None,
    }

    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_file() => {}
        _ => return None,
    }

    let file = std::fs::OpenOptions::new().append(true).open(path).ok()?;
    imposta_permessi_owner(&file);
    Some(file)
}

/// Estrae dal payload del panic un testo leggibile, gestendo sia `&str`
/// (caso comune, es. `panic!("...")` / `.expect("...")`) sia `String`.
#[allow(deprecated)]
fn estrai_payload(info: &PanicInfo<'_>) -> String {
    info.payload()
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "<payload panic non testuale>".to_string())
}

/// Formatta una sezione di crash completa: timestamp unix, messaggio,
/// posizione (file:riga:colonna) e backtrace best-effort.
#[allow(deprecated)]
fn formatta_crash(info: &PanicInfo<'_>) -> String {
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

    // `payload` va su una riga TUTTA SUA (non in coda a "messaggio: " sulla
    // stessa riga fisica): la redazione content-based di `redigi_valori_header`
    // (H1, review PR #589) è ancorata a inizio riga (`(?m)^`) — se un header
    // sensibile fosse l'INTERO payload del panic (es.
    // `.expect(&format!("{header}: {valore}"))`) e restasse in coda a
    // "messaggio: " sulla stessa riga fisica, non verrebbe mai riconosciuto
    // come "inizio riga". Isolarlo su una riga propria copre anche questo
    // caso limite, oltre a quello già coperto (payload multi-riga con \r\n
    // letterali, es. un dump di header HTTP incorporato nel messaggio).
    format!(
        "\n=== PANIC (unix {ora_unix}) ===\n\
         messaggio:\n{payload}\n\
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
        // Redazione CONTENT-based (non target-based, vedi doc di modulo):
        // il target di questo record è sempre `pap_lib::panic_diagnostics`,
        // mai `ureq::*`, quindi la redazione ancorata al target di
        // `log_redazione` non scatterebbe mai su un payload di panic. Va
        // applicata qui, PRIMA sia di `log::error!` sia della scrittura su
        // file, così nessuna delle due vie d'uscita può contenere una
        // chiave API in chiaro.
        let riga = log_redazione::redigi_valori_header(&riga);

        // 1. Via `log::error!` — raggiunge `pap.log` se il logger è già
        //    inizializzato e il processo sopravvive abbastanza da svuotare
        //    il buffer.
        log::error!("{riga}");

        // 2. Scrittura sincrona diretta sul file di crash dedicato, con
        //    flush esplicito: unica garanzia affidabile quando `panic =
        //    "abort"` segue immediatamente l'esecuzione dell'hook — il
        //    buffer del logger potrebbe non essere mai svuotato.
        //    `apri_crash_log_safe` (fix H2, review PR #589) resiste a un
        //    symlink pre-piazzato sul percorso e nasce/resta a permessi 0600.
        if let Some(mut file) = apri_crash_log_safe(&percorso_crash_log()) {
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
    /// i test sotto che li manipolano DEVONO essere serializzati fra loro
    /// (altrimenti `take_hook` concorrenti possono perdersi a vicenda l'hook
    /// salvato). Nessun altro test nel crate tocca l'hook di panic
    /// ESPLICITAMENTE (vedi grep in fase di sviluppo), quindi questo mutex
    /// locale basta per QUESTI test. Fragilità nota, non garanzia
    /// dell'isolamento generale: l'harness di test standard manipola anch'esso
    /// l'hook globale per ogni `#[should_panic]` (installa il proprio,
    /// silenzia stdout, poi ripristina) senza comparire in un grep testuale
    /// per `set_hook`/`take_hook` — se in futuro un `#[should_panic]` girasse
    /// in parallelo a uno di questi test, l'esecuzione concorrente
    /// resterebbe una fonte di interferenza non coperta da questo mutex
    /// (oggi non ci sono `#[should_panic]` nel crate, quindi non è un
    /// problema attuale).
    static TEST_HOOK_LOCK: Mutex<()> = Mutex::new(());

    /// Directory dei test per il file di crash: `CRASH_LOG_DIR` è una
    /// `OnceLock` GLOBALE al processo (il primo chiamante vince, vedi doc su
    /// `imposta_directory_crash_log`) — condivisa fra TUTTI i test di questo
    /// binario, non solo quelli di questo modulo. Un secondo test che
    /// chiamasse `imposta_directory_crash_log` con un proprio `tempdir()`
    /// otterrebbe silenziosamente un no-op e finirebbe per cercare il file
    /// nella directory del PRIMO test invece che nella propria. Questo
    /// helper impone una directory di test condivisa e stabile per l'intero
    /// processo (creata una sola volta, mai droppata — altrimenti il
    /// percorso resterebbe valido ma la directory sparirebbe da sotto): i
    /// test che scrivono sul file di crash devono chiamarlo invece di
    /// costruire un proprio `tempdir()` e passarlo a
    /// `imposta_directory_crash_log`.
    fn crash_log_dir_di_test() -> &'static std::path::Path {
        static DIR: OnceLock<tempfile::TempDir> = OnceLock::new();
        let dir = DIR.get_or_init(|| tempfile::tempdir().unwrap());
        imposta_directory_crash_log(dir.path().to_path_buf());
        dir.path()
    }

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

        // Cattura temporaneamente l'hook per estrarre il `PanicInfo`
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

        let dir = crash_log_dir_di_test();

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

        let contenuto = std::fs::read_to_string(dir.join(NOME_CRASH_LOG))
            .expect("il file di crash deve esistere");
        assert!(contenuto.contains("crash di prova per il test del file dedicato"));
    }

    // ─── H1 (review PR #589): redazione content-based del payload di panic ───

    /// Un payload di panic che incorpora un dump multi-riga con un header
    /// sensibile (stessa forma del preludio HTTP di `ureq`, con `\r\n`
    /// letterali) non deve MAI raggiungere il file di crash in chiaro — né
    /// via `formatta_crash` (usato anche dal path `log::error!`).
    #[test]
    fn formatta_crash_seguito_da_redazione_non_espone_chiave_in_header_multi_riga() {
        let _guard = TEST_HOOK_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let catturato = std::sync::Arc::new(Mutex::new(String::new()));
        let catturato_clone = catturato.clone();
        let precedente = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            *catturato_clone.lock().unwrap() = formatta_crash(info);
        }));
        let risultato = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            panic!("bad response: POST /v1/messages HTTP/1.1\r\nx-api-key: sk-ant-segreto-panic\r\n\r\n");
        }));
        std::panic::set_hook(precedente);
        assert!(risultato.is_err());

        let grezzo = catturato.lock().unwrap().clone();
        // Prima della redazione la chiave È nel testo grezzo (altrimenti il
        // test non proverebbe nulla).
        assert!(grezzo.contains("sk-ant-segreto-panic"), "{grezzo}");

        let redatto = log_redazione::redigi_valori_header(&grezzo);
        assert!(!redatto.contains("sk-ant-segreto-panic"), "{redatto}");
        assert!(redatto.contains("x-api-key: ***"), "{redatto}");
    }

    /// Caso limite: l'INTERO payload del panic è un singolo header
    /// sensibile (nessuna riga di continuazione). Isolare il payload su una
    /// riga propria in `formatta_crash` (invece di "messaggio: {payload}"
    /// sulla stessa riga fisica) è ciò che permette alla redazione ancorata
    /// a inizio riga di riconoscerlo.
    #[test]
    fn formatta_crash_payload_singola_riga_di_solo_header_viene_redatto() {
        let _guard = TEST_HOOK_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let catturato = std::sync::Arc::new(Mutex::new(String::new()));
        let catturato_clone = catturato.clone();
        let precedente = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            *catturato_clone.lock().unwrap() = formatta_crash(info);
        }));
        let risultato = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            panic!("x-api-key: sk-ant-segreto-riga-singola");
        }));
        std::panic::set_hook(precedente);
        assert!(risultato.is_err());

        let grezzo = catturato.lock().unwrap().clone();
        let redatto = log_redazione::redigi_valori_header(&grezzo);
        assert!(!redatto.contains("sk-ant-segreto-riga-singola"), "{redatto}");
        assert!(redatto.contains("x-api-key: ***"), "{redatto}");
    }

    /// End-to-end: l'hook installato da `installa()` NON deve mai scrivere
    /// la chiave in chiaro sul file di crash dedicato — copre la via 2
    /// (scrittura diretta) descritta in H1.
    #[test]
    fn installa_non_scrive_chiavi_in_chiaro_sul_file_di_crash() {
        let _guard = TEST_HOOK_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let dir = crash_log_dir_di_test();

        let hook_originale_processo = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_info| {}));

        installa();

        let risultato = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            panic!("upstream dump:\r\nx-goog-api-key: AIzaSegretoPanicE2E\r\ncontent-type: application/json\r\n");
        }));

        std::panic::set_hook(hook_originale_processo);
        assert!(risultato.is_err());

        let contenuto = std::fs::read_to_string(dir.join(NOME_CRASH_LOG))
            .expect("il file di crash deve esistere");
        assert!(
            !contenuto.contains("AIzaSegretoPanicE2E"),
            "la chiave non deve comparire in chiaro nel file di crash: {contenuto}"
        );
        assert!(contenuto.contains("x-goog-api-key: ***"), "{contenuto}");
    }

    // ─── H2 (review PR #589): O_EXCL/symlink e permessi sul file di crash ───

    /// Un symlink pre-piazzato sul percorso del file di crash (scenario
    /// reale: nome prevedibile in `/tmp`, condivisa fra utenti locali)
    /// deve far fallire l'apertura invece di essere seguito — stesso
    /// principio della #569 (`embeddings::apri_tmp_symlink_safe`).
    #[cfg(unix)]
    #[test]
    fn apri_crash_log_safe_fallisce_su_symlink_pre_piazzato_senza_seguirlo() {
        let dir = tempfile::tempdir().unwrap();
        let bersaglio = dir.path().join("bersaglio-esterno");
        std::fs::write(&bersaglio, b"contenuto-originale").unwrap();
        let percorso = dir.path().join(NOME_CRASH_LOG);
        std::os::unix::fs::symlink(&bersaglio, &percorso).unwrap();

        let r = apri_crash_log_safe(&percorso);

        assert!(r.is_none(), "un symlink pre-piazzato deve far fallire l'apertura");
        assert_eq!(
            std::fs::read(&bersaglio).unwrap(),
            b"contenuto-originale",
            "il bersaglio del symlink non deve essere stato toccato"
        );
    }

    /// Sul path libero, il file nasce con permessi 0600 (solo owner) fin
    /// dalla creazione — niente finestra TOCTOU fra creazione e
    /// `set_permissions`.
    #[cfg(unix)]
    #[test]
    fn apri_crash_log_safe_crea_il_file_con_permessi_0600() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let percorso = dir.path().join(NOME_CRASH_LOG);

        let file = apri_crash_log_safe(&percorso).expect("su path libero l'apertura deve riuscire");
        let mode = file.metadata().unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    /// Un secondo crash nella stessa sessione deve poter riaprire in
    /// append il file regolare già creato dal primo — non solo il primo
    /// crash deve riuscire a scrivere.
    #[test]
    fn apri_crash_log_safe_riapre_in_append_un_file_regolare_preesistente() {
        let dir = tempfile::tempdir().unwrap();
        let percorso = dir.path().join(NOME_CRASH_LOG);

        {
            let mut f = apri_crash_log_safe(&percorso).expect("prima apertura deve riuscire");
            f.write_all(b"primo crash\n").unwrap();
        }
        {
            let mut f = apri_crash_log_safe(&percorso).expect("seconda apertura deve riuscire (append)");
            f.write_all(b"secondo crash\n").unwrap();
        }

        let contenuto = std::fs::read_to_string(&percorso).unwrap();
        assert!(contenuto.contains("primo crash"));
        assert!(contenuto.contains("secondo crash"));
    }
}
