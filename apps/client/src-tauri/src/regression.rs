// Modulo regression — Fase 4 Step 8 (golden examples + regression testing).
//
// Sub-step 8a (questa PR): schema + CRUD per `PromptGoldens`. La parte di
// run/observation/provider AI atterra in PR successive (8b-8g).
//
// Un *golden* è un caso di test salvato sul prompt: dato un certo
// input (variabili compilate), ci si aspetta un certo output. PaP
// misura nel tempo se il prompt produce ancora output coerente.
//
// MVP scope:
// - 4 comandi Tauri: golden_crea / golden_aggiorna / golden_elimina / golden_lista
// - Validazione client-side: SimilarityFn ∈ enum, soglia ∈ [0,1], JSON
//   valido per InputVars
// - Soft-delete con `DeletedAt`, coerente col resto dello schema

use regex::Regex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tauri::State;

use crate::embeddings::EmbeddingsState;
use crate::errore::PapErrore;
use crate::provider_ai::{AIProvider, OllamaProvider};
use crate::similarity;
use crate::vault::VaultState;

const USER_LOCALE: &str = "usr-locale";

const SIMILARITY_FN_VALIDE: &[&str] = &["cosine", "llm-judge", "exact-match", "regex"];

#[derive(Debug, Clone, Serialize)]
pub struct Golden {
    pub id: String,
    pub prompt_id: String,
    pub etichetta: String,
    pub input_vars: String,
    pub expected_output: String,
    pub similarity_fn: String,
    pub soglia_tolleranza: f64,
    pub creato_a: String,
    pub aggiornato_a: String,
}

#[derive(Debug, Deserialize)]
pub struct NuovoGolden {
    pub prompt_id: String,
    pub etichetta: String,
    pub input_vars: String,
    pub expected_output: String,
    #[serde(default = "default_similarity_fn")]
    pub similarity_fn: String,
    #[serde(default = "default_soglia")]
    pub soglia_tolleranza: f64,
}

#[derive(Debug, Deserialize)]
pub struct AggiornamentoGolden {
    pub id: String,
    pub etichetta: String,
    pub input_vars: String,
    pub expected_output: String,
    pub similarity_fn: String,
    pub soglia_tolleranza: f64,
}

fn default_similarity_fn() -> String {
    "cosine".to_string()
}

fn default_soglia() -> f64 {
    0.85
}

fn genera_id() -> String {
    genera_id_con_prefix("gld")
}

fn genera_id_con_prefix(prefix: &str) -> String {
    use rand::RngCore;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut rnd = [0u8; 4];
    rand::rngs::OsRng.fill_bytes(&mut rnd);
    format!(
        "{prefix}-{:012x}{:02x}{:02x}{:02x}{:02x}",
        ts, rnd[0], rnd[1], rnd[2], rnd[3]
    )
}

fn re_segnaposto() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap())
}

/// Sostituisce i segnaposti `{{nome}}` nel `body` con i valori in
/// `input_vars` (JSON object). Segnaposti senza match sono lasciati
/// invariati. NON espande gli `{{import "..."}}` (la regex `\w+` non
/// matcha le stringhe quotate).
pub(crate) fn compila_per_golden(body: &str, input_vars_json: &str) -> Result<String, PapErrore> {
    let vars: serde_json::Value = serde_json::from_str(input_vars_json)
        .map_err(|e| PapErrore::Generico(format!("InputVars JSON invalido: {e}")))?;
    let map = vars
        .as_object()
        .ok_or_else(|| PapErrore::Generico("InputVars deve essere un oggetto JSON".into()))?;
    let re = re_segnaposto();
    let result = re.replace_all(body, |caps: &regex::Captures| {
        let name = &caps[1];
        match map.get(name) {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None => caps[0].to_string(),
        }
    });
    Ok(result.into_owned())
}

fn valida(input: &str, expected: &str, similarity_fn: &str, soglia: f64) -> Result<(), PapErrore> {
    if input.trim().is_empty() {
        return Err(PapErrore::Generico(
            "InputVars vuoto: passa almeno '{}' come JSON".into(),
        ));
    }
    serde_json::from_str::<serde_json::Value>(input)
        .map_err(|e| PapErrore::Generico(format!("InputVars non è JSON valido: {e}")))?;
    if expected.trim().is_empty() {
        return Err(PapErrore::Generico("ExpectedOutput vuoto".into()));
    }
    if !SIMILARITY_FN_VALIDE.contains(&similarity_fn) {
        return Err(PapErrore::Generico(format!(
            "SimilarityFn '{similarity_fn}' non valida. Ammesse: {:?}",
            SIMILARITY_FN_VALIDE
        )));
    }
    if !(0.0..=1.0).contains(&soglia) {
        return Err(PapErrore::Generico(format!(
            "Soglia tolleranza {soglia} fuori range [0,1]"
        )));
    }
    Ok(())
}

fn verifica_prompt_esiste(conn: &Connection, prompt_id: &str) -> Result<(), PapErrore> {
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
            [prompt_id],
            |_| Ok(true),
        )
        .unwrap_or(false);
    if !exists {
        return Err(PapErrore::Generico(format!(
            "Prompt '{prompt_id}' non esiste o è eliminato"
        )));
    }
    Ok(())
}

// ─────────── Helper pure (testabili senza Tauri State) ───────────

pub(crate) fn crea_pure(conn: &Connection, dati: &NuovoGolden) -> Result<String, PapErrore> {
    valida(
        &dati.input_vars,
        &dati.expected_output,
        &dati.similarity_fn,
        dati.soglia_tolleranza,
    )?;
    verifica_prompt_esiste(conn, &dati.prompt_id)?;
    let id = genera_id();
    conn.execute(
        "INSERT INTO PromptGoldens (
            Id, PromptId, Etichetta, InputVars, ExpectedOutput,
            SimilarityFn, SoglieTolleranza, CreatedAt, UpdatedAt
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'), datetime('now'))",
        params![
            id,
            dati.prompt_id,
            dati.etichetta,
            dati.input_vars,
            dati.expected_output,
            dati.similarity_fn,
            dati.soglia_tolleranza,
        ],
    )?;
    crate::audit::registra(conn, "golden.creato", "Golden", &id, Some(&dati.etichetta));
    Ok(id)
}

pub(crate) fn aggiorna_pure(
    conn: &Connection,
    dati: &AggiornamentoGolden,
) -> Result<(), PapErrore> {
    valida(
        &dati.input_vars,
        &dati.expected_output,
        &dati.similarity_fn,
        dati.soglia_tolleranza,
    )?;
    let n = conn.execute(
        "UPDATE PromptGoldens
         SET Etichetta = ?2, InputVars = ?3, ExpectedOutput = ?4,
             SimilarityFn = ?5, SoglieTolleranza = ?6,
             UpdatedAt = datetime('now')
         WHERE Id = ?1 AND DeletedAt IS NULL",
        params![
            dati.id,
            dati.etichetta,
            dati.input_vars,
            dati.expected_output,
            dati.similarity_fn,
            dati.soglia_tolleranza,
        ],
    )?;
    if n == 0 {
        return Err(PapErrore::Generico(format!(
            "Golden '{}' non trovato o già eliminato",
            dati.id
        )));
    }
    crate::audit::registra(
        conn,
        "golden.aggiornato",
        "Golden",
        &dati.id,
        Some(&dati.etichetta),
    );
    Ok(())
}

pub(crate) fn elimina_pure(conn: &Connection, id: &str) -> Result<(), PapErrore> {
    let n = conn.execute(
        "UPDATE PromptGoldens
         SET DeletedAt = datetime('now'), UpdatedAt = datetime('now')
         WHERE Id = ?1 AND DeletedAt IS NULL",
        [id],
    )?;
    if n == 0 {
        return Err(PapErrore::Generico(format!(
            "Golden '{id}' non trovato o già eliminato"
        )));
    }
    crate::audit::registra(conn, "golden.eliminato", "Golden", id, None);
    Ok(())
}

pub(crate) fn lista_pure(conn: &Connection, prompt_id: &str) -> Result<Vec<Golden>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT Id, PromptId, Etichetta, InputVars, ExpectedOutput,
                SimilarityFn, SoglieTolleranza, CreatedAt, UpdatedAt
         FROM PromptGoldens
         WHERE PromptId = ?1 AND DeletedAt IS NULL
         ORDER BY CreatedAt ASC, Id ASC",
    )?;
    let rows: Vec<Golden> = stmt
        .query_map([prompt_id], |r| {
            Ok(Golden {
                id: r.get(0)?,
                prompt_id: r.get(1)?,
                etichetta: r.get(2)?,
                input_vars: r.get(3)?,
                expected_output: r.get(4)?,
                similarity_fn: r.get(5)?,
                soglia_tolleranza: r.get(6)?,
                creato_a: r.get(7)?,
                aggiornato_a: r.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

// ─────────── Esecuzione golden + observations (Step 8d) ───────────

#[derive(Debug, Clone, Serialize)]
pub struct Observation {
    pub id: String,
    pub prompt_version_id: String,
    pub golden_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub actual_output: String,
    pub similarita: Option<f64>,
    pub passed: bool,
    pub latenza_ms: Option<u64>,
    pub tokens_used: Option<u32>,
    pub costo_stimato: Option<f64>,
    pub errore: Option<String>,
    pub ran_at: String,
    pub ran_by: String,
}

/// Restituisce l'`Id` di `PromptVersions` corrispondente alla `Version`
/// corrente del prompt. Se non esiste ancora una row in `PromptVersions`
/// (vault legacy senza versioning popolato), genera un id placeholder
/// derivato da `prompt_id + version` per coerenza referenziale logica.
fn current_version_id(conn: &Connection, prompt_id: &str) -> Result<String, PapErrore> {
    let id: Option<String> = conn
        .query_row(
            "SELECT pv.Id FROM PromptVersions pv
             JOIN Prompts p ON p.Id = pv.PromptId AND p.Version = pv.Version
             WHERE pv.PromptId = ?1
             ORDER BY pv.Version DESC
             LIMIT 1",
            [prompt_id],
            |r| r.get(0),
        )
        .ok();
    match id {
        Some(v) => Ok(v),
        None => Err(PapErrore::Generico(format!(
            "PromptVersions vuoto per '{prompt_id}'. Salva il prompt almeno una volta prima di eseguire un golden."
        ))),
    }
}

fn carica_golden(conn: &Connection, golden_id: &str) -> Result<Golden, PapErrore> {
    conn.query_row(
        "SELECT Id, PromptId, Etichetta, InputVars, ExpectedOutput,
                SimilarityFn, SoglieTolleranza, CreatedAt, UpdatedAt
         FROM PromptGoldens
         WHERE Id = ?1 AND DeletedAt IS NULL",
        [golden_id],
        |r| {
            Ok(Golden {
                id: r.get(0)?,
                prompt_id: r.get(1)?,
                etichetta: r.get(2)?,
                input_vars: r.get(3)?,
                expected_output: r.get(4)?,
                similarity_fn: r.get(5)?,
                soglia_tolleranza: r.get(6)?,
                creato_a: r.get(7)?,
                aggiornato_a: r.get(8)?,
            })
        },
    )
    .map_err(|_| PapErrore::Generico(format!("Golden '{golden_id}' non trovato")))
}

fn carica_prompt_body(conn: &Connection, prompt_id: &str) -> Result<String, PapErrore> {
    conn.query_row(
        "SELECT Body FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
        [prompt_id],
        |r| r.get(0),
    )
    .map_err(|_| PapErrore::Generico(format!("Prompt '{prompt_id}' non trovato")))
}

fn insert_observation(conn: &Connection, obs: &Observation) -> Result<(), PapErrore> {
    conn.execute(
        "INSERT INTO PromptRunObservations (
            Id, PromptVersionId, GoldenId, Provider, Model, ActualOutput,
            Similarita, Passed, LatenzaMs, TokensUsed, CostoStimato,
            Errore, RanAt, RanBy
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            obs.id,
            obs.prompt_version_id,
            obs.golden_id,
            obs.provider,
            obs.model,
            obs.actual_output,
            obs.similarita,
            obs.passed as i64,
            obs.latenza_ms.map(|v| v as i64),
            obs.tokens_used.map(|v| v as i64),
            obs.costo_stimato,
            obs.errore,
            obs.ran_at,
            obs.ran_by,
        ],
    )?;
    Ok(())
}

/// Orchestrazione end-to-end di un golden: carica → compila body con
/// input_vars → chiama provider → calcola similarità → salva observation.
///
/// Il caller passa `provider` come trait object così la funzione resta
/// testabile con un mock provider senza spinning-up Ollama vero.
///
/// Errori del provider (HTTP, network, modello sconosciuto) vengono
/// catturati e salvati come observation con `Errore` valorizzato e
/// `Passed = false` — non bubbled-up al chiamante. Errori di lookup DB
/// (golden inesistente, prompt eliminato) sono invece propagati.
pub(crate) fn esegui_pure_con_provider(
    conn: &Connection,
    rt: Option<&EmbeddingsState>,
    golden_id: &str,
    provider: &dyn AIProvider,
    model: &str,
    user_id: &str,
) -> Result<Observation, PapErrore> {
    let golden = carica_golden(conn, golden_id)?;
    let body = carica_prompt_body(conn, &golden.prompt_id)?;
    let prompt_compilato = compila_per_golden(&body, &golden.input_vars)?;
    let prompt_version_id = current_version_id(conn, &golden.prompt_id)?;

    let ran_at: String = conn.query_row("SELECT datetime('now')", [], |r| r.get(0))?;
    let id = genera_id_con_prefix("obs");

    let observation = match provider.generate(&prompt_compilato, model) {
        Ok(out) => {
            let sim_result = similarity::calcola(
                &golden.similarity_fn,
                &golden.expected_output,
                &out.content,
                rt,
            );
            let (similarita, errore) = match sim_result {
                Ok(s) => (Some(s), None),
                Err(e) => (None, Some(format!("Similarity: {e}"))),
            };
            let passed = similarita
                .map(|s| {
                    similarity::is_passed(&golden.similarity_fn, s, golden.soglia_tolleranza)
                })
                .unwrap_or(false);
            Observation {
                id,
                prompt_version_id,
                golden_id: Some(golden.id.clone()),
                provider: provider.name().to_string(),
                model: out.model,
                actual_output: out.content,
                similarita,
                passed,
                latenza_ms: Some(out.latency_ms),
                tokens_used: out.tokens_used,
                costo_stimato: None,
                errore,
                ran_at,
                ran_by: user_id.to_string(),
            }
        }
        Err(e) => Observation {
            id,
            prompt_version_id,
            golden_id: Some(golden.id.clone()),
            provider: provider.name().to_string(),
            model: model.to_string(),
            actual_output: String::new(),
            similarita: None,
            passed: false,
            latenza_ms: None,
            tokens_used: None,
            costo_stimato: None,
            errore: Some(format!("Provider: {e}")),
            ran_at,
            ran_by: user_id.to_string(),
        },
    };

    insert_observation(conn, &observation)?;
    crate::audit::registra(
        conn,
        "golden.eseguito",
        "Observation",
        &observation.id,
        Some(&golden.etichetta),
    );
    Ok(observation)
}

// ─────────── Tauri commands (delegano agli helper) ───────────

#[tauri::command]
pub fn golden_crea(
    dati: NuovoGolden,
    state: State<'_, VaultState>,
) -> Result<String, PapErrore> {
    state.with_conn(|conn| crea_pure(conn, &dati))
}

#[tauri::command]
pub fn golden_aggiorna(
    dati: AggiornamentoGolden,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| aggiorna_pure(conn, &dati))
}

#[tauri::command]
pub fn golden_elimina(id: String, state: State<'_, VaultState>) -> Result<(), PapErrore> {
    state.with_conn(|conn| elimina_pure(conn, &id))
}

#[tauri::command]
pub fn golden_lista(
    prompt_id: String,
    state: State<'_, VaultState>,
) -> Result<Vec<Golden>, PapErrore> {
    state.with_conn(|conn| lista_pure(conn, &prompt_id))
}

/// Esegue un golden via il provider scelto. Solo Ollama supportato in
/// 8d (provider remote in 8f). `model` è il nome del modello come noto
/// al provider (es. "llama3.2"). `base_url` è opzionale per override
/// dell'endpoint Ollama (default `http://localhost:11434`).
#[tauri::command]
pub fn golden_esegui(
    golden_id: String,
    provider_kind: String,
    model: String,
    base_url: Option<String>,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<Observation, PapErrore> {
    if provider_kind != "ollama" {
        return Err(PapErrore::Generico(format!(
            "Provider '{provider_kind}' non supportato in v0.4 alpha (solo 'ollama'). Provider remote arrivano in 8f."
        )));
    }
    let provider: OllamaProvider = match base_url {
        Some(u) if !u.trim().is_empty() => OllamaProvider::new(u),
        _ => OllamaProvider::default(),
    };
    state.with_conn(|conn| {
        esegui_pure_con_provider(conn, Some(&rt_state), &golden_id, &provider, &model, USER_LOCALE)
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use rusqlite::params;

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        // Inserisci un prompt-padre per i golden.
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                Visibility, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-1', 'ws-personale', 'usr-locale', 'p', 'body', 'private', 1,
                     datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn
    }

    fn inserisci_golden(
        conn: &Connection,
        id: &str,
        prompt_id: &str,
        etichetta: &str,
        similarity_fn: &str,
    ) {
        conn.execute(
            "INSERT INTO PromptGoldens (Id, PromptId, Etichetta, InputVars,
                ExpectedOutput, SimilarityFn, SoglieTolleranza,
                CreatedAt, UpdatedAt)
             VALUES (?1, ?2, ?3, '{\"contesto\":\"x\"}', 'output atteso',
                     ?4, 0.85, datetime('now'), datetime('now'))",
            params![id, prompt_id, etichetta, similarity_fn],
        )
        .unwrap();
    }

    #[test]
    fn valida_input_vars_json_invalido() {
        let r = valida("non-json", "out", "cosine", 0.85);
        assert!(r.is_err());
    }

    #[test]
    fn valida_similarity_fn_sconosciuta() {
        let r = valida("{}", "out", "fuzzy-magic", 0.85);
        assert!(r.is_err());
    }

    #[test]
    fn valida_soglia_fuori_range() {
        assert!(valida("{}", "out", "cosine", -0.1).is_err());
        assert!(valida("{}", "out", "cosine", 1.5).is_err());
        assert!(valida("{}", "out", "cosine", 0.0).is_ok());
        assert!(valida("{}", "out", "cosine", 1.0).is_ok());
    }

    #[test]
    fn valida_expected_vuoto() {
        assert!(valida("{}", "", "cosine", 0.85).is_err());
        assert!(valida("{}", "   ", "cosine", 0.85).is_err());
    }

    #[test]
    fn valida_similarity_fn_tutte_quelle_attese() {
        for fn_nome in ["cosine", "llm-judge", "exact-match", "regex"] {
            assert!(valida("{}", "out", fn_nome, 0.85).is_ok());
        }
    }

    #[test]
    fn id_generato_ha_prefix_gld() {
        let id = genera_id();
        assert!(id.starts_with("gld-"));
    }

    // ─────────── Test scenari DB ───────────

    #[test]
    fn migrazioni_creano_tabelle_golden_e_observations() {
        let conn = db_test();
        // V008 + V009 atterrate.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type='table' AND name IN ('PromptGoldens','PromptRunObservations')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn lista_golden_per_prompt_ordina_per_created_at() {
        let conn = db_test();
        inserisci_golden(&conn, "gld-1", "prm-1", "primo", "cosine");
        // Sleep di 1ms per garantire ordering deterministico via datetime('now').
        // SQLite datetime ha granularità al secondo, quindi gli inserti
        // ravvicinati potrebbero avere stesso CreatedAt — ordering tolerant.
        inserisci_golden(&conn, "gld-2", "prm-1", "secondo", "exact-match");
        inserisci_golden(&conn, "gld-3", "prm-1", "terzo", "regex");

        let mut stmt = conn
            .prepare(
                "SELECT Id FROM PromptGoldens
                 WHERE PromptId = 'prm-1' AND DeletedAt IS NULL
                 ORDER BY CreatedAt ASC, Id ASC",
            )
            .unwrap();
        let ids: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(ids, vec!["gld-1", "gld-2", "gld-3"]);
    }

    #[test]
    fn check_constraint_similarity_fn_blocca_valori_invalidi() {
        let conn = db_test();
        let r = conn.execute(
            "INSERT INTO PromptGoldens (Id, PromptId, Etichetta, InputVars,
                ExpectedOutput, SimilarityFn, SoglieTolleranza,
                CreatedAt, UpdatedAt)
             VALUES ('gld-x', 'prm-1', 'l', '{}', 'o',
                     'fuzzy', 0.85, datetime('now'), datetime('now'))",
            [],
        );
        assert!(r.is_err(), "CHECK su SimilarityFn deve rifiutare 'fuzzy'");
    }

    #[test]
    fn check_constraint_soglia_blocca_fuori_range() {
        let conn = db_test();
        let r = conn.execute(
            "INSERT INTO PromptGoldens (Id, PromptId, Etichetta, InputVars,
                ExpectedOutput, SimilarityFn, SoglieTolleranza,
                CreatedAt, UpdatedAt)
             VALUES ('gld-x', 'prm-1', 'l', '{}', 'o',
                     'cosine', 1.5, datetime('now'), datetime('now'))",
            [],
        );
        assert!(r.is_err(), "CHECK su SoglieTolleranza deve rifiutare 1.5");
    }

    #[test]
    fn soft_delete_rimuove_da_lista() {
        let conn = db_test();
        inserisci_golden(&conn, "gld-1", "prm-1", "x", "cosine");
        conn.execute(
            "UPDATE PromptGoldens SET DeletedAt = datetime('now') WHERE Id = 'gld-1'",
            [],
        )
        .unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptGoldens
                 WHERE PromptId = 'prm-1' AND DeletedAt IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn observations_table_esiste_con_indici() {
        let conn = db_test();
        let indici: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master
                 WHERE type='index' AND tbl_name='PromptRunObservations'
                 ORDER BY name",
            )
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(indici.iter().any(|i| i.contains("prompt_ranat")));
        assert!(indici.iter().any(|i| i.contains("model_passed")));
    }

    // ─────────── Test helper pure ───────────

    fn nuovo_default(prompt_id: &str, etichetta: &str) -> NuovoGolden {
        NuovoGolden {
            prompt_id: prompt_id.to_string(),
            etichetta: etichetta.to_string(),
            input_vars: r#"{"contesto":"x"}"#.to_string(),
            expected_output: "output atteso".to_string(),
            similarity_fn: "cosine".to_string(),
            soglia_tolleranza: 0.85,
        }
    }

    #[test]
    fn crea_pure_inserisce_e_ritorna_id() {
        let conn = db_test();
        let id = crea_pure(&conn, &nuovo_default("prm-1", "primo")).unwrap();
        assert!(id.starts_with("gld-"));
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptGoldens WHERE PromptId = 'prm-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn crea_pure_fallisce_se_prompt_non_esiste() {
        let conn = db_test();
        let r = crea_pure(&conn, &nuovo_default("prm-fantasma", "x"));
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("non esiste"));
    }

    #[test]
    fn crea_pure_fallisce_se_input_vars_non_json() {
        let conn = db_test();
        let mut dati = nuovo_default("prm-1", "x");
        dati.input_vars = "non-json".to_string();
        assert!(crea_pure(&conn, &dati).is_err());
    }

    #[test]
    fn aggiorna_pure_modifica_campi() {
        let conn = db_test();
        let id = crea_pure(&conn, &nuovo_default("prm-1", "originale")).unwrap();
        let agg = AggiornamentoGolden {
            id: id.clone(),
            etichetta: "modificato".to_string(),
            input_vars: "{}".to_string(),
            expected_output: "nuovo output".to_string(),
            similarity_fn: "exact-match".to_string(),
            soglia_tolleranza: 1.0,
        };
        aggiorna_pure(&conn, &agg).unwrap();
        let lista = lista_pure(&conn, "prm-1").unwrap();
        assert_eq!(lista.len(), 1);
        assert_eq!(lista[0].etichetta, "modificato");
        assert_eq!(lista[0].similarity_fn, "exact-match");
        assert_eq!(lista[0].soglia_tolleranza, 1.0);
    }

    #[test]
    fn aggiorna_pure_fallisce_su_id_inesistente() {
        let conn = db_test();
        let agg = AggiornamentoGolden {
            id: "gld-fantasma".to_string(),
            etichetta: "x".to_string(),
            input_vars: "{}".to_string(),
            expected_output: "y".to_string(),
            similarity_fn: "cosine".to_string(),
            soglia_tolleranza: 0.85,
        };
        let r = aggiorna_pure(&conn, &agg);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("non trovato"));
    }

    #[test]
    fn elimina_pure_marca_deleted_at() {
        let conn = db_test();
        let id = crea_pure(&conn, &nuovo_default("prm-1", "x")).unwrap();
        elimina_pure(&conn, &id).unwrap();
        // Lista non lo include più.
        assert_eq!(lista_pure(&conn, "prm-1").unwrap().len(), 0);
        // Ma la riga è ancora in DB con DeletedAt valorizzato.
        let deleted: Option<String> = conn
            .query_row(
                "SELECT DeletedAt FROM PromptGoldens WHERE Id = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(deleted.is_some());
    }

    #[test]
    fn elimina_pure_idempotente_fallisce_su_seconda() {
        let conn = db_test();
        let id = crea_pure(&conn, &nuovo_default("prm-1", "x")).unwrap();
        assert!(elimina_pure(&conn, &id).is_ok());
        let r = elimina_pure(&conn, &id);
        assert!(r.is_err(), "seconda elimina su soft-deleted deve fallire");
    }

    #[test]
    fn lista_pure_solo_per_prompt_id_richiesto() {
        let conn = db_test();
        // Crea un secondo prompt per scopare il filtro.
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                Visibility, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-2', 'ws-personale', 'usr-locale', 'p2', 'b2',
                     'private', 1, datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        crea_pure(&conn, &nuovo_default("prm-1", "g1")).unwrap();
        crea_pure(&conn, &nuovo_default("prm-2", "g2")).unwrap();

        let lista_1 = lista_pure(&conn, "prm-1").unwrap();
        let lista_2 = lista_pure(&conn, "prm-2").unwrap();
        assert_eq!(lista_1.len(), 1);
        assert_eq!(lista_2.len(), 1);
        assert_eq!(lista_1[0].etichetta, "g1");
        assert_eq!(lista_2[0].etichetta, "g2");
    }

    #[test]
    fn lista_pure_db_vuoto_ritorna_vec_vuoto() {
        let conn = db_test();
        assert_eq!(lista_pure(&conn, "prm-1").unwrap().len(), 0);
    }

    // ─────────── Test compila_per_golden ───────────

    #[test]
    fn compila_sostituisce_segnaposto_stringa() {
        let body = "Saluta {{nome}} con tono {{tono}}.";
        let vars = r#"{"nome":"Luca","tono":"formale"}"#;
        let r = compila_per_golden(body, vars).unwrap();
        assert_eq!(r, "Saluta Luca con tono formale.");
    }

    #[test]
    fn compila_segnaposto_non_trovato_lasciato_invariato() {
        let body = "Hello {{ignoto}} world";
        let vars = r#"{"altro":"x"}"#;
        let r = compila_per_golden(body, vars).unwrap();
        assert_eq!(r, "Hello {{ignoto}} world");
    }

    #[test]
    fn compila_valore_numerico_serializzato() {
        let body = "N = {{n}}";
        let vars = r#"{"n":42}"#;
        let r = compila_per_golden(body, vars).unwrap();
        assert_eq!(r, "N = 42");
    }

    #[test]
    fn compila_input_vars_invalido_e_errore() {
        let r = compila_per_golden("x", "non-json");
        assert!(r.is_err());
    }

    #[test]
    fn compila_input_vars_array_e_errore() {
        // InputVars deve essere oggetto, non array.
        let r = compila_per_golden("x", "[1,2,3]");
        assert!(r.is_err());
    }

    #[test]
    fn compila_non_tocca_import_quotati() {
        // Sentinel: il regex \w+ NON matcha {{import "..."}}.
        let body = r#"{{import "ruolo"}} prefix {{nome}}"#;
        let vars = r#"{"nome":"X"}"#;
        let r = compila_per_golden(body, vars).unwrap();
        assert_eq!(r, r#"{{import "ruolo"}} prefix X"#);
    }

    // ─────────── Test esegui_pure_con_provider con mock provider ───────────

    /// Mock di AIProvider per i test offline.
    struct MockProvider {
        nome: &'static str,
        risposta: String,
        errore: Option<String>,
        latency: u64,
    }

    impl crate::provider_ai::AIProvider for MockProvider {
        fn name(&self) -> &'static str {
            self.nome
        }
        fn generate(
            &self,
            _prompt: &str,
            model: &str,
        ) -> Result<crate::provider_ai::GenerateOutput, PapErrore> {
            if let Some(err) = &self.errore {
                return Err(PapErrore::Generico(err.clone()));
            }
            Ok(crate::provider_ai::GenerateOutput {
                content: self.risposta.clone(),
                latency_ms: self.latency,
                tokens_used: Some(10),
                provider: self.nome,
                model: model.to_string(),
            })
        }
    }

    fn db_test_con_versione() -> (Connection, String) {
        let conn = db_test();
        // Snapshot v1 in PromptVersions per soddisfare current_version_id.
        crate::versioning::snapshot_versione(&conn, "prm-1", "usr-locale").unwrap();
        let v_id: String = conn
            .query_row(
                "SELECT Id FROM PromptVersions WHERE PromptId = 'prm-1' ORDER BY Version DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        (conn, v_id)
    }

    #[test]
    fn esegui_exact_match_passed_uno() {
        let (conn, v_id) = db_test_con_versione();
        let mut g = nuovo_default("prm-1", "lab");
        g.expected_output = "ciao mondo".into();
        g.similarity_fn = "exact-match".into();
        let golden_id = crea_pure(&conn, &g).unwrap();

        let provider = MockProvider {
            nome: "ollama",
            risposta: "ciao mondo".into(),
            errore: None,
            latency: 50,
        };
        let obs = esegui_pure_con_provider(
            &conn,
            None,
            &golden_id,
            &provider,
            "llama3.2",
            "usr-locale",
        )
        .unwrap();

        assert!(obs.passed, "exact-match identico deve passare");
        assert_eq!(obs.similarita, Some(1.0));
        assert_eq!(obs.actual_output, "ciao mondo");
        assert_eq!(obs.prompt_version_id, v_id);
        assert_eq!(obs.provider, "ollama");
        assert_eq!(obs.errore, None);
    }

    #[test]
    fn esegui_exact_match_diverso_failed() {
        let (conn, _) = db_test_con_versione();
        let mut g = nuovo_default("prm-1", "lab");
        g.expected_output = "atteso".into();
        g.similarity_fn = "exact-match".into();
        let golden_id = crea_pure(&conn, &g).unwrap();

        let provider = MockProvider {
            nome: "ollama",
            risposta: "qualcos'altro".into(),
            errore: None,
            latency: 50,
        };
        let obs =
            esegui_pure_con_provider(&conn, None, &golden_id, &provider, "x", "usr-locale")
                .unwrap();

        assert!(!obs.passed);
        assert_eq!(obs.similarita, Some(0.0));
        assert_eq!(obs.errore, None);
    }

    #[test]
    fn esegui_provider_errore_persiste_observation_con_errore() {
        let (conn, _) = db_test_con_versione();
        let g = nuovo_default("prm-1", "lab");
        let golden_id = crea_pure(&conn, &g).unwrap();

        let provider = MockProvider {
            nome: "ollama",
            risposta: "".into(),
            errore: Some("connessione rifiutata".into()),
            latency: 0,
        };
        let obs = esegui_pure_con_provider(
            &conn,
            None,
            &golden_id,
            &provider,
            "llama3.2",
            "usr-locale",
        )
        .unwrap();

        assert!(!obs.passed);
        assert_eq!(obs.similarita, None);
        assert!(obs.errore.is_some());
        assert!(obs.errore.unwrap().contains("connessione rifiutata"));
        // Observation persistita comunque.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptRunObservations WHERE Id = ?1",
                [&obs.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn esegui_regex_pattern_match() {
        let (conn, _) = db_test_con_versione();
        let mut g = nuovo_default("prm-1", "lab");
        g.expected_output = r"^\d{3}-\d{4}$".into();
        g.similarity_fn = "regex".into();
        let golden_id = crea_pure(&conn, &g).unwrap();

        let provider = MockProvider {
            nome: "ollama",
            risposta: "123-4567".into(),
            errore: None,
            latency: 10,
        };
        let obs = esegui_pure_con_provider(
            &conn,
            None,
            &golden_id,
            &provider,
            "x",
            "usr-locale",
        )
        .unwrap();

        assert!(obs.passed);
        assert_eq!(obs.similarita, Some(1.0));
    }

    #[test]
    fn esegui_golden_inesistente_e_errore() {
        let (conn, _) = db_test_con_versione();
        let provider = MockProvider {
            nome: "ollama",
            risposta: "x".into(),
            errore: None,
            latency: 0,
        };
        let r = esegui_pure_con_provider(
            &conn,
            None,
            "gld-fantasma",
            &provider,
            "x",
            "usr-locale",
        );
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("non trovato"));
    }

    #[test]
    fn esegui_senza_promptversions_e_errore() {
        // Salto db_test_con_versione: PromptVersions vuoto.
        let conn = db_test();
        let golden_id = crea_pure(&conn, &nuovo_default("prm-1", "lab")).unwrap();
        let provider = MockProvider {
            nome: "ollama",
            risposta: "x".into(),
            errore: None,
            latency: 0,
        };
        let r =
            esegui_pure_con_provider(&conn, None, &golden_id, &provider, "x", "usr-locale");
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("PromptVersions vuoto"));
    }

    #[test]
    fn esegui_observation_persistita_correttamente() {
        let (conn, v_id) = db_test_con_versione();
        let mut g = nuovo_default("prm-1", "lab");
        g.expected_output = "match".into();
        g.similarity_fn = "exact-match".into();
        let golden_id = crea_pure(&conn, &g).unwrap();

        let provider = MockProvider {
            nome: "ollama",
            risposta: "match".into(),
            errore: None,
            latency: 123,
        };
        let obs = esegui_pure_con_provider(
            &conn,
            None,
            &golden_id,
            &provider,
            "llama3.2",
            "usr-locale",
        )
        .unwrap();

        // Verifica persistenza completa.
        let (db_passed, db_sim, db_lat, db_tokens, db_provider, db_model): (
            i64,
            Option<f64>,
            Option<i64>,
            Option<i64>,
            String,
            String,
        ) = conn
            .query_row(
                "SELECT Passed, Similarita, LatenzaMs, TokensUsed, Provider, Model
                 FROM PromptRunObservations WHERE Id = ?1",
                [&obs.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .unwrap();
        assert_eq!(db_passed, 1);
        assert_eq!(db_sim, Some(1.0));
        assert_eq!(db_lat, Some(123));
        assert_eq!(db_tokens, Some(10));
        assert_eq!(db_provider, "ollama");
        assert_eq!(db_model, "llama3.2");

        // E che il PromptVersionId punti effettivamente alla versione corrente.
        assert_eq!(obs.prompt_version_id, v_id);
    }

    #[test]
    fn audit_log_registra_eventi_lifecycle() {
        let conn = db_test();
        let id = crea_pure(&conn, &nuovo_default("prm-1", "x")).unwrap();
        let agg = AggiornamentoGolden {
            id: id.clone(),
            etichetta: "y".to_string(),
            input_vars: "{}".to_string(),
            expected_output: "out".to_string(),
            similarity_fn: "cosine".to_string(),
            soglia_tolleranza: 0.85,
        };
        aggiorna_pure(&conn, &agg).unwrap();
        elimina_pure(&conn, &id).unwrap();

        let actions: Vec<String> = conn
            .prepare(
                "SELECT Action FROM AuditLog
                 WHERE EntityType = 'Golden' AND EntityId = ?1
                 ORDER BY OccurredAt ASC, Id ASC",
            )
            .unwrap()
            .query_map([&id], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        // I 3 eventi devono essere registrati.
        assert!(actions.contains(&"golden.creato".to_string()));
        assert!(actions.contains(&"golden.aggiornato".to_string()));
        assert!(actions.contains(&"golden.eliminato".to_string()));
    }
}
