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

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

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
    use rand::RngCore;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut rnd = [0u8; 4];
    rand::rngs::OsRng.fill_bytes(&mut rnd);
    format!(
        "gld-{:012x}{:02x}{:02x}{:02x}{:02x}",
        ts, rnd[0], rnd[1], rnd[2], rnd[3]
    )
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
