//! v0.8.7 Sezione Sviluppo → Debug log: cmd Tauri per gestire il file
//! di log scritto da `tauri-plugin-log`.
//!
//! Path su `app_log_dir()`:
//! - Linux: `~/.local/share/com.pap.client/logs/pap.log`
//! - Windows: `%APPDATA%\com.pap.client\logs\pap.log`
//! - macOS: `~/Library/Logs/com.pap.client/pap.log`
//!
//! Cmd esposti:
//! - `debug_log_info` — path + lista file rotati con size/mtime
//! - `debug_log_apri_cartella` — apre LogDir con file manager OS
//! - `debug_log_pulisci` — truncate del file corrente **ed elimina anche
//!   i file rotati** (`pap.log.1`, `pap.log.2`, …) — vedi Azione 9,
//!   review PR #588: con `RotationStrategy::KeepAll` (`lib.rs`) i rotati
//!   non vengono mai eliminati automaticamente, quindi chi ha usato
//!   l'app prima della redazione CWE-532 (v0.8.44) può averli ancora
//!   pieni di chiavi API in chiaro, e la versione precedente di questo
//!   comando li lasciava intatti dando comunque esito "successo"
//! - `debug_log_esporta_zip` — apre il dialog di salvataggio nativo e
//!   crea uno ZIP con file log + metadata nel percorso scelto
//!   dall'utente (issue #558 punto 2: prima scriveva sempre nella stessa
//!   cartella fissa)
//!
//! **Nota di sicurezza (review avversariale pre-merge PR #570, HIGH):**
//! il dialog di salvataggio viene aperto **qui**, lato Rust, con
//! `tauri_plugin_dialog::DialogExt::dialog().file().blocking_save_file()`
//! — il comando non accetta più un percorso di destinazione come
//! argomento dal frontend. Questa app non ha un manifest ACL
//! (`permissions/`), quindi l'IPC dispatch per i comandi applicativi
//! (non `plugin:*`) di una finestra locale salta `resolve_access`
//! (vedi `tauri` 2.11.5, `src/webview/mod.rs`): un `destinazione: String`
//! passato dal frontend sarebbe stato un path scelto da qualunque JS
//! nella webview, scritto con `truncate(true)` — sovrascrittura
//! arbitraria di file, non solo lettura. Il permesso `dialog:allow-save`
//! non c'entra: regola l'apertura del dialogo nativo, non l'argomento di
//! un comando Rust.
//!
//! **Fix sicurezza MEDIUM (CWE-532), difesa in profondità:** i log
//! scritti **prima** della fix in `log_redazione` possono contenere
//! chiavi API dei provider AI in chiaro (header `x-api-key`/
//! `x-goog-api-key` non mascherati da `ureq`, vedi doc del modulo
//! `log_redazione`). `debug_log_esporta_zip` e `debug_log_leggi`
//! applicano `log_redazione::redigi_testo_log_storico` al testo prima di
//! usarlo, così le due vie d'uscita dei log (archivio ZIP, viewer
//! in-app) non espongono più quelle chiavi anche per file scritti da
//! versioni precedenti dell'app. Residuo noto: chi usa «Apri cartella»
//! e allega `pap.log` (o un rotato) a mano bypassa questa redazione —
//! non c'è modo di intercettarlo lato Rust; l'unico modo per ripulire
//! quei file è `debug_log_pulisci` prima di allegarli — da Azione 9
//! (review PR #588) elimina anche i rotati, che prima restavano intatti
//! per sempre con `RotationStrategy::KeepAll` (CWE-532 pre-v0.8.44
//! altrimenti mai rimediabile per chi ha già quei file su disco).
//!
//! **Indagine issue #584 ("pulisci log non pulisce nulla", segnalata su
//! Windows, non riprodotta dal vivo — macchina di sviluppo Linux
//! headless):** la conferma UI (`conferma()`, pass-through diretto a
//! `window.confirm` fuori da macOS) e un lock esclusivo Windows sono
//! state escluse come causa — vedi analisi in PR. **Rettifica
//! post-review (PR #588):** la diagnosi iniziale ipotizzava una corsa
//! critica reale tra `debug_log_pulisci` e il buffer interno del writer
//! di `tauri-plugin-log`, ma leggendo il sorgente vendorizzato delle
//! versioni pinnate nel lock (`tauri-plugin-log` 2.9.0 +
//! `fern` 0.7.1, `writer_log_impl!`) quella corsa **non esiste**: ogni
//! chiamata a `log()` fa `write!` + `flush()` sotto lo stesso lock,
//! quindi il buffer non sopravvive mai da un evento di log al
//! successivo. Il `log::logger().flush()` aggiunto qui sotto resta un
//! flush difensivo ragionevole, ma il suo effetto sul sintomo #584 **non
//! è dimostrato** — vedi doc di `tronca_file_log` più sotto per
//! l'ipotesi più credibile (flush fallito che lascia il buffer pieno) e
//! per i limiti noti che restano comunque irrisolti. **Non chiude la
//! #584**: la causa del sintomo riportato su Windows resta ignota.

use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::errore::PapErrore;
use crate::log_redazione;

/// Nome del file di log configurato in `lib.rs::run` (tauri-plugin-log
/// TargetKind::LogDir { file_name: Some("pap".to_string()) }).
/// Plugin appende `.log` automaticamente, e rotati si chiamano
/// `pap.log.1`, `pap.log.2`, etc.
const NOME_LOG: &str = "pap";

#[derive(Debug, Serialize)]
pub struct FileLog {
    pub name: String,
    pub size_bytes: u64,
    /// Timestamp ISO-8601 modificato (best-effort, "" se non disponibile)
    pub modified_at: String,
}

#[derive(Debug, Serialize)]
pub struct InfoDebugLog {
    /// Path al file di log corrente (anche se non ancora creato)
    pub path_corrente: String,
    /// Directory contenente i log (LogDir di Tauri)
    pub directory: String,
    /// Lista di tutti i file `pap.log*` esistenti, ordinati alfabeticamente
    /// (il corrente per primo, poi rotati 1, 2, …)
    pub files: Vec<FileLog>,
}

fn log_dir(app: &tauri::AppHandle) -> Result<PathBuf, PapErrore> {
    app.path()
        .app_log_dir()
        .map_err(|e| PapErrore::dominio("Impossibile determinare la cartella dei log.", e))
}

/// Crea (o sovrascrive) il file ZIP di export già con permessi 0600 su
/// Unix, invece di creare il file con i permessi di default (spesso 0644,
/// leggibili da altri utenti locali) e restringerli solo a fine scrittura.
///
/// Fix MEDIUM (review PR #481): `File::create` + `restringi_permessi_owner`
/// alla fine lasciava una finestra TOCTOU — tra la creazione del file e lo
/// `set_permissions` finale, il file esisteva con i permessi di default
/// mentre veniva scritto (ZipWriter + contenuto dei log). Aprire il file
/// già in modalità 0600 elimina la finestra.
fn crea_file_export(path: &Path) -> std::io::Result<File> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
    }
    #[cfg(not(unix))]
    {
        File::create(path)
    }
}

/// Applica permessi 0600 (solo owner) su Unix. No-op su altre piattaforme.
/// Best-effort: un fallimento viene loggato ma non interrompe l'export.
/// Belt-and-suspenders dopo `crea_file_export`: su Unix è già ridondante
/// (il file nasce 0600), ma resta utile come rete di sicurezza se in
/// futuro il file venisse creato altrove senza passare da quella funzione.
fn restringi_permessi_owner(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o600)) {
            log::warn!("set_permissions 0600 su {} fallito: {e}", path.display());
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

fn iso_mtime(meta: &fs::Metadata) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let modified = meta.modified().ok();
    let Some(m) = modified else {
        return String::new();
    };
    let dur = m.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    let secs = dur.as_secs() as i64;
    format_iso_utc(secs)
}

/// Formato ISO 8601 UTC senza dipendenze esterne. Algoritmo civil-from-days
/// di Hinnant (2013). Accurato per date 1970-9999.
fn format_iso_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let time_of_day = secs.rem_euclid(86_400);
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };

    format!("{year:04}-{month:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn raccogli_file_log(dir: &Path) -> Vec<FileLog> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<FileLog> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy().to_string();
            // Match pap.log, pap.log.1, pap.log.2, …
            s.starts_with(NOME_LOG) && (s.ends_with(".log") || s.contains(".log."))
        })
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            let name = e.file_name().to_string_lossy().to_string();
            Some(FileLog {
                name,
                size_bytes: meta.len(),
                modified_at: iso_mtime(&meta),
            })
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

#[tauri::command]
pub fn debug_log_info(app: tauri::AppHandle) -> Result<InfoDebugLog, PapErrore> {
    let dir = log_dir(&app)?;
    let path_corrente = dir.join(format!("{NOME_LOG}.log"));
    let files = raccogli_file_log(&dir);
    Ok(InfoDebugLog {
        path_corrente: path_corrente.to_string_lossy().to_string(),
        directory: dir.to_string_lossy().to_string(),
        files,
    })
}

/// Apre la cartella `app_log_dir()` con il file manager del sistema operativo.
/// Linux: xdg-open, macOS: open, Windows: explorer.
#[tauri::command]
pub fn debug_log_apri_cartella(app: tauri::AppHandle) -> Result<(), PapErrore> {
    let dir = log_dir(&app)?;
    let _ = fs::create_dir_all(&dir);

    #[cfg(target_os = "linux")]
    let cmd_name = "xdg-open";
    #[cfg(target_os = "macos")]
    let cmd_name = "open";
    #[cfg(target_os = "windows")]
    let cmd_name = "explorer";

    Command::new(cmd_name)
        .arg(&dir)
        .spawn()
        .map_err(|e| PapErrore::dominio("Impossibile aprire la cartella dei log.", e))?;
    Ok(())
}

/// Nucleo puro di `debug_log_pulisci`: tronca `path` a zero byte e ritorna
/// `(size_prima, size_dopo)` in byte. Estratto a parte per essere
/// testabile senza `AppHandle` (stesso motivo di `ultime_righe_parsate`).
///
/// **Indagine #584 ("pulisci log non pulisce nulla"): cosa si sa e cosa
/// resta ignoto.** La conferma UI e i permessi/`share_mode` su Windows
/// sono stati esclusi come causa (vedi PR): `File::create` sui default
/// Rust concede già `FILE_SHARE_READ|WRITE|DELETE`.
///
/// **Rettifica post-review (PR #588): la corsa critica dichiarata nella
/// prima versione di questo commento NON esiste.** Leggendo il sorgente
/// vendorizzato pinnato nel lock — `tauri-plugin-log-2.9.0/src/lib.rs`
/// (`TargetKind::LogDir` usa `fern::Output::writer(Box::new(rotator),
/// "\n")`) e `fern-0.7.1/src/log_impl.rs` (macro `writer_log_impl!`) —
/// ogni singola chiamata a `log()` esegue `write!(writer, ...)?;
/// writer.flush()?;` **tenendo il `Mutex` bloccato per tutta la
/// write+flush**. Il buffer di `RotatingFile` non sopravvive quindi mai
/// da un evento di log al successivo: non c'è nulla "in volo" che possa
/// ricomparire dopo un truncate nel percorso normale. Il
/// `log::logger().flush()` sotto resta un flush difensivo ragionevole
/// (costa poco, non fa danno), ma **non va presentato come la causa
/// trovata e chiusa** del sintomo #584 — non lo è.
///
/// **Ipotesi principale, da verificare al prossimo collaudo Windows (non
/// accertata):** `RotatingFile::flush()` (`lib.rs:310-330` del plugin)
/// fa `file.write_all(&self.buffer)` poi `file.flush()` e **solo se
/// entrambi riescono** esegue `self.buffer.clear()`. Se una delle due
/// fallisce — es. *sharing violation* su Windows (antivirus, OneDrive,
/// editor con lock esclusivo sul file) — il metodo ritorna `Err`
/// **prima** dello `clear()`, e il buffer resta pieno finché un flush
/// successivo non riesce. È un candidato molto più credibile per #584
/// su Windows di quanto lo fosse la corsa critica smentita sopra.
///
/// **Buco correlato: il fallimento silenzioso di `flush()`.**
/// `log::Log::flush(&self) -> ()` non ritorna `Result`
/// (`fern-0.7.1/src/log_impl.rs:648-653`: `let _ =
/// self.stream.lock()....flush();`). Se il flush qui sotto fallisce
/// proprio per la causa più plausibile appena descritta, non abbiamo
/// alcun modo di saperlo da questa chiamata — l'errore è inghiottito
/// dentro il crate `log` — e il truncate procede comunque. Rilevante
/// perché uno degli obiettivi di questa PR è dare osservabilità a
/// "pulisci log": su questo punto specifico non possiamo averne.
///
/// **Race residua, non chiusa:** tra `log::logger().flush()` e
/// `File::create(path)` qui sotto non c'è alcun lock condiviso con il
/// percorso di scrittura del logger — è una TOCTOU strutturale. Un altro
/// thread può scrivere una riga proprio in quella finestra; quella riga
/// specifica andrebbe persa dal truncate. Non è la stessa cosa del
/// "contenuto pre-pulizia che ricompare" di #584, ma resta una finestra
/// reale che questa funzione non chiude.
///
/// **Limite noto, non risolto qui:** il contatore `current_size` interno
/// di `RotatingFile` resta comunque disallineato dopo il nostro truncate
/// (memorizza la dimensione pre-pulizia, non 0) fino alla prossima
/// rotazione naturale del plugin. Non è solo una reportistica incoerente:
/// con `max_file_size` a 5 MB (`lib.rs`), se `current_size` è rimasto
/// vicino a quella soglia, alla riga di log successiva
/// `current_size + buffer.len() > max_size` può risultare vera con il
/// file appena svuotato — una **rotazione prematura reale**, che produce
/// un file ruotato spurio quasi vuoto, visibile sia in `debug_log_info`
/// sia nell'export ZIP. Non esiste modo di riallineare quel contatore da
/// qui: `log::set_boxed_logger` (crate `log`) è impostabile una sola
/// volta per processo — non c'è "unset" — e `RotatingFile` non espone né
/// un metodo di reset né un modo per ottenerne un riferimento
/// dall'`AppHandle`. Riallinearlo davvero richiederebbe di
/// forkare/patchare `tauri-plugin-log` per esporre un hook di reset,
/// cosa che non facciamo in questa PR.
fn tronca_file_log(path: &Path) -> std::io::Result<(u64, Option<u64>)> {
    let size_prima = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    // Flush difensivo prima di troncare: vedi doc sopra — NON è dimostrato
    // che chiuda #584 (la corsa critica ipotizzata inizialmente non esiste
    // nel percorso normale), ma non costa nulla e potrebbe aiutare
    // nell'ipotesi "flush precedente fallito" descritta sopra.
    log::logger().flush();
    File::create(path)?;
    // `None` se lo stat post-truncate fallisce: NON va confuso con "0
    // byte" (Azione 6, review PR #588) — il truncate stesso (riga sopra)
    // è comunque riuscito a questo punto, solo la verifica no. Prima di
    // questa correzione un fallimento qui produceva `unwrap_or(0)`,
    // indistinguibile da un vero truncate a 0 byte: proprio la
    // distinzione che questa telemetria dovrebbe rendere possibile.
    let size_dopo = fs::metadata(path).ok().map(|m| m.len());
    Ok((size_prima, size_dopo))
}

/// Un file rotato eliminato con successo da `debug_log_pulisci`.
#[derive(Debug, Serialize)]
pub struct FileRotatoRimosso {
    pub name: String,
    pub size_bytes: u64,
}

/// Un file rotato che `debug_log_pulisci` NON è riuscito a eliminare
/// (permessi, lock esclusivo su Windows, …). Azione 9 (review PR #588):
/// il fallimento va riportato esplicitamente, non nascosto dietro un
/// esito `Ok` generico — niente falsi successi.
#[derive(Debug, Serialize)]
pub struct FileRotatoNonRimosso {
    pub name: String,
    pub errore: String,
}

/// Esito di `debug_log_pulisci`. Sempre `Ok` con questo payload finché il
/// truncate del file corrente riesce (vedi sotto); i fallimenti sui
/// singoli rotati finiscono in `rotati_falliti`, non fanno fallire
/// l'intero comando — la pulizia degli altri file prosegue comunque.
#[derive(Debug, Serialize)]
pub struct EsitoPulizia {
    /// Dimensione del file corrente prima del truncate.
    pub size_prima: u64,
    /// Dimensione dopo il truncate. `None` se non è stato possibile
    /// verificarla via stat (il truncate stesso è comunque riuscito a
    /// questo punto) — v. doc di `tronca_file_log`, Azione 6.
    pub size_dopo: Option<u64>,
    /// File rotati (`pap.log.1`, `pap.log.2`, …) eliminati con successo.
    pub rotati_rimossi: Vec<FileRotatoRimosso>,
    /// File rotati che NON è stato possibile eliminare. Il frontend deve
    /// controllare questo campo e non dichiarare successo pieno se non è
    /// vuoto (Azione 9, review PR #588).
    pub rotati_falliti: Vec<FileRotatoNonRimosso>,
}

/// Nucleo puro di `debug_log_pulisci`: dato `dir` (la cartella dei log),
/// tronca il file corrente ed elimina i rotati. Estratto a parte per
/// essere testabile senza `AppHandle` — stesso motivo di
/// `tronca_file_log`/`ultime_righe_parsate` — e necessario per Azione 9
/// (review PR #588): la feature `tauri::test` non è cablata nei
/// dev-dependencies del crate (v. commento su
/// `debug_log_esporta_zip_non_accetta_piu_un_percorso_dal_chiamante`),
/// quindi il "caso positivo" (rotati rimossi davvero) e la "rimozione
/// parziale" richiesti dalla review vanno testati qui, non sul comando.
///
/// Solo il fallimento del truncate del file corrente è un errore duro
/// (propagato come `Err`): i fallimenti sui singoli rotati sono soft,
/// finiscono in `EsitoPulizia::rotati_falliti` e non interrompono la
/// pulizia degli altri file.
fn pulisci_log_in_dir(dir: &Path) -> std::io::Result<EsitoPulizia> {
    let path_corrente = dir.join(format!("{NOME_LOG}.log"));

    let (size_prima, size_dopo) = if path_corrente.exists() {
        tronca_file_log(&path_corrente)?
    } else {
        (0, Some(0))
    };

    let mut rotati_rimossi = Vec::new();
    let mut rotati_falliti = Vec::new();
    for f in raccogli_file_log(dir) {
        if f.name == format!("{NOME_LOG}.log") {
            continue; // il corrente è gestito sopra, non va eliminato
        }
        let path_rotato = dir.join(&f.name);
        match fs::remove_file(&path_rotato) {
            Ok(()) => rotati_rimossi.push(FileRotatoRimosso {
                name: f.name,
                size_bytes: f.size_bytes,
            }),
            Err(e) => rotati_falliti.push(FileRotatoNonRimosso {
                name: f.name,
                errore: e.to_string(),
            }),
        }
    }

    Ok(EsitoPulizia {
        size_prima,
        size_dopo,
        rotati_rimossi,
        rotati_falliti,
    })
}

/// Truncate il file `pap.log` corrente **ed elimina anche i file
/// rotati** (`pap.log.1`, `pap.log.2`, …). Best-effort sul file
/// corrente: se non esiste è un no-op (non un errore) per quella parte.
///
/// **Azione 9 (review PR #588), fix di un HIGH preesistente (CWE-532):**
/// prima di questa PR i rotati non venivano mai toccati da "Pulisci
/// log", e con `RotationStrategy::KeepAll` (`lib.rs`) non vengono nemmeno
/// mai eliminati automaticamente dal plugin. Chi ha usato l'app prima
/// della redazione introdotta in v0.8.44 può quindi avere chiavi API
/// Anthropic/Gemini in chiaro nei rotati, e la versione precedente di
/// questo comando restituiva comunque "successo" senza averli toccati.
/// Qui i rotati vengono eliminati (non troncati: non hanno motivo di
/// restare — a differenza del corrente non sono più scritti dal
/// logger), riusando `raccogli_file_log` per individuarli. Un
/// fallimento su un singolo rotato (permessi, lock esclusivo Windows)
/// NON interrompe la pulizia degli altri: finisce in `rotati_falliti`,
/// che il chiamante deve controllare esplicitamente (niente falsi
/// successi — questo è esattamente il tipo di esito silenzioso che ha
/// reso #584 difficile da diagnosticare la prima volta).
///
/// **Sovrascrittura fisica dei blocchi prima dell'eliminazione: fuori
/// perimetro per decisione esplicita** (i file vengono eliminati con
/// `fs::remove_file`, i blocchi su disco possono restare recuperabili
/// per un po' con strumenti di file recovery — stesso limite di
/// qualunque cancellazione file "normale" su tutti e tre gli OS target).
///
/// Logga sempre dimensione prima/dopo del truncate (issue #584): serve a
/// distinguere, al prossimo collaudo, i tre scenari possibili — errore
/// mostrato in UI (`debugErrore`), "falso successo" (comando torna `Ok`
/// ma `size_dopo` non è 0), oppure svuotamento riuscito ma che si
/// riempie di nuovo subito dopo (size_dopo=0 in questo log, ma il file
/// torna a crescere appena dopo — visibile solo riaprendo
/// `debug_log_info` più tardi, non da qui).
///
/// **Azione 8 (review PR #588) — perché `async`:** `flush()` prende il
/// lock globale del logger, fa I/O sincrono e ora può innescare
/// l'eliminazione di più file — non operazioni "veloci" su disco di
/// rete/OneDrive/antivirus, cioè proprio la piattaforma sotto indagine
/// per #584. `debug_log_esporta_zip`, nello stesso file, è già marcato
/// `async` per lo stesso motivo (non bloccare il thread IPC — v. il suo
/// commento). Marcare anche questo comando `async` è la scelta coerente
/// con quel pattern, non un'eccezione da giustificare caso per caso.
#[tauri::command(async)]
pub fn debug_log_pulisci(app: tauri::AppHandle) -> Result<EsitoPulizia, PapErrore> {
    let dir = log_dir(&app)?;
    if !dir.join(format!("{NOME_LOG}.log")).exists() {
        log::info!("Debug log: pulizia richiesta ma il file corrente non esiste (no-op sul corrente).");
    }
    let esito = pulisci_log_in_dir(&dir)
        .map_err(|e| PapErrore::dominio("Pulizia dei log non riuscita.", e))?;
    let size_dopo_msg = esito
        .size_dopo
        .map(|s| format!("{s} byte"))
        .unwrap_or_else(|| "sconosciuto (stat fallito dopo il truncate)".to_string());
    log::info!(
        "Debug log pulito (truncate): {} byte -> {size_dopo_msg}. Rotati eliminati: {}, \
         falliti: {}. Nota: il contatore di dimensione interno del writer del plugin di \
         log resta disallineato fino alla prossima rotazione naturale (limite noto, vedi #584).",
        esito.size_prima,
        esito.rotati_rimossi.len(),
        esito.rotati_falliti.len(),
    );
    for f in &esito.rotati_falliti {
        log::warn!("Debug log: eliminazione rotato {} fallita: {}", f.name, f.errore);
    }
    Ok(esito)
}

/// Nome file suggerito nel dialog di salvataggio nativo, es.
/// `pap-debug-log-2026-08-01-17-49-52.zip`.
fn nome_file_zip_suggerito(timestamp_iso: &str) -> String {
    // "YYYY-MM-DDTHH:MM:SSZ" -> "YYYY-MM-DD-HH-MM-SS" (stesso schema che
    // aveva il frontend prima di questo fix, in nomeFileZipLogSuggerito()).
    let senza_zona = timestamp_iso.trim_end_matches('Z');
    let slug = senza_zona.replace([':', 'T'], "-");
    format!("pap-debug-log-{slug}.zip")
}

/// Apre il dialog di salvataggio nativo e crea uno ZIP con il file log
/// corrente + i rotati + un file `metadata.txt` (versione app, OS,
/// timestamp) nel percorso scelto dall'utente (issue #558 punto 2: prima
/// l'export finiva sempre nella stessa cartella fissa
/// `app_data_dir()/debug-exports`, senza possibilità di scegliere dove).
/// Il file nasce comunque con permessi 0600 su Unix (solo owner — vedi
/// `crea_file_export`/`restringi_permessi_owner`), a prescindere da dove
/// l'utente lo salvi.
///
/// Il dialog viene aperto **qui**, non nel frontend: vedi la nota di
/// sicurezza in testa al modulo (review avversariale pre-merge PR #570).
/// `blocking_save_file` pompa una nested event loop nativa e non deve
/// girare sul thread principale — questo comando è marcato `async` così
/// l'IPC lo esegue fuori da quel thread (stesso pattern usato da
/// `tauri-plugin-dialog` stesso nel comando `save` interno).
///
/// Ritorna `None` se l'utente ha annullato il dialog (non è un errore),
/// altrimenti il path assoluto al file ZIP creato.
#[tauri::command(async)]
pub fn debug_log_esporta_zip(app: tauri::AppHandle) -> Result<Option<String>, PapErrore> {
    let timestamp = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as i64;
        format_iso_utc(s)
    };

    let destinazione = app
        .dialog()
        .file()
        .set_file_name(nome_file_zip_suggerito(&timestamp))
        .add_filter("Archivio ZIP", &["zip"])
        .blocking_save_file();
    let Some(destinazione) = destinazione else {
        // Utente ha annullato il dialog: non è un errore.
        return Ok(None);
    };
    let zip_path = destinazione
        .into_path()
        .map_err(|e| PapErrore::dominio("Percorso di destinazione non valido.", e))?;

    let dir = log_dir(&app)?;
    let files = raccogli_file_log(&dir);

    if let Some(parent) = zip_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = crea_file_export(&zip_path)
        .map_err(|e| PapErrore::dominio("Creazione dell'archivio dei log non riuscita.", e))?;
    let mut zip = ZipWriter::new(BufWriter::new(file));
    let opts: SimpleFileOptions =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let metadata = format!(
        "Prompt a Porter — Debug log export\n\
        Generated at: {ts}\n\
        App version: {ver}\n\
        OS: {os}\n\
        Arch: {arch}\n\
        Log dir: {dir}\n\
        Files: {count}\n",
        ts = timestamp,
        ver = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        dir = dir.display(),
        count = files.len(),
    );
    zip.start_file("metadata.txt", opts)
        .map_err(|e| PapErrore::dominio("Creazione dell'archivio dei log non riuscita.", e))?;
    zip.write_all(metadata.as_bytes())
        .map_err(|e| PapErrore::dominio("Creazione dell'archivio dei log non riuscita.", e))?;

    for f in &files {
        let src = dir.join(&f.name);
        let Ok(mut fh) = File::open(&src) else {
            continue;
        };
        let mut buf = Vec::with_capacity(f.size_bytes as usize);
        if fh.read_to_end(&mut buf).is_err() {
            continue;
        }
        // Difesa in profondità CWE-532 (vedi nota di sicurezza in testa al
        // modulo): redige le chiavi API eventualmente scritte in chiaro
        // prima della fix del formatter. `from_utf8_lossy` è safe su bytes
        // non-UTF-8 (sostituisce con U+FFFD, non panica) — il file di log
        // è comunque testo prodotto da `log`/`write!`, quindi UTF-8 valido
        // nella quasi totalità dei casi reali.
        let testo = String::from_utf8_lossy(&buf);
        let testo_redatto = log_redazione::redigi_testo_log_storico(&testo);
        zip.start_file(&f.name, opts)
            .map_err(|e| PapErrore::dominio("Creazione dell'archivio dei log non riuscita.", e))?;
        zip.write_all(testo_redatto.as_bytes())
            .map_err(|e| PapErrore::dominio("Creazione dell'archivio dei log non riuscita.", e))?;
    }

    zip.finish()
        .map_err(|e| PapErrore::dominio("Creazione dell'archivio dei log non riuscita.", e))?;
    restringi_permessi_owner(&zip_path);
    log::info!("Debug log esportato: {}", zip_path.display());
    Ok(Some(zip_path.to_string_lossy().to_string()))
}

// ─── v0.8.7 PR-C: lettura log per viewer in-app ───

const MAX_RIGHE: usize = 5000;
const DEFAULT_RIGHE: usize = 200;

#[derive(Debug, Serialize)]
pub struct RigaLog {
    /// Timestamp ISO senza parentesi, "" se parsing fallito
    pub timestamp: String,
    /// "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR" | "" (unknown)
    pub level: String,
    /// Module path / target (es. "pap_lib::editor"), "" se parsing fallito
    pub target: String,
    /// Messaggio user-readable
    pub message: String,
    /// Linea raw originale (utile per debug del parser stesso)
    pub raw: String,
}

/// Parsing best-effort del formato REALMENTE prodotto dal formatter di
/// default di `tauri-plugin-log` (verificato riproducendo `Builder::default()`
/// del crate — `time::format_description!` + `format_args!("{}[{}][{}] {}",
/// timestamp, record.target(), record.level(), message)`):
///
/// `[YYYY-MM-DD][HH:MM:SS][target][LEVEL] message`
///
/// **Nota storica (bug #558 punto 1):** la doc precedente di questa
/// funzione assumeva l'ordine `[LEVEL][target]`, invertito rispetto a
/// quello vero. Con l'ordine sbagliato `r.level` finiva per contenere il
/// module path (es. "pap_lib::editor") invece di "INFO"/"WARN"/ecc.: il
/// filtro per livello nel viewer non trovava mai corrispondenze (ogni
/// riga finiva silenziosamente esclusa non appena si selezionava un
/// livello), e ogni riga risultava etichettata/colorata in modo errato.
///
/// Se il formato non corrisponde (es. linea di continuazione panic
/// trace), ritorna RigaLog con campi vuoti tranne `raw` e `message=raw`.
fn parse_riga(line: &str) -> RigaLog {
    let raw = line.to_string();

    // Helper: estrai contenuto tra `[` e `]` consumando, ritorna (contenuto, resto)
    fn estrai_bracket(s: &str) -> Option<(&str, &str)> {
        let s = s.strip_prefix('[')?;
        let end = s.find(']')?;
        let (dentro, dopo) = s.split_at(end);
        let dopo = dopo.strip_prefix(']').unwrap_or(dopo);
        Some((dentro, dopo))
    }

    let mut resto = line;
    let Some((data, r1)) = estrai_bracket(resto) else {
        return RigaLog {
            timestamp: String::new(),
            level: String::new(),
            target: String::new(),
            message: raw.clone(),
            raw,
        };
    };
    resto = r1;
    let Some((ora, r2)) = estrai_bracket(resto) else {
        return RigaLog {
            timestamp: String::new(),
            level: String::new(),
            target: String::new(),
            message: raw.clone(),
            raw,
        };
    };
    resto = r2;
    let Some((target, r3)) = estrai_bracket(resto) else {
        return RigaLog {
            timestamp: format!("{data} {ora}"),
            level: String::new(),
            target: String::new(),
            message: resto.trim().to_string(),
            raw,
        };
    };
    resto = r3;
    let Some((level, r4)) = estrai_bracket(resto) else {
        return RigaLog {
            timestamp: format!("{data} {ora}"),
            level: String::new(),
            target: target.to_string(),
            message: resto.trim().to_string(),
            raw,
        };
    };

    let message = r4.trim().to_string();
    RigaLog {
        timestamp: format!("{data} {ora}"),
        level: level.to_string(),
        target: target.to_string(),
        message,
        raw,
    }
}

/// Nucleo puro di `debug_log_leggi`: dato il contenuto testuale del file
/// e `n`, ritorna le ultime `n` righe parsate. Estratta a parte per poter
/// scrivere test su un "file fittizio" (stringa in memoria o scritta su
/// un file temporaneo) senza dover simulare un `AppHandle` Tauri.
///
/// Applica prima `log_redazione::redigi_testo_log_storico` (difesa in
/// profondità CWE-532, vedi nota di sicurezza in testa al modulo): il
/// viewer in-app non deve mostrare chiavi API scritte in chiaro da
/// versioni precedenti dell'app. Va fatto sul testo INTERO, non riga per
/// riga dopo lo split — la redazione è stateful (eredita il target dalla
/// riga con prefisso precedente per le righe di continuazione del
/// preludio HTTP di `ureq`).
fn ultime_righe_parsate(contenuto: &str, n: usize) -> Vec<RigaLog> {
    let contenuto_redatto = log_redazione::redigi_testo_log_storico(contenuto);
    let righe: Vec<&str> = contenuto_redatto.lines().collect();
    let inizio = righe.len().saturating_sub(n);
    righe[inizio..].iter().map(|l| parse_riga(l)).collect()
}

/// Legge le ultime `n` righe dal file `pap.log` corrente. `n` clampato
/// a [1, 5000]. Per efficienza legge l'intero file (max 5MB per rotation
/// strategy) e prende slice finale. Per file > 5MB usare i rotati.
#[tauri::command]
pub fn debug_log_leggi(
    app: tauri::AppHandle,
    n_righe: Option<usize>,
) -> Result<Vec<RigaLog>, PapErrore> {
    let n = n_righe.unwrap_or(DEFAULT_RIGHE).clamp(1, MAX_RIGHE);
    let dir = log_dir(&app)?;
    let path = dir.join(format!("{NOME_LOG}.log"));
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contenuto = fs::read_to_string(&path)
        .map_err(|e| PapErrore::dominio("Lettura dei log non riuscita.", e))?;
    Ok(ultime_righe_parsate(&contenuto, n))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn format_iso_unix_epoch() {
        assert_eq!(format_iso_utc(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn format_iso_anno_2026() {
        // 2026-05-11T13:00:00Z = 1778540400 secondi epoch
        let s = format_iso_utc(1_778_540_400);
        assert!(s.starts_with("2026-05-11T"), "got: {s}");
    }

    #[test]
    fn raccogli_match_solo_pap_log() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pap.log"), b"line1\n").unwrap();
        fs::write(dir.path().join("pap.log.1"), b"old\n").unwrap();
        fs::write(dir.path().join("altro.txt"), b"skip").unwrap();
        fs::write(dir.path().join("README.md"), b"skip").unwrap();

        let files = raccogli_file_log(dir.path());
        let nomi: Vec<String> = files.iter().map(|f| f.name.clone()).collect();
        assert_eq!(nomi, vec!["pap.log".to_string(), "pap.log.1".to_string()]);
    }

    #[test]
    fn raccogli_size_corretto() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pap.log"), b"hello").unwrap();
        let files = raccogli_file_log(dir.path());
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].size_bytes, 5);
    }

    #[test]
    fn raccogli_vuoto_per_dir_inesistente() {
        let files = raccogli_file_log(Path::new("/nonexistent/dir/xyz"));
        assert!(files.is_empty());
    }

    #[test]
    fn parse_riga_formato_standard() {
        // Ordine REALE del formatter di default di tauri-plugin-log:
        // [data][ora][target][LIVELLO] messaggio — verificato riproducendo
        // `Builder::default()` del crate (bug #558 punto 1: la doc/i test
        // precedenti assumevano l'ordine [LIVELLO][target], invertito).
        let line = "[2026-05-11][14:23:12][pap_lib::editor][INFO] prompt salvato id=abc";
        let r = parse_riga(line);
        assert_eq!(r.timestamp, "2026-05-11 14:23:12");
        assert_eq!(r.level, "INFO");
        assert_eq!(r.target, "pap_lib::editor");
        assert_eq!(r.message, "prompt salvato id=abc");
    }

    #[test]
    fn parse_riga_formato_3_brackets_solo() {
        // Manca il bracket del livello: parser deve estrarre il target e
        // mettere il resto in message (fallback difensivo — righe così
        // non dovrebbero comparire dal formatter reale, che emette sempre
        // 4 gruppi, ma una riga tronca/malformata non deve far panicare
        // né perdere silenziosamente il testo).
        let line = "[2026-05-11][14:23:12][pap_lib::vault] panic in module";
        let r = parse_riga(line);
        assert!(r.level.is_empty());
        assert_eq!(r.target, "pap_lib::vault");
        assert_eq!(r.message, "panic in module");
    }

    #[test]
    fn parse_riga_non_match_fallback_raw() {
        let line = "  at module::function (src/file.rs:42)";
        let r = parse_riga(line);
        assert!(r.timestamp.is_empty());
        assert!(r.level.is_empty());
        assert_eq!(r.message, line);
        assert_eq!(r.raw, line);
    }

    #[test]
    fn parse_riga_messaggio_vuoto() {
        let line = "[2026-05-11][14:23:12][pap_lib::vault][ERROR] ";
        let r = parse_riga(line);
        assert_eq!(r.level, "ERROR");
        assert_eq!(r.target, "pap_lib::vault");
        assert_eq!(r.message, "");
    }

    // ─── fix sicurezza CWE-532: `log_redazione::formatta_riga` sostituisce
    // il formatter di default di `tauri-plugin-log` in `lib.rs::run`. Il
    // layout `[data][ora][target][LIVELLO] messaggio` DEVE restare
    // identico — è ciò da cui dipende `parse_riga` qui sopra. Roundtrip
    // per tutti i livelli, con e senza redazione. ───

    #[test]
    fn riga_del_nuovo_formatter_passa_per_parse_riga_per_ogni_livello() {
        for (level, atteso) in [
            (log::Level::Trace, "TRACE"),
            (log::Level::Debug, "DEBUG"),
            (log::Level::Info, "INFO"),
            (log::Level::Warn, "WARN"),
            (log::Level::Error, "ERROR"),
        ] {
            let riga = crate::log_redazione::formatta_riga(
                "pap_lib::editor",
                level,
                "prompt salvato id=abc",
            );
            let r = parse_riga(&riga);
            assert!(!r.timestamp.is_empty(), "livello {atteso}: {riga}");
            assert_eq!(r.level, atteso, "livello {atteso}: {riga}");
            assert_eq!(r.target, "pap_lib::editor", "livello {atteso}: {riga}");
            assert_eq!(
                r.message, "prompt salvato id=abc",
                "livello {atteso}: {riga}"
            );
        }
    }

    #[test]
    fn riga_del_nuovo_formatter_con_target_ureq_passa_per_parse_riga_e_resta_redatta() {
        let riga = crate::log_redazione::formatta_riga(
            "ureq::unit",
            log::Level::Debug,
            "writing prelude: POST / HTTP/1.1\r\nx-api-key: sk-segreto\r\n\r\n",
        );
        let r = parse_riga(&riga);
        assert_eq!(r.level, "DEBUG");
        assert_eq!(r.target, "ureq::unit");
        assert!(!r.message.contains("sk-segreto"));
        assert!(r.message.contains("x-api-key: ***"));
    }

    // ─── #558 punto 1: ultime_righe_parsate / debug_log_leggi ───

    #[test]
    fn ultime_righe_parsate_rispetta_il_clamp_e_l_ordine() {
        let contenuto = "\
[2026-05-11][10:00:00][pap_lib::a][INFO] uno
[2026-05-11][10:00:01][pap_lib::b][WARN] due
[2026-05-11][10:00:02][pap_lib::c][ERROR] tre
";
        let righe = ultime_righe_parsate(contenuto, 2);
        assert_eq!(righe.len(), 2);
        assert_eq!(righe[0].message, "due");
        assert_eq!(righe[1].message, "tre");
        assert_eq!(righe[1].level, "ERROR");
    }

    #[test]
    fn ultime_righe_parsate_n_maggiore_delle_righe_disponibili() {
        let contenuto = "[2026-05-11][10:00:00][pap_lib::a][INFO] unica\n";
        let righe = ultime_righe_parsate(contenuto, 200);
        assert_eq!(righe.len(), 1);
    }

    #[test]
    fn ultime_righe_parsate_file_vuoto() {
        assert!(ultime_righe_parsate("", 200).is_empty());
    }

    /// Difesa in profondità CWE-532: una riga già scritta su disco (da
    /// prima della fix del formatter) con la chiave in chiaro nel target
    /// `ureq` deve arrivare redatta al viewer in-app.
    #[test]
    fn ultime_righe_parsate_redige_chiavi_di_log_storici_target_ureq() {
        let contenuto = "\
[2026-05-11][10:00:00][ureq::unit][DEBUG] x-api-key: sk-vecchia-in-chiaro
[2026-05-11][10:00:01][pap_lib::sync][WARN] prompt cookie: scartato per conflitto
";
        let righe = ultime_righe_parsate(contenuto, 10);
        assert_eq!(righe.len(), 2);
        assert!(!righe[0].message.contains("sk-vecchia-in-chiaro"));
        assert!(righe[0].message.contains("x-api-key: ***"));
        // Il messaggio applicativo con "cookie:" per coincidenza nel testo
        // (target NON ureq) non deve perdere la coda (trappola #2).
        assert_eq!(righe[1].message, "prompt cookie: scartato per conflitto");
    }

    /// Riproduce `debug_log_leggi` end-to-end su un file di log fittizio
    /// scritto su disco (non un `AppHandle` reale — non necessario: la
    /// sola dipendenza da Tauri in `debug_log_leggi` è la risoluzione
    /// della directory, già coperta da `log_dir`/`raccogli_file_log`).
    #[test]
    fn debug_log_leggi_pipeline_su_file_fittizio() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pap.log");
        let righe_scritte: Vec<String> = (0..5)
            .map(|i| format!("[2026-05-11][10:00:0{i}][pap_lib::editor][INFO] evento numero {i}"))
            .collect();
        fs::write(&path, righe_scritte.join("\n") + "\n").unwrap();

        let contenuto = fs::read_to_string(&path).unwrap();
        let righe = ultime_righe_parsate(&contenuto, 3);

        assert_eq!(righe.len(), 3);
        assert_eq!(righe[0].message, "evento numero 2");
        assert_eq!(righe[2].message, "evento numero 4");
        assert!(righe.iter().all(|r| r.level == "INFO"));
        assert!(righe.iter().all(|r| r.target == "pap_lib::editor"));
    }

    // ─── #462: export ZIP non più in /tmp con nome prevedibile ───

    #[cfg(unix)]
    #[test]
    fn restringi_permessi_owner_applica_0600() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let path = dir.path().join("export-test.zip");
        fs::write(&path, b"contenuto zip finto").unwrap();

        // Prima del fix il file poteva restare con i permessi di default
        // dell'umask (spesso 0644, leggibile da altri utenti su /tmp).
        restringi_permessi_owner(&path);

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "il file esportato deve essere leggibile solo dall'owner"
        );
    }

    #[test]
    fn restringi_permessi_owner_file_inesistente_non_panica() {
        // Best-effort: un path inesistente logga un warning ma non deve
        // panicare né interrompere il chiamante.
        restringi_permessi_owner(Path::new("/nonexistent/dir/xyz.zip"));
    }

    /// Fix MEDIUM (review PR #481): il file deve nascere già 0600, non
    /// solo essere ristretto a export completato — niente finestra TOCTOU
    /// tra creazione e scrittura in cui è leggibile con i permessi di
    /// default dell'umask.
    #[cfg(unix)]
    #[test]
    fn crea_file_export_nasce_gia_0600() {
        use std::io::Write as _;
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let path = dir.path().join("export-test.zip");

        let mut file = crea_file_export(&path).unwrap();
        // I permessi devono essere già 0600 SUBITO dopo l'apertura,
        // prima ancora di scrivere qualunque byte.
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "il file deve nascere 0600 fin dalla creazione, non solo a fine scrittura"
        );

        file.write_all(b"contenuto").unwrap();
        let mode_dopo_scrittura = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode_dopo_scrittura, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn crea_file_export_tronca_file_preesistente() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("export-test.zip");
        fs::write(&path, b"contenuto vecchio lungo").unwrap();

        let file = crea_file_export(&path).unwrap();
        drop(file);

        assert_eq!(fs::metadata(&path).unwrap().len(), 0);
    }

    // ─── issue #584 "pulisci log non pulisce nulla": debug_log_pulisci /
    // tronca_file_log. Prima di questa PR non esisteva NESSUN test per
    // debug_log_pulisci — solo permessi/export erano coperti. ───

    /// Caso positivo (mancava del tutto): un file popolato viene
    /// effettivamente ridotto a zero byte, non solo "non fallisce".
    #[test]
    fn tronca_file_log_azzera_davvero_un_file_popolato() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pap.log");
        fs::write(&path, "riga vecchia 1\nriga vecchia 2\n").unwrap();
        let size_iniziale = fs::metadata(&path).unwrap().len();
        assert!(
            size_iniziale > 0,
            "precondizione: il file deve partire popolato"
        );

        let (prima, dopo) = tronca_file_log(&path).unwrap();

        assert_eq!(prima, size_iniziale);
        assert_eq!(dopo, Some(0));
        assert_eq!(
            fs::metadata(&path).unwrap().len(),
            0,
            "il file su disco deve essere davvero vuoto, non solo il valore ritornato"
        );
    }

    #[test]
    fn tronca_file_log_su_file_gia_vuoto_resta_a_zero() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pap.log");
        fs::write(&path, b"").unwrap();

        let (prima, dopo) = tronca_file_log(&path).unwrap();

        assert_eq!(prima, 0);
        assert_eq!(dopo, Some(0));
    }

    /// Scrittore concorrente (thread separato, come farebbe il logger
    /// globale su un evento asincrono) che continua ad appendere DOPO il
    /// truncate: non deve panicare, perdere né corrompere le righe scritte
    /// dopo la pulizia — solo il contenuto pre-pulizia deve sparire.
    #[test]
    fn tronca_file_log_con_scrittore_concorrente_che_continua_a_loggare() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pap.log");
        fs::write(&path, "contenuto da pulire, non deve sopravvivere\n").unwrap();

        let (prima, dopo) = tronca_file_log(&path).unwrap();
        assert!(prima > 0);
        assert_eq!(dopo, Some(0));

        let path_thread = path.clone();
        std::thread::spawn(move || {
            use std::fs::OpenOptions;
            let mut f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path_thread)
                .unwrap();
            f.write_all(b"riga scritta dopo la pulizia\n").unwrap();
        })
        .join()
        .unwrap();

        let contenuto = fs::read_to_string(&path).unwrap();
        assert_eq!(contenuto, "riga scritta dopo la pulizia\n");
        assert!(!contenuto.contains("da pulire"));
    }

    /// **Ipotesi SCARTATA (rettifica post-review PR #588), non il
    /// comportamento reale.** Questo writer modella un writer *ipotetico*
    /// con buffering differito arbitrario (accumula in memoria, scrive su
    /// disco solo quando qualcuno chiama esplicitamente `flush()`, senza
    /// mai farlo da solo). Leggendo il sorgente vendorizzato pinnato nel
    /// lock (`tauri-plugin-log-2.9.0` + `fern-0.7.1`, `writer_log_impl!`)
    /// si vede che il vero `RotatingFile` NON si comporta così nel
    /// percorso normale: ogni `log()` fa `write!` + `flush()` sotto lo
    /// stesso lock, quindi il buffer reale non sopravvive mai da una
    /// chiamata alla successiva. I due test sotto dimostrano solo che
    /// QUESTO mock si comporta come è stato scritto — non provano nulla
    /// sul comportamento reale del plugin. Il meccanismo reale candidato
    /// per #584 (flush che fallisce e lascia il buffer pieno) è testato
    /// separatamente più sotto, con `RotatingFileSimulato`.
    struct WriterIpoteticoBufferingDifferito {
        path: PathBuf,
        buffer: Vec<u8>,
    }

    impl WriterIpoteticoBufferingDifferito {
        fn scrivi(&mut self, s: &str) {
            self.buffer.extend_from_slice(s.as_bytes());
        }

        fn flush(&mut self) {
            if self.buffer.is_empty() {
                return;
            }
            use std::fs::OpenOptions;
            let mut f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
                .unwrap();
            f.write_all(&self.buffer).unwrap();
            self.buffer.clear();
        }
    }

    /// Ipotesi scartata, caso "senza flush": con un writer che bufferizza
    /// in modo differito arbitrario (NON il comportamento reale, vedi doc
    /// sopra), non chiamare `flush()` prima del truncate lascia
    /// ricomparire il contenuto bufferizzato al flush successivo.
    #[test]
    fn ipotesi_scartata_writer_con_buffering_differito_senza_flush_le_righe_ricompaiono() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pap.log");
        fs::write(&path, "riga vecchia\n").unwrap();

        let mut writer = WriterIpoteticoBufferingDifferito {
            path: path.clone(),
            buffer: Vec::new(),
        };
        writer.scrivi("riga bufferizzata non ancora su disco\n");
        // Nessun flush qui: comportamento del mock, NON dimostrato per il
        // writer reale del plugin (v. doc della struct sopra).

        File::create(&path).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().len(),
            0,
            "il truncate azzera subito"
        );

        writer.flush();
        let contenuto = fs::read_to_string(&path).unwrap();
        assert!(
            contenuto.contains("riga bufferizzata"),
            "con QUESTO mock, senza flush prima del truncate il contenuto bufferizzato ricompare"
        );
    }

    /// Stesso mock ipotetico, ma con l'ordine usato da `tronca_file_log`
    /// (`flush()` prima del truncate): dimostra solo che il mock si
    /// comporta come scritto, non che questo sia il motivo per cui il
    /// flush difensivo in `tronca_file_log` aiuta nel caso reale.
    #[test]
    fn ipotesi_scartata_writer_con_buffering_differito_con_flush_le_righe_non_ricompaiono() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pap.log");
        fs::write(&path, "riga vecchia\n").unwrap();

        let mut writer = WriterIpoteticoBufferingDifferito {
            path: path.clone(),
            buffer: Vec::new(),
        };
        writer.scrivi("riga bufferizzata non ancora su disco\n");

        writer.flush(); // ordine analogo a tronca_file_log: flush prima del truncate
        File::create(&path).unwrap();

        let contenuto = fs::read_to_string(&path).unwrap();
        assert!(
            contenuto.is_empty(),
            "con QUESTO mock, flush prima del truncate non lascia ricomparire nulla"
        );
    }

    /// **Meccanismo REALE candidato per #584 (Azione 2, review PR #588),
    /// non accertato ma il più credibile trovato leggendo il sorgente.**
    /// Modella `RotatingFile::flush()` (tauri-plugin-log 2.9.0,
    /// `lib.rs:310-330`): fa `write_all` poi `file.flush()` e chiama
    /// `self.buffer.clear()` **solo se entrambi riescono**. Se uno dei
    /// due fallisce — es. sharing violation Windows — il buffer resta
    /// pieno oltre la singola chiamata: è questo, non un buffering
    /// differito "normale", il modo reale per cui il contenuto
    /// pre-pulizia può ricomparire dopo un truncate esterno.
    struct RotatingFileSimulato {
        path: PathBuf,
        buffer: Vec<u8>,
        prossimo_flush_fallisce: bool,
    }

    impl RotatingFileSimulato {
        fn scrivi(&mut self, s: &str) {
            self.buffer.extend_from_slice(s.as_bytes());
        }

        /// Ritorna `Err` se `prossimo_flush_fallisce`, senza svuotare il
        /// buffer — replica `RotatingFile::flush()` reale, dove
        /// `buffer.clear()` è raggiunto solo dopo write+flush riusciti.
        fn flush(&mut self) -> std::io::Result<()> {
            if self.prossimo_flush_fallisce {
                return Err(std::io::Error::other("sharing violation simulata"));
            }
            if self.buffer.is_empty() {
                return Ok(());
            }
            use std::fs::OpenOptions;
            let mut f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;
            f.write_all(&self.buffer)?;
            self.buffer.clear();
            Ok(())
        }
    }

    /// Riproduce il meccanismo Azione 2: un flush fallito lascia il
    /// buffer pieno; il truncate esterno procede comunque (Azione 4:
    /// `log::Log::flush(&self) -> ()` non ritorna `Result`, quindi il
    /// chiamante non avrebbe comunque modo di sapere che è fallito); un
    /// flush successivo che riesce scarica il contenuto vecchio, che
    /// "ricompare" davvero — con QUESTO meccanismo, non con quello
    /// smentito nei due test sopra.
    #[test]
    fn meccanismo_reale_flush_fallito_lascia_il_buffer_pieno_oltre_la_singola_chiamata() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pap.log");

        let mut writer = RotatingFileSimulato {
            path: path.clone(),
            buffer: Vec::new(),
            prossimo_flush_fallisce: true,
        };
        writer.scrivi("riga che dovrebbe sparire dopo la pulizia\n");

        let esito = writer.flush();
        assert!(esito.is_err(), "precondizione: il flush simulato fallisce");
        assert!(
            !writer.buffer.is_empty(),
            "il buffer resta pieno perché clear() non viene mai raggiunto dopo un Err"
        );

        // Il truncate esterno procede comunque: nessun modo di sapere
        // dall'esterno che l'ultimo flush del logger è fallito.
        File::create(&path).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().len(), 0);

        // Un flush successivo che stavolta riesce scarica il buffer
        // vecchio: la riga pre-pulizia ricompare per davvero.
        writer.prossimo_flush_fallisce = false;
        writer.flush().unwrap();
        let contenuto = fs::read_to_string(&path).unwrap();
        assert!(contenuto.contains("riga che dovrebbe sparire"));
    }

    // ─── Azione 9 (review PR #588): "Pulisci log" ora elimina anche i
    // rotati (HIGH preesistente CWE-532 — con RotationStrategy::KeepAll
    // non venivano mai eliminati automaticamente). ───

    /// Caso positivo: rotati presenti vengono davvero rimossi, e il conteggio
    /// byte/file torna corretto.
    #[test]
    fn pulisci_log_in_dir_rimuove_davvero_i_rotati_presenti() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pap.log"), "corrente\n").unwrap();
        fs::write(dir.path().join("pap.log.1"), "rotato vecchio 1, con chiave in chiaro").unwrap();
        fs::write(dir.path().join("pap.log.2"), "rotato vecchio 2").unwrap();

        let esito = pulisci_log_in_dir(dir.path()).unwrap();

        assert_eq!(esito.size_dopo, Some(0));
        assert!(esito.rotati_falliti.is_empty());
        let nomi: Vec<String> = esito.rotati_rimossi.iter().map(|f| f.name.clone()).collect();
        assert_eq!(nomi, vec!["pap.log.1".to_string(), "pap.log.2".to_string()]);
        assert!(esito.rotati_rimossi.iter().all(|f| f.size_bytes > 0));

        assert!(
            !dir.path().join("pap.log.1").exists(),
            "il rotato deve essere sparito dal disco, non solo dal report"
        );
        assert!(!dir.path().join("pap.log.2").exists());
        assert!(
            dir.path().join("pap.log").exists(),
            "il corrente deve restare (troncato, non eliminato)"
        );
    }

    #[test]
    fn pulisci_log_in_dir_senza_rotati_non_riporta_nulla_da_rimuovere() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pap.log"), "solo corrente\n").unwrap();

        let esito = pulisci_log_in_dir(dir.path()).unwrap();

        assert!(esito.rotati_rimossi.is_empty());
        assert!(esito.rotati_falliti.is_empty());
    }

    /// Rimozione parziale: un rotato eliminabile e uno no (simulato
    /// rendendo la sua "directory" un file, così `fs::remove_file` sul
    /// path costruito sotto fallisce con NotADirectory) — l'esito deve
    /// riportare entrambi gli scenari, non un falso successo generico.
    #[cfg(unix)]
    #[test]
    fn pulisci_log_in_dir_riporta_un_fallimento_parziale_senza_falso_successo() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pap.log"), "corrente\n").unwrap();
        fs::write(dir.path().join("pap.log.1"), "rotato eliminabile").unwrap();
        fs::write(dir.path().join("pap.log.2"), "rotato NON eliminabile").unwrap();

        // Toglie il permesso di scrittura sulla directory: `remove_file`
        // richiede il permesso di scrittura sulla dir contenente, quindi
        // ENTRAMBI i rotati falliranno — sufficiente a dimostrare che un
        // fallimento (qui totale sui rotati) viene riportato in
        // `rotati_falliti`, non nascosto da un `Ok` generico, mentre il
        // truncate del corrente (già avvenuto via metadata prima) resta
        // comunque riportato correttamente.
        let mode_originale = fs::metadata(dir.path()).unwrap().permissions().mode();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o500)).unwrap();

        let esito = pulisci_log_in_dir(dir.path());

        // Ripristina i permessi PRIMA di qualunque assert/unwrap che possa
        // fallire, altrimenti `tempdir` non riesce a ripulirsi a fine test.
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(mode_originale)).unwrap();

        let esito = esito.unwrap();
        assert!(
            esito.rotati_rimossi.is_empty(),
            "nessun rotato deve risultare rimosso con la dir non scrivibile"
        );
        assert_eq!(
            esito.rotati_falliti.len(),
            2,
            "entrambi i rotati devono finire in rotati_falliti, non in un Ok silenzioso"
        );
        let nomi_falliti: Vec<String> = esito
            .rotati_falliti
            .iter()
            .map(|f| f.name.clone())
            .collect();
        assert!(nomi_falliti.contains(&"pap.log.1".to_string()));
        assert!(nomi_falliti.contains(&"pap.log.2".to_string()));
        assert!(esito.rotati_falliti.iter().all(|f| !f.errore.is_empty()));
    }

    // ─── HIGH review avversariale pre-merge PR #570: dialog spostato lato Rust ───

    #[test]
    fn nome_file_zip_suggerito_ha_lo_schema_atteso() {
        let nome = nome_file_zip_suggerito("2026-08-01T17:49:52Z");
        assert_eq!(nome, "pap-debug-log-2026-08-01-17-49-52.zip");
    }

    /// Verifica statica (a tempo di compilazione, non a runtime): la firma
    /// del comando accetta solo l'`AppHandle`, nessun parametro
    /// `destinazione` proveniente dal chiamante IPC. Se in futuro
    /// qualcuno reintroducesse `destinazione: String`, questa riga
    /// smetterebbe di compilare — l'arità/i tipi non coinciderebbero più
    /// con la coercizione a `fn(tauri::AppHandle) -> ...` — e la build/i
    /// test fallirebbero.
    ///
    /// Il resto del comportamento (dialog aperto lato Rust, annullamento
    /// gestito senza errore restituendo `Ok(None)`) non è verificabile
    /// con un `AppHandle` reale in questo crate — la feature `tauri::test`
    /// non è attualmente cablata nei dev-dependencies del progetto — ed è
    /// invece coperto lato frontend in
    /// `debug-log-export-logic.test.ts` (`eseguiEsportaLogZip`), che
    /// verifica esplicitamente che l'annullamento (`null`) non produca un
    /// errore e che l'invocazione non passi più alcun argomento.
    #[test]
    fn debug_log_esporta_zip_non_accetta_piu_un_percorso_dal_chiamante() {
        let _f: fn(tauri::AppHandle) -> Result<Option<String>, PapErrore> = debug_log_esporta_zip;
    }
}
