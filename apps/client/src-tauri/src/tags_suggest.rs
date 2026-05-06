// Modulo tags_suggest — Fase 3 Step 4.
//
// Auto-suggerimento dei tag pertinenti per un nuovo prompt o per un body
// arbitrario. Usa l'embedding del testo per trovare i tag esistenti il
// cui nome ha embedding più vicino, sopra una soglia di similarità.
//
// Fallback: se ci sono pochi tag con embedding (< MIN_TAG_PER_SEMANTIC),
// si torna alla strategia "tag più frequenti", più robusta su workspace
// piccoli o appena creati.

use rusqlite::Connection;
use serde::Serialize;
use tauri::State;

use crate::embeddings::{compute_embedding_opt, EmbeddingsState};
use crate::embeddings_store::{conta_tag, search_nearest_tags};
use crate::errore::PapErrore;
use crate::vault::VaultState;

/// Limite ragionevole per evitare lookup costosi su workspace grandi.
const MAX_LIMIT: usize = 20;

/// Default richiesto dal frontend se non specificato. Doc Fase 3 Step 4
/// raccomanda top-5 chip cliccabili nell'editor.
const DEFAULT_LIMIT: usize = 5;

/// Soglia di L2 distance oltre la quale il tag è considerato "non
/// pertinente" e scartato dal suggerimento. Per embeddings L2-normalized
/// (MiniLM-L12-v2 lo è), L2 distance ∈ [0, 2]. Valore tipico cosine
/// similarity ≥ 0.5 ↔ L2 distance ≤ ~1.0; threshold prudente.
const SOGLIA_DISTANZA: f64 = 1.0;

/// Sotto questa soglia (numero di tag con embedding), invece della ricerca
/// vettoriale si fa fallback a "tag più frequenti".
const MIN_TAG_PER_SEMANTIC: i64 = 10;

#[derive(Debug, Serialize, Clone)]
pub struct TagSuggerito {
    pub id: String,
    pub nome: String,
    pub colore: String,
    /// Punteggio di pertinenza ∈ [0, 1] (1 = massima pertinenza).
    /// Per il path semantico è `1 - L2_distance / 2` (cosine similarity).
    /// Per il path "frequenza" è `uso_count / max_uso_count` clampato a [0,1].
    pub score: f64,
    /// Sorgente del suggerimento, utile per debug/UI ("vector" | "frequenza").
    pub sorgente: &'static str,
}

/// Tag esistenti ordinati per frequenza d'uso (PromptTags), top-N.
fn tag_frequenti(conn: &Connection, limite: usize) -> Result<Vec<TagSuggerito>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT t.Id, t.Name, COALESCE(t.Color, ''),
                COUNT(pt.PromptId) AS uso
         FROM Tags t
         LEFT JOIN PromptTags pt ON pt.TagId = t.Id
         LEFT JOIN Prompts p ON p.Id = pt.PromptId AND p.DeletedAt IS NULL
         WHERE t.DeletedAt IS NULL
         GROUP BY t.Id, t.Name, t.Color
         ORDER BY uso DESC, t.Name COLLATE NOCASE
         LIMIT ?1",
    )?;
    let rows: Vec<(String, String, String, i64)> = stmt
        .query_map(rusqlite::params![limite as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let max_uso = rows.iter().map(|r| r.3).max().unwrap_or(1).max(1);
    Ok(rows
        .into_iter()
        .map(|(id, nome, colore, uso)| TagSuggerito {
            id,
            nome,
            colore,
            score: (uso as f64 / max_uso as f64).clamp(0.0, 1.0),
            sorgente: "frequenza",
        })
        .collect())
}

/// Tag più simili semanticamente al `query_emb`, sopra soglia di distanza.
fn tag_semantici(
    conn: &Connection,
    query_emb: &[f32],
    limite: usize,
) -> Result<Vec<TagSuggerito>, PapErrore> {
    // Tiriamo top-K più ampio della soglia richiesta, perché alcuni
    // potrebbero essere scartati dal threshold di distanza.
    let raw = search_nearest_tags(conn, query_emb, limite * 2)?;
    let mut suggeriti: Vec<TagSuggerito> = Vec::with_capacity(limite);

    for (tag_id, distance) in raw {
        if distance > SOGLIA_DISTANZA {
            continue;
        }
        // Carica metadata tag (nome, colore).
        let r = conn.query_row(
            "SELECT Name, COALESCE(Color, '') FROM Tags
             WHERE Id = ?1 AND DeletedAt IS NULL",
            [&tag_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );
        if let Ok((nome, colore)) = r {
            // Cosine similarity da L2 distance (assumendo embeddings
            // L2-normalized): sim = 1 - L2² / 2. Clampato a [0, 1].
            let cos = (1.0 - (distance * distance) / 2.0).clamp(0.0, 1.0);
            suggeriti.push(TagSuggerito {
                id: tag_id,
                nome,
                colore,
                score: cos,
                sorgente: "vector",
            });
        }
        if suggeriti.len() >= limite {
            break;
        }
    }
    Ok(suggeriti)
}

/// Logica pura del comando `tags_suggest`, testabile senza Tauri State.
/// Il command Tauri pubblico (sotto) si limita a fare il dispatch.
pub(crate) fn suggerisci_per_testo(
    conn: &Connection,
    rt_state: &EmbeddingsState,
    testo_pulito: &str,
    limite: usize,
) -> Result<Vec<TagSuggerito>, PapErrore> {
    // Soglia: se ci sono pochi tag con embedding o Session non loaded,
    // fallback al path frequenza.
    let n_tag_emb = conta_tag(conn)?;
    if n_tag_emb < MIN_TAG_PER_SEMANTIC {
        log::debug!(
            "tags_suggest: solo {n_tag_emb} tag con embedding, fallback a tag_frequenti"
        );
        return tag_frequenti(conn, limite);
    }

    let query_emb = match compute_embedding_opt(rt_state, testo_pulito)? {
        Some(emb) => emb,
        None => {
            log::debug!("tags_suggest: Session non loaded, fallback a tag_frequenti");
            return tag_frequenti(conn, limite);
        }
    };

    let suggeriti = tag_semantici(conn, &query_emb, limite)?;
    if suggeriti.is_empty() {
        // Tutti sotto soglia → fallback frequenza per non lasciare l'utente
        // con zero suggerimenti.
        log::debug!("tags_suggest: nessun tag sopra soglia, fallback a tag_frequenti");
        return tag_frequenti(conn, limite);
    }
    Ok(suggeriti)
}

#[tauri::command]
pub fn tags_suggest(
    testo: String,
    limit: Option<u32>,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<Vec<TagSuggerito>, PapErrore> {
    let limite = limit
        .map(|l| l as usize)
        .unwrap_or(DEFAULT_LIMIT)
        .min(MAX_LIMIT);

    let testo_pulito = testo.trim();
    if testo_pulito.is_empty() {
        return Ok(vec![]);
    }

    state.with_conn(|conn| suggerisci_per_testo(conn, &rt_state, testo_pulito, limite))
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
        conn
    }

    fn inserisci_tag(conn: &Connection, id: &str, nome: &str) {
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', ?2, datetime('now'), datetime('now'))",
            params![id, nome],
        )
        .unwrap();
    }

    fn inserisci_prompt_con_tag(conn: &Connection, prompt_id: &str, tag_ids: &[&str]) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                Visibility, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', 't', 'b', 'private', 1,
                     datetime('now'), datetime('now'))",
            params![prompt_id],
        )
        .unwrap();
        for tag_id in tag_ids {
            conn.execute(
                "INSERT INTO PromptTags (PromptId, TagId) VALUES (?1, ?2)",
                params![prompt_id, tag_id],
            )
            .unwrap();
        }
    }

    #[test]
    fn tag_frequenti_ordina_per_uso_count_desc() {
        let conn = db_test();
        inserisci_tag(&conn, "t-poco", "poco");
        inserisci_tag(&conn, "t-molto", "molto");
        inserisci_tag(&conn, "t-medio", "medio");
        // t-molto: 3 prompt
        inserisci_prompt_con_tag(&conn, "prm-1", &["t-molto"]);
        inserisci_prompt_con_tag(&conn, "prm-2", &["t-molto", "t-medio"]);
        inserisci_prompt_con_tag(&conn, "prm-3", &["t-molto"]);
        // t-medio: 1 prompt (oltre quello sopra è 2 totale ma)
        // ricalcolo: prm-2 ha medio. Inseriamone un altro.
        inserisci_prompt_con_tag(&conn, "prm-4", &["t-medio"]);

        let r = tag_frequenti(&conn, 10).unwrap();
        assert_eq!(r.len(), 3);
        assert_eq!(r[0].id, "t-molto");
        assert_eq!(r[1].id, "t-medio");
        assert_eq!(r[2].id, "t-poco");
        assert_eq!(r[0].score, 1.0); // max uso = 3, score = 3/3
        assert!(r[2].score < 0.5);
        assert_eq!(r[0].sorgente, "frequenza");
    }

    #[test]
    fn tag_frequenti_su_db_vuoto_ritorna_vuoto() {
        let conn = db_test();
        let r = tag_frequenti(&conn, 10).unwrap();
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn soglia_costanti_sensate() {
        // Sentinel: se qualcuno cambia accidentalmente i parametri.
        assert!(SOGLIA_DISTANZA > 0.0 && SOGLIA_DISTANZA <= 2.0);
        assert!(DEFAULT_LIMIT > 0 && DEFAULT_LIMIT <= MAX_LIMIT);
        assert!(MIN_TAG_PER_SEMANTIC > 0);
    }

    // ─────────── Smoke test: fallback grace senza Session ───────────

    #[test]
    fn suggerisci_senza_session_fa_fallback_a_frequenza() {
        // Quality gate Step 10 — grace degradation: con tag a sufficienza
        // (>= MIN_TAG_PER_SEMANTIC) ma Session NON loaded, il path
        // semantico non parte e dobbiamo cadere su tag_frequenti.
        let conn = db_test();
        // Inserisci 12 tag (sopra MIN_TAG_PER_SEMANTIC=10) con
        // un embedding fittizio in TagsEmbeddings così conta_tag ritorna 12.
        for i in 0..12 {
            let id = format!("t-{i}");
            inserisci_tag(&conn, &id, &format!("nome_{i}"));
            // Inserisci embedding stub di dimensione 384 (richiesto da vec0).
            let emb: Vec<f32> = vec![0.0_f32; 384];
            crate::embeddings_store::upsert_tag_embedding(&conn, &id, &emb).unwrap();
        }
        // Almeno un PromptTag per ordinare via uso.
        inserisci_prompt_con_tag(&conn, "prm-1", &["t-0", "t-1"]);

        let rt = EmbeddingsState::new(); // session = None

        let r = suggerisci_per_testo(&conn, &rt, "qualcosa", 5).unwrap();
        // Ottengo risultati (dal fallback frequenza), no errore.
        assert!(!r.is_empty(), "fallback a tag_frequenti deve restituire qualcosa");
        // Tutti devono avere sorgente=frequenza, non vector.
        assert!(
            r.iter().all(|t| t.sorgente == "frequenza"),
            "senza session, sorgente deve essere frequenza"
        );
    }

    #[test]
    fn suggerisci_pochi_tag_fa_fallback_a_frequenza() {
        // Anche con Session in teoria caricabile, se pochi tag con embedding
        // (< MIN_TAG_PER_SEMANTIC) il path semantico è skippato. Test
        // sentinella per evitare regressioni di soglia.
        let conn = db_test();
        inserisci_tag(&conn, "t-1", "alpha");
        inserisci_prompt_con_tag(&conn, "prm-1", &["t-1"]);

        let rt = EmbeddingsState::new();
        let r = suggerisci_per_testo(&conn, &rt, "qualcosa", 5).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].sorgente, "frequenza");
    }
}
