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

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

const OLLAMA_DEFAULT_URL: &str = "http://localhost:11434";
const ANTHROPIC_DEFAULT_URL: &str = "https://api.anthropic.com";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const OPENAI_DEFAULT_URL: &str = "https://api.openai.com";
const GEMINI_DEFAULT_URL: &str = "https://generativelanguage.googleapis.com";

/// Timeout HTTP generoso: i modelli Ollama locali su CPU possono
/// impiegare 30-60s per generare una risposta lunga; gli endpoint
/// remote (Anthropic/OpenAI) tipicamente rispondono in <30s.
const HTTP_TIMEOUT_SEC: u64 = 180;

/// Default `max_tokens` per Anthropic — campo obbligatorio nelle
/// /v1/messages. 4096 è un compromesso tra "abbastanza per qualunque
/// risposta normale" e "non sprecare crediti se il modello allucina".
const ANTHROPIC_MAX_TOKENS_DEFAULT: u32 = 4096;

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

// ─────────── Anthropic ───────────

// No `Debug`: contiene `api_key`. Evita leak accidentali via `{:?}`/log;
// se servisse Debug, implementarlo a mano mascherando la chiave.
#[derive(Clone)]
pub struct AnthropicProvider {
    pub base_url: String,
    pub api_key: String,
    pub max_tokens: u32,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        Self {
            base_url: base_url
                .filter(|u| !u.trim().is_empty())
                .unwrap_or_else(|| ANTHROPIC_DEFAULT_URL.to_string()),
            api_key: api_key.into(),
            max_tokens: ANTHROPIC_MAX_TOKENS_DEFAULT,
        }
    }

    pub(crate) fn endpoint(&self) -> String {
        format!("{}/v1/messages", self.base_url.trim_end_matches('/'))
    }
}

#[derive(Serialize)]
struct AnthropicMsg<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<AnthropicMsg<'a>>,
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    type_: String,
    #[serde(default)]
    text: String,
}

#[derive(Deserialize, Default)]
struct AnthropicUsage {
    #[serde(default)]
    output_tokens: u32,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
    #[serde(default)]
    usage: AnthropicUsage,
}

/// Estrae `(content, output_tokens)` dalla response /v1/messages.
/// Concatena tutti i block di tipo `text` (di solito ce n'è uno solo).
pub(crate) fn parse_anthropic_response(json: &str) -> Result<(String, Option<u32>), PapErrore> {
    let r: AnthropicResponse = serde_json::from_str(json)
        .map_err(|e| PapErrore::Generico(format!("Risposta Anthropic malformata: {e}")))?;
    let content: String = r
        .content
        .iter()
        .filter(|b| b.type_ == "text")
        .map(|b| b.text.clone())
        .collect::<Vec<_>>()
        .join("");
    let tokens = if r.usage.output_tokens > 0 {
        Some(r.usage.output_tokens)
    } else {
        None
    };
    Ok((content, tokens))
}

impl AIProvider for AnthropicProvider {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn generate(&self, prompt: &str, model: &str) -> Result<GenerateOutput, PapErrore> {
        let req = AnthropicRequest {
            model,
            max_tokens: self.max_tokens,
            messages: vec![AnthropicMsg {
                role: "user",
                content: prompt,
            }],
        };
        let body = serde_json::to_string(&req)
            .map_err(|e| PapErrore::Generico(format!("serializzazione richiesta: {e}")))?;

        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SEC))
            .build();

        let start = Instant::now();
        let resp = agent
            .post(&self.endpoint())
            .set("x-api-key", &self.api_key)
            .set("anthropic-version", ANTHROPIC_VERSION)
            .set("content-type", "application/json")
            .send_string(&body)
            .map_err(|e| PapErrore::Generico(format!("Anthropic HTTP: {e}")))?;
        let latency_ms = start.elapsed().as_millis() as u64;

        let json = resp
            .into_string()
            .map_err(|e| PapErrore::Generico(format!("Anthropic body: {e}")))?;

        let (content, tokens) = parse_anthropic_response(&json)?;

        Ok(GenerateOutput {
            content,
            latency_ms,
            tokens_used: tokens,
            provider: "anthropic",
            model: model.to_string(),
        })
    }
}

// ─────────── OpenAI ───────────

// No `Debug`: contiene `api_key`.
#[derive(Clone)]
pub struct OpenAIProvider {
    pub base_url: String,
    pub api_key: String,
}

impl OpenAIProvider {
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        Self {
            base_url: base_url
                .filter(|u| !u.trim().is_empty())
                .unwrap_or_else(|| OPENAI_DEFAULT_URL.to_string()),
            api_key: api_key.into(),
        }
    }

    pub(crate) fn endpoint(&self) -> String {
        format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        )
    }
}

#[derive(Serialize)]
struct OpenAIMsg<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct OpenAIRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAIMsg<'a>>,
}

#[derive(Deserialize)]
struct OpenAIChoiceMessage {
    #[serde(default)]
    content: String,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIChoiceMessage,
}

#[derive(Deserialize, Default)]
struct OpenAIUsage {
    #[serde(default)]
    total_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    #[serde(default)]
    usage: OpenAIUsage,
}

/// Estrae `(content, completion_tokens)` dalla response /v1/chat/completions.
/// Usa `choices[0].message.content` (il primo, che è quello che il caller
/// si aspetta non avendo richiesto `n>1`).
pub(crate) fn parse_openai_response(json: &str) -> Result<(String, Option<u32>), PapErrore> {
    let r: OpenAIResponse = serde_json::from_str(json)
        .map_err(|e| PapErrore::Generico(format!("Risposta OpenAI malformata: {e}")))?;
    let content = r
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();
    // Preferisci completion_tokens (più preciso); fallback a total se manca.
    let tokens = if r.usage.completion_tokens > 0 {
        Some(r.usage.completion_tokens)
    } else if r.usage.total_tokens > 0 {
        Some(r.usage.total_tokens)
    } else {
        None
    };
    Ok((content, tokens))
}

impl AIProvider for OpenAIProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn generate(&self, prompt: &str, model: &str) -> Result<GenerateOutput, PapErrore> {
        let req = OpenAIRequest {
            model,
            messages: vec![OpenAIMsg {
                role: "user",
                content: prompt,
            }],
        };
        let body = serde_json::to_string(&req)
            .map_err(|e| PapErrore::Generico(format!("serializzazione richiesta: {e}")))?;

        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SEC))
            .build();

        let start = Instant::now();
        let resp = agent
            .post(&self.endpoint())
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("content-type", "application/json")
            .send_string(&body)
            .map_err(|e| PapErrore::Generico(format!("OpenAI HTTP: {e}")))?;
        let latency_ms = start.elapsed().as_millis() as u64;

        let json = resp
            .into_string()
            .map_err(|e| PapErrore::Generico(format!("OpenAI body: {e}")))?;

        let (content, tokens) = parse_openai_response(&json)?;

        Ok(GenerateOutput {
            content,
            latency_ms,
            tokens_used: tokens,
            provider: "openai",
            model: model.to_string(),
        })
    }
}

// ─────────── Gemini (Google AI) ───────────

// No `Debug`: contiene `api_key`.
#[derive(Clone)]
pub struct GeminiProvider {
    pub base_url: String,
    pub api_key: String,
}

impl GeminiProvider {
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        Self {
            base_url: base_url
                .filter(|u| !u.trim().is_empty())
                .unwrap_or_else(|| GEMINI_DEFAULT_URL.to_string()),
            api_key: api_key.into(),
        }
    }

    /// Endpoint `<base>/v1beta/models/<model>:generateContent`.
    /// Il model è path-encoded direttamente nell'URL, niente body.
    pub(crate) fn endpoint(&self, model: &str) -> String {
        format!(
            "{}/v1beta/models/{}:generateContent",
            self.base_url.trim_end_matches('/'),
            model
        )
    }
}

#[derive(Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct GeminiContent<'a> {
    parts: Vec<GeminiPart<'a>>,
}

#[derive(Serialize)]
struct GeminiRequest<'a> {
    contents: Vec<GeminiContent<'a>>,
}

#[derive(Deserialize)]
struct GeminiPartResp {
    #[serde(default)]
    text: String,
}

#[derive(Deserialize)]
struct GeminiContentResp {
    #[serde(default)]
    parts: Vec<GeminiPartResp>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResp,
}

#[derive(Deserialize, Default)]
struct GeminiUsage {
    #[serde(default, rename = "candidatesTokenCount")]
    candidates_token_count: u32,
    #[serde(default, rename = "totalTokenCount")]
    total_token_count: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    #[serde(default)]
    candidates: Vec<GeminiCandidate>,
    #[serde(default, rename = "usageMetadata")]
    usage_metadata: GeminiUsage,
}

/// Estrae `(content, candidates_token_count)` dalla response generateContent.
/// Concatena tutti i `parts` del primo `candidate` (di solito ce n'è uno).
pub(crate) fn parse_gemini_response(json: &str) -> Result<(String, Option<u32>), PapErrore> {
    let r: GeminiResponse = serde_json::from_str(json)
        .map_err(|e| PapErrore::Generico(format!("Risposta Gemini malformata: {e}")))?;
    let content: String = r
        .candidates
        .first()
        .map(|c| {
            c.content
                .parts
                .iter()
                .map(|p| p.text.clone())
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();
    // Preferisci candidates_token_count (output) per coerenza con OpenAI.
    let tokens = if r.usage_metadata.candidates_token_count > 0 {
        Some(r.usage_metadata.candidates_token_count)
    } else if r.usage_metadata.total_token_count > 0 {
        Some(r.usage_metadata.total_token_count)
    } else {
        None
    };
    Ok((content, tokens))
}

impl AIProvider for GeminiProvider {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn generate(&self, prompt: &str, model: &str) -> Result<GenerateOutput, PapErrore> {
        let req = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: prompt }],
            }],
        };
        let body = serde_json::to_string(&req)
            .map_err(|e| PapErrore::Generico(format!("serializzazione richiesta: {e}")))?;

        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SEC))
            .build();

        let start = Instant::now();
        let resp = agent
            .post(&self.endpoint(model))
            .set("x-goog-api-key", &self.api_key)
            .set("content-type", "application/json")
            .send_string(&body)
            .map_err(|e| PapErrore::Generico(format!("Gemini HTTP: {e}")))?;
        let latency_ms = start.elapsed().as_millis() as u64;

        let json = resp
            .into_string()
            .map_err(|e| PapErrore::Generico(format!("Gemini body: {e}")))?;

        let (content, tokens) = parse_gemini_response(&json)?;

        Ok(GenerateOutput {
            content,
            latency_ms,
            tokens_used: tokens,
            provider: "gemini",
            model: model.to_string(),
        })
    }
}

// ─────────── ProviderConfig storage (V010) ───────────

// No `Debug`: può contenere `api_key`.
#[derive(Clone, Serialize, Deserialize)]
pub struct ProviderConfigItem {
    pub provider: String,
    /// `None` quando il chiamante è la UI di lista — non rinviamo mai
    /// la API key in chiaro al frontend a meno che non la chieda
    /// esplicitamente con un comando dedicato.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
    pub abilitato: bool,
    pub creato_a: String,
    pub aggiornato_a: String,
}

// No `Debug`: può contenere `api_key`.
#[derive(Deserialize)]
pub struct ProviderConfigInput {
    pub provider: String,
    /// `None` o stringa vuota = "non toccare la chiave esistente". Utile
    /// quando l'utente modifica solo `default_model` da UI senza re-incollare
    /// la API key.
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default = "default_abilitato")]
    pub abilitato: bool,
}

fn default_abilitato() -> bool {
    true
}

const PROVIDERS_VALIDI: &[&str] = &[
    "anthropic",
    "openai",
    "ollama",
    "openai-compat",
    "gemini",
];

fn valida_provider(name: &str) -> Result<(), PapErrore> {
    if !PROVIDERS_VALIDI.contains(&name) {
        return Err(PapErrore::Generico(format!(
            "Provider '{name}' non riconosciuto. Validi: {:?}",
            PROVIDERS_VALIDI
        )));
    }
    Ok(())
}

/// Host locali consentiti per `http://` (letterali ESATTI, confronto
/// case-insensitive già normalizzato a monte). Deliberatamente NON usiamo
/// `Ipv4Addr::is_loopback()`/`Ipv6Addr::is_loopback()` sull'host parsato:
/// accetterebbe silenziosamente forme alternative dello stesso indirizzo
/// (decimale `2130706433`, ottale `0177.0.0.1`, ecc.) che WHATWG/`url`
/// normalizzano a `127.0.0.1` — un validatore "intelligente" diventerebbe
/// esso stesso un vettore di offuscamento. L'allowlist testuale stretta è
/// volutamente noiosa da forzare.
const HOST_LOCALI_CONSENTITI: &[&str] = &["localhost", "127.0.0.1", "::1"];

/// Fix #457 (review CRITICAL): valida lo schema/host di un `base_url`
/// custom fornito dall'utente per un provider AI (config salvata o
/// smoke-test Ollama).
///
/// Usa `url::Url::parse` (conforme WHATWG) invece di string-splitting
/// manuale: un parser manuale che cerca il primo `:`/`/` dopo lo schema
/// interpreta MALE un URL con userinfo come `http://127.0.0.1@evil.com/`
/// (vede host=127.0.0.1, ma il client HTTP si connette DAVVERO a
/// evil.com, che riceve la API key in chiaro — vedi audit #457 follow-up).
///
/// Regole:
/// - stringa vuota/whitespace → Ok: significa "usa il default del
///   provider", non c'è nessun input da validare.
/// - qualunque userinfo (`user:pass@host` o anche solo `user@host`) →
///   SEMPRE rifiutato: non esiste un caso d'uso legittimo per un
///   provider AI, ed è il principale vettore per ingannare un parser
///   ingenuo sull'host reale.
/// - `https://<host>` → accettato (host non vuoto).
/// - `http://<host>` → accettato SOLO se l'host è uno dei letterali
///   `HOST_LOCALI_CONSENTITI` (confronto ESATTO, case-insensitive):
///   Ollama gira tipicamente in locale senza TLS. `http://` verso un
///   host remoto trasmetterebbe la API key (se presente in header) in
///   chiaro sulla rete.
/// - qualunque altro schema (`file://`, `ftp://`, `javascript:`,
///   nessuno schema, ecc.) → rifiutato: `file://` permetterebbe di
///   leggere file locali arbitrari attraverso il client HTTP, altri
///   schemi non sono endpoint HTTP validi.
pub(crate) fn valida_base_url(url: &str) -> Result<(), PapErrore> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let parsed = url::Url::parse(trimmed)
        .map_err(|_| PapErrore::Generico("base_url non valido: URL non analizzabile.".into()))?;

    // Fix #457 CRITICAL: rifiuta qualunque userinfo PRIMA di ispezionare
    // l'host — altrimenti un validatore "furbo" costruito sopra l'host
    // parsato correttamente potrebbe comunque essere aggirato da varianti
    // future di questa funzione che tornassero a ispezionare la stringa
    // grezza. Qui lo controlliamo direttamente sui campi strutturati.
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(PapErrore::Generico(
            "base_url non valido: credenziali (user:pass@) non consentite nell'URL.".into(),
        ));
    }

    match parsed.scheme() {
        "https" => {
            if parsed.host_str().unwrap_or("").is_empty() {
                return Err(PapErrore::Generico(
                    "base_url non valido: host mancante dopo https://".into(),
                ));
            }
            Ok(())
        }
        "http" => {
            // A questo punto l'URL non ha userinfo (verificato sopra): il
            // testo tra `http://` e la prima `/`/`?`/`#` è esattamente
            // `host[:porta]` come scritto dall'utente, senza ambiguità.
            // Confronto testuale ESATTO con l'allowlist (non semantico):
            // vedi doc-comment di HOST_LOCALI_CONSENTITI sul perché.
            let lower = trimmed.to_ascii_lowercase();
            let resto = lower.strip_prefix("http://").unwrap_or("");
            let host_grezzo = if let Some(dentro_parentesi) = resto.strip_prefix('[') {
                dentro_parentesi.split(']').next().unwrap_or("")
            } else {
                resto.split(['/', ':', '?', '#']).next().unwrap_or("")
            };
            if HOST_LOCALI_CONSENTITI.contains(&host_grezzo) {
                Ok(())
            } else {
                Err(PapErrore::Generico(
                    "base_url non valido: http:// è consentito solo per host locali \
                     (localhost/127.0.0.1); usa https:// per host remoti."
                        .into(),
                ))
            }
        }
        _ => Err(PapErrore::Generico(
            "base_url non valido: schema non consentito. Richiesto https://, \
             o http:// solo per host locali."
                .into(),
        )),
    }
}

pub(crate) fn config_lista_pure(
    conn: &Connection,
) -> Result<Vec<ProviderConfigItem>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT Provider, BaseUrl, DefaultModel, Abilitato, CreatedAt, UpdatedAt
         FROM ProviderConfig
         ORDER BY Provider ASC",
    )?;
    let rows: Vec<ProviderConfigItem> = stmt
        .query_map([], |r| {
            Ok(ProviderConfigItem {
                provider: r.get(0)?,
                api_key: None,
                base_url: r.get(1)?,
                default_model: r.get(2)?,
                abilitato: r.get::<_, i64>(3)? != 0,
                creato_a: r.get(4)?,
                aggiornato_a: r.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Fix #456: rifiuta il salvataggio di una API key quando il vault
/// corrente non è cifrato. Le API key sono segreti long-lived (a
/// differenza della password del vault, che l'utente digita ogni volta):
/// persisterle in un DB SQLite in chiaro le espone a chiunque abbia
/// accesso al filesystem. Il chiamante (comando Tauri) determina
/// `vault_cifrato` leggendo `vault-meta.json` PRIMA di aprire la
/// connessione, così questa funzione resta pura e testabile senza
/// dover costruire un `VaultState` reale.
pub(crate) fn config_salva_pure(
    conn: &Connection,
    input: &ProviderConfigInput,
    vault_cifrato: bool,
) -> Result<(), PapErrore> {
    valida_provider(&input.provider)?;
    // Fix #457: valida lo schema del base_url custom (previene SSRF/leak
    // della API key verso host remoti non-TLS o lettura file:// locali).
    if let Some(url) = &input.base_url {
        valida_base_url(url)?;
    }
    // Se api_key è None o stringa vuota, manteniamo la chiave esistente.
    let manteni_chiave = input
        .api_key
        .as_ref()
        .map(|k| k.is_empty())
        .unwrap_or(true);

    // Rifiuta di persistere una NUOVA api_key se il vault non è cifrato.
    // Non blocca l'aggiornamento di base_url/default_model/abilitato quando
    // la chiave non viene toccata (manteni_chiave=true): in quel caso non
    // stiamo scrivendo nessun nuovo segreto in chiaro.
    if !manteni_chiave && !vault_cifrato {
        return Err(PapErrore::VaultNonCifrato);
    }

    if manteni_chiave {
        conn.execute(
            "INSERT INTO ProviderConfig (Provider, ApiKey, BaseUrl, DefaultModel,
                Abilitato, CreatedAt, UpdatedAt)
             VALUES (?1, NULL, ?2, ?3, ?4, datetime('now'), datetime('now'))
             ON CONFLICT(Provider) DO UPDATE SET
                BaseUrl = excluded.BaseUrl,
                DefaultModel = excluded.DefaultModel,
                Abilitato = excluded.Abilitato,
                UpdatedAt = datetime('now')",
            params![
                input.provider,
                input.base_url,
                input.default_model,
                input.abilitato as i64,
            ],
        )?;
    } else {
        conn.execute(
            "INSERT INTO ProviderConfig (Provider, ApiKey, BaseUrl, DefaultModel,
                Abilitato, CreatedAt, UpdatedAt)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))
             ON CONFLICT(Provider) DO UPDATE SET
                ApiKey = excluded.ApiKey,
                BaseUrl = excluded.BaseUrl,
                DefaultModel = excluded.DefaultModel,
                Abilitato = excluded.Abilitato,
                UpdatedAt = datetime('now')",
            params![
                input.provider,
                input.api_key,
                input.base_url,
                input.default_model,
                input.abilitato as i64,
            ],
        )?;
    }
    crate::audit::registra(
        conn,
        "provider.salvato",
        "ProviderConfig",
        &input.provider,
        None,
    );
    Ok(())
}

pub(crate) fn config_elimina_pure(
    conn: &Connection,
    provider: &str,
) -> Result<(), PapErrore> {
    let n = conn.execute(
        "DELETE FROM ProviderConfig WHERE Provider = ?1",
        [provider],
    )?;
    if n == 0 {
        return Err(PapErrore::Generico(format!(
            "Provider '{provider}' non configurato"
        )));
    }
    crate::audit::registra(
        conn,
        "provider.eliminato",
        "ProviderConfig",
        provider,
        None,
    );
    Ok(())
}

/// Carica la config completa (api_key inclusa) per uso interno
/// (`regression::golden_esegui` la usa per istanziare il provider).
pub(crate) fn config_carica_completa(
    conn: &Connection,
    provider: &str,
) -> Result<ProviderConfigItem, PapErrore> {
    conn.query_row(
        "SELECT Provider, ApiKey, BaseUrl, DefaultModel, Abilitato, CreatedAt, UpdatedAt
         FROM ProviderConfig WHERE Provider = ?1",
        [provider],
        |r| {
            Ok(ProviderConfigItem {
                provider: r.get(0)?,
                api_key: r.get(1)?,
                base_url: r.get(2)?,
                default_model: r.get(3)?,
                abilitato: r.get::<_, i64>(4)? != 0,
                creato_a: r.get(5)?,
                aggiornato_a: r.get(6)?,
            })
        },
    )
    .map_err(|_| PapErrore::Generico(format!("Provider '{provider}' non configurato")))
}

/// Costruisce un'istanza `Box<dyn AIProvider>` a partire da una
/// `ProviderConfigItem` caricata dal DB. Errore se la config richiede
/// una API key e questa è mancante.
pub(crate) fn istanzia_provider(
    cfg: &ProviderConfigItem,
) -> Result<Box<dyn AIProvider>, PapErrore> {
    match cfg.provider.as_str() {
        "ollama" => Ok(Box::new(match cfg.base_url.clone() {
            Some(u) if !u.trim().is_empty() => OllamaProvider::new(u),
            _ => OllamaProvider::default(),
        })),
        "anthropic" => {
            let key = cfg.api_key.as_ref().ok_or_else(|| {
                PapErrore::Generico("Anthropic richiede una API key configurata".into())
            })?;
            Ok(Box::new(AnthropicProvider::new(
                key.clone(),
                cfg.base_url.clone(),
            )))
        }
        "openai" | "openai-compat" => {
            let key = cfg.api_key.as_ref().ok_or_else(|| {
                PapErrore::Generico(format!(
                    "{} richiede una API key configurata",
                    cfg.provider
                ))
            })?;
            Ok(Box::new(OpenAIProvider::new(
                key.clone(),
                cfg.base_url.clone(),
            )))
        }
        "gemini" => {
            let key = cfg.api_key.as_ref().ok_or_else(|| {
                PapErrore::Generico("Gemini richiede una API key configurata".into())
            })?;
            Ok(Box::new(GeminiProvider::new(
                key.clone(),
                cfg.base_url.clone(),
            )))
        }
        altro => Err(PapErrore::Generico(format!(
            "Provider '{altro}' non supportato"
        ))),
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
    // Fix #457: valida lo schema del base_url anche sul percorso di smoke
    // test, non solo al salvataggio della config — un base_url malevolo
    // passato direttamente dal frontend deve essere rifiutato qui.
    if let Some(url) = &base_url {
        valida_base_url(url)?;
    }
    let provider = match base_url {
        Some(u) if !u.trim().is_empty() => OllamaProvider::new(u),
        _ => OllamaProvider::default(),
    };
    provider.generate(&prompt, &model)
}

#[tauri::command]
pub fn provider_config_lista(
    state: State<'_, VaultState>,
) -> Result<Vec<ProviderConfigItem>, PapErrore> {
    state.with_conn(config_lista_pure)
}

#[tauri::command]
pub fn provider_config_salva(
    input: ProviderConfigInput,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    // Fix #456: determina se il vault è cifrato PRIMA di scrivere, così
    // `config_salva_pure` può rifiutare il salvataggio di una nuova API key
    // quando il vault è in chiaro.
    let vault_cifrato = crate::vault::vault_cifrato_impl(&state)?;
    state.with_conn(|conn| config_salva_pure(conn, &input, vault_cifrato))
}

#[tauri::command]
pub fn provider_config_elimina(
    provider: String,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| config_elimina_pure(conn, &provider))
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

    // ─────────── Anthropic ───────────

    #[test]
    fn anthropic_endpoint_e_default_url() {
        let p = AnthropicProvider::new("k", None);
        assert_eq!(p.endpoint(), "https://api.anthropic.com/v1/messages");
        assert_eq!(p.api_key, "k");
    }

    #[test]
    fn anthropic_endpoint_custom_url() {
        let p = AnthropicProvider::new("k", Some("https://my-proxy.local/".into()));
        assert_eq!(p.endpoint(), "https://my-proxy.local/v1/messages");
    }

    #[test]
    fn anthropic_endpoint_empty_url_e_default() {
        // Stringa vuota significa "non override".
        let p = AnthropicProvider::new("k", Some("  ".into()));
        assert_eq!(p.endpoint(), "https://api.anthropic.com/v1/messages");
    }

    #[test]
    fn parse_anthropic_response_un_blocco() {
        let json = r#"{"content":[{"type":"text","text":"ciao"}],"usage":{"input_tokens":10,"output_tokens":5}}"#;
        let (content, tokens) = parse_anthropic_response(json).unwrap();
        assert_eq!(content, "ciao");
        assert_eq!(tokens, Some(5));
    }

    #[test]
    fn parse_anthropic_response_blocchi_misti_concatena_solo_text() {
        // Content può contenere `tool_use`, `image`, ecc. — prendiamo solo `text`.
        let json = r#"{"content":[{"type":"text","text":"prima"},{"type":"tool_use","text":"ignored"},{"type":"text","text":" parte"}],"usage":{"output_tokens":7}}"#;
        let (content, tokens) = parse_anthropic_response(json).unwrap();
        assert_eq!(content, "prima parte");
        assert_eq!(tokens, Some(7));
    }

    #[test]
    fn parse_anthropic_response_no_usage_e_none() {
        let json = r#"{"content":[{"type":"text","text":"x"}]}"#;
        let (_, tokens) = parse_anthropic_response(json).unwrap();
        assert_eq!(tokens, None);
    }

    #[test]
    fn parse_anthropic_response_json_invalido_e_errore() {
        assert!(parse_anthropic_response("non-json").is_err());
    }

    #[test]
    fn anthropic_provider_name() {
        let p = AnthropicProvider::new("k", None);
        assert_eq!(p.name(), "anthropic");
    }

    // ─────────── OpenAI ───────────

    #[test]
    fn openai_endpoint_e_default_url() {
        let p = OpenAIProvider::new("k", None);
        assert_eq!(p.endpoint(), "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn openai_endpoint_custom_url() {
        let p = OpenAIProvider::new("k", Some("http://localhost:1234".into()));
        assert_eq!(p.endpoint(), "http://localhost:1234/v1/chat/completions");
    }

    #[test]
    fn parse_openai_response_choice_zero() {
        let json = r#"{"choices":[{"message":{"role":"assistant","content":"ciao"}}],"usage":{"total_tokens":15,"completion_tokens":5}}"#;
        let (content, tokens) = parse_openai_response(json).unwrap();
        assert_eq!(content, "ciao");
        // Preferisce completion_tokens.
        assert_eq!(tokens, Some(5));
    }

    #[test]
    fn parse_openai_response_solo_total_tokens() {
        let json = r#"{"choices":[{"message":{"content":"x"}}],"usage":{"total_tokens":12}}"#;
        let (_, tokens) = parse_openai_response(json).unwrap();
        assert_eq!(tokens, Some(12));
    }

    #[test]
    fn parse_openai_response_no_usage() {
        let json = r#"{"choices":[{"message":{"content":"x"}}]}"#;
        let (content, tokens) = parse_openai_response(json).unwrap();
        assert_eq!(content, "x");
        assert_eq!(tokens, None);
    }

    #[test]
    fn parse_openai_response_no_choices_content_vuoto() {
        let json = r#"{"choices":[],"usage":{"total_tokens":0}}"#;
        let (content, tokens) = parse_openai_response(json).unwrap();
        assert_eq!(content, "");
        assert_eq!(tokens, None);
    }

    #[test]
    fn parse_openai_response_json_invalido_e_errore() {
        assert!(parse_openai_response("non-json").is_err());
    }

    #[test]
    fn openai_provider_name() {
        let p = OpenAIProvider::new("k", None);
        assert_eq!(p.name(), "openai");
    }

    // ─────────── Gemini ───────────

    #[test]
    fn gemini_endpoint_e_default_url() {
        let p = GeminiProvider::new("k", None);
        assert_eq!(
            p.endpoint("gemini-2.5-flash"),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent"
        );
        assert_eq!(p.api_key, "k");
    }

    #[test]
    fn gemini_endpoint_custom_url() {
        let p = GeminiProvider::new("k", Some("https://my-proxy.local/".into()));
        assert_eq!(
            p.endpoint("gemini-2.5-pro"),
            "https://my-proxy.local/v1beta/models/gemini-2.5-pro:generateContent"
        );
    }

    #[test]
    fn gemini_endpoint_empty_url_usa_default() {
        let p = GeminiProvider::new("k", Some("  ".into()));
        assert_eq!(
            p.endpoint("g"),
            "https://generativelanguage.googleapis.com/v1beta/models/g:generateContent"
        );
    }

    #[test]
    fn parse_gemini_response_un_part() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"ciao"}]}}],"usageMetadata":{"promptTokenCount":3,"candidatesTokenCount":5,"totalTokenCount":8}}"#;
        let (content, tokens) = parse_gemini_response(json).unwrap();
        assert_eq!(content, "ciao");
        assert_eq!(tokens, Some(5));
    }

    #[test]
    fn parse_gemini_response_parts_multipli_concatena() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"prima"},{"text":" seconda"}]}}],"usageMetadata":{"candidatesTokenCount":7}}"#;
        let (content, tokens) = parse_gemini_response(json).unwrap();
        assert_eq!(content, "prima seconda");
        assert_eq!(tokens, Some(7));
    }

    #[test]
    fn parse_gemini_response_fallback_total_tokens() {
        // Se candidatesTokenCount è 0/missing, usa totalTokenCount.
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"x"}]}}],"usageMetadata":{"totalTokenCount":12}}"#;
        let (_, tokens) = parse_gemini_response(json).unwrap();
        assert_eq!(tokens, Some(12));
    }

    #[test]
    fn parse_gemini_response_no_usage_e_no_tokens() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"x"}]}}]}"#;
        let (content, tokens) = parse_gemini_response(json).unwrap();
        assert_eq!(content, "x");
        assert_eq!(tokens, None);
    }

    #[test]
    fn parse_gemini_response_no_candidates_content_vuoto() {
        let json = r#"{"candidates":[]}"#;
        let (content, _) = parse_gemini_response(json).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn parse_gemini_response_json_invalido_e_errore() {
        assert!(parse_gemini_response("non-json").is_err());
    }

    #[test]
    fn gemini_provider_name() {
        let p = GeminiProvider::new("k", None);
        assert_eq!(p.name(), "gemini");
    }

    #[test]
    fn istanzia_provider_gemini_richiede_api_key() {
        let cfg = ProviderConfigItem {
            provider: "gemini".to_string(),
            api_key: None,
            base_url: None,
            default_model: None,
            abilitato: true,
            creato_a: "2026-05-07T00:00:00Z".to_string(),
            aggiornato_a: "2026-05-07T00:00:00Z".to_string(),
        };
        assert!(istanzia_provider(&cfg).is_err());
    }

    #[test]
    fn istanzia_provider_gemini_con_api_key_ok() {
        let cfg = ProviderConfigItem {
            provider: "gemini".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: None,
            default_model: None,
            abilitato: true,
            creato_a: "2026-05-07T00:00:00Z".to_string(),
            aggiornato_a: "2026-05-07T00:00:00Z".to_string(),
        };
        let p = istanzia_provider(&cfg).unwrap();
        assert_eq!(p.name(), "gemini");
    }

    // ─────────── ProviderConfig (storage) ───────────

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    #[test]
    fn config_lista_db_vuoto_e_vec_vuoto() {
        let conn = db_test();
        let r = config_lista_pure(&conn).unwrap();
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn config_salva_inserisce_e_lista_lo_ritorna() {
        let conn = db_test();
        let input = ProviderConfigInput {
            provider: "anthropic".into(),
            api_key: Some("sk-ant-xyz".into()),
            base_url: None,
            default_model: Some("claude-sonnet-4.6".into()),
            abilitato: true,
        };
        config_salva_pure(&conn, &input, true).unwrap();
        let r = config_lista_pure(&conn).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].provider, "anthropic");
        // La lista NON espone la api_key.
        assert!(r[0].api_key.is_none());
        assert_eq!(r[0].default_model.as_deref(), Some("claude-sonnet-4.6"));
        assert!(r[0].abilitato);
    }

    #[test]
    fn config_carica_completa_include_api_key() {
        let conn = db_test();
        let input = ProviderConfigInput {
            provider: "openai".into(),
            api_key: Some("sk-test".into()),
            base_url: None,
            default_model: None,
            abilitato: true,
        };
        config_salva_pure(&conn, &input, true).unwrap();
        let cfg = config_carica_completa(&conn, "openai").unwrap();
        assert_eq!(cfg.api_key.as_deref(), Some("sk-test"));
    }

    #[test]
    fn config_salva_provider_invalido_e_errore() {
        let conn = db_test();
        let input = ProviderConfigInput {
            provider: "fantasma".into(),
            api_key: None,
            base_url: None,
            default_model: None,
            abilitato: true,
        };
        let r = config_salva_pure(&conn, &input, true);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("non riconosciuto"));
    }

    #[test]
    fn config_salva_upsert_aggiorna_senza_perdere_chiave() {
        let conn = db_test();
        // Primo insert con chiave.
        config_salva_pure(
            &conn,
            &ProviderConfigInput {
                provider: "anthropic".into(),
                api_key: Some("sk-ant-VECCHIA".into()),
                base_url: None,
                default_model: Some("claude-3.7".into()),
                abilitato: true,
            },
            true,
        )
        .unwrap();
        // Upsert SENZA passare api_key (None) — deve preservarla.
        config_salva_pure(
            &conn,
            &ProviderConfigInput {
                provider: "anthropic".into(),
                api_key: None,
                base_url: None,
                default_model: Some("claude-sonnet-4.6".into()),
                abilitato: true,
            },
            true,
        )
        .unwrap();
        let cfg = config_carica_completa(&conn, "anthropic").unwrap();
        assert_eq!(cfg.api_key.as_deref(), Some("sk-ant-VECCHIA"));
        assert_eq!(cfg.default_model.as_deref(), Some("claude-sonnet-4.6"));
    }

    #[test]
    fn config_salva_upsert_aggiorna_chiave_se_passata() {
        let conn = db_test();
        config_salva_pure(
            &conn,
            &ProviderConfigInput {
                provider: "openai".into(),
                api_key: Some("sk-VECCHIA".into()),
                base_url: None,
                default_model: None,
                abilitato: true,
            },
            true,
        )
        .unwrap();
        config_salva_pure(
            &conn,
            &ProviderConfigInput {
                provider: "openai".into(),
                api_key: Some("sk-NUOVA".into()),
                base_url: None,
                default_model: None,
                abilitato: true,
            },
            true,
        )
        .unwrap();
        let cfg = config_carica_completa(&conn, "openai").unwrap();
        assert_eq!(cfg.api_key.as_deref(), Some("sk-NUOVA"));
    }

    #[test]
    fn config_elimina_rimuove_dalla_lista() {
        let conn = db_test();
        config_salva_pure(
            &conn,
            &ProviderConfigInput {
                provider: "anthropic".into(),
                api_key: Some("k".into()),
                base_url: None,
                default_model: None,
                abilitato: true,
            },
            true,
        )
        .unwrap();
        config_elimina_pure(&conn, "anthropic").unwrap();
        assert_eq!(config_lista_pure(&conn).unwrap().len(), 0);
    }

    #[test]
    fn config_elimina_su_provider_inesistente_e_errore() {
        let conn = db_test();
        let r = config_elimina_pure(&conn, "anthropic");
        assert!(r.is_err());
    }

    // ─────────── #456: rifiuto API key su vault non cifrato ───────────

    #[test]
    fn config_salva_nuova_api_key_su_vault_non_cifrato_e_rifiutata() {
        let conn = db_test();
        let input = ProviderConfigInput {
            provider: "anthropic".into(),
            api_key: Some("sk-ant-xyz".into()),
            base_url: None,
            default_model: None,
            abilitato: true,
        };
        let r = config_salva_pure(&conn, &input, false);
        assert!(matches!(r, Err(PapErrore::VaultNonCifrato)));
        // Nessuna riga deve essere stata scritta.
        assert_eq!(config_lista_pure(&conn).unwrap().len(), 0);
    }

    #[test]
    fn config_salva_senza_api_key_su_vault_non_cifrato_e_permessa() {
        // Aggiornare base_url/default_model senza toccare la chiave deve
        // funzionare anche su vault non cifrato: non stiamo persistendo
        // nessun nuovo segreto in chiaro.
        let conn = db_test();
        let input = ProviderConfigInput {
            provider: "ollama".into(),
            api_key: None,
            base_url: Some("http://localhost:11434".into()),
            default_model: Some("llama3.1".into()),
            abilitato: true,
        };
        config_salva_pure(&conn, &input, false).unwrap();
        let r = config_lista_pure(&conn).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].provider, "ollama");
    }

    #[test]
    fn config_salva_api_key_vuota_su_vault_non_cifrato_e_permessa() {
        // Stringa vuota == "non toccare la chiave" (manteni_chiave=true),
        // stesso trattamento di None.
        let conn = db_test();
        let input = ProviderConfigInput {
            provider: "ollama".into(),
            api_key: Some("".into()),
            base_url: None,
            default_model: None,
            abilitato: true,
        };
        config_salva_pure(&conn, &input, false).unwrap();
        assert_eq!(config_lista_pure(&conn).unwrap().len(), 1);
    }

    #[test]
    fn config_salva_nuova_api_key_su_vault_cifrato_e_permessa() {
        let conn = db_test();
        let input = ProviderConfigInput {
            provider: "anthropic".into(),
            api_key: Some("sk-ant-xyz".into()),
            base_url: None,
            default_model: None,
            abilitato: true,
        };
        config_salva_pure(&conn, &input, true).unwrap();
        assert_eq!(config_lista_pure(&conn).unwrap().len(), 1);
    }

    #[test]
    fn vault_non_cifrato_messaggio_menziona_cifratura() {
        let msg = PapErrore::VaultNonCifrato.to_string();
        assert!(msg.contains("cifr"), "Il messaggio deve menzionare la cifratura: {msg}");
    }

    // ─────────── #457: validazione base_url (SSRF / key exfiltration) ───────────

    #[test]
    fn valida_base_url_https_ok() {
        assert!(valida_base_url("https://api.anthropic.com").is_ok());
        assert!(valida_base_url("https://my-proxy.example.com:8443/v1").is_ok());
    }

    #[test]
    fn valida_base_url_http_localhost_ok() {
        assert!(valida_base_url("http://localhost:11434").is_ok());
        assert!(valida_base_url("http://127.0.0.1:11434").is_ok());
        assert!(valida_base_url("http://[::1]:11434").is_ok());
    }

    #[test]
    fn valida_base_url_http_remoto_rifiutato() {
        let r = valida_base_url("http://example.com");
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("http://"));
    }

    #[test]
    fn valida_base_url_file_scheme_rifiutato() {
        let r = valida_base_url("file:///etc/passwd");
        assert!(r.is_err());
    }

    #[test]
    fn valida_base_url_garbage_rifiutato() {
        assert!(valida_base_url("not-a-url").is_err());
        assert!(valida_base_url("ftp://example.com").is_err());
        assert!(valida_base_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn valida_base_url_stringa_vuota_ok() {
        // Vuota/whitespace = "usa il default del provider".
        assert!(valida_base_url("").is_ok());
        assert!(valida_base_url("   ").is_ok());
    }

    #[test]
    fn valida_base_url_https_senza_host_rifiutato() {
        assert!(valida_base_url("https://").is_err());
    }

    // ─── Review CRITICAL: bypass via userinfo (user:pass@host) ───

    #[test]
    fn valida_base_url_userinfo_ipv4_bypass_rifiutato() {
        // Il validatore precedente vedeva host="127.0.0.1" (username),
        // ma il client HTTP si connette DAVVERO a evil.com (authority
        // reale dopo la @) — la API key sarebbe stata inviata in chiaro
        // a evil.com. Deve essere rifiutato SEMPRE, non "accettato perché
        // sembra localhost".
        let r = valida_base_url("http://127.0.0.1:11434@evil.com/");
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("credenziali"));
    }

    #[test]
    fn valida_base_url_userinfo_localhost_bypass_rifiutato() {
        let r = valida_base_url("http://localhost:1@evil.com/");
        assert!(r.is_err());
    }

    #[test]
    fn valida_base_url_userinfo_ipv6_bypass_rifiutato() {
        let r = valida_base_url("http://[::1]:80@evil.com/");
        assert!(r.is_err());
    }

    #[test]
    fn valida_base_url_userinfo_su_host_locale_legittimo_rifiutato_comunque() {
        // Anche se l'host DOPO la @ è realmente locale, le credenziali
        // nell'URL non hanno un caso d'uso legittimo per un provider AI:
        // rifiutiamo a prescindere per non lasciare aperta la sintassi.
        let r = valida_base_url("http://user:pass@localhost/");
        assert!(r.is_err());
    }

    // ─── Review CRITICAL: host camuffati (non letterali) sempre rifiutati ───

    #[test]
    fn valida_base_url_sottodominio_di_localhost_rifiutato() {
        // "localhost.attacker.com" NON è "localhost": un confronto per
        // suffisso/substring sarebbe stato un'altra falla.
        let r = valida_base_url("http://localhost.attacker.com/");
        assert!(r.is_err());
    }

    #[test]
    fn valida_base_url_ipv4_decimale_rifiutato() {
        // 2130706433 == 127.0.0.1 in decimale a 32 bit: WHATWG/`url`
        // normalizzerebbero silenziosamente a "127.0.0.1" se usassimo
        // un controllo semantico (is_loopback()). L'allowlist testuale
        // stretta lo rifiuta perché non è uno dei letterali consentiti.
        let r = valida_base_url("http://2130706433/");
        assert!(r.is_err());
    }

    #[test]
    fn valida_base_url_0000_rifiutato() {
        // 0.0.0.0 non è loopback (non è nemmeno in HOST_LOCALI_CONSENTITI).
        let r = valida_base_url("http://0.0.0.0/");
        assert!(r.is_err());
    }

    #[test]
    fn valida_base_url_trailing_dot_rifiutato() {
        // "localhost." (FQDN con punto finale) non è il letterale esatto
        // "localhost": rifiutato per rigore, nessun uso legittimo lo richiede.
        let r = valida_base_url("http://localhost./");
        assert!(r.is_err());
    }

    #[test]
    fn valida_base_url_uppercase_host_accettato() {
        // Case-insensitivity è normalizzazione DNS legittima (non un
        // bypass): "LOCALHOST" e "localhost" sono lo stesso host.
        assert!(valida_base_url("http://LOCALHOST:11434/").is_ok());
        assert!(valida_base_url("http://LoCaLhOsT/").is_ok());
    }

    #[test]
    fn config_salva_base_url_invalido_e_rifiutato() {
        let conn = db_test();
        let input = ProviderConfigInput {
            provider: "ollama".into(),
            api_key: None,
            base_url: Some("file:///etc/passwd".into()),
            default_model: None,
            abilitato: true,
        };
        let r = config_salva_pure(&conn, &input, false);
        assert!(r.is_err());
        assert_eq!(config_lista_pure(&conn).unwrap().len(), 0);
    }

    #[test]
    fn config_salva_base_url_http_remoto_e_rifiutato() {
        let conn = db_test();
        let input = ProviderConfigInput {
            provider: "openai-compat".into(),
            api_key: None,
            base_url: Some("http://malicious.example.com".into()),
            default_model: None,
            abilitato: true,
        };
        let r = config_salva_pure(&conn, &input, false);
        assert!(r.is_err());
    }

    #[test]
    fn provider_ollama_genera_base_url_invalido_rifiutato_prima_della_chiamata_http() {
        // Non possiamo verificare l'assenza di chiamate HTTP direttamente,
        // ma possiamo verificare che l'errore di validazione arrivi PRIMA
        // di qualunque tentativo di rete (nessun timeout HTTP nel test).
        let r = provider_ollama_genera(
            "ciao".into(),
            "llama3.1".into(),
            Some("file:///etc/passwd".into()),
        );
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("base_url"));
    }

    #[test]
    fn istanzia_provider_anthropic_con_chiave() {
        let cfg = ProviderConfigItem {
            provider: "anthropic".into(),
            api_key: Some("k".into()),
            base_url: None,
            default_model: None,
            abilitato: true,
            creato_a: "x".into(),
            aggiornato_a: "x".into(),
        };
        let p = istanzia_provider(&cfg).unwrap();
        assert_eq!(p.name(), "anthropic");
    }

    #[test]
    fn istanzia_provider_anthropic_senza_chiave_e_errore() {
        let cfg = ProviderConfigItem {
            provider: "anthropic".into(),
            api_key: None,
            base_url: None,
            default_model: None,
            abilitato: true,
            creato_a: "x".into(),
            aggiornato_a: "x".into(),
        };
        let r = istanzia_provider(&cfg);
        assert!(r.is_err());
    }

    #[test]
    fn istanzia_provider_ollama_no_key_richiesta() {
        let cfg = ProviderConfigItem {
            provider: "ollama".into(),
            api_key: None,
            base_url: None,
            default_model: None,
            abilitato: true,
            creato_a: "x".into(),
            aggiornato_a: "x".into(),
        };
        let p = istanzia_provider(&cfg).unwrap();
        assert_eq!(p.name(), "ollama");
    }

    #[test]
    fn istanzia_provider_kind_sconosciuto_e_errore() {
        let cfg = ProviderConfigItem {
            provider: "fantasma".into(),
            api_key: Some("k".into()),
            base_url: None,
            default_model: None,
            abilitato: true,
            creato_a: "x".into(),
            aggiornato_a: "x".into(),
        };
        let r = istanzia_provider(&cfg);
        assert!(r.is_err());
    }
}
