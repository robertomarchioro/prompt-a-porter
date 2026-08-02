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
//! - `debug_log_pulisci` — truncate file corrente (mantiene i rotati)
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
//! e allega `pap.log` a mano bypassa questa redazione — non c'è modo di
//! intercettarlo lato Rust; l'unico modo per ripulire quel file è
//! `debug_log_pulisci` prima di allegarlo.

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

/// Truncate il file `pap.log` corrente. I file rotati restano intatti.
/// Best-effort: se il file non esiste è un no-op (non un errore).
#[tauri::command]
pub fn debug_log_pulisci(app: tauri::AppHandle) -> Result<(), PapErrore> {
    let dir = log_dir(&app)?;
    let path = dir.join(format!("{NOME_LOG}.log"));
    if !path.exists() {
        return Ok(());
    }
    File::create(&path).map_err(|e| PapErrore::dominio("Pulizia dei log non riuscita.", e))?;
    log::info!("Debug log pulito (truncate)");
    Ok(())
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
