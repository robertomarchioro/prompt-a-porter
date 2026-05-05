// Modulo embeddings — Fase 3 Step 1.
//
// Calcolo embeddings locali via ONNX Runtime (`ort` crate). Modello:
// `paraphrase-multilingual-MiniLM-L12-v2` (384 dim, ~118 MB ONNX
// quantizzato), scelto in Spike 3 v2 (vedi
// `docs/architettura/decisioni/embedding-model.md`).
//
// Architettura distribuzione:
// - Modello scaricato lazy al primo uso da HuggingFace (Xenova fork)
// - Cache locale in `${data_dir}/models/multilingual-MiniLM-L12-v2/`
// - libonnxruntime caricata via `load-dynamic` (vedi ADR onnx-bundle.md)
//
// Stato implementazione:
// - PR 1/3: scaffolding + comando `embeddings_status`
// - PR 2/3 (questa): comando `embeddings_download` per scaricare i due
//   file modello (model_quantized.onnx + tokenizer.json) da HuggingFace
//   con progress via Tauri events
// - PR 3/3: download libonnxruntime + Session ort + comando
//   `embeddings_compute`

use serde::Serialize;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{Emitter, State};

use crate::errore::PapErrore;
use crate::vault::VaultState;

/// ID del modello scelto. Usato come nome cartella sotto `${data_dir}/models/`
/// per supportare future migrations a modelli diversi (vedi ADR
/// embedding-model.md → "Migration path se in futuro si vuole cambiare modello").
pub const MODEL_ID: &str = "multilingual-MiniLM-L12-v2";

/// Dimensione embedding output (384 per MiniLM-L12-v2).
pub const EMBEDDING_DIM: usize = 384;

/// Repository HuggingFace di Xenova con la versione ONNX quantizzata.
const HF_REPO: &str = "Xenova/paraphrase-multilingual-MiniLM-L12-v2";

/// File da scaricare. Coppia `(path-su-HF, nome-locale)`.
/// model_quantized.onnx ≈ 118 MB; tokenizer.json ≈ 17 MB.
const FILES: &[(&str, &str)] = &[
    ("onnx/model_quantized.onnx", "model.onnx"),
    ("tokenizer.json", "tokenizer.json"),
];

/// Timeout HTTP generoso per file da centinaia di MB su connessioni lente.
const HTTP_TIMEOUT_SEC: u64 = 600;

/// Status corrente del modulo embeddings dal punto di vista del frontend.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "stato")]
pub enum EmbeddingsStato {
    /// Il modello non è stato scaricato. Il frontend deve mostrare la
    /// progress UI di download al primo uso.
    NonScaricato { model_id: String, path_atteso: String },
    /// Il modello è scaricato su disco ma non è ancora caricato in memoria.
    /// L'init è stato richiesto e/o il modello è pronto per l'uso.
    Pronto {
        model_id: String,
        path: String,
        size_mb: u64,
    },
    /// Il modello è caricato in sessione ort, embedding ready.
    Caricato { model_id: String, dimensione: usize },
}

/// Evento di progress emesso al frontend durante il download.
/// Channel: `embeddings:download:progress`.
#[derive(Debug, Clone, Serialize)]
pub struct ProgressDownload {
    pub file: String,
    /// Bytes scaricati per il file corrente.
    pub bytes: u64,
    /// Bytes totali del file (se Content-Length disponibile, altrimenti `None`).
    pub total: Option<u64>,
    /// Indice del file corrente fra quelli da scaricare (1-based).
    pub indice_file: usize,
    /// Numero totale di file da scaricare.
    pub totale_file: usize,
}

fn percorso_modello(state: &VaultState) -> PathBuf {
    state.data_dir().join("models").join(MODEL_ID)
}

fn dim_cartella_mb(path: &PathBuf) -> u64 {
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

fn modello_completo(path: &PathBuf) -> bool {
    FILES.iter().all(|(_, locale)| path.join(locale).is_file())
}

#[tauri::command]
pub fn embeddings_status(state: State<'_, VaultState>) -> Result<EmbeddingsStato, PapErrore> {
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

/// Scarica un singolo file con streaming + progress event.
fn scarica_file(
    app: &tauri::AppHandle,
    url: &str,
    dest: &PathBuf,
    indice_file: usize,
    totale_file: usize,
    nome_visibile: &str,
) -> Result<(), PapErrore> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(HTTP_TIMEOUT_SEC))
        .build();

    let resp = agent
        .get(url)
        .call()
        .map_err(|e| PapErrore::Generico(format!("Download fallito ({}): {e}", url)))?;

    let total: Option<u64> = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok());

    // Scrittura su file temporaneo + rename atomico → nessun file parziale
    // in caso di crash a metà download.
    let dest_tmp = dest.with_extension("download-partial");
    if let Some(parent) = dest_tmp.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut out = fs::File::create(&dest_tmp)?;
    let mut reader = resp.into_reader();
    let mut buf = [0u8; 64 * 1024];
    let mut acc: u64 = 0;
    let mut last_emit_acc: u64 = 0;

    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| PapErrore::Generico(format!("Read stream: {e}")))?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n])?;
        acc += n as u64;

        // Throttling: emit progress ogni 256 KB per non saturare il bus eventi
        // su file grandi.
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

    out.flush()?;
    drop(out);
    fs::rename(&dest_tmp, dest)?;

    // Emit finale a 100%
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
    Ok(())
}

#[tauri::command]
pub fn embeddings_download(
    app: tauri::AppHandle,
    state: State<'_, VaultState>,
) -> Result<EmbeddingsStato, PapErrore> {
    let dir = percorso_modello(&state);
    fs::create_dir_all(&dir)?;

    let totale_file = FILES.len();
    for (idx, (path_remoto, nome_locale)) in FILES.iter().enumerate() {
        let dest = dir.join(nome_locale);
        if dest.is_file() {
            log::info!(
                "embeddings_download: {} già presente, skip",
                dest.display()
            );
            continue;
        }
        let url = format!("https://huggingface.co/{HF_REPO}/resolve/main/{path_remoto}");
        log::info!("embeddings_download: scarico {url} -> {}", dest.display());
        scarica_file(&app, &url, &dest, idx + 1, totale_file, nome_locale)?;
    }

    log::info!(
        "embeddings_download: completato, dir = {}",
        dir.display()
    );

    embeddings_status(state)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn model_id_coerente_con_adr() {
        // Sentinel test: se cambiamo modello (es. in futuro EmbeddingGemma),
        // questo test deve essere aggiornato esplicitamente.
        assert_eq!(MODEL_ID, "multilingual-MiniLM-L12-v2");
        assert_eq!(EMBEDDING_DIM, 384);
    }

    #[test]
    fn dim_cartella_dir_inesistente() {
        let p = Path::new("/percorso/inesistente").to_path_buf();
        assert_eq!(dim_cartella_mb(&p), 0);
    }

    #[test]
    fn modello_completo_richiede_entrambi_i_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        assert!(!modello_completo(&p), "dir vuota deve essere incompleto");
        std::fs::write(p.join("model.onnx"), b"stub").unwrap();
        assert!(!modello_completo(&p), "solo model.onnx non basta");
        std::fs::write(p.join("tokenizer.json"), b"stub").unwrap();
        assert!(modello_completo(&p), "entrambi i file presenti = completo");
    }

    #[test]
    fn files_coerenti_con_ids_modello() {
        // Sanity: i file nel manifest devono essere quelli che ci aspettiamo
        // di trovare sul HF Xenova repo. Se cambia il modello bisogna anche
        // aggiornare FILES.
        assert_eq!(FILES.len(), 2);
        assert_eq!(FILES[0].1, "model.onnx");
        assert_eq!(FILES[1].1, "tokenizer.json");
    }
}
