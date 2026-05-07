// Modulo rating — Fase 4 Step 2.
//
// Feedback discreto post-uso del prompt. 3 valori (-1/0/+1), append-only:
// ogni feedback è una riga separata con timestamp, così emerge la
// traiettoria nel tempo (un prompt molto usato che inizia a prendere
// rating bassi è candidato per refactor).
//
// MVP scope (questa PR):
// - rating_aggiungi(prompt_id, rating, nota?, used_with_model?): insert
//   nuova row con UserId='usr-locale' (single-user)
// - rating_aggregato(prompt_id): media + conteggio + distribuzione
//   pos/neu/neg, tutto on-the-fly via aggregazione SQL
// - Validazione: rating IN (-1, 0, 1), prompt esistente
// - Audit log evento rating.aggiunto
//
// Out of scope (futuro):
// - Vista "Migliori prompt" sort_by_quality (richiede integrazione in libreria_lista)
// - Aggregato per modello (filtro UsedWithModel)
// - Modale "Aggiungi nota" UI per voto negativo

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

const USER_LOCALE: &str = "usr-locale";

#[derive(Debug, Clone, Serialize)]
pub struct RatingAggregato {
    /// Media dei rating in [-1, 1], `None` se nessun rating.
    pub media: Option<f64>,
    pub conteggio: i64,
    pub positivi: i64,
    pub neutri: i64,
    pub negativi: i64,
}

#[derive(Debug, Deserialize)]
pub struct NuovoRating {
    pub prompt_id: String,
    pub rating: i64,
    #[serde(default)]
    pub nota: Option<String>,
    #[serde(default)]
    pub used_with_model: Option<String>,
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
        "rtg-{:012x}{:02x}{:02x}{:02x}{:02x}",
        ts, rnd[0], rnd[1], rnd[2], rnd[3]
    )
}

fn valida_rating(rating: i64) -> Result<(), PapErrore> {
    if !(-1..=1).contains(&rating) {
        return Err(PapErrore::Generico(format!(
            "Rating {rating} fuori range. Ammessi: -1 (negativo), 0 (neutro), 1 (positivo)"
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

/// Logica pura di `rating_aggiungi`, testabile senza Tauri State.
pub(crate) fn aggiungi_pure(
    conn: &Connection,
    nuovo: &NuovoRating,
) -> Result<String, PapErrore> {
    valida_rating(nuovo.rating)?;
    verifica_prompt_esiste(conn, &nuovo.prompt_id)?;

    // Trim della nota: stringa vuota dopo trim diventa NULL.
    let nota_trim = nuovo
        .nota
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let model_trim = nuovo
        .used_with_model
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let id = genera_id();
    conn.execute(
        "INSERT INTO PromptRatings
            (Id, PromptId, UserId, Rating, Note, UsedWithModel, CreatedAt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))",
        params![
            id,
            nuovo.prompt_id,
            USER_LOCALE,
            nuovo.rating,
            nota_trim,
            model_trim,
        ],
    )?;

    crate::audit::registra(
        conn,
        "rating.aggiunto",
        "PromptRating",
        &id,
        Some(&format!("rating={} su {}", nuovo.rating, nuovo.prompt_id)),
    );
    log::info!(
        "Rating aggiunto: {id} (prompt {}, valore {})",
        nuovo.prompt_id,
        nuovo.rating
    );
    Ok(id)
}

pub(crate) fn aggregato_pure(
    conn: &Connection,
    prompt_id: &str,
) -> Result<RatingAggregato, PapErrore> {
    let row: (Option<f64>, i64, i64, i64, i64) = conn
        .query_row(
            "SELECT
                AVG(CAST(Rating AS REAL)) AS media,
                COUNT(*) AS tot,
                SUM(CASE WHEN Rating = 1 THEN 1 ELSE 0 END) AS pos,
                SUM(CASE WHEN Rating = 0 THEN 1 ELSE 0 END) AS neu,
                SUM(CASE WHEN Rating = -1 THEN 1 ELSE 0 END) AS neg
             FROM PromptRatings WHERE PromptId = ?1",
            [prompt_id],
            |r| {
                Ok((
                    r.get::<_, Option<f64>>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, Option<i64>>(2)?.unwrap_or(0),
                    r.get::<_, Option<i64>>(3)?.unwrap_or(0),
                    r.get::<_, Option<i64>>(4)?.unwrap_or(0),
                ))
            },
        )
        // Se non ci sono rating, AVG ritorna NULL ma il SELECT torna comunque 1 row.
        .unwrap_or((None, 0, 0, 0, 0));

    Ok(RatingAggregato {
        media: row.0,
        conteggio: row.1,
        positivi: row.2,
        neutri: row.3,
        negativi: row.4,
    })
}

#[tauri::command]
pub fn rating_aggiungi(
    nuovo: NuovoRating,
    state: State<'_, VaultState>,
) -> Result<String, PapErrore> {
    state.with_conn(|conn| aggiungi_pure(conn, &nuovo))
}

#[tauri::command]
pub fn rating_aggregato(
    prompt_id: String,
    state: State<'_, VaultState>,
) -> Result<RatingAggregato, PapErrore> {
    state.with_conn(|conn| aggregato_pure(conn, &prompt_id))
}

#[cfg(test)]
mod test {
    use super::*;

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
                Visibility, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-1', 'ws-personale', 'usr-locale', 't', 'd', 'b',
                     'private', 1, datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn id_inizia_con_rtg() {
        assert!(genera_id().starts_with("rtg-"));
    }

    #[test]
    fn valida_rating_accetta_range() {
        assert!(valida_rating(-1).is_ok());
        assert!(valida_rating(0).is_ok());
        assert!(valida_rating(1).is_ok());
    }

    #[test]
    fn valida_rating_rifiuta_fuori_range() {
        assert!(valida_rating(2).is_err());
        assert!(valida_rating(-2).is_err());
        assert!(valida_rating(100).is_err());
    }

    fn nuovo_rating(rating: i64) -> NuovoRating {
        NuovoRating {
            prompt_id: "prm-1".into(),
            rating,
            nota: None,
            used_with_model: None,
        }
    }

    #[test]
    fn aggiungi_pure_inserisce_e_ritorna_id() {
        let conn = db_test();
        let id = aggiungi_pure(&conn, &nuovo_rating(1)).unwrap();
        assert!(id.starts_with("rtg-"));
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptRatings WHERE Id = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn aggiungi_pure_persiste_user_locale() {
        let conn = db_test();
        let id = aggiungi_pure(&conn, &nuovo_rating(0)).unwrap();
        let user: String = conn
            .query_row(
                "SELECT UserId FROM PromptRatings WHERE Id = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(user, "usr-locale");
    }

    #[test]
    fn aggiungi_pure_rifiuta_rating_invalido() {
        let conn = db_test();
        let r = aggiungi_pure(&conn, &nuovo_rating(5));
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("fuori range"));
    }

    #[test]
    fn aggiungi_pure_rifiuta_prompt_inesistente() {
        let conn = db_test();
        let mut n = nuovo_rating(1);
        n.prompt_id = "prm-fantasma".into();
        let r = aggiungi_pure(&conn, &n);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("non esiste"));
    }

    #[test]
    fn aggiungi_pure_persiste_nota() {
        let conn = db_test();
        let mut n = nuovo_rating(-1);
        n.nota = Some("Risposta troppo verbosa".into());
        let id = aggiungi_pure(&conn, &n).unwrap();
        let nota: Option<String> = conn
            .query_row(
                "SELECT Note FROM PromptRatings WHERE Id = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(nota.as_deref(), Some("Risposta troppo verbosa"));
    }

    #[test]
    fn aggiungi_pure_nota_vuota_diventa_null() {
        let conn = db_test();
        let mut n = nuovo_rating(0);
        n.nota = Some("   ".into());
        let id = aggiungi_pure(&conn, &n).unwrap();
        let nota: Option<String> = conn
            .query_row(
                "SELECT Note FROM PromptRatings WHERE Id = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(nota.is_none(), "Nota whitespace-only deve essere NULL");
    }

    #[test]
    fn aggiungi_pure_persiste_used_with_model() {
        let conn = db_test();
        let mut n = nuovo_rating(1);
        n.used_with_model = Some("claude-sonnet-4.6".into());
        let id = aggiungi_pure(&conn, &n).unwrap();
        let m: Option<String> = conn
            .query_row(
                "SELECT UsedWithModel FROM PromptRatings WHERE Id = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(m.as_deref(), Some("claude-sonnet-4.6"));
    }

    #[test]
    fn aggiungi_pure_audit_log_registra_evento() {
        let conn = db_test();
        let id = aggiungi_pure(&conn, &nuovo_rating(1)).unwrap();
        let azione: String = conn
            .query_row(
                "SELECT Action FROM AuditLog WHERE EntityType = 'PromptRating'
                 AND EntityId = ?1 ORDER BY OccurredAt DESC LIMIT 1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(azione, "rating.aggiunto");
    }

    #[test]
    fn check_constraint_blocca_rating_invalido_dal_db() {
        let conn = db_test();
        // Bypassa la validazione applicativa testando il CHECK SQL.
        let r = conn.execute(
            "INSERT INTO PromptRatings (Id, PromptId, UserId, Rating)
             VALUES ('rtg-x', 'prm-1', 'usr-locale', 99)",
            [],
        );
        assert!(r.is_err(), "CHECK su Rating deve rifiutare 99");
    }

    // ─────────── Test aggregato_pure ───────────

    #[test]
    fn aggregato_db_vuoto_e_zero() {
        let conn = db_test();
        let agg = aggregato_pure(&conn, "prm-1").unwrap();
        assert_eq!(agg.conteggio, 0);
        assert_eq!(agg.positivi, 0);
        assert_eq!(agg.neutri, 0);
        assert_eq!(agg.negativi, 0);
        assert!(agg.media.is_none());
    }

    #[test]
    fn aggregato_calcola_media_e_distribuzione() {
        let conn = db_test();
        aggiungi_pure(&conn, &nuovo_rating(1)).unwrap();
        aggiungi_pure(&conn, &nuovo_rating(1)).unwrap();
        aggiungi_pure(&conn, &nuovo_rating(1)).unwrap();
        aggiungi_pure(&conn, &nuovo_rating(0)).unwrap();
        aggiungi_pure(&conn, &nuovo_rating(-1)).unwrap();

        let agg = aggregato_pure(&conn, "prm-1").unwrap();
        assert_eq!(agg.conteggio, 5);
        assert_eq!(agg.positivi, 3);
        assert_eq!(agg.neutri, 1);
        assert_eq!(agg.negativi, 1);
        // Media = (3 - 1) / 5 = 0.4
        assert!((agg.media.unwrap() - 0.4).abs() < 1e-6);
    }

    #[test]
    fn aggregato_isolato_per_prompt() {
        let conn = db_test();
        // Secondo prompt
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                Visibility, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-2', 'ws-personale', 'usr-locale', 't2', 'b2',
                     'private', 1, datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        aggiungi_pure(&conn, &nuovo_rating(1)).unwrap();
        let mut n2 = nuovo_rating(-1);
        n2.prompt_id = "prm-2".into();
        aggiungi_pure(&conn, &n2).unwrap();

        let agg1 = aggregato_pure(&conn, "prm-1").unwrap();
        let agg2 = aggregato_pure(&conn, "prm-2").unwrap();
        assert_eq!(agg1.conteggio, 1);
        assert_eq!(agg1.positivi, 1);
        assert_eq!(agg2.conteggio, 1);
        assert_eq!(agg2.negativi, 1);
    }
}
