// Modulo provider_ai — Fase 4 Step 8b.
//
// Astrazione per i provider AI usati dai golden examples (Step 8d).
// Un provider espone una funzione `generate(prompt, model)` che ritorna
// l'output del modello + metadati (latenza, tokens). Il chiamante
// (regression::esegui) si occupa di salvare l'observation e calcolare
// la similarità rispetto all'expected del golden.
//
// MVP scope (questa PR):
// - trait AIProvider con metodo generate
// - GenerateOutput come tipo di ritorno (content + metadati)
// - OllamaProvider come prima implementazione concreta — locale, no API
//   key richiesta. Endpoint default `http://localhost:11434/api/generate`,
//   override possibile via base_url
// - Comando Tauri `provider_ollama_genera` per smoke test dal frontend
//
// Provider remote (Anthropic, OpenAI, ecc.) atterrano in 8f con gestione
// API key cifrate nel vault.

use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::errore::PapErrore;

const OLLAMA_DEFAULT_URL: &str = "http://localhost:11434";
/// Timeout HTTP generoso: i modelli Ollama locali su CPU possono
/// impiegare 30-60s per generare una risposta lunga.
const HTTP_TIMEOUT_SEC: u64 = 180;

#[derive(Debug, Clone, Serialize)]
pub struct GenerateOutput {
    pub content: String,
    pub latency_ms: u64,
    pub tokens_used: Option<u32>,
    pub provider: &'static str,
    pub model: String,
}

/// Trait per qualunque provider AI. Implementazioni: Ollama (8b),
/// Anthropic/OpenAI (8f), OpenAI-compatible (futura).
pub trait AIProvider {
    fn name(&self) -> &'static str;
    fn generate(&self, prompt: &str, model: &str) -> Result<GenerateOutput, PapErrore>;
}

// ─────────── Ollama ───────────

#[derive(Debug, Clone)]
pub struct OllamaProvider {
    pub base_url: String,
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self {
            base_url: OLLAMA_DEFAULT_URL.to_string(),
        }
    }
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// Endpoint completo `<base>/api/generate`, normalizzato eliminando
    /// eventuale `/` finale dall'input utente.
    pub(crate) fn endpoint(&self) -> String {
        format!("{}/api/generate", self.base_url.trim_end_matches('/'))
    }
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    /// Modalità non-streaming: una sola risposta JSON, niente parsing
    /// chunk-by-chunk. Tradeoff accettabile per un test/regression run
    /// dove ci interessa il risultato finale.
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    #[allow(dead_code)]
    model: String,
    response: String,
    done: bool,
    /// Numero di token generati. Disponibile sui modelli moderni;
    /// `None` su risposte di modelli vecchi o errate.
    eval_count: Option<u32>,
}

/// Parser puro della risposta Ollama. Esposto per test unit senza
/// dover spinning-up un server vero.
pub(crate) fn parse_ollama_response(json: &str) -> Result<(String, Option<u32>), PapErrore> {
    let r: OllamaResponse = serde_json::from_str(json)
        .map_err(|e| PapErrore::Generico(format!("Risposta Ollama malformata: {e}")))?;
    if !r.done {
        return Err(PapErrore::Generico(
            "Risposta Ollama incompleta (done=false, streaming non supportato)".into(),
        ));
    }
    Ok((r.response, r.eval_count))
}

impl AIProvider for OllamaProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn generate(&self, prompt: &str, model: &str) -> Result<GenerateOutput, PapErrore> {
        let req = OllamaRequest {
            model,
            prompt,
            stream: false,
        };
        let body = serde_json::to_string(&req)
            .map_err(|e| PapErrore::Generico(format!("serializzazione richiesta: {e}")))?;

        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SEC))
            .build();

        let start = Instant::now();
        let resp = agent
            .post(&self.endpoint())
            .set("Content-Type", "application/json")
            .send_string(&body)
            .map_err(|e| PapErrore::Generico(format!("Ollama HTTP: {e}")))?;
        let latency_ms = start.elapsed().as_millis() as u64;

        let json = resp
            .into_string()
            .map_err(|e| PapErrore::Generico(format!("Ollama body: {e}")))?;

        let (content, tokens) = parse_ollama_response(&json)?;

        Ok(GenerateOutput {
            content,
            latency_ms,
            tokens_used: tokens,
            provider: "ollama",
            model: model.to_string(),
        })
    }
}

// ─────────── Tauri commands ───────────

/// Genera con Ollama. Utile per smoke test dal frontend
/// (Impostazioni > Provider AI > "Verifica connessione").
/// Il run vero per regression testing passa da `regression::golden_esegui`
/// (Step 8d).
#[tauri::command]
pub fn provider_ollama_genera(
    prompt: String,
    model: String,
    base_url: Option<String>,
) -> Result<GenerateOutput, PapErrore> {
    let provider = match base_url {
        Some(u) if !u.trim().is_empty() => OllamaProvider::new(u),
        _ => OllamaProvider::default(),
    };
    provider.generate(&prompt, &model)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_response_done_true() {
        let json = r#"{"model":"llama3.1","response":"ciao","done":true,"eval_count":42}"#;
        let (content, tokens) = parse_ollama_response(json).unwrap();
        assert_eq!(content, "ciao");
        assert_eq!(tokens, Some(42));
    }

    #[test]
    fn parse_response_done_false_fallisce() {
        let json = r#"{"model":"llama3.1","response":"par","done":false}"#;
        let r = parse_ollama_response(json);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("incompleta"));
    }

    #[test]
    fn parse_response_no_eval_count_ok() {
        // Modelli vecchi o errore di runtime: campo opzionale.
        let json = r#"{"model":"x","response":"out","done":true}"#;
        let (content, tokens) = parse_ollama_response(json).unwrap();
        assert_eq!(content, "out");
        assert_eq!(tokens, None);
    }

    #[test]
    fn parse_response_json_invalido_fallisce() {
        let r = parse_ollama_response("non-json");
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("malformata"));
    }

    #[test]
    fn parse_response_response_vuota_ok() {
        // Edge case: il modello non genera nulla ma done=true. Lasciamo
        // passare al chiamante (regression decidera se passa la soglia).
        let json = r#"{"model":"x","response":"","done":true,"eval_count":0}"#;
        let (content, tokens) = parse_ollama_response(json).unwrap();
        assert_eq!(content, "");
        assert_eq!(tokens, Some(0));
    }

    #[test]
    fn parse_response_unicode_preserved() {
        let json =
            r#"{"model":"x","response":"ciào 你好 🦀","done":true,"eval_count":3}"#;
        let (content, _) = parse_ollama_response(json).unwrap();
        assert_eq!(content, "ciào 你好 🦀");
    }

    #[test]
    fn endpoint_normalizza_trailing_slash() {
        let p1 = OllamaProvider::new("http://localhost:11434");
        let p2 = OllamaProvider::new("http://localhost:11434/");
        let p3 = OllamaProvider::new("http://localhost:11434///");
        assert_eq!(p1.endpoint(), "http://localhost:11434/api/generate");
        assert_eq!(p2.endpoint(), "http://localhost:11434/api/generate");
        assert_eq!(p3.endpoint(), "http://localhost:11434/api/generate");
    }

    #[test]
    fn default_provider_url() {
        let p = OllamaProvider::default();
        assert_eq!(p.base_url, "http://localhost:11434");
    }

    #[test]
    fn provider_custom_url() {
        let p = OllamaProvider::new("http://192.168.1.10:11434");
        assert_eq!(p.endpoint(), "http://192.168.1.10:11434/api/generate");
    }

    #[test]
    fn provider_name_ollama() {
        let p = OllamaProvider::default();
        assert_eq!(p.name(), "ollama");
    }
}
