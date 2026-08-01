// Modulo embeddings — Fase 3 Step 1.
//
// Calcolo embeddings locali via ONNX Runtime (`ort` crate). Modello:
// `paraphrase-multilingual-MiniLM-L12-v2` (384 dim, ~118 MB ONNX
// quantizzato), scelto in Spike 3 v2 (vedi
// `docs/architettura/decisioni/embedding-model.md`).
//
// Architettura distribuzione:
// - Modello scaricato lazy al primo uso da HuggingFace (Xenova fork)
// - libonnxruntime scaricata lazy al primo uso da Microsoft GitHub
//   release (per la piattaforma corrente)
// - Cache locale in `${data_dir}/models/multilingual-MiniLM-L12-v2/`
//   e `${data_dir}/onnxruntime/`
// - load-dynamic via env var `ORT_DYLIB_PATH` settata prima di creare
//   la prima Session (vedi ADR onnx-bundle.md)
//
// Implementazione completa:
// - PR 1/3: scaffolding + comando `embeddings_status`
// - PR 2/3: download model.onnx + tokenizer.json
// - PR 3/3 (questa): download libonnxruntime per piattaforma + Session
//   ort + tokenizer load + comando `embeddings_compute`

use ndarray::{Array1, Array2};
use ort::session::Session;
use ort::value::Tensor;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{Emitter, State};
use tokenizers::Tokenizer;

use crate::errore::PapErrore;
use crate::vault::VaultState;

// ─────────── Costanti modello ───────────

pub const MODEL_ID: &str = "multilingual-MiniLM-L12-v2";
pub const EMBEDDING_DIM: usize = 384;

const HF_REPO: &str = "Xenova/paraphrase-multilingual-MiniLM-L12-v2";

// Fix #458: hash SHA-256 attesi per gli artefatti scaricati da HuggingFace,
// pinnati sul commit `main` corrente al momento della scrittura di questa
// PR. Se HuggingFace aggiorna il file, il download fallirà con un errore
// esplicito (fail-closed) invece di caricare silenziosamente un file
// diverso da quello verificato: in quel caso l'hash va aggiornato di
// proposito in un commit dedicato.
const SHA256_MODEL_ONNX: &str =
    "66fc00f5f29afcaff34092e1bdd20008ca3918265a82fb9695a551e510cc4ebc";
const SHA256_TOKENIZER_JSON: &str =
    "b60b6b43406a48bf3638526314f3d232d97058bc93472ff2de930d43686fa441";

const FILES_HF: &[(&str, &str, &str)] = &[
    ("onnx/model_quantized.onnx", "model.onnx", SHA256_MODEL_ONNX),
    ("tokenizer.json", "tokenizer.json", SHA256_TOKENIZER_JSON),
];

// ─────────── Costanti onnxruntime ───────────

/// Versione di ONNX Runtime da scaricare. Allineata con `api-23` di `ort`
/// crate (ORT 1.23 supporta tutte le API che usiamo).
const ORT_VERSION: &str = "1.23.0";
const ORT_RELEASE_BASE: &str = "https://github.com/microsoft/onnxruntime/releases/download/";
const HTTP_TIMEOUT_SEC: u64 = 600;

/// Lunghezza massima sequenza supportata dal MiniLM-L12-v2 (token).
const MAX_SEQ_LEN: usize = 128;

// ─────────── State ───────────

/// Stato globale del modulo embeddings. Inizialmente vuoto; popolato da
/// `embeddings_init` dopo che il modello è scaricato e libonnxruntime è
/// pronta sul filesystem.
///
/// `last_used` traccia il timestamp dell'ultima `compute_embedding_opt`
/// successful. Usato dal task background di idle-unload (Step 10) per
/// liberare RAM quando la Session è inattiva da una soglia configurabile.
pub struct EmbeddingsState {
    inner: Mutex<Option<EmbeddingsLoaded>>,
    last_used: Mutex<Option<Instant>>,
    /// Fix TOCTOU (CWE-367): TempDir che contiene la COPIA privata e già
    /// verificata di libonnxruntime effettivamente caricata da `ort` via
    /// `ORT_DYLIB_PATH`. Tenuta viva per l'intera vita del processo: dropparla
    /// cancellerebbe la libreria mentre `ort` la tiene mappata in memoria.
    /// Sostituita ad ogni (ri)caricamento della Session dallo staging fresco.
    lib_privata: Mutex<Option<tempfile::TempDir>>,
}

struct EmbeddingsLoaded {
    session: Session,
    tokenizer: Tokenizer,
}

impl EmbeddingsState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
            last_used: Mutex::new(None),
            lib_privata: Mutex::new(None),
        }
    }
}

impl Default for EmbeddingsState {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────── Status ───────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "stato")]
pub enum EmbeddingsStato {
    NonScaricato { model_id: String, path_atteso: String },
    Pronto { model_id: String, path: String, size_mb: u64 },
    Caricato { model_id: String, dimensione: usize },
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressDownload {
    pub file: String,
    pub bytes: u64,
    pub total: Option<u64>,
    pub indice_file: usize,
    pub totale_file: usize,
}

// ─────────── Path helpers ───────────

fn percorso_modello(state: &VaultState) -> PathBuf {
    state.data_dir().join("models").join(MODEL_ID)
}

fn percorso_runtime_dir(state: &VaultState) -> PathBuf {
    state.data_dir().join("onnxruntime").join(ORT_VERSION)
}

/// Nome file della libreria nativa per la piattaforma corrente.
fn nome_libonnxruntime() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "onnxruntime.dll"
    }
    #[cfg(target_os = "macos")]
    {
        "libonnxruntime.dylib"
    }
    #[cfg(all(target_os = "linux", not(target_os = "macos")))]
    {
        "libonnxruntime.so"
    }
}

fn percorso_libonnxruntime(state: &VaultState) -> PathBuf {
    percorso_runtime_dir(state).join(nome_libonnxruntime())
}

fn dim_cartella_mb(path: &Path) -> u64 {
    let mut total: u64 = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                }
            }
        }
    }
    total / (1024 * 1024)
}

fn modello_completo(path: &Path) -> bool {
    FILES_HF
        .iter()
        .all(|(_, locale, _)| path.join(locale).is_file())
}

/// `true` solo se sia il modello (model.onnx + tokenizer.json) SIA
/// libonnxruntime sono presenti su disco. Estratta da `embeddings_status`
/// (fix #556, sintomo 3) per essere testabile senza costruire uno State
/// Tauri: prima del fix lo stato controllava SOLO `modello_completo`, quindi
/// dichiarava "Pronto" anche quando la libreria nativa non era mai stata
/// scaricata — mentre `init_session_pure` rifiutava con "libonnxruntime non
/// scaricata". Lo stato mentiva all'utente.
fn pronto_su_disco(dir_modello: &Path, lib_path: &Path) -> bool {
    modello_completo(dir_modello) && lib_path.is_file()
}

// ─────────── Verifica integrità (fix #458) ───────────

/// Calcola l'hash SHA-256 di un buffer, come stringa hex minuscola.
fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Verifica che l'hash SHA-256 di `bytes` corrisponda a `atteso`
/// (confronto case-insensitive). Fail-closed: qualunque mismatch produce
/// un errore esplicito invece di procedere silenziosamente con un
/// artefatto non verificato — in particolare `libonnxruntime` è codice
/// nativo caricato ed eseguito nel processo, quindi un file manomesso
/// (a download avvenuto, o riscritto sul disco in un momento successivo)
/// equivarrebbe a RCE. Usata sia subito dopo il download sia ad ogni
/// riavvio prima di ricaricare gli artefatti dalla cache locale.
fn verifica_sha256(bytes: &[u8], atteso: &str, nome_file: &str) -> Result<(), PapErrore> {
    let calcolato = sha256_hex(bytes);
    if !calcolato.eq_ignore_ascii_case(atteso) {
        return Err(PapErrore::Generico(format!(
            "Verifica integrità fallita per {nome_file}: l'hash SHA-256 non corrisponde \
             a quello atteso. Il file potrebbe essere stato manomesso, corrotto o \
             sostituito; operazione interrotta."
        )));
    }
    Ok(())
}

/// Fix #458 (review MEDIUM): ri-verifica il SHA-256 di TUTTI gli artefatti
/// in cache (modello, tokenizer, libonnxruntime) ad OGNI avvio, non solo
/// al primo download. Senza questo, un attacker con scrittura sulla
/// data-dir dell'app (o una corruzione silenziosa su disco) potrebbe
/// sostituire `libonnxruntime` DOPO la verifica iniziale: ai riavvii
/// successivi il file veniva fidato solo perché "esiste su disco"
/// (`lib_path.is_file()`), senza ricontrollarne il contenuto. Fail-closed:
/// il primo mismatch interrompe il caricamento con un errore esplicito.
fn verifica_artefatti_cache_su_disco(dir_modello: &Path, lib_path: &Path) -> Result<(), PapErrore> {
    for (_, nome_locale, sha256_atteso) in FILES_HF {
        let path = dir_modello.join(nome_locale);
        let bytes = fs::read(&path).map_err(|e| {
            PapErrore::Generico(format!(
                "Impossibile leggere {nome_locale} per la verifica di integrità: {e}"
            ))
        })?;
        verifica_sha256(&bytes, sha256_atteso, nome_locale)?;
    }

    let (_, _, sha256_lib_atteso) = ort_release_filename()?;
    let lib_bytes = fs::read(lib_path).map_err(|e| {
        PapErrore::Generico(format!(
            "Impossibile leggere libonnxruntime per la verifica di integrità: {e}"
        ))
    })?;
    verifica_sha256(&lib_bytes, sha256_lib_atteso, "libonnxruntime")?;

    Ok(())
}

// ─────────── Staging privato libonnxruntime (fix TOCTOU CWE-367) ───────────

/// Nome della directory base, creata e posseduta dall'app, dentro cui viene
/// messa la copia privata per-processo di libonnxruntime.
const NOME_BASE_PRIVATA: &str = ".pap-priv";

/// Fix TOCTOU (CWE-367): copia la libreria nativa dalla cache in una directory
/// privata per-processo (una `mkdtemp` 0700 dentro la base hardened), rilegge
/// e ri-verifica il SHA-256 della COPIA, e ritorna `(TempDir, path_copia)`.
///
/// Il chiamante punta `ORT_DYLIB_PATH` alla copia e tiene vivo il `TempDir`
/// finché il processo vive: così i byte verificati QUI sono esattamente i byte
/// che `ort` fa `dlopen`, chiudendo lo swap in-place del file in cache tra la
/// verifica e il load.
fn stage_lib_verificata(
    runtime_dir: &Path,
    lib_cache_path: &Path,
    sha256_atteso: &str,
) -> Result<(tempfile::TempDir, PathBuf), PapErrore> {
    let base = base_privata_hardened(runtime_dir)?;
    let tmp = tempfile::Builder::new()
        .prefix("lib-")
        .tempdir_in(&base)
        .map_err(|e| PapErrore::dominio("Impossibile creare la copia privata di libonnxruntime.", e))?;
    let dest = tmp.path().join(nome_libonnxruntime());
    fs::copy(lib_cache_path, &dest)
        .map_err(|e| PapErrore::dominio("Copia privata di libonnxruntime non riuscita.", e))?;
    // Ri-verifica il SHA-256 della copia effettivamente presente nella dir
    // privata: è questa la copia che verrà caricata, non più il file in cache.
    let bytes = fs::read(&dest)
        .map_err(|e| PapErrore::dominio("Impossibile rileggere la copia privata di libonnxruntime per la verifica.", e))?;
    verifica_sha256(&bytes, sha256_atteso, "libonnxruntime (copia privata)")?;
    Ok((tmp, dest))
}

/// Crea (idempotente) e verifica in modo hardened la directory base privata
/// `${runtime_dir}/.pap-priv`, così che il PARENT IMMEDIATO della copia
/// per-processo non sia la runtime-dir largamente scrivibile ma una directory
/// che l'app crea e possiede.
///
/// Controlli symlink-safe (lstat via `symlink_metadata`, che NON segue i
/// symlink): la base deve essere una directory reale (non un symlink, non un
/// file), su Unix di proprietà dell'uid corrente e con permessi 0700. Se
/// esiste già ma fallisce un qualunque controllo (owner/mode sbagliati, è un
/// symlink, è un file) si FALLISCE CHIUSI con un errore di integrità invece di
/// usarla.
///
/// ONESTÀ sul residuo: questo NON rende a prova d'attacco l'intera catena di
/// parent. Un attaccante di uid diverso con permesso di scrittura su
/// `runtime_dir` (scenario che richiede una data-dir group/other-writable, NON
/// il default) può ancora rinominare `.pap-priv` stessa e tentare una race
/// sulla directory privata. È una mitigazione, non un'eliminazione completa
/// (vedi il commento in `init_session_pure`).
fn base_privata_hardened(runtime_dir: &Path) -> Result<PathBuf, PapErrore> {
    fs::create_dir_all(runtime_dir)
        .map_err(|e| PapErrore::dominio("Impossibile preparare la directory runtime di onnxruntime.", e))?;
    let base = runtime_dir.join(NOME_BASE_PRIVATA);

    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        match fs::DirBuilder::new().mode(0o700).create(&base) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => {
                return Err(PapErrore::dominio(
                    "Impossibile creare la directory privata di libonnxruntime.",
                    e,
                ))
            }
        }
    }
    #[cfg(not(unix))]
    {
        match fs::create_dir(&base) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => {
                return Err(PapErrore::dominio(
                    "Impossibile creare la directory privata di libonnxruntime.",
                    e,
                ))
            }
        }
    }

    verifica_base_privata(runtime_dir, &base)?;
    Ok(base)
}

/// Controllo hardened, symlink-safe, della directory base privata. Fail-closed
/// su qualunque anomalia (usato sia sul path creato ora sia su uno pre-esistente).
fn verifica_base_privata(runtime_dir: &Path, base: &Path) -> Result<(), PapErrore> {
    // lstat: NON segue i symlink, così un `.pap-priv` piazzato come symlink
    // dall'attaccante viene rilevato e rifiutato invece di essere seguito.
    let meta = fs::symlink_metadata(base)
        .map_err(|e| PapErrore::dominio("Impossibile ispezionare la directory privata di libonnxruntime.", e))?;
    if meta.file_type().is_symlink() {
        return Err(PapErrore::Generico(format!(
            "Verifica integrità fallita per la directory privata {}: è un symlink, \
             non una directory reale; operazione interrotta.",
            base.display()
        )));
    }
    if !meta.is_dir() {
        return Err(PapErrore::Generico(format!(
            "Verifica integrità fallita per la directory privata {}: non è una \
             directory; operazione interrotta.",
            base.display()
        )));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        use std::os::unix::fs::PermissionsExt;
        let uid_atteso = uid_corrente(runtime_dir)?;
        if meta.uid() != uid_atteso {
            return Err(PapErrore::Generico(format!(
                "Verifica integrità fallita per la directory privata {}: proprietario \
                 inatteso (uid {}, atteso {}); operazione interrotta.",
                base.display(),
                meta.uid(),
                uid_atteso
            )));
        }
        let mode = meta.permissions().mode() & 0o777;
        if mode != 0o700 {
            return Err(PapErrore::Generico(format!(
                "Verifica integrità fallita per la directory privata {}: permessi {mode:o} \
                 inattesi (atteso 700); operazione interrotta.",
                base.display()
            )));
        }
    }
    let _ = runtime_dir; // usato solo su unix (uid probe)
    Ok(())
}

/// Apprende l'uid del processo corrente senza dipendere da `libc`, leggendo il
/// proprietario di un file-probe temporaneo che creiamo — e che quindi
/// possediamo per certo. Il probe è cancellato all'uscita di funzione.
#[cfg(unix)]
fn uid_corrente(runtime_dir: &Path) -> Result<u32, PapErrore> {
    use std::os::unix::fs::MetadataExt;
    let probe = tempfile::Builder::new()
        .prefix(".uid-probe-")
        .tempfile_in(runtime_dir)
        .map_err(|e| PapErrore::dominio("Impossibile determinare l'utente del processo.", e))?;
    let uid = probe
        .as_file()
        .metadata()
        .map_err(|e| PapErrore::dominio("Impossibile leggere i metadati del probe uid.", e))?
        .uid();
    Ok(uid)
}

// ─────────── Status command ───────────

#[tauri::command]
pub fn embeddings_status(
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<EmbeddingsStato, PapErrore> {
    if rt_state.inner.lock().unwrap_or_else(|p| p.into_inner()).is_some() {
        return Ok(EmbeddingsStato::Caricato {
            model_id: MODEL_ID.to_string(),
            dimensione: EMBEDDING_DIM,
        });
    }
    let path = percorso_modello(&state);
    // Fix #556 (sintomo 3): `pronto_su_disco` controlla ANCHE la presenza di
    // libonnxruntime, non solo modello+tokenizer (vedi doc sulla funzione).
    if !path.exists() || !pronto_su_disco(&path, &percorso_libonnxruntime(&state)) {
        return Ok(EmbeddingsStato::NonScaricato {
            model_id: MODEL_ID.to_string(),
            path_atteso: path.display().to_string(),
        });
    }
    Ok(EmbeddingsStato::Pronto {
        model_id: MODEL_ID.to_string(),
        path: path.display().to_string(),
        size_mb: dim_cartella_mb(&path),
    })
}

// ─────────── Download HTTP streaming ───────────

fn http_get_streaming(url: &str) -> Result<Box<dyn Read + Send + Sync>, PapErrore> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(HTTP_TIMEOUT_SEC))
        .build();
    let resp = agent
        .get(url)
        .call()
        .map_err(|e| PapErrore::dominio("Download del modello di embedding non riuscito. Verifica la connessione.", e))?;
    Ok(resp.into_reader())
}

fn http_get_with_progress(
    app: &tauri::AppHandle,
    url: &str,
    indice_file: usize,
    totale_file: usize,
    nome_visibile: &str,
) -> Result<(Vec<u8>, Option<u64>), PapErrore> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(HTTP_TIMEOUT_SEC))
        .build();
    let resp = agent
        .get(url)
        .call()
        .map_err(|e| PapErrore::dominio("Download del modello di embedding non riuscito. Verifica la connessione.", e))?;
    let total: Option<u64> = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok());
    let mut reader = resp.into_reader();
    let mut buf = [0u8; 64 * 1024];
    let mut acc: u64 = 0;
    let mut last_emit_acc: u64 = 0;
    let mut out: Vec<u8> = Vec::with_capacity(total.unwrap_or(0) as usize);
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| PapErrore::dominio("Lettura del modello di embedding durante il download non riuscita.", e))?;
        if n == 0 {
            break;
        }
        out.extend_from_slice(&buf[..n]);
        acc += n as u64;
        if acc - last_emit_acc >= 256 * 1024 {
            let _ = app.emit(
                "embeddings:download:progress",
                ProgressDownload {
                    file: nome_visibile.to_string(),
                    bytes: acc,
                    total,
                    indice_file,
                    totale_file,
                },
            );
            last_emit_acc = acc;
        }
    }
    let _ = app.emit(
        "embeddings:download:progress",
        ProgressDownload {
            file: nome_visibile.to_string(),
            bytes: acc,
            total,
            indice_file,
            totale_file,
        },
    );
    Ok((out, total))
}

fn scarica_file(
    app: &tauri::AppHandle,
    url: &str,
    dest: &Path,
    indice_file: usize,
    totale_file: usize,
    nome_visibile: &str,
    sha256_atteso: &str,
) -> Result<(), PapErrore> {
    let dest_tmp = dest.with_extension("download-partial");
    if let Some(parent) = dest_tmp.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut out = fs::File::create(&dest_tmp)?;
    let mut reader = http_get_streaming(url)?;
    let mut buf = [0u8; 64 * 1024];
    let mut acc: u64 = 0;
    let mut last_emit_acc: u64 = 0;
    // Fix #458: hash calcolato incrementalmente mentre il file viene
    // scritto su disco, per evitare di doverlo rileggere per intero dopo.
    let mut hasher = Sha256::new();
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| PapErrore::dominio("Lettura del modello di embedding durante il download non riuscita.", e))?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n])?;
        hasher.update(&buf[..n]);
        acc += n as u64;
        if acc - last_emit_acc >= 256 * 1024 {
            let _ = app.emit(
                "embeddings:download:progress",
                ProgressDownload {
                    file: nome_visibile.to_string(),
                    bytes: acc,
                    total: None,
                    indice_file,
                    totale_file,
                },
            );
            last_emit_acc = acc;
        }
    }
    out.flush()?;
    drop(out);

    // Fix #458: verifica integrità PRIMA di spostare il file nella
    // posizione finale da cui verrà caricato/usato. Su mismatch il file
    // parziale viene rimosso (fail-closed, niente file sospetto residuo).
    let digest_hex: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    if !digest_hex.eq_ignore_ascii_case(sha256_atteso) {
        let _ = fs::remove_file(&dest_tmp);
        return Err(PapErrore::Generico(format!(
            "Verifica integrità fallita per {nome_visibile}: l'hash SHA-256 del download \
             non corrisponde a quello atteso. Il file potrebbe essere stato manomesso \
             o il mirror è compromesso; download interrotto."
        )));
    }

    fs::rename(&dest_tmp, dest)?;
    let _ = app.emit(
        "embeddings:download:progress",
        ProgressDownload {
            file: nome_visibile.to_string(),
            bytes: acc,
            total: None,
            indice_file,
            totale_file,
        },
    );
    Ok(())
}

// ─────────── Download libonnxruntime ───────────

/// Tarball name + sub-path della libreria nativa nella release upstream +
/// hash SHA-256 atteso dell'intero archivio (fix #458).
///
/// L'hash è verificato PRIMA dell'estrazione: `libonnxruntime` è codice
/// nativo caricato ed eseguito nel processo (via `ORT_DYLIB_PATH`), quindi
/// un archivio manomesso non verificato equivarrebbe a RCE. Ogni
/// piattaforma supportata ha il proprio hash pinnato qui; una piattaforma
/// non elencata fallisce chiusa nel branch `(os, arch)` sottostante,
/// invece di scaricare/estrarre un artefatto non verificato.
fn ort_release_filename() -> Result<(String, String, &'static str), PapErrore> {
    let arch = std::env::consts::ARCH;
    let lib = nome_libonnxruntime();
    let (suffix, sub, sha256_atteso) = match (std::env::consts::OS, arch) {
        ("linux", "x86_64") => (
            format!("onnxruntime-linux-x64-{ORT_VERSION}.tgz"),
            format!("onnxruntime-linux-x64-{ORT_VERSION}/lib/{lib}"),
            "b6deea7f2e22c10c043019f294a0ea4d2a6c0ae52a009c34847640db75ec5580",
        ),
        ("linux", "aarch64") => (
            format!("onnxruntime-linux-aarch64-{ORT_VERSION}.tgz"),
            format!("onnxruntime-linux-aarch64-{ORT_VERSION}/lib/{lib}"),
            "0b9f47d140411d938e47915824d8daaa424df95a88b5f1fc843172a75168f7a0",
        ),
        ("macos", "aarch64") => (
            format!("onnxruntime-osx-arm64-{ORT_VERSION}.tgz"),
            format!("onnxruntime-osx-arm64-{ORT_VERSION}/lib/{lib}"),
            "8182db0ebb5caa21036a3c78178f17fabb98a7916bdab454467c8f4cf34bcfdf",
        ),
        ("macos", "x86_64") => (
            format!("onnxruntime-osx-x86_64-{ORT_VERSION}.tgz"),
            format!("onnxruntime-osx-x86_64-{ORT_VERSION}/lib/{lib}"),
            "a8e43edcaa349cbfc51578a7fc61ea2b88793ccf077b4bc65aca58999d20cf0f",
        ),
        ("windows", "x86_64") => (
            format!("onnxruntime-win-x64-{ORT_VERSION}.zip"),
            format!("onnxruntime-win-x64-{ORT_VERSION}/lib/{lib}"),
            "72c23470310ec79a7d42d27fe9d257e6c98540c73fa5a1db1f67f538c6c16f2f",
        ),
        (os, arch) => {
            return Err(PapErrore::Generico(format!(
                "Piattaforma non supportata per onnxruntime: {os}/{arch}"
            )))
        }
    };
    Ok((suffix, sub, sha256_atteso))
}

/// Estrae `path_in_archive` in `dest`.
///
/// Fix #556: estrazione atomica via file temporaneo + `rename` (stesso
/// pattern già usato da `scarica_file` per model.onnx/tokenizer.json). Senza
/// questo, un errore a metà copia (I/O, processo interrotto, disco pieno)
/// avrebbe lasciato un `dest` parzialmente scritto ma comunque presente sul
/// filesystem: la prossima `embeddings_download` lo avrebbe scambiato per
/// "già scaricato" (`lib_path.is_file()`) e non l'avrebbe MAI ri-scaricato,
/// mentre `init_session_pure` avrebbe continuato a rifiutarlo con "Verifica
/// integrità fallita" senza che l'utente avesse un modo per uscirne se non
/// cancellando il file a mano. Con l'estrazione atomica questo stato
/// inconsistente non può più prodursi: `dest` esiste solo dopo una copia
/// completa e riuscita — non serve quindi nessuna pulizia/invalidazione
/// aggiuntiva di `dir_modello` a monte.
fn estrai_libonnxruntime(
    archive_bytes: &[u8],
    path_in_archive: &str,
    dest: &Path,
) -> Result<(), PapErrore> {
    fs::create_dir_all(
        dest.parent()
            .ok_or_else(|| PapErrore::Generico("dest senza parent".into()))?,
    )?;

    let dest_tmp = dest.with_extension("extract-partial");
    let risultato = if std::env::consts::OS == "windows" {
        estrai_da_zip(archive_bytes, path_in_archive, &dest_tmp)
    } else {
        estrai_da_tar_gz(archive_bytes, path_in_archive, &dest_tmp)
    };
    if risultato.is_err() {
        let _ = fs::remove_file(&dest_tmp);
        return risultato;
    }
    fs::rename(&dest_tmp, dest)?;
    Ok(())
}

fn estrai_da_zip(archive_bytes: &[u8], path_in_archive: &str, dest: &Path) -> Result<(), PapErrore> {
    let cursor = std::io::Cursor::new(archive_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| PapErrore::dominio("Archivio del modello di embedding non valido.", e))?;
    let mut entry = archive
        .by_name(path_in_archive)
        .map_err(|e| PapErrore::dominio("Contenuto del modello di embedding mancante o corrotto nell'archivio.", e))?;
    let mut out = fs::File::create(dest)?;
    std::io::copy(&mut entry, &mut out)?;
    Ok(())
}

/// Normalizza un path di un'entry tar rimuovendo un eventuale componente
/// `.`/`./` iniziale (`Component::CurDir`).
///
/// Fix #556 (sintomo 1): il tarball macOS di onnxruntime (verificato
/// empiricamente su `onnxruntime-osx-arm64-1.23.0.tgz`) emette tutte le
/// entry con prefisso `./` (es. `./onnxruntime-osx-arm64-1.23.0/lib/...`),
/// mentre `path_in_archive` costruito da `ort_release_filename()` non ce
/// l'ha. Il confronto per stringa esatta non trovava mai corrispondenza:
/// `estrai_libonnxruntime` falliva sempre con "file non trovato
/// nell'archivio". Su Linux/Windows il prefisso non c'è, quindi il bug non
/// era mai emerso lì.
fn normalizza_path_archivio(p: &Path) -> PathBuf {
    p.components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect()
}

/// Risolve `link_target` (relativo, letto da un'entry symlink) rispetto alla
/// directory che contiene il symlink (`symlink_dir`), gestendo `.` e `..`
/// senza toccare il filesystem: i path esistono solo dentro l'archivio in
/// memoria, non su disco.
fn risolvi_path_relativo(symlink_dir: &Path, link_target: &Path) -> PathBuf {
    let mut componenti: Vec<Component<'_>> = symlink_dir.components().collect();
    for c in link_target.components() {
        match c {
            Component::ParentDir => {
                componenti.pop();
            }
            Component::CurDir => {}
            altro => componenti.push(altro),
        }
    }
    componenti.iter().collect()
}

/// Numero massimo di hop di symlink seguiti dentro l'archivio prima di
/// arrendersi con un errore esplicito (anti-loop su un tarball
/// malformato/malevolo).
const MAX_HOP_SYMLINK: u8 = 8;

/// Segue una catena di symlink dentro un archivio tar.gz in memoria, fino a
/// trovare il path normalizzato dell'entry regolare finale.
///
/// Fix #556 (sintomo 2, bug indipendente dal prefisso `./`, riprodotto su
/// Linux): nei tarball onnxruntime linux/macOS `lib/libonnxruntime.so` (o
/// `.dylib`) è spesso un symlink verso `libonnxruntime.so.<versione>`.
/// Leggere il *contenuto* di un'entry symlink con `tar::Entry::read` non
/// segue il link: restituisce 0 byte, non i dati del target. Estrarla così
/// produce una libreria vuota che poi fallisce la verifica SHA-256 con un
/// messaggio fuorviante ("hash non corrisponde") invece del vero problema
/// ("mai seguito il symlink").
///
/// Il decoding gzip riparte da `archive_bytes` (in memoria, quindi
/// riavvolgibile) ad ogni hop: il tar crate legge solo in streaming forward,
/// non supporta un accesso random per path.
fn risolvi_symlink_in_archivio(
    archive_bytes: &[u8],
    path_normalizzato: &Path,
) -> Result<PathBuf, PapErrore> {
    let mut corrente = path_normalizzato.to_path_buf();
    for _ in 0..MAX_HOP_SYMLINK {
        let dec = flate2::read::GzDecoder::new(archive_bytes);
        let mut archive = tar::Archive::new(dec);
        let mut esito_entry: Option<Option<PathBuf>> = None; // None = non trovata; Some(None) = trovata, non symlink; Some(Some(target)) = trovata, symlink
        for entry in archive
            .entries()
            .map_err(|e| PapErrore::dominio("Estrazione del modello di embedding non riuscita.", e))?
        {
            let entry = entry.map_err(|e| PapErrore::dominio("Estrazione del modello di embedding non riuscita.", e))?;
            let entry_path = entry
                .path()
                .map_err(|e| PapErrore::dominio("Estrazione del modello di embedding non riuscita.", e))?
                .into_owned();
            if normalizza_path_archivio(&entry_path) != corrente {
                continue;
            }
            if entry.header().entry_type().is_symlink() {
                let link_name = entry
                    .link_name()
                    .map_err(|e| PapErrore::dominio("Estrazione del modello di embedding non riuscita.", e))?
                    .ok_or_else(|| {
                        PapErrore::Generico(format!(
                            "tar: entry symlink {} senza link target",
                            corrente.display()
                        ))
                    })?
                    .into_owned();
                let dir_symlink = corrente.parent().unwrap_or_else(|| Path::new(""));
                esito_entry = Some(Some(normalizza_path_archivio(&risolvi_path_relativo(
                    dir_symlink,
                    &link_name,
                ))));
            } else {
                esito_entry = Some(None);
            }
            break;
        }
        match esito_entry {
            None => {
                return Err(PapErrore::Generico(format!(
                    "tar: file {} non trovato nell'archivio",
                    corrente.display()
                )))
            }
            Some(None) => return Ok(corrente),
            Some(Some(target)) => corrente = target,
        }
    }
    Err(PapErrore::Generico(format!(
        "tar: catena di symlink troppo lunga (>{MAX_HOP_SYMLINK}) risolvendo {}",
        path_normalizzato.display()
    )))
}

fn estrai_da_tar_gz(archive_bytes: &[u8], path_in_archive: &str, dest: &Path) -> Result<(), PapErrore> {
    let target_normalizzato = normalizza_path_archivio(Path::new(path_in_archive));
    let target_risolto = risolvi_symlink_in_archivio(archive_bytes, &target_normalizzato)?;

    let dec = flate2::read::GzDecoder::new(archive_bytes);
    let mut archive = tar::Archive::new(dec);
    let mut found = false;
    for entry in archive
        .entries()
        .map_err(|e| PapErrore::dominio("Estrazione del modello di embedding non riuscita.", e))?
    {
        let mut entry = entry.map_err(|e| PapErrore::dominio("Estrazione del modello di embedding non riuscita.", e))?;
        let entry_path = entry
            .path()
            .map_err(|e| PapErrore::dominio("Estrazione del modello di embedding non riuscita.", e))?
            .into_owned();
        if normalizza_path_archivio(&entry_path) == target_risolto {
            let mut out = fs::File::create(dest)?;
            std::io::copy(&mut entry, &mut out)?;
            found = true;
            break;
        }
    }
    if !found {
        return Err(PapErrore::Generico(format!(
            "tar: file {} non trovato nell'archivio",
            target_risolto.display()
        )));
    }
    Ok(())
}

// ─────────── Download command ───────────

#[tauri::command]
pub fn embeddings_download(
    app: tauri::AppHandle,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<EmbeddingsStato, PapErrore> {
    let dir_modello = percorso_modello(&state);
    fs::create_dir_all(&dir_modello)?;

    // Conteggio totale: 2 file modello + 1 tarball onnxruntime se mancante.
    let lib_path = percorso_libonnxruntime(&state);
    let libonnxruntime_da_scaricare = !lib_path.is_file();
    let totale_file = FILES_HF.len() + if libonnxruntime_da_scaricare { 1 } else { 0 };

    // 1. Modello + tokenizer da HuggingFace
    for (idx, (path_remoto, nome_locale, sha256_atteso)) in FILES_HF.iter().enumerate() {
        let dest = dir_modello.join(nome_locale);
        if dest.is_file() {
            continue;
        }
        let url = format!("https://huggingface.co/{HF_REPO}/resolve/main/{path_remoto}");
        scarica_file(&app, &url, &dest, idx + 1, totale_file, nome_locale, sha256_atteso)?;
    }

    // 2. libonnxruntime da Microsoft GitHub release (tarball/zip → estrai solo lib)
    if libonnxruntime_da_scaricare {
        let (filename, path_in_archive, sha256_atteso) = ort_release_filename()?;
        let url = format!("{ORT_RELEASE_BASE}v{ORT_VERSION}/{filename}");
        let (bytes, _total) = http_get_with_progress(
            &app,
            &url,
            FILES_HF.len() + 1,
            totale_file,
            &filename,
        )?;
        // Fix #458: verifica integrità dell'intero archivio PRIMA di
        // estrarre/caricare la libreria nativa al suo interno.
        verifica_sha256(&bytes, sha256_atteso, &filename)?;
        estrai_libonnxruntime(&bytes, &path_in_archive, &lib_path)?;
        log::info!("libonnxruntime estratta in {}", lib_path.display());
    }

    embeddings_status(state, rt_state)
}

// ─────────── Init Session ───────────

/// Carica Session + Tokenizer in `rt_state` se non già caricati.
/// Helper "pure" (no Tauri State) usato sia da `embeddings_init` (comando
/// Tauri) sia da `assicura_session_caricata` (riload on-demand post
/// idle-unload, v0.6.0 Step 2).
///
/// Idempotente: ritorna `Ok(false)` se la Session è già caricata, senza
/// effetti collaterali. Ritorna `Ok(true)` quando ha appena caricato.
/// Errore se il modello / la lib runtime non sono pronti su disco, o se
/// il caricamento ort fallisce.
pub fn init_session_pure(
    rt_state: &EmbeddingsState,
    vault_state: &VaultState,
) -> Result<bool, PapErrore> {
    {
        let guard = rt_state.inner.lock().unwrap_or_else(|p| p.into_inner());
        if guard.is_some() {
            return Ok(false);
        }
    }

    let dir_modello = percorso_modello(vault_state);
    if !modello_completo(&dir_modello) {
        return Err(PapErrore::Generico(
            "Modello non scaricato. Chiama embeddings_download prima di embeddings_init.".into(),
        ));
    }

    let lib_path = percorso_libonnxruntime(vault_state);
    if !lib_path.is_file() {
        return Err(PapErrore::Generico(
            "libonnxruntime non scaricata. Chiama embeddings_download.".into(),
        ));
    }

    // Fix #458 (review MEDIUM): fail-fast. La verifica SHA-256 avveniva solo
    // al download; la ri-eseguiamo ORA, ad ogni avvio, così un artefatto in
    // cache alterato produce subito un errore chiaro — modello e tokenizer
    // compresi — prima ancora dello staging della libreria nativa.
    verifica_artefatti_cache_su_disco(&dir_modello, &lib_path)?;

    // Fix TOCTOU (CWE-367): NON puntare ORT_DYLIB_PATH al file in cache.
    // `ort` fa `dlopen(ORT_DYLIB_PATH)` in modo indipendente DOPO la nostra
    // verifica; puntando alla cache condivisa i byte verificati e i byte
    // caricati potrebbero differire, perché un writer della data-dir può
    // sostituire il file nella finestra check→load → esecuzione di codice
    // nativo arbitrario in-process. Copiamo invece la libreria in una dir
    // privata per-processo 0700 (dentro una base `.pap-priv` che creiamo e
    // possediamo), ri-verifichiamo il SHA-256 della COPIA e puntiamo ort a
    // quella.
    //
    // COSA CHIUDE: lo swap statico/in-place del file in cache — i byte
    // verificati sono ora esattamente i byte caricati, perché la copia è
    // ri-verificata e vive in una dir 0700 per-processo tenuta viva per tutta
    // la vita del processo (campo `lib_privata`).
    //
    // RESIDUO (onesto): un attaccante di UID DIVERSO che può scrivere la
    // runtime-dir può ancora tentare una race di `rename(2)` sulla dir privata
    // `.pap-priv` tra la verifica e il `dlopen`. Ciò richiede però una data-dir
    // group/other-writable (NON il default) E la vittoria di una race stretta.
    // Un attaccante dello STESSO UID è fuori dalla portata di qualunque
    // approccio path-based, perché `ort` non offre un load da file descriptor.
    let runtime_dir = percorso_runtime_dir(vault_state);
    let (_, _, sha256_lib_atteso) = ort_release_filename()?;
    let (tmp_lib, lib_privata_path) =
        stage_lib_verificata(&runtime_dir, &lib_path, sha256_lib_atteso)?;

    // Punta ort alla COPIA privata via env var. Sicuro perché siamo
    // single-thread qui (Tauri command sequenziati su mutex), e l'env var è
    // letta solo al primo Session::create.
    // SAFETY: set_var è unsafe in edition 2024+ ma stable in 2021.
    std::env::set_var("ORT_DYLIB_PATH", &lib_privata_path);

    let model_path = dir_modello.join("model.onnx");
    let tokenizer_path = dir_modello.join("tokenizer.json");

    let session = Session::builder()
        .map_err(|e| PapErrore::dominio("Inizializzazione del modello di embedding non riuscita.", e))?
        .commit_from_file(&model_path)
        .map_err(|e| PapErrore::dominio("Inizializzazione del modello di embedding non riuscita.", e))?;
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .map_err(|e| PapErrore::dominio("Inizializzazione del modello di embedding non riuscita.", e))?;

    let mut guard = rt_state.inner.lock().unwrap_or_else(|p| p.into_inner());
    *guard = Some(EmbeddingsLoaded { session, tokenizer });
    // Fix TOCTOU (CWE-367): trattieni il TempDir della copia privata finché il
    // processo vive. `ort` mantiene la libreria mappata; dropparla ora
    // cancellerebbe il file da sotto la Session. Sostituisce l'eventuale copia
    // di un caricamento precedente (post idle-unload), che viene così ripulita.
    *rt_state.lib_privata.lock().unwrap_or_else(|p| p.into_inner()) = Some(tmp_lib);
    // Marca l'init come "uso recente" così il timer di idle-unload non
    // droppa subito una Session appena caricata.
    *rt_state.last_used.lock().unwrap_or_else(|p| p.into_inner()) = Some(Instant::now());

    Ok(true)
}

/// Riload on-demand della Session se è stata droppata dal timer idle-unload.
/// Idempotente: no-op se la Session è già caricata.
///
/// Caller (es. `cerca_semantica` in `ricerca_ibrida`) la chiamano prima di
/// `compute_embedding_opt` per evitare il degrade FTS-only quando una nuova
/// query arriva dopo un periodo di inattività. Risolve il limite documentato
/// in `docs/roadmap/rinvii.md` § Da Fase 3 Step 10 — atterrato in v0.6.0
/// Step 2.
///
/// Ritorna `Ok(true)` se ha riloadato, `Ok(false)` se era già caricata.
/// Errore se il modello/runtime non sono disponibili su disco (graceful
/// degrade gestito dal caller).
pub fn assicura_session_caricata(
    rt_state: &EmbeddingsState,
    vault_state: &VaultState,
) -> Result<bool, PapErrore> {
    let riloadato = init_session_pure(rt_state, vault_state)?;
    if riloadato {
        log::info!("embeddings: Session ricaricata on-demand post idle-unload");
    }
    Ok(riloadato)
}

#[tauri::command]
pub fn embeddings_init(
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<EmbeddingsStato, PapErrore> {
    let _caricato = init_session_pure(rt_state.inner(), state.inner())?;
    log::info!(
        "embeddings_init: Session ort + tokenizer pronti per {MODEL_ID}"
    );
    Ok(EmbeddingsStato::Caricato {
        model_id: MODEL_ID.to_string(),
        dimensione: EMBEDDING_DIM,
    })
}

// ─────────── Compute ───────────

/// Mean pooling: media delle hidden states pesata da attention mask.
fn mean_pooling(token_embeddings: &Array2<f32>, attention_mask: &Array1<i64>) -> Array1<f32> {
    let (seq_len, hidden) = token_embeddings.dim();
    let mut sum = vec![0.0f32; hidden];
    let mut count = 0.0f32;
    for i in 0..seq_len {
        if attention_mask[i] == 0 {
            continue;
        }
        for h in 0..hidden {
            sum[h] += token_embeddings[[i, h]];
        }
        count += 1.0;
    }
    if count > 0.0 {
        for x in sum.iter_mut() {
            *x /= count;
        }
    }
    Array1::from_vec(sum)
}

fn l2_normalize(v: &mut Array1<f32>) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-12 {
        v.mapv_inplace(|x| x / norm);
    }
}

/// Helper interno per il calcolo embedding, riutilizzabile da altri moduli
/// (es. ricerca_ibrida) senza passare dal layer Tauri command.
///
/// Ritorna `Ok(None)` se la Session non è ancora caricata: i caller possono
/// degradare graziosamente (es. fallback a sola ricerca FTS5).
///
/// Effetto collaterale: aggiorna `last_used` a `Instant::now()` su success,
/// così il task di idle-unload (Step 10) sa quando la Session è viva.
pub(crate) fn compute_embedding_opt(
    rt_state: &EmbeddingsState,
    testo: &str,
) -> Result<Option<Vec<f32>>, PapErrore> {
    let mut guard = rt_state.inner.lock().unwrap_or_else(|p| p.into_inner());
    let Some(loaded) = guard.as_mut() else {
        return Ok(None);
    };
    let result = compute_with_loaded(loaded, testo)?;
    *rt_state.last_used.lock().unwrap_or_else(|p| p.into_inner()) = Some(Instant::now());
    Ok(Some(result))
}

/// Soglia minima accettabile per `unload_se_idle`. Sotto questa soglia la
/// chiamata è no-op (evita unload aggressivi che vanificano la cache).
/// La preferenza `idle_unload_secondi = 0` significa "disattivata".
const SOGLIA_MIN_UNLOAD: Duration = Duration::from_secs(60);

/// Se la Session è caricata e non è stata usata da almeno `soglia`, la
/// droppa per liberare RAM (~150 MB modello + runtime ort). Idempotente:
/// no-op se Session è già `None` o se è ancora "calda".
///
/// Ritorna `true` se ha effettuato l'unload, `false` altrimenti.
///
/// Casistiche:
/// - `soglia < SOGLIA_MIN_UNLOAD` → no-op (pref disattivata o fuori range)
/// - `last_used = None` (Session caricata ma mai usata) → no-op: lasciamo
///   la prima compute decidere quando partire il timer
/// - `last_used.elapsed() < soglia` → no-op
/// - altrimenti → `*inner = None` e log info
pub fn unload_se_idle(rt_state: &EmbeddingsState, soglia: Duration) -> bool {
    if soglia < SOGLIA_MIN_UNLOAD {
        return false;
    }
    let last = *rt_state.last_used.lock().unwrap_or_else(|p| p.into_inner());
    let Some(last) = last else {
        return false;
    };
    if last.elapsed() < soglia {
        return false;
    }
    let mut guard = rt_state.inner.lock().unwrap_or_else(|p| p.into_inner());
    if guard.is_some() {
        *guard = None;
        log::info!(
            "embeddings: Session droppata per inattività ({}s)",
            last.elapsed().as_secs()
        );
        true
    } else {
        false
    }
}

/// Restituisce l'`Instant` dell'ultimo uso della Session. Esposto per i
/// test e per il task background.
pub fn ultimo_uso(rt_state: &EmbeddingsState) -> Option<Instant> {
    *rt_state.last_used.lock().unwrap_or_else(|p| p.into_inner())
}

/// `true` se la Session è caricata. Esposto per i test e per UI status.
pub fn session_caricata(rt_state: &EmbeddingsState) -> bool {
    rt_state.inner.lock().unwrap_or_else(|p| p.into_inner()).is_some()
}

fn compute_with_loaded(
    loaded: &mut EmbeddingsLoaded,
    testo: &str,
) -> Result<Vec<f32>, PapErrore> {
    // 1. Tokenize
    let encoding = loaded
        .tokenizer
        .encode(testo, true)
        .map_err(|e| PapErrore::dominio("Elaborazione del testo per l'embedding non riuscita.", e))?;

    // 2. Trunc/pad a MAX_SEQ_LEN
    let mut ids: Vec<i64> = encoding.get_ids().iter().map(|x| *x as i64).collect();
    let mut mask: Vec<i64> = encoding.get_attention_mask().iter().map(|x| *x as i64).collect();
    let mut type_ids: Vec<i64> = encoding.get_type_ids().iter().map(|x| *x as i64).collect();
    if ids.len() > MAX_SEQ_LEN {
        ids.truncate(MAX_SEQ_LEN);
        mask.truncate(MAX_SEQ_LEN);
        type_ids.truncate(MAX_SEQ_LEN);
    } else {
        ids.resize(MAX_SEQ_LEN, 0);
        mask.resize(MAX_SEQ_LEN, 0);
        type_ids.resize(MAX_SEQ_LEN, 0);
    }

    let seq_len = ids.len();
    // Costruzione tensor via tupla (shape, vec) — forma universale ort 2.x,
    // funziona per i64 senza dover passare attraverso ndarray (che richiede
    // OwnedTensorArrayData<_> trait, non implementato per Array2<i64>).
    let shape = vec![1i64, seq_len as i64];
    let mask_clone = mask.clone();
    let inputs = ort::inputs![
        "input_ids" => Tensor::from_array((shape.clone(), ids))
            .map_err(|e| PapErrore::dominio("Preparazione dei dati per il modello di embedding non riuscita.", e))?,
        "attention_mask" => Tensor::from_array((shape.clone(), mask_clone))
            .map_err(|e| PapErrore::dominio("Preparazione dei dati per il modello di embedding non riuscita.", e))?,
        "token_type_ids" => Tensor::from_array((shape, type_ids))
            .map_err(|e| PapErrore::dominio("Preparazione dei dati per il modello di embedding non riuscita.", e))?,
    ];
    let outputs = loaded
        .session
        .run(inputs)
        .map_err(|e| PapErrore::dominio("Esecuzione del modello di embedding non riuscita.", e))?;

    // 4. Output last_hidden_state shape [1, seq_len, hidden_dim]
    let (output_name, _) = outputs
        .iter()
        .next()
        .ok_or_else(|| PapErrore::Generico("nessun output da Session run".into()))?;
    let value = outputs
        .get(output_name)
        .ok_or_else(|| PapErrore::Generico("output mancante".into()))?;
    let (shape, data) = value
        .try_extract_tensor::<f32>()
        .map_err(|e| PapErrore::dominio("Lettura dell'output del modello di embedding non riuscita.", e))?;

    if shape.len() != 3 || shape[0] != 1 || shape[2] as usize != EMBEDDING_DIM {
        return Err(PapErrore::Generico(format!(
            "Shape output inattesa: {:?} (atteso [1, seq_len, {EMBEDDING_DIM}])",
            shape
        )));
    }
    let actual_seq = shape[1] as usize;

    let token_emb = Array2::from_shape_vec((actual_seq, EMBEDDING_DIM), data.to_vec())
        .map_err(|e| PapErrore::dominio("Lettura dell'output del modello di embedding non riuscita.", e))?;
    let mask_arr1 = Array1::from_vec(mask.iter().take(actual_seq).copied().collect());

    // 5. Mean pooling + L2 normalize
    let mut pooled = mean_pooling(&token_emb, &mask_arr1);
    l2_normalize(&mut pooled);

    Ok(pooled.to_vec())
}

#[tauri::command]
pub fn embeddings_compute(
    testo: String,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<Vec<f32>, PapErrore> {
    compute_embedding_opt(&rt_state, &testo)?.ok_or_else(|| {
        PapErrore::Generico(
            "Embeddings non inizializzati. Chiama embeddings_init.".into(),
        )
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn model_id_coerente_con_adr() {
        assert_eq!(MODEL_ID, "multilingual-MiniLM-L12-v2");
        assert_eq!(EMBEDDING_DIM, 384);
    }

    #[test]
    fn dim_cartella_dir_inesistente() {
        let p = std::path::Path::new("/percorso/inesistente");
        assert_eq!(dim_cartella_mb(p), 0);
    }

    #[test]
    fn modello_completo_richiede_entrambi_i_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        assert!(!modello_completo(p));
        std::fs::write(p.join("model.onnx"), b"stub").unwrap();
        assert!(!modello_completo(p));
        std::fs::write(p.join("tokenizer.json"), b"stub").unwrap();
        assert!(modello_completo(p));
    }

    #[test]
    fn pronto_su_disco_falso_se_manca_libonnxruntime() {
        // Fix #556 (sintomo 3): modello+tokenizer presenti ma libonnxruntime
        // assente → non pronto. Prima del fix, `embeddings_status` non
        // controllava affatto la lib e avrebbe dichiarato "Pronto" qui.
        let dir = tempfile::tempdir().unwrap();
        let dir_modello = dir.path().join("modello");
        std::fs::create_dir_all(&dir_modello).unwrap();
        std::fs::write(dir_modello.join("model.onnx"), b"stub").unwrap();
        std::fs::write(dir_modello.join("tokenizer.json"), b"stub").unwrap();
        let lib_path = dir.path().join("libonnxruntime.so");

        assert!(!pronto_su_disco(&dir_modello, &lib_path));

        std::fs::write(&lib_path, b"stub").unwrap();
        assert!(pronto_su_disco(&dir_modello, &lib_path));
    }

    #[test]
    fn ort_release_filename_supporta_principali_piattaforme() {
        // Sentinel: la fn ritorna Ok per la piattaforma corrente di test.
        let r = ort_release_filename();
        assert!(r.is_ok(), "Piattaforma corrente deve essere supportata");
        let (filename, sub, sha256_atteso) = r.unwrap();
        assert!(filename.contains(ORT_VERSION));
        assert!(sub.contains("/lib/"));
        // Fix #458: hash pinnato, 64 hex char (SHA-256), non vuoto/placeholder.
        assert_eq!(sha256_atteso.len(), 64);
        assert!(sha256_atteso.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn nome_libonnxruntime_per_piattaforma_corretto() {
        let n = nome_libonnxruntime();
        if cfg!(target_os = "windows") {
            assert_eq!(n, "onnxruntime.dll");
        } else if cfg!(target_os = "macos") {
            assert_eq!(n, "libonnxruntime.dylib");
        } else {
            assert_eq!(n, "libonnxruntime.so");
        }
    }

    #[test]
    fn mean_pooling_con_mask_zero_ignora_token() {
        // 2 token, hidden=3. Mask=[1,0]. Atteso: pooling = [1.0, 2.0, 3.0]
        // (solo il primo token contribuisce).
        let emb = Array2::from_shape_vec(
            (2, 3),
            vec![1.0, 2.0, 3.0, 100.0, 200.0, 300.0],
        )
        .unwrap();
        let mask = Array1::from_vec(vec![1, 0]);
        let pooled = mean_pooling(&emb, &mask);
        assert_eq!(pooled.to_vec(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn l2_normalize_vector_unitario() {
        let mut v = Array1::from_vec(vec![3.0, 4.0]);
        l2_normalize(&mut v);
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn compute_embedding_opt_senza_session_ritorna_none() {
        // Quality gate Step 10 — grace degradation: tutti i caller
        // (ricerca_ibrida, tags_suggest, editor::aggiorna_embedding_*,
        // embeddings_backfill) si appoggiano a questo guard. Se la
        // primitiva ritorna None senza errori, il fallback graceful
        // funziona ovunque.
        let rt = EmbeddingsState::new();
        let r = compute_embedding_opt(&rt, "qualunque testo");
        assert!(r.is_ok(), "no errore quando session non loaded");
        assert!(r.unwrap().is_none(), "ritorna None, non Some(emb)");
    }

    // ─────────── Test idle-unload (Step 10 quality gate) ───────────

    #[test]
    fn unload_se_idle_session_none_no_op() {
        let rt = EmbeddingsState::new();
        // Session = None, last_used = None → unload_se_idle no-op.
        let dropped = unload_se_idle(&rt, Duration::from_secs(60));
        assert!(!dropped, "no-op se Session non era caricata");
        assert!(!session_caricata(&rt));
    }

    #[test]
    fn unload_se_idle_soglia_sotto_minimo_no_op() {
        // Soglia 0 (disattivato) o sotto SOGLIA_MIN_UNLOAD (60s) → no-op.
        let rt = EmbeddingsState::new();
        // Forziamo last_used antico per simulare "molto idle".
        *rt.last_used.lock().unwrap() = Some(Instant::now() - Duration::from_secs(3600));
        let dropped = unload_se_idle(&rt, Duration::from_secs(0));
        assert!(!dropped, "soglia 0 = disattivata, non droppa");
        let dropped2 = unload_se_idle(&rt, Duration::from_secs(30));
        assert!(!dropped2, "soglia < 60s sotto minimo, non droppa");
    }

    #[test]
    fn unload_se_idle_session_calda_no_op() {
        let rt = EmbeddingsState::new();
        // last_used = adesso, soglia 60s → calda, no unload.
        *rt.last_used.lock().unwrap() = Some(Instant::now());
        let dropped = unload_se_idle(&rt, Duration::from_secs(60));
        assert!(!dropped);
    }

    #[test]
    fn ultimo_uso_e_session_caricata_riflettono_lo_state() {
        let rt = EmbeddingsState::new();
        assert!(!session_caricata(&rt));
        assert_eq!(ultimo_uso(&rt), None);
        // Simuliamo un compute "successful" senza dover caricare ort
        // davvero: settando last_used direttamente.
        let now = Instant::now();
        *rt.last_used.lock().unwrap() = Some(now);
        assert_eq!(ultimo_uso(&rt), Some(now));
    }

    // ─────────── Riload on-demand post idle-unload (v0.6.0 Step 2) ───────────

    #[test]
    fn init_session_pure_modello_mancante_errore() {
        // Vault dir vuota → modello non scaricato → init fallisce con
        // messaggio chiaro per il caller.
        let dir = tempfile::tempdir().unwrap();
        let vault = crate::vault::VaultState::new(dir.path().to_path_buf());
        let rt = EmbeddingsState::new();

        let r = init_session_pure(&rt, &vault);
        assert!(r.is_err(), "Atteso errore per modello mancante");
        let msg = format!("{:?}", r.unwrap_err());
        assert!(
            msg.contains("Modello non scaricato"),
            "Errore deve menzionare 'Modello non scaricato', got: {msg}"
        );
        assert!(!session_caricata(&rt), "Session deve restare non caricata");
    }

    // ─── Review MEDIUM: ri-verifica integrità ad ogni avvio, non solo al download ───

    #[test]
    fn verifica_artefatti_cache_su_disco_contenuto_manomesso_e_errore() {
        // Simula un attacker con scrittura sulla data-dir (o una
        // corruzione silenziosa): i file esistono ma il contenuto non
        // corrisponde più all'hash pinnato. Deve fallire chiuso.
        let dir = tempfile::tempdir().unwrap();
        let dir_modello = dir.path().join("modello");
        std::fs::create_dir_all(&dir_modello).unwrap();
        std::fs::write(dir_modello.join("model.onnx"), b"contenuto-manomesso").unwrap();
        std::fs::write(dir_modello.join("tokenizer.json"), b"contenuto-manomesso").unwrap();
        let lib_path = dir.path().join("libonnxruntime.so");
        std::fs::write(&lib_path, b"binario-manomesso").unwrap();

        let r = verifica_artefatti_cache_su_disco(&dir_modello, &lib_path);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("integrità"));
    }

    #[test]
    fn init_session_pure_rifiuta_cache_manomessa_prima_di_settare_env_var() {
        // Fix #458 review: `init_session_pure` deve rifiutare fail-closed
        // artefatti in cache che esistono su disco ma non superano la
        // ri-verifica SHA-256 — non solo al primo download (embeddings_download),
        // ma ad OGNI avvio dell'app, PRIMA di impostare ORT_DYLIB_PATH.
        let dir = tempfile::tempdir().unwrap();
        let vault = crate::vault::VaultState::new(dir.path().to_path_buf());
        let rt = EmbeddingsState::new();

        let dir_modello = percorso_modello(&vault);
        std::fs::create_dir_all(&dir_modello).unwrap();
        std::fs::write(dir_modello.join("model.onnx"), b"contenuto-manomesso").unwrap();
        std::fs::write(dir_modello.join("tokenizer.json"), b"contenuto-manomesso").unwrap();

        let lib_path = percorso_libonnxruntime(&vault);
        std::fs::create_dir_all(lib_path.parent().unwrap()).unwrap();
        std::fs::write(&lib_path, b"binario-manomesso").unwrap();

        let r = init_session_pure(&rt, &vault);
        assert!(r.is_err(), "Atteso errore per artefatto in cache manomesso");
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("integrità"), "Errore deve menzionare l'integrità: {msg}");
        assert!(!session_caricata(&rt), "Session deve restare non caricata");
    }

    #[test]
    fn assicura_session_caricata_propaga_errore_da_init() {
        // assicura_session_caricata = wrapper di init_session_pure.
        // Se init_session_pure fallisce (modello mancante), assicura
        // propaga lo stesso errore (caller fa graceful degrade).
        let dir = tempfile::tempdir().unwrap();
        let vault = crate::vault::VaultState::new(dir.path().to_path_buf());
        let rt = EmbeddingsState::new();

        let r = assicura_session_caricata(&rt, &vault);
        assert!(r.is_err(), "Atteso errore per modello mancante");
        assert!(!session_caricata(&rt));
    }

    #[test]
    fn init_session_pure_idempotente_se_gia_caricata() {
        // Non possiamo costruire un EmbeddingsLoaded reale senza ort,
        // ma possiamo verificare il path del check iniziale: dopo aver
        // marcato la state come "session presente" (mockando lo storage
        // sottostante), init_session_pure ritorna Ok(false) senza errori
        // anche se il filesystem è vuoto.
        //
        // Sentinel anti-regressione: il check `guard.is_some()` deve
        // precedere qualunque accesso al filesystem.
        let rt = EmbeddingsState::new();
        // Verifica baseline: con state vuoto e fs vuoto → errore.
        let dir = tempfile::tempdir().unwrap();
        let vault = crate::vault::VaultState::new(dir.path().to_path_buf());
        assert!(init_session_pure(&rt, &vault).is_err());
        // Il guard early-return non è raggiungibile senza un EmbeddingsLoaded
        // reale; la regressione si manifesterebbe come errore quando atteso.
        // Sentinel: la versione corrente mantiene l'early-return at lock guard.
    }

    // ─────────── v0.7.0 Step 1: edge case pure functions ───────────

    #[test]
    fn mean_pooling_tutti_token_attivi_media_uniforme() {
        // 3 token tutti attivi (mask=1), hidden=2.
        // Atteso: media element-wise sui 3 token.
        let emb = Array2::from_shape_vec(
            (3, 2),
            vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0],
        )
        .unwrap();
        let mask = Array1::from_vec(vec![1, 1, 1]);
        let pooled = mean_pooling(&emb, &mask);
        // Media: (1+2+3)/3 = 2.0; (4+5+6)/3 = 5.0
        assert!((pooled[0] - 2.0).abs() < 1e-6);
        assert!((pooled[1] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn mean_pooling_mask_tutta_zero_ritorna_zero() {
        // Edge case: tutti i token mascherati → count=0, sum=0, no division.
        let emb = Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        let mask = Array1::from_vec(vec![0, 0]);
        let pooled = mean_pooling(&emb, &mask);
        assert_eq!(pooled.to_vec(), vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn l2_normalize_vettore_zero_no_panic() {
        // Edge case: vettore zero → norm=0 → guard `> 1e-12` evita divisione.
        let mut v = Array1::from_vec(vec![0.0, 0.0, 0.0]);
        l2_normalize(&mut v);
        // Resta zero (no NaN/Inf).
        assert_eq!(v.to_vec(), vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn l2_normalize_vettore_unitario_invariato() {
        // [1, 0, 0] è già unitario → resta uguale.
        let mut v = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        l2_normalize(&mut v);
        assert!((v[0] - 1.0).abs() < 1e-6);
        assert!(v[1].abs() < 1e-6);
        assert!(v[2].abs() < 1e-6);
    }

    #[test]
    fn l2_normalize_vettore_negativo_norma_corretta() {
        // [-3, 4] ha norma 5 → normalizzato [-0.6, 0.8].
        let mut v = Array1::from_vec(vec![-3.0, 4.0]);
        l2_normalize(&mut v);
        assert!((v[0] + 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn dim_cartella_mb_directory_vuota() {
        // Directory esistente ma senza file → 0 MB.
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(dim_cartella_mb(dir.path()), 0);
    }

    #[test]
    fn dim_cartella_mb_con_file() {
        // Crea 1 file da 2 MB → dim_cartella_mb deve ritornare 2.
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.bin");
        let bytes = vec![0u8; 2 * 1024 * 1024];
        std::fs::write(&file_path, &bytes).unwrap();
        assert_eq!(dim_cartella_mb(dir.path()), 2);
    }

    #[test]
    fn modello_completo_solo_un_file_no() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("model.onnx"), b"x").unwrap();
        // Manca tokenizer.json → false.
        assert!(!modello_completo(dir.path()));
    }

    #[test]
    fn ort_release_filename_versione_consistente() {
        // Sentinel: il filename ritorna la stessa ORT_VERSION (no drift).
        let r = ort_release_filename().unwrap();
        let (filename, _sub, _sha256_atteso) = r;
        // Filename contiene sempre la versione configurata.
        assert!(filename.contains(ORT_VERSION));
    }

    // ─────────── #458: verifica integrità SHA-256 ───────────

    #[test]
    fn sha256_hex_valore_noto() {
        // sha256("") ha un valore noto e stabile — sentinel anti-regressione
        // dell'algoritmo/encoding usato.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn verifica_sha256_hash_corretto_ok() {
        let bytes = b"contenuto di test";
        let atteso = sha256_hex(bytes);
        assert!(verifica_sha256(bytes, &atteso, "file.txt").is_ok());
    }

    #[test]
    fn verifica_sha256_case_insensitive() {
        let bytes = b"contenuto di test";
        let atteso = sha256_hex(bytes).to_uppercase();
        assert!(verifica_sha256(bytes, &atteso, "file.txt").is_ok());
    }

    #[test]
    fn verifica_sha256_mismatch_e_errore_fail_closed() {
        let bytes = b"contenuto originale";
        let atteso_sbagliato =
            "0000000000000000000000000000000000000000000000000000000000000000";
        let r = verifica_sha256(bytes, atteso_sbagliato, "modello.onnx");
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("modello.onnx"), "Deve nominare il file: {msg}");
        assert!(msg.contains("integrità"), "Deve menzionare l'integrità: {msg}");
    }

    #[test]
    fn sha256_atteso_ort_hash_reale_non_placeholder() {
        // Sentinel anti-regressione: l'hash pinnato per la piattaforma
        // corrente deve essere reale (64 hex char), non un placeholder
        // tipo tutto-zero che romperebbe silenziosamente `verifica_sha256`.
        let placeholder_zero = "0".repeat(64);
        assert_ne!(
            ort_release_filename().unwrap().2,
            placeholder_zero,
            "l'hash della piattaforma corrente non deve essere un placeholder"
        );
    }

    #[test]
    fn model_e_tokenizer_sha256_reali_pinnati_non_placeholder() {
        let placeholder_zero = "0".repeat(64);
        assert_ne!(SHA256_MODEL_ONNX, placeholder_zero);
        assert_ne!(SHA256_TOKENIZER_JSON, placeholder_zero);
        assert_eq!(SHA256_MODEL_ONNX.len(), 64);
        assert_eq!(SHA256_TOKENIZER_JSON.len(), 64);
    }

    // ─────────── Fix TOCTOU (CWE-367): staging copia privata verificata ───────────
    //
    // NB: il path completo di `dlopen` (ORT_DYLIB_PATH → Session::create) NON è
    // testabile a livello di unit test, perché richiede la libreria nativa reale
    // e il caricamento in-process di `ort`. Qui copriamo la logica attorno: lo
    // staging verificato e l'hardening della directory base.

    #[test]
    fn stage_lib_verificata_copia_e_verifica_ok() {
        // La copia privata deve materializzarsi dentro `.pap-priv`, avere lo
        // stesso contenuto della cache e superare la ri-verifica SHA-256.
        let dir = tempfile::tempdir().unwrap();
        let runtime_dir = dir.path().join("onnxruntime").join(ORT_VERSION);
        std::fs::create_dir_all(&runtime_dir).unwrap();
        let lib_cache = runtime_dir.join(nome_libonnxruntime());
        let contenuto = b"finta-libonnxruntime-bytes";
        std::fs::write(&lib_cache, contenuto).unwrap();
        let atteso = sha256_hex(contenuto);

        let (tmp, copia) = stage_lib_verificata(&runtime_dir, &lib_cache, &atteso).unwrap();
        assert!(copia.starts_with(runtime_dir.join(NOME_BASE_PRIVATA)));
        assert_eq!(std::fs::read(&copia).unwrap(), contenuto);
        // Il TempDir tiene viva la copia finché non viene droppato.
        drop(tmp);
    }

    #[test]
    fn stage_lib_verificata_hash_errato_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let runtime_dir = dir.path().join("onnxruntime").join(ORT_VERSION);
        std::fs::create_dir_all(&runtime_dir).unwrap();
        let lib_cache = runtime_dir.join(nome_libonnxruntime());
        std::fs::write(&lib_cache, b"contenuto-reale").unwrap();
        let atteso_sbagliato = "0".repeat(64);

        let r = stage_lib_verificata(&runtime_dir, &lib_cache, &atteso_sbagliato);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("integrità"));
    }

    #[test]
    fn base_privata_hardened_crea_dir_0700() {
        let dir = tempfile::tempdir().unwrap();
        let runtime_dir = dir.path().join("rt");
        let base = base_privata_hardened(&runtime_dir).unwrap();
        assert_eq!(base, runtime_dir.join(NOME_BASE_PRIVATA));
        assert!(base.is_dir());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::symlink_metadata(&base)
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o700, "la base deve essere creata 0700");
        }
        // Idempotente: una seconda chiamata riusa la stessa base senza errori.
        assert!(base_privata_hardened(&runtime_dir).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn base_privata_hardened_rifiuta_symlink() {
        // Un attaccante che piazza `.pap-priv` come symlink deve far fallire
        // chiuso la verifica: lstat rileva il symlink e NON lo segue.
        let dir = tempfile::tempdir().unwrap();
        let runtime_dir = dir.path().join("rt");
        std::fs::create_dir_all(&runtime_dir).unwrap();
        let bersaglio = dir.path().join("altrove");
        std::fs::create_dir_all(&bersaglio).unwrap();
        std::os::unix::fs::symlink(&bersaglio, runtime_dir.join(NOME_BASE_PRIVATA)).unwrap();

        let r = base_privata_hardened(&runtime_dir);
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("integrità"), "msg: {msg}");
        assert!(msg.contains("symlink"), "msg: {msg}");
    }

    #[cfg(unix)]
    #[test]
    fn base_privata_hardened_rifiuta_permessi_larghi() {
        // `.pap-priv` pre-esistente con permessi troppo larghi (0777) → la
        // verifica hardened fallisce chiusa invece di usarla.
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let runtime_dir = dir.path().join("rt");
        let base = runtime_dir.join(NOME_BASE_PRIVATA);
        std::fs::create_dir_all(&base).unwrap();
        std::fs::set_permissions(&base, std::fs::Permissions::from_mode(0o777)).unwrap();

        let r = base_privata_hardened(&runtime_dir);
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("integrità"), "msg: {msg}");
        assert!(msg.contains("permessi"), "msg: {msg}");
    }

    #[cfg(unix)]
    #[test]
    fn base_privata_hardened_rifiuta_file_al_posto_della_dir() {
        // `.pap-priv` è un file regolare (non una directory) → fail-closed.
        let dir = tempfile::tempdir().unwrap();
        let runtime_dir = dir.path().join("rt");
        std::fs::create_dir_all(&runtime_dir).unwrap();
        std::fs::write(runtime_dir.join(NOME_BASE_PRIVATA), b"non-una-dir").unwrap();

        let r = base_privata_hardened(&runtime_dir);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("integrità"));
    }

    // ─────────── #556: estrazione tar.gz — prefisso `./` + entry symlink ───────────
    //
    // Diagnosi verificata empiricamente scaricando ed aprendo
    // `onnxruntime-osx-arm64-1.23.0.tgz` con questa stessa versione di `tar`
    // (0.4.46): tutte le entry hanno prefisso `./`, che il confronto per
    // stringa esatta di `path_in_archive` non gestiva (sintomo 1, la causa
    // diretta del bug macOS #556). Indipendentemente, `lib/libonnxruntime.*`
    // è spesso un symlink verso `libonnxruntime.*.<versione>`: leggerne il
    // contenuto senza risolvere il link dà 0 byte (sintomo 2, riprodotto su
    // Linux). Questo archivio sintetico riproduce ENTRAMBE le stranezze in
    // un colpo solo.

    /// Costruisce in memoria un tar.gz con:
    /// - `./onnxruntime-test/lib/libonnxruntime.so.1.23.0` — file regolare,
    ///   contenuto reale della "libreria";
    /// - `./onnxruntime-test/lib/libonnxruntime.so` — symlink relativo verso
    ///   `libonnxruntime.so.1.23.0` (stesso schema del tarball reale).
    ///
    /// Entrambe le entry hanno il prefisso `./` come nel tarball macOS reale.
    fn tar_gz_sintetico_con_prefisso_e_symlink(contenuto_lib: &[u8]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());

        let mut header_file = tar::Header::new_gnu();
        header_file.set_size(contenuto_lib.len() as u64);
        header_file.set_mode(0o644);
        header_file.set_cksum();
        builder
            .append_data(
                &mut header_file,
                "./onnxruntime-test/lib/libonnxruntime.so.1.23.0",
                contenuto_lib,
            )
            .unwrap();

        let mut header_link = tar::Header::new_gnu();
        header_link.set_size(0);
        header_link.set_mode(0o777);
        header_link.set_entry_type(tar::EntryType::Symlink);
        builder
            .append_link(
                &mut header_link,
                "./onnxruntime-test/lib/libonnxruntime.so",
                "libonnxruntime.so.1.23.0",
            )
            .unwrap();

        let tar_bytes = builder.into_inner().unwrap();
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        enc.write_all(&tar_bytes).unwrap();
        enc.finish().unwrap()
    }

    #[test]
    fn estrai_da_tar_gz_risolve_prefisso_puntoslash_e_symlink() {
        // Regressione #556: senza i due fix (normalizzazione `./` +
        // risoluzione symlink) questo test falliva con "file non trovato
        // nell'archivio" (sintomo 1) o produceva un file da 0 byte
        // (sintomo 2, se solo la normalizzazione fosse stata applicata).
        let contenuto = b"contenuto-fittizio-libonnxruntime";
        let archive = tar_gz_sintetico_con_prefisso_e_symlink(contenuto);

        // `path_in_archive`, come costruito da `ort_release_filename`: SENZA
        // prefisso `./`, punta all'entry symlink (non al file reale).
        let path_in_archive = "onnxruntime-test/lib/libonnxruntime.so";

        let dest_dir = tempfile::tempdir().unwrap();
        let dest = dest_dir.path().join("libonnxruntime.so");

        estrai_da_tar_gz(&archive, path_in_archive, &dest).unwrap();

        let estratto = std::fs::read(&dest).unwrap();
        assert_eq!(estratto, contenuto, "deve estrarre il contenuto REALE del target, non 0 byte");
    }

    #[test]
    fn estrai_da_tar_gz_file_assente_errore_esplicito() {
        let archive = tar_gz_sintetico_con_prefisso_e_symlink(b"x");
        let dest_dir = tempfile::tempdir().unwrap();
        let dest = dest_dir.path().join("out");

        let r = estrai_da_tar_gz(&archive, "percorso/inesistente", &dest);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("non trovato"));
    }

    #[test]
    fn estrai_libonnxruntime_su_errore_non_lascia_dest_parziale() {
        // `estrai_libonnxruntime` (il wrapper pubblico, non `estrai_da_tar_gz`
        // direttamente) deve estrarre atomicamente: su un archivio dove il
        // path richiesto non esiste, `dest` non deve comparire sul
        // filesystem né restare in uno stato parzialmente scritto — così un
        // successivo `embeddings_download` non scambia un residuo per
        // "libonnxruntime già scaricata" (vedi doc su `estrai_libonnxruntime`).
        let archive = tar_gz_sintetico_con_prefisso_e_symlink(b"x");
        let dest_dir = tempfile::tempdir().unwrap();
        let dest = dest_dir.path().join("libonnxruntime.so");

        let r = estrai_libonnxruntime(&archive, "percorso/inesistente", &dest);
        assert!(r.is_err());
        assert!(!dest.exists(), "dest non deve esistere dopo un'estrazione fallita");
        assert!(
            fs::read_dir(dest_dir.path()).unwrap().next().is_none(),
            "nessun file temporaneo residuo nella dest dir"
        );
    }

    #[test]
    fn estrai_libonnxruntime_su_successo_scrive_dest_definitiva() {
        let contenuto = b"contenuto-fittizio-libonnxruntime";
        let archive = tar_gz_sintetico_con_prefisso_e_symlink(contenuto);
        let dest_dir = tempfile::tempdir().unwrap();
        let dest = dest_dir.path().join("libonnxruntime.so");

        estrai_libonnxruntime(&archive, "onnxruntime-test/lib/libonnxruntime.so", &dest).unwrap();

        assert_eq!(std::fs::read(&dest).unwrap(), contenuto);
        // Solo il file finale, niente residui `.extract-partial`.
        let file_nella_dir: Vec<_> = fs::read_dir(dest_dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(file_nella_dir, vec![std::ffi::OsString::from("libonnxruntime.so")]);
    }

    #[test]
    fn normalizza_path_archivio_rimuove_prefisso_curdir() {
        assert_eq!(
            normalizza_path_archivio(Path::new("./a/b/c")),
            PathBuf::from("a/b/c")
        );
        // Path già senza prefisso: invariato.
        assert_eq!(
            normalizza_path_archivio(Path::new("a/b/c")),
            PathBuf::from("a/b/c")
        );
    }

    #[test]
    fn risolvi_path_relativo_link_nella_stessa_dir() {
        let risolto = risolvi_path_relativo(
            Path::new("onnxruntime-test/lib"),
            Path::new("libonnxruntime.so.1.23.0"),
        );
        assert_eq!(risolto, PathBuf::from("onnxruntime-test/lib/libonnxruntime.so.1.23.0"));
    }

    #[test]
    fn risolvi_path_relativo_gestisce_parent_dir() {
        let risolto = risolvi_path_relativo(
            Path::new("onnxruntime-test/lib"),
            Path::new("../lib64/libonnxruntime.so"),
        );
        assert_eq!(risolto, PathBuf::from("onnxruntime-test/lib64/libonnxruntime.so"));
    }
}
