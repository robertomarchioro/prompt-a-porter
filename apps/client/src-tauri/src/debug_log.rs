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
//! - `debug_log_esporta_zip` — crea ZIP in temp con file log + metadata,
//!   ritorna path per attach a GitHub issue

use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use tauri::Manager;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::errore::PapErrore;

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

/// #462 (security review, LOW): lo ZIP di export del debug log NON deve
/// finire in `std::env::temp_dir()` (`/tmp` è world-readable su sistemi
/// Unix multi-utente, ed era comunque un nome di file prevedibile). Usiamo
/// `app_data_dir()` — già per-utente sia su Unix che su Windows — in una
/// sottocartella dedicata, e su Unix restringiamo i permessi a 0600 come
/// fatto in `preferenze.rs::salva_pure`.
fn export_dir(app: &tauri::AppHandle) -> Result<PathBuf, PapErrore> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| PapErrore::dominio("Impossibile determinare la cartella dati.", e))?
        .join("debug-exports");
    fs::create_dir_all(&dir)?;
    Ok(dir)
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
    File::create(&path).map_err(|e| {
        PapErrore::dominio("Pulizia dei log non riuscita.", e)
    })?;
    log::info!("Debug log pulito (truncate)");
    Ok(())
}

/// Crea uno ZIP con il file log corrente + i rotati + un file `metadata.txt`
/// (versione app, OS, timestamp) in `app_data_dir()/debug-exports` (Unix:
/// permessi 0600, solo owner — vedi `export_dir`/`restringi_permessi_owner`).
///
/// Ritorna il path assoluto al file ZIP creato; il frontend può poi
/// proporre "salva con nome" o aprire il file manager su quel path.
#[tauri::command]
pub fn debug_log_esporta_zip(app: tauri::AppHandle) -> Result<String, PapErrore> {
    let dir = log_dir(&app)?;
    let files = raccogli_file_log(&dir);

    let timestamp = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as i64;
        // ':' sostituiti con '-' per compatibilità filename Windows
        format_iso_utc(s).replace(':', "-")
    };
    let zip_path = export_dir(&app)?.join(format!("pap-debug-log-{timestamp}.zip"));

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
        zip.start_file(&f.name, opts)
            .map_err(|e| PapErrore::dominio("Creazione dell'archivio dei log non riuscita.", e))?;
        zip.write_all(&buf)
            .map_err(|e| PapErrore::dominio("Creazione dell'archivio dei log non riuscita.", e))?;
    }

    zip.finish()
        .map_err(|e| PapErrore::dominio("Creazione dell'archivio dei log non riuscita.", e))?;
    restringi_permessi_owner(&zip_path);
    log::info!("Debug log esportato: {}", zip_path.display());
    Ok(zip_path.to_string_lossy().to_string())
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

/// Parsing best-effort del formato default di tauri-plugin-log:
/// `[YYYY-MM-DD][HH:MM:SS.mmm +TZ][LEVEL][target] message`
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
    let Some((level, r3)) = estrai_bracket(resto) else {
        return RigaLog {
            timestamp: String::new(),
            level: String::new(),
            target: String::new(),
            message: raw.clone(),
            raw,
        };
    };
    resto = r3;
    let Some((target, r4)) = estrai_bracket(resto) else {
        return RigaLog {
            timestamp: format!("{data} {ora}"),
            level: level.to_string(),
            target: String::new(),
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
    let righe: Vec<&str> = contenuto.lines().collect();
    let inizio = righe.len().saturating_sub(n);
    let parsed: Vec<RigaLog> = righe[inizio..].iter().map(|l| parse_riga(l)).collect();
    Ok(parsed)
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
        let line = "[2026-05-11][14:23:12.345 +02:00][INFO][pap_lib::editor] prompt salvato id=abc";
        let r = parse_riga(line);
        assert_eq!(r.timestamp, "2026-05-11 14:23:12.345 +02:00");
        assert_eq!(r.level, "INFO");
        assert_eq!(r.target, "pap_lib::editor");
        assert_eq!(r.message, "prompt salvato id=abc");
    }

    #[test]
    fn parse_riga_formato_3_brackets_solo() {
        // Manca target bracket, parser deve estrarre level e mettere il resto in message
        let line = "[2026-05-11][14:23:12][WARN] panic in module";
        let r = parse_riga(line);
        assert_eq!(r.level, "WARN");
        assert!(r.target.is_empty());
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
        let line = "[2026-05-11][14:23:12][ERROR][pap_lib::vault] ";
        let r = parse_riga(line);
        assert_eq!(r.level, "ERROR");
        assert_eq!(r.target, "pap_lib::vault");
        assert_eq!(r.message, "");
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
        assert_eq!(mode, 0o600, "il file esportato deve essere leggibile solo dall'owner");
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
}
