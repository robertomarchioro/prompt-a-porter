// Modulo embeddings_store — Fase 3 Step 2.
//
// Storage degli embeddings via sqlite-vec (virtual table vec0). Compatibilità
// con SQLCipher verificata in Spike 1 (vedi
// `docs/architettura/decisioni/sqlite-vec-sqlcipher.md`).
//
// Pattern: registra `sqlite_vec_init` come auto-extension statica una sola
// volta a app start, prima della prima `Connection::open`. Tutte le connessioni
// successive (compreso il vault SQLCipher) avranno automaticamente vec0
// disponibile.
//
// Questo modulo contiene SOLO storage helper. Il calcolo dell'embedding
// (Session ort + tokenizer) vive in `embeddings.rs`. La ricerca ibrida
// (FTS5 + vec0 con RRF) sarà aggiunta in Step 3 di Fase 3.

use rusqlite::{ffi, params, Connection};
use std::sync::Once;

use crate::errore::PapErrore;
use crate::embeddings::EMBEDDING_DIM;

static REGISTRA_ONCE: Once = Once::new();

/// Registra `sqlite_vec_init` come auto-extension statica per tutte le
/// connessioni SQLite successive. **Deve essere chiamata una sola volta
/// a app start, prima di qualunque `Connection::open`.** Idempotente
/// grazie a `Once`.
///
/// Pattern e razionale: vedi `docs/architettura/decisioni/sqlite-vec-sqlcipher.md`.
pub fn registra_auto_extension() {
    REGISTRA_ONCE.call_once(|| {
        // SAFETY: chiamata una sola volta, prima di qualunque connessione.
        // sqlite3_auto_extension richiede una fn pointer con la firma C
        // standard di un init di estensione SQLite; sqlite_vec espone
        // sqlite3_vec_init come `unsafe extern "C" fn()` per semplicità,
        // quindi serve transmute alla firma attesa da rusqlite::ffi.
        unsafe {
            let init: unsafe extern "C" fn(
                *mut ffi::sqlite3,
                *mut *mut i8,
                *const ffi::sqlite3_api_routines,
            ) -> i32 = std::mem::transmute(sqlite_vec::sqlite3_vec_init as *const ());
            let rc = ffi::sqlite3_auto_extension(Some(init));
            if rc != ffi::SQLITE_OK {
                log::error!(
                    "sqlite3_auto_extension(sqlite_vec) fallito: rc={rc}"
                );
            } else {
                log::info!("sqlite-vec registrata come auto-extension");
            }
        }
    });
}

/// Serializza un embedding f32 nel formato little-endian raw atteso da vec0
/// (FLOAT[N] in sqlite-vec è uno slice contiguo di f32 LE).
fn embedding_a_bytes(emb: &[f32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(emb.len() * 4);
    for &v in emb {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    buf
}

/// Inserisce o aggiorna l'embedding di un prompt.
///
/// Pre-condizione: `embedding.len() == EMBEDDING_DIM`. La validazione è strict
/// per evitare INSERT con shape sbagliata che potrebbe corrompere vec0.
pub fn upsert_embedding(
    conn: &Connection,
    prompt_id: &str,
    embedding: &[f32],
) -> Result<(), PapErrore> {
    if embedding.len() != EMBEDDING_DIM {
        return Err(PapErrore::Generico(format!(
            "embedding shape errata: attesa {EMBEDDING_DIM}, ricevuta {}",
            embedding.len()
        )));
    }
    let bytes = embedding_a_bytes(embedding);

    // vec0 non supporta INSERT OR REPLACE né ON CONFLICT (limite della
    // virtual table). DELETE + INSERT è il pattern raccomandato. La
    // sequenza è eseguita all'interno di una transazione implicita di
    // SQLite (each statement è atomico, e in single-thread context il
    // gap è invisibile).
    conn.execute(
        "DELETE FROM PromptsEmbeddings WHERE PromptId = ?1",
        params![prompt_id],
    )?;
    conn.execute(
        "INSERT INTO PromptsEmbeddings (PromptId, Embedding) VALUES (?1, ?2)",
        params![prompt_id, bytes],
    )?;
    Ok(())
}

/// Rimuove l'embedding di un prompt. No-op se assente.
pub fn delete_embedding(conn: &Connection, prompt_id: &str) -> Result<(), PapErrore> {
    conn.execute(
        "DELETE FROM PromptsEmbeddings WHERE PromptId = ?1",
        params![prompt_id],
    )?;
    Ok(())
}

/// Cerca i prompt più simili a un embedding query, top-K per L2 distance.
/// Ritorna `Vec<(prompt_id, distance)>` ordinato per distance ASC (più simili
/// prima). Per cosine similarity su embeddings L2-normalized, L2 distance e
/// cosine sono monotonicamente equivalenti per il ranking.
pub fn search_nearest(
    conn: &Connection,
    query_embedding: &[f32],
    k: usize,
) -> Result<Vec<(String, f64)>, PapErrore> {
    if query_embedding.len() != EMBEDDING_DIM {
        return Err(PapErrore::Generico(format!(
            "query embedding shape errata: attesa {EMBEDDING_DIM}, ricevuta {}",
            query_embedding.len()
        )));
    }
    let bytes = embedding_a_bytes(query_embedding);

    let mut stmt = conn.prepare(
        "SELECT PromptId, distance FROM PromptsEmbeddings
         WHERE Embedding MATCH ?1 AND k = ?2
         ORDER BY distance",
    )?;
    let rows = stmt
        .query_map(params![bytes, k as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Conta gli embeddings memorizzati. Utile per progress UI di backfill e
/// quality gate.
pub fn conta(conn: &Connection) -> Result<i64, PapErrore> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM PromptsEmbeddings", [], |r| r.get(0))?;
    Ok(n)
}

// ─────────── Tag embeddings (Fase 3 Step 4) ───────────
//
// Pattern simmetrico a quello dei prompt: tabella vec0 separata
// `TagsEmbeddings`. Usato dall'auto-suggerimento tag per trovare i tag
// esistenti il cui nome è più vicino al body del prompt corrente.

/// Inserisce o aggiorna l'embedding del nome di un tag.
pub fn upsert_tag_embedding(
    conn: &Connection,
    tag_id: &str,
    embedding: &[f32],
) -> Result<(), PapErrore> {
    if embedding.len() != EMBEDDING_DIM {
        return Err(PapErrore::Generico(format!(
            "tag embedding shape errata: attesa {EMBEDDING_DIM}, ricevuta {}",
            embedding.len()
        )));
    }
    let bytes = embedding_a_bytes(embedding);
    // Stesso pattern DELETE+INSERT del prompt embedding: vec0 non
    // supporta OR REPLACE (vedi commit history per il razionale).
    conn.execute(
        "DELETE FROM TagsEmbeddings WHERE TagId = ?1",
        params![tag_id],
    )?;
    conn.execute(
        "INSERT INTO TagsEmbeddings (TagId, Embedding) VALUES (?1, ?2)",
        params![tag_id, bytes],
    )?;
    Ok(())
}

/// Rimuove l'embedding di un tag. No-op se assente.
pub fn delete_tag_embedding(conn: &Connection, tag_id: &str) -> Result<(), PapErrore> {
    conn.execute(
        "DELETE FROM TagsEmbeddings WHERE TagId = ?1",
        params![tag_id],
    )?;
    Ok(())
}

/// Cerca i tag più simili a un embedding query, top-K. Ritorna
/// `Vec<(tag_id, distance)>` ordinato per distance ASC.
pub fn search_nearest_tags(
    conn: &Connection,
    query_embedding: &[f32],
    k: usize,
) -> Result<Vec<(String, f64)>, PapErrore> {
    if query_embedding.len() != EMBEDDING_DIM {
        return Err(PapErrore::Generico(format!(
            "query embedding shape errata: attesa {EMBEDDING_DIM}, ricevuta {}",
            query_embedding.len()
        )));
    }
    let bytes = embedding_a_bytes(query_embedding);
    let mut stmt = conn.prepare(
        "SELECT TagId, distance FROM TagsEmbeddings
         WHERE Embedding MATCH ?1 AND k = ?2
         ORDER BY distance",
    )?;
    let rows = stmt
        .query_map(params![bytes, k as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Conta gli embeddings dei tag memorizzati.
pub fn conta_tag(conn: &Connection) -> Result<i64, PapErrore> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM TagsEmbeddings", [], |r| r.get(0))?;
    Ok(n)
}

/// Lista i tag che NON hanno ancora un embedding (candidati per backfill).
pub fn tag_senza_embedding(
    conn: &Connection,
    limite: usize,
) -> Result<Vec<(String, String)>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT t.Id, t.Name FROM Tags t
         WHERE t.DeletedAt IS NULL
           AND NOT EXISTS (
               SELECT 1 FROM TagsEmbeddings e WHERE e.TagId = t.Id
           )
         LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limite as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

// ─────────── Prompt embeddings backfill helper ───────────

/// Lista i prompt che NON hanno ancora un embedding (candidati per il
/// backfill iniziale al primo unlock con feature attivata).
pub fn prompt_senza_embedding(
    conn: &Connection,
    limite: usize,
) -> Result<Vec<String>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT p.Id FROM Prompts p
         WHERE p.DeletedAt IS NULL
           AND NOT EXISTS (
               SELECT 1 FROM PromptsEmbeddings e WHERE e.PromptId = p.Id
           )
         LIMIT ?1",
    )?;
    let ids = stmt
        .query_map(params![limite as i64], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(ids)
}

#[cfg(test)]
mod test {
    use super::*;

    fn db_test() -> Connection {
        registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    fn embedding_dummy(seed: f32) -> Vec<f32> {
        (0..EMBEDDING_DIM)
            .map(|i| seed + (i as f32) * 0.001)
            .collect()
    }

    fn inserisci_prompt_minimo(conn: &Connection, id: &str) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                Visibility, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', 'T', 'b', 'private', 1,
                datetime('now'), datetime('now'))",
            params![id],
        )
        .unwrap();
    }

    #[test]
    fn auto_extension_idempotente() {
        // Doppia chiamata non deve causare problemi (Once garantisce single-init).
        registra_auto_extension();
        registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        // vec_version() è disponibile solo se l'extension è registrata.
        let v: String = conn.query_row("SELECT vec_version()", [], |r| r.get(0)).unwrap();
        assert!(v.starts_with("v"), "Version deve essere stringa tipo 'v0.x.y'");
    }

    #[test]
    fn upsert_e_search() {
        let conn = db_test();
        inserisci_prompt_minimo(&conn, "prm-1");
        inserisci_prompt_minimo(&conn, "prm-2");
        inserisci_prompt_minimo(&conn, "prm-3");

        upsert_embedding(&conn, "prm-1", &embedding_dummy(0.1)).unwrap();
        upsert_embedding(&conn, "prm-2", &embedding_dummy(0.2)).unwrap();
        upsert_embedding(&conn, "prm-3", &embedding_dummy(0.9)).unwrap();

        // Query vicina a 0.1 → prm-1 deve essere primo
        let risultati = search_nearest(&conn, &embedding_dummy(0.11), 3).unwrap();
        assert_eq!(risultati.len(), 3);
        assert_eq!(risultati[0].0, "prm-1");
        assert_eq!(risultati[1].0, "prm-2");
        assert_eq!(risultati[2].0, "prm-3");
    }

    #[test]
    fn upsert_replace_su_id_esistente() {
        let conn = db_test();
        inserisci_prompt_minimo(&conn, "prm-1");

        upsert_embedding(&conn, "prm-1", &embedding_dummy(0.1)).unwrap();
        upsert_embedding(&conn, "prm-1", &embedding_dummy(0.5)).unwrap();

        assert_eq!(conta(&conn).unwrap(), 1, "Una sola riga dopo replace");

        // Search vicino al nuovo embedding (0.5) deve trovarlo come primo
        let risultati = search_nearest(&conn, &embedding_dummy(0.5), 1).unwrap();
        assert_eq!(risultati[0].0, "prm-1");
    }

    #[test]
    fn delete_embedding_idempotente() {
        let conn = db_test();
        inserisci_prompt_minimo(&conn, "prm-1");

        upsert_embedding(&conn, "prm-1", &embedding_dummy(0.1)).unwrap();
        delete_embedding(&conn, "prm-1").unwrap();
        delete_embedding(&conn, "prm-1").unwrap(); // no-op
        delete_embedding(&conn, "non-esistente").unwrap(); // no-op

        assert_eq!(conta(&conn).unwrap(), 0);
    }

    #[test]
    fn upsert_rifiuta_shape_sbagliata() {
        let conn = db_test();
        inserisci_prompt_minimo(&conn, "prm-1");

        let r = upsert_embedding(&conn, "prm-1", &vec![0.0f32; 100]);
        assert!(r.is_err(), "shape != EMBEDDING_DIM deve essere rifiutata");
    }

    #[test]
    fn search_rifiuta_shape_sbagliata() {
        let conn = db_test();
        let r = search_nearest(&conn, &vec![0.0f32; 100], 5);
        assert!(r.is_err());
    }

    #[test]
    fn prompt_senza_embedding_lista_correttamente() {
        let conn = db_test();
        inserisci_prompt_minimo(&conn, "prm-1");
        inserisci_prompt_minimo(&conn, "prm-2");
        inserisci_prompt_minimo(&conn, "prm-3");

        upsert_embedding(&conn, "prm-1", &embedding_dummy(0.1)).unwrap();
        // prm-2 e prm-3 senza embedding

        let candidati = prompt_senza_embedding(&conn, 100).unwrap();
        assert_eq!(candidati.len(), 2);
        assert!(candidati.contains(&"prm-2".to_string()));
        assert!(candidati.contains(&"prm-3".to_string()));
    }

    #[test]
    fn embedding_a_bytes_dim_corretta() {
        let v = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = embedding_a_bytes(&v);
        assert_eq!(b.len(), 16, "4 float32 = 16 bytes");
    }

    // ─────────── Tag embeddings ───────────

    fn inserisci_tag_minimo(conn: &Connection, id: &str, nome: &str) {
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', ?2, datetime('now'), datetime('now'))",
            params![id, nome],
        )
        .unwrap();
    }

    #[test]
    fn upsert_e_search_tag() {
        let conn = db_test();
        inserisci_tag_minimo(&conn, "tag-a", "marketing");
        inserisci_tag_minimo(&conn, "tag-b", "tecnico");
        inserisci_tag_minimo(&conn, "tag-c", "creativo");

        upsert_tag_embedding(&conn, "tag-a", &embedding_dummy(0.1)).unwrap();
        upsert_tag_embedding(&conn, "tag-b", &embedding_dummy(0.2)).unwrap();
        upsert_tag_embedding(&conn, "tag-c", &embedding_dummy(0.9)).unwrap();

        let risultati = search_nearest_tags(&conn, &embedding_dummy(0.11), 3).unwrap();
        assert_eq!(risultati.len(), 3);
        assert_eq!(risultati[0].0, "tag-a");
        assert_eq!(risultati[1].0, "tag-b");
    }

    #[test]
    fn upsert_tag_replace_su_id_esistente() {
        let conn = db_test();
        inserisci_tag_minimo(&conn, "tag-a", "x");
        upsert_tag_embedding(&conn, "tag-a", &embedding_dummy(0.1)).unwrap();
        upsert_tag_embedding(&conn, "tag-a", &embedding_dummy(0.5)).unwrap();
        assert_eq!(conta_tag(&conn).unwrap(), 1);
    }

    #[test]
    fn delete_tag_idempotente() {
        let conn = db_test();
        delete_tag_embedding(&conn, "non-esistente").unwrap();
        inserisci_tag_minimo(&conn, "tag-a", "x");
        upsert_tag_embedding(&conn, "tag-a", &embedding_dummy(0.1)).unwrap();
        delete_tag_embedding(&conn, "tag-a").unwrap();
        delete_tag_embedding(&conn, "tag-a").unwrap();
        assert_eq!(conta_tag(&conn).unwrap(), 0);
    }

    #[test]
    fn upsert_tag_rifiuta_shape_sbagliata() {
        let conn = db_test();
        inserisci_tag_minimo(&conn, "tag-a", "x");
        let r = upsert_tag_embedding(&conn, "tag-a", &vec![0.0f32; 100]);
        assert!(r.is_err());
    }

    #[test]
    fn tag_senza_embedding_lista_correttamente() {
        let conn = db_test();
        inserisci_tag_minimo(&conn, "tag-a", "alfa");
        inserisci_tag_minimo(&conn, "tag-b", "beta");
        upsert_tag_embedding(&conn, "tag-a", &embedding_dummy(0.1)).unwrap();
        // tag-b senza embedding

        let candidati = tag_senza_embedding(&conn, 100).unwrap();
        assert_eq!(candidati.len(), 1);
        assert_eq!(candidati[0].0, "tag-b");
        assert_eq!(candidati[0].1, "beta");
    }
}
