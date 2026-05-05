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
// - PR 1/3 (questa): scaffolding + comando `embeddings_status` (path,
//   esistenza file modello, dimensione cache)
// - PR 2/3: download del modello + libonnxruntime con progress
// - PR 3/3: tokenizer + Session ort + comando `embeddings_compute`
//
// La presenza del modulo (anche solo scaffolding) verifica che ort linki
// con la configurazione `default-features = false, features = ["load-dynamic",
// "api-23", "ndarray"]` (vedi ADR onnx-bundle.md update 2026-05-05).

use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

/// ID del modello scelto. Usato come nome cartella sotto `${data_dir}/models/`
/// per supportare future migrations a modelli diversi (vedi ADR
/// embedding-model.md → "Migration path se in futuro si vuole cambiare modello").
pub const MODEL_ID: &str = "multilingual-MiniLM-L12-v2";

/// Dimensione embedding output (384 per MiniLM-L12-v2).
pub const EMBEDDING_DIM: usize = 384;

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

fn percorso_modello(state: &VaultState) -> PathBuf {
    // VaultState::data_dir non è pubblico — esponiamo via metodo dedicato.
    // Per ora usiamo un sottodirettorio "models" del data dir del vault.
    state.data_dir().join("models").join(MODEL_ID)
}

fn dim_cartella_mb(path: &PathBuf) -> u64 {
    let mut total: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
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

#[tauri::command]
pub fn embeddings_status(state: State<'_, VaultState>) -> Result<EmbeddingsStato, PapErrore> {
    let path = percorso_modello(&state);
    // File minimi che ci aspettiamo dopo il download: model.onnx + tokenizer.json.
    // Per ora ci basta verificare l'esistenza della cartella per la PR 1/3.
    if !path.exists() {
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
}
