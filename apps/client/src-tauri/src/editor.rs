use rand::RngCore;
use rusqlite::Connection;
use serde::Deserialize;
use tauri::State;

use crate::embeddings::{compute_embedding_opt, EmbeddingsState};
use crate::embeddings_store;
use crate::errore::PapErrore;
use crate::prompt_componibili;
use crate::vault::VaultState;

/// Hook embedding: dopo una INSERT/UPDATE su `Prompts`, se la Session ort è
/// caricata, calcola l'embedding del body e fa upsert in vec0. Graceful
/// skip se la Session non è disponibile (utente non ha attivato la feature
/// o modello non scaricato): in quel caso il prompt resta senza embedding
/// e verrà processato dal backfill quando la feature viene attivata.
pub(crate) fn aggiorna_embedding(
    conn: &Connection,
    rt_state: &EmbeddingsState,
    prompt_id: &str,
    body: &str,
) -> Result<(), PapErrore> {
    match compute_embedding_opt(rt_state, body)? {
        Some(emb) => {
            embeddings_store::upsert_embedding(conn, prompt_id, &emb)?;
            log::debug!("embedding upserted per {prompt_id}");
        }
        None => {
            log::debug!("embedding skipped per {prompt_id} (Session non loaded)");
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct NuovoPrompt {
    pub titolo: String,
    pub descrizione: String,
    pub body: String,
    pub visibilita: String,
    pub tag_nomi: Vec<String>,
    #[serde(default)]
    pub target_model: Option<String>,
    #[serde(default)]
    pub folder_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AggiornamentoPrompt {
    pub id: String,
    pub titolo: String,
    pub descrizione: String,
    pub body: String,
    pub visibilita: String,
    pub tag_nomi: Vec<String>,
    #[serde(default)]
    pub target_model: Option<String>,
    #[serde(default)]
    pub folder_id: Option<String>,
    /// Issue #158: se `false` skippa lo snapshot in `PromptVersions`
    /// (autosave silenzioso, no spam cronologia). Default `true` per
    /// back-compat con client che non passano il flag (= comportamento
    /// pre v0.8.5: ogni save creava versione).
    #[serde(default = "default_crea_snapshot")]
    pub crea_snapshot: bool,
}

fn default_crea_snapshot() -> bool {
    true
}

fn normalizza_target_model(v: &Option<String>) -> Option<String> {
    v.as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub(crate) fn genera_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut rnd = [0u8; 4];
    rand::rngs::OsRng.fill_bytes(&mut rnd);
    format!(
        "{:012x}{:02x}{:02x}{:02x}{:02x}",
        ts, rnd[0], rnd[1], rnd[2], rnd[3]
    )
}

fn sincronizza_tags(
    conn: &Connection,
    rt_state: &EmbeddingsState,
    prompt_id: &str,
    tag_nomi: &[String],
) -> Result<(), PapErrore> {
    conn.execute("DELETE FROM PromptTags WHERE PromptId = ?1", [prompt_id])?;

    for nome in tag_nomi {
        let nome = nome.trim();
        if nome.is_empty() {
            continue;
        }

        let tag_id: String = match conn.query_row(
            "SELECT Id FROM Tags
             WHERE Name = ?1 AND WorkspaceId = 'ws-personale' AND DeletedAt IS NULL",
            [nome],
            |r| r.get(0),
        ) {
            Ok(id) => id,
            Err(_) => {
                let id = format!("tag-{}", genera_id());
                conn.execute(
                    "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
                     VALUES (?1, 'ws-personale', ?2, datetime('now'), datetime('now'))",
                    rusqlite::params![id, nome],
                )?;
                // Hook embedding tag (Fase 3 Step 4): se Session loaded,
                // calcola embedding del nome e upsert in TagsEmbeddings.
                // No-op se non disponibile (backfill futuro popolerà).
                if let Ok(Some(emb)) = compute_embedding_opt(rt_state, nome) {
                    if let Err(e) = embeddings_store::upsert_tag_embedding(conn, &id, &emb) {
                        log::warn!("upsert tag embedding fallito per {id}: {e}");
                    }
                }
                id
            }
        };

        conn.execute(
            "INSERT OR IGNORE INTO PromptTags (PromptId, TagId) VALUES (?1, ?2)",
            rusqlite::params![prompt_id, tag_id],
        )?;
    }

    Ok(())
}

pub(crate) fn ricostruisci_fts(conn: &Connection) -> Result<(), PapErrore> {
    conn.execute_batch("DELETE FROM PromptsFts")?;
    conn.execute_batch(
        "INSERT INTO PromptsFts(PromptId, Title, Description, Body, Tags)
         SELECT p.Id, p.Title, COALESCE(p.Description, ''), p.Body,
                COALESCE(GROUP_CONCAT(t.Name, ' '), '')
         FROM Prompts p
         LEFT JOIN PromptTags pt ON pt.PromptId = p.Id
         LEFT JOIN Tags t ON t.Id = pt.TagId
         WHERE p.DeletedAt IS NULL
         GROUP BY p.Id",
    )?;
    Ok(())
}

#[tauri::command]
pub fn prompt_crea(
    dati: NuovoPrompt,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<String, PapErrore> {
    state.with_conn(|conn| {
        let id = format!("prm-{}", genera_id());
        let target = normalizza_target_model(&dati.target_model);
        let folder = normalizza_target_model(&dati.folder_id);
        let body_clean = dati.body.trim();
        conn.execute(
            "INSERT INTO Prompts
                (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
                 Visibility, TargetModel, FolderId, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, ?4, ?5, ?6, ?7, 1,
                     datetime('now'), datetime('now'))",
            rusqlite::params![
                id,
                dati.titolo.trim(),
                dati.descrizione.trim(),
                body_clean,
                dati.visibilita,
                target,
                folder,
            ],
        )?;
        sincronizza_tags(conn, &rt_state, &id, &dati.tag_nomi)?;
        // Snapshot v1 in PromptVersions (Fase 2 versioning).
        crate::versioning::snapshot_versione(conn, &id, "usr-locale")?;
        ricostruisci_fts(conn)?;
        // Hook embedding (Fase 3 Step 3): no-op se Session non loaded.
        aggiorna_embedding(conn, &rt_state, &id, body_clean)?;
        // Hook import graph (Fase 3 Step 8): popola PromptImports.
        prompt_componibili::aggiorna_imports(conn, &id, body_clean)?;
        crate::audit::registra(conn, "prompt.creato", "Prompt", &id, Some(dati.titolo.trim()));
        log::info!("Prompt creato: {id}");
        Ok(id)
    })
}

#[tauri::command]
pub fn prompt_aggiorna(
    dati: AggiornamentoPrompt,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        let target = normalizza_target_model(&dati.target_model);
        let folder = normalizza_target_model(&dati.folder_id);
        let body_clean = dati.body.trim();
        // Issue #158: Version cresce solo al snapshot per mantenere
        // 1:1 tra Version field e righe in PromptVersions. Autosave
        // (crea_snapshot=false) aggiorna body/meta + UpdatedAt ma
        // lascia Version invariato.
        conn.execute(
            "UPDATE Prompts
             SET Title = ?1, Description = ?2, Body = ?3, Visibility = ?4,
                 TargetModel = ?5, FolderId = ?6,
                 Version = CASE WHEN ?8 THEN Version + 1 ELSE Version END,
                 UpdatedAt = datetime('now'),
                 UpdatedByUserId = 'usr-locale'
             WHERE Id = ?7 AND DeletedAt IS NULL",
            rusqlite::params![
                dati.titolo.trim(),
                dati.descrizione.trim(),
                body_clean,
                dati.visibilita,
                target,
                folder,
                dati.id,
                dati.crea_snapshot,
            ],
        )?;
        sincronizza_tags(conn, &rt_state, &dati.id, &dati.tag_nomi)?;
        // Issue #158: snapshot solo se richiesto dal client (default true
        // per back-compat). Autosave passa `crea_snapshot: false` per
        // evitare spam cronologia; salvataggio manuale "Salva" passa true.
        // Version comunque incrementata dall'UPDATE sopra (autosave la
        // bumpa anche senza snapshot — ok perché la riga corrente in
        // Prompts riflette sempre l'ultimo stato).
        if dati.crea_snapshot {
            crate::versioning::snapshot_versione(conn, &dati.id, "usr-locale")?;
        }
        ricostruisci_fts(conn)?;
        // Hook embedding (Fase 3 Step 3): re-compute perché il body è cambiato.
        // No-op se Session non loaded.
        aggiorna_embedding(conn, &rt_state, &dati.id, body_clean)?;
        // Hook import graph (Fase 3 Step 8): re-popola PromptImports.
        prompt_componibili::aggiorna_imports(conn, &dati.id, body_clean)?;
        crate::audit::registra(conn, "prompt.aggiornato", "Prompt", &dati.id, Some(dati.titolo.trim()));
        log::info!("Prompt aggiornato: {}", dati.id);
        Ok(())
    })
}

#[tauri::command]
pub fn prompt_registra_uso(id: String, state: State<'_, VaultState>) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        conn.execute(
            "UPDATE Prompts SET UseCount = UseCount + 1, LastUsedAt = datetime('now')
             WHERE Id = ?1 AND DeletedAt IS NULL",
            [&id],
        )?;
        Ok(())
    })
}

#[tauri::command]
pub fn prompt_elimina(id: String, state: State<'_, VaultState>) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        conn.execute(
            "UPDATE Prompts SET DeletedAt = datetime('now') WHERE Id = ?1 AND DeletedAt IS NULL",
            [&id],
        )?;
        conn.execute("DELETE FROM PromptTags WHERE PromptId = ?1", [&id])?;
        ricostruisci_fts(conn)?;
        // Pulizia vec0: la riga embedding non serve più. Sicuro chiamarlo
        // anche se non c'era mai stato un embedding (delete è no-op).
        embeddings_store::delete_embedding(conn, &id)?;
        // Cleanup PromptImports (Fase 3 Step 8).
        prompt_componibili::pulisci_imports(conn, &id)?;
        crate::audit::registra(conn, "prompt.eliminato", "Prompt", &id, None);
        log::info!("Prompt eliminato: {id}");
        Ok(())
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    #[test]
    fn genera_id_formato() {
        let id = genera_id();
        assert_eq!(id.len(), 20);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn genera_id_univoco() {
        let id1 = genera_id();
        let id2 = genera_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn crea_prompt_e_ricostruisci_fts() {
        let conn = db_test();
        let id = format!("prm-{}", genera_id());
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
             Visibility, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', 'Test Prompt', 'desc', 'corpo', 'private', 1,
             datetime('now'), datetime('now'))",
            rusqlite::params![id],
        )
        .unwrap();
        ricostruisci_fts(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptsFts WHERE PromptsFts MATCH 'Test*'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn sincronizza_tags_crea_e_associa() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt)
             VALUES ('prm-t1', 'ws-personale', 'usr-locale', 'T', 'b', 'private', 1,
             datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let tags = vec!["rust".to_string(), "test".to_string()];
        sincronizza_tags(&conn, &EmbeddingsState::new(), "prm-t1", &tags).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptTags WHERE PromptId = 'prm-t1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn sincronizza_tags_ignora_vuoti() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt)
             VALUES ('prm-t2', 'ws-personale', 'usr-locale', 'T', 'b', 'private', 1,
             datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let tags = vec!["rust".to_string(), "".to_string(), "  ".to_string()];
        sincronizza_tags(&conn, &EmbeddingsState::new(), "prm-t2", &tags).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptTags WHERE PromptId = 'prm-t2'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn sincronizza_tags_riuso_esistenti() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('tag-pre', 'ws-personale', 'esistente', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt)
             VALUES ('prm-t3', 'ws-personale', 'usr-locale', 'T', 'b', 'private', 1,
             datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let tags = vec!["esistente".to_string()];
        sincronizza_tags(&conn, &EmbeddingsState::new(), "prm-t3", &tags).unwrap();

        let tag_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM Tags", [], |r| r.get(0)).unwrap();
        assert_eq!(tag_count, 1, "Non deve creare duplicati");
    }

    #[test]
    fn ricostruisci_fts_idempotente() {
        let conn = db_test();
        ricostruisci_fts(&conn).unwrap();
        ricostruisci_fts(&conn).unwrap();
    }

    #[test]
    fn normalizza_target_model_casi() {
        assert_eq!(normalizza_target_model(&None), None);
        assert_eq!(normalizza_target_model(&Some("".into())), None);
        assert_eq!(normalizza_target_model(&Some("   ".into())), None);
        assert_eq!(
            normalizza_target_model(&Some("claude-opus".into())),
            Some("claude-opus".into())
        );
        assert_eq!(
            normalizza_target_model(&Some("  gpt-4  ".into())),
            Some("gpt-4".into())
        );
    }
}
