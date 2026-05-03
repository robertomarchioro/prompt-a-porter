use rand::RngCore;
use rusqlite::Connection;
use serde::Deserialize;
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

#[derive(Debug, Deserialize)]
pub struct NuovoPrompt {
    pub titolo: String,
    pub descrizione: String,
    pub body: String,
    pub visibilita: String,
    pub tag_nomi: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AggiornamentoPrompt {
    pub id: String,
    pub titolo: String,
    pub descrizione: String,
    pub body: String,
    pub visibilita: String,
    pub tag_nomi: Vec<String>,
}

fn genera_id() -> String {
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
) -> Result<String, PapErrore> {
    state.with_conn(|conn| {
        let id = format!("prm-{}", genera_id());
        conn.execute(
            "INSERT INTO Prompts
                (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
                 Visibility, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, ?4, ?5, 1,
                     datetime('now'), datetime('now'))",
            rusqlite::params![
                id,
                dati.titolo.trim(),
                dati.descrizione.trim(),
                dati.body.trim(),
                dati.visibilita
            ],
        )?;
        sincronizza_tags(conn, &id, &dati.tag_nomi)?;
        ricostruisci_fts(conn)?;
        crate::audit::registra(conn, "prompt.creato", "Prompt", &id, Some(dati.titolo.trim()));
        log::info!("Prompt creato: {id}");
        Ok(id)
    })
}

#[tauri::command]
pub fn prompt_aggiorna(
    dati: AggiornamentoPrompt,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        conn.execute(
            "UPDATE Prompts
             SET Title = ?1, Description = ?2, Body = ?3, Visibility = ?4,
                 Version = Version + 1, UpdatedAt = datetime('now'),
                 UpdatedByUserId = 'usr-locale'
             WHERE Id = ?5 AND DeletedAt IS NULL",
            rusqlite::params![
                dati.titolo.trim(),
                dati.descrizione.trim(),
                dati.body.trim(),
                dati.visibilita,
                dati.id
            ],
        )?;
        sincronizza_tags(conn, &dati.id, &dati.tag_nomi)?;
        ricostruisci_fts(conn)?;
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
        crate::audit::registra(conn, "prompt.eliminato", "Prompt", &id, None);
        log::info!("Prompt eliminato: {id}");
        Ok(())
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn db_test() -> Connection {
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
        sincronizza_tags(&conn, "prm-t1", &tags).unwrap();

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
        sincronizza_tags(&conn, "prm-t2", &tags).unwrap();

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
        sincronizza_tags(&conn, "prm-t3", &tags).unwrap();

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
}
