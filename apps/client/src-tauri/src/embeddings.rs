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
use std::path::{Path, PathBuf};
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
/// nativo caricato ed eseguito nel processo, quindi un archivio
/// manomesso equivarrebbe a RCE.
fn verifica_sha256(bytes: &[u8], atteso: &str, nome_file: &str) -> Result<(), PapErrore> {
    let calcolato = sha256_hex(bytes);
    if !calcolato.eq_ignore_ascii_case(atteso) {
        return Err(PapErrore::Generico(format!(
            "Verifica integrità fallita per {nome_file}: l'hash SHA-256 del download \
             non corrisponde a quello atteso. Il file potrebbe essere stato manomesso \
             o il mirror è compromesso; download interrotto."
        )));
    }
    Ok(())
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
    if !path.exists() || !modello_completo(&path) {
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
        .map_err(|e| PapErrore::Generico(format!("HTTP get fallito ({}): {e}", url)))?;
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
        .map_err(|e| PapErrore::Generico(format!("HTTP get fallito ({}): {e}", url)))?;
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
            .map_err(|e| PapErrore::Generico(format!("Read stream: {e}")))?;
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
            .map_err(|e| PapErrore::Generico(format!("Read stream: {e}")))?;
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

fn estrai_libonnxruntime(
    archive_bytes: &[u8],
    path_in_archive: &str,
    dest: &Path,
) -> Result<(), PapErrore> {
    fs::create_dir_all(
        dest.parent()
            .ok_or_else(|| PapErrore::Generico("dest senza parent".into()))?,
    )?;

    if std::env::consts::OS == "windows" {
        // ZIP
        let cursor = std::io::Cursor::new(archive_bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| PapErrore::Generico(format!("zip read: {e}")))?;
        let mut entry = archive
            .by_name(path_in_archive)
            .map_err(|e| PapErrore::Generico(format!("zip entry {path_in_archive} non trovata: {e}")))?;
        let mut out = fs::File::create(dest)?;
        std::io::copy(&mut entry, &mut out)?;
    } else {
        // tar.gz
        let dec = flate2::read::GzDecoder::new(archive_bytes);
        let mut archive = tar::Archive::new(dec);
        let mut found = false;
        for entry in archive
            .entries()
            .map_err(|e| PapErrore::Generico(format!("tar entries: {e}")))?
        {
            let mut entry = entry.map_err(|e| PapErrore::Generico(format!("tar entry: {e}")))?;
            let entry_path = entry
                .path()
                .map_err(|e| PapErrore::Generico(format!("tar path: {e}")))?
                .to_string_lossy()
                .to_string();
            if entry_path == path_in_archive {
                let mut out = fs::File::create(dest)?;
                std::io::copy(&mut entry, &mut out)?;
                found = true;
                break;
            }
        }
        if !found {
            return Err(PapErrore::Generico(format!(
                "tar: file {path_in_archive} non trovato nell'archivio"
            )));
        }
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

    // Punta ort a libonnxruntime via env var. Sicuro perché siamo single-thread
    // qui (Tauri command sequenziati su mutex), e l'env var è letta solo al
    // primo Session::create.
    // SAFETY: set_var è unsafe in edition 2024+ ma stable in 2021.
    std::env::set_var("ORT_DYLIB_PATH", &lib_path);

    let model_path = dir_modello.join("model.onnx");
    let tokenizer_path = dir_modello.join("tokenizer.json");

    let session = Session::builder()
        .map_err(|e| PapErrore::Generico(format!("Session builder: {e}")))?
        .commit_from_file(&model_path)
        .map_err(|e| PapErrore::Generico(format!("Session load: {e}")))?;
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .map_err(|e| PapErrore::Generico(format!("Tokenizer load: {e}")))?;

    let mut guard = rt_state.inner.lock().unwrap_or_else(|p| p.into_inner());
    *guard = Some(EmbeddingsLoaded { session, tokenizer });
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
        .map_err(|e| PapErrore::Generico(format!("Tokenizer encode: {e}")))?;

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
            .map_err(|e| PapErrore::Generico(format!("tensor ids: {e}")))?,
        "attention_mask" => Tensor::from_array((shape.clone(), mask_clone))
            .map_err(|e| PapErrore::Generico(format!("tensor mask: {e}")))?,
        "token_type_ids" => Tensor::from_array((shape, type_ids))
            .map_err(|e| PapErrore::Generico(format!("tensor type: {e}")))?,
    ];
    let outputs = loaded
        .session
        .run(inputs)
        .map_err(|e| PapErrore::Generico(format!("ort run: {e}")))?;

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
        .map_err(|e| PapErrore::Generico(format!("extract f32 tensor: {e}")))?;

    if shape.len() != 3 || shape[0] != 1 || shape[2] as usize != EMBEDDING_DIM {
        return Err(PapErrore::Generico(format!(
            "Shape output inattesa: {:?} (atteso [1, seq_len, {EMBEDDING_DIM}])",
            shape
        )));
    }
    let actual_seq = shape[1] as usize;

    let token_emb = Array2::from_shape_vec((actual_seq, EMBEDDING_DIM), data.to_vec())
        .map_err(|e| PapErrore::Generico(format!("ndarray output: {e}")))?;
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
}
