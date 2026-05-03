use serde::Deserialize;
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

#[derive(Debug, Deserialize)]
pub struct SyncDelta {
    pub prompts: Vec<SyncPrompt>,
    pub tags: Vec<SyncTag>,
    #[serde(rename = "promptTags")]
    pub prompt_tags: Vec<SyncPromptTag>,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPrompt {
    pub id: String,
    pub workspace_id: String,
    pub author_user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    pub visibility: String,
    pub target_model: Option<String>,
    pub is_favorite: i32,
    pub use_count: i32,
    pub last_used_at: Option<String>,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
    pub updated_by_user_id: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncTag {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPromptTag {
    pub prompt_id: String,
    pub tag_id: String,
}

#[tauri::command]
pub fn sync_applica_delta(delta: SyncDelta, state: State<'_, VaultState>) -> Result<u32, PapErrore> {
    state.with_conn(|conn| {
        let mut applicati: u32 = 0;

        for tag in &delta.tags {
            let esiste: bool = conn
                .query_row("SELECT COUNT(*) FROM Tags WHERE Id = ?1", [&tag.id], |r| {
                    r.get::<_, i32>(0)
                })
                .map(|c| c > 0)
                .unwrap_or(false);

            if esiste {
                conn.execute(
                    "UPDATE Tags SET Name = ?1, Color = ?2, UpdatedAt = ?3, DeletedAt = ?4
                     WHERE Id = ?5 AND UpdatedAt < ?3",
                    rusqlite::params![tag.name, tag.color, tag.updated_at, tag.deleted_at, tag.id],
                )?;
            } else {
                conn.execute(
                    "INSERT OR IGNORE INTO Tags (Id, WorkspaceId, Name, Color, CreatedAt, UpdatedAt, DeletedAt)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        tag.id,
                        tag.workspace_id,
                        tag.name,
                        tag.color,
                        tag.created_at,
                        tag.updated_at,
                        tag.deleted_at
                    ],
                )?;
            }
            applicati += 1;
        }

        for p in &delta.prompts {
            let esiste: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM Prompts WHERE Id = ?1",
                    [&p.id],
                    |r| r.get::<_, i32>(0),
                )
                .map(|c| c > 0)
                .unwrap_or(false);

            if esiste {
                conn.execute(
                    "UPDATE Prompts SET Title = ?1, Description = ?2, Body = ?3,
                     Visibility = ?4, TargetModel = ?5, Version = ?6,
                     UpdatedAt = ?7, UpdatedByUserId = ?8, DeletedAt = ?9
                     WHERE Id = ?10 AND UpdatedAt < ?7",
                    rusqlite::params![
                        p.title,
                        p.description,
                        p.body,
                        p.visibility,
                        p.target_model,
                        p.version,
                        p.updated_at,
                        p.updated_by_user_id,
                        p.deleted_at,
                        p.id
                    ],
                )?;
            } else {
                conn.execute(
                    "INSERT OR IGNORE INTO Prompts
                     (Id, WorkspaceId, AuthorUserId, Title, Description, Body, Visibility,
                      TargetModel, IsFavorite, UseCount, LastUsedAt, Version,
                      CreatedAt, UpdatedAt, UpdatedByUserId, DeletedAt)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                    rusqlite::params![
                        p.id,
                        p.workspace_id,
                        p.author_user_id,
                        p.title,
                        p.description,
                        p.body,
                        p.visibility,
                        p.target_model,
                        p.is_favorite,
                        p.use_count,
                        p.last_used_at,
                        p.version,
                        p.created_at,
                        p.updated_at,
                        p.updated_by_user_id,
                        p.deleted_at
                    ],
                )?;
            }
            applicati += 1;
        }

        for pt in &delta.prompt_tags {
            conn.execute(
                "INSERT OR IGNORE INTO PromptTags (PromptId, TagId) VALUES (?1, ?2)",
                rusqlite::params![pt.prompt_id, pt.tag_id],
            )?;
        }

        crate::editor::ricostruisci_fts(conn)?;
        crate::audit::registra(conn, "sync.delta_applicato", "Sync", "", Some(&format!("{applicati} entità")));

        Ok(applicati)
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn db_test() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    #[test]
    fn inserisci_tag_nuovo() {
        let conn = db_test();
        conn.execute(
            "INSERT OR IGNORE INTO Tags (Id, WorkspaceId, Name, Color, CreatedAt, UpdatedAt)
             VALUES ('tag-s1', 'ws-personale', 'synced', '#ff0000', '2024-01-01', '2024-01-01')",
            [],
        )
        .unwrap();

        let name: String = conn
            .query_row(
                "SELECT Name FROM Tags WHERE Id = 'tag-s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(name, "synced");
    }

    #[test]
    fn inserisci_prompt_nuovo() {
        let conn = db_test();
        conn.execute(
            "INSERT OR IGNORE INTO Prompts
             (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-s1','ws-personale','usr-locale','Synced','body','workspace',1,
             '2024-01-01','2024-01-01')",
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM Prompts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn aggiorna_solo_se_piu_recente() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('tag-1', 'ws-personale', 'originale', '2024-01-01', '2024-01-01 12:00:00')",
            [],
        )
        .unwrap();

        let changed = conn
            .execute(
                "UPDATE Tags SET Name = 'vecchio' WHERE Id = 'tag-1' AND UpdatedAt < '2024-01-01 10:00:00'",
                [],
            )
            .unwrap();
        assert_eq!(changed, 0);

        let name: String = conn
            .query_row("SELECT Name FROM Tags WHERE Id = 'tag-1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "originale");

        let changed = conn
            .execute(
                "UPDATE Tags SET Name = 'nuovo' WHERE Id = 'tag-1' AND UpdatedAt < '2024-01-02 00:00:00'",
                [],
            )
            .unwrap();
        assert_eq!(changed, 1);

        let name: String = conn
            .query_row("SELECT Name FROM Tags WHERE Id = 'tag-1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "nuovo");
    }

    #[test]
    fn prompt_tag_insert_or_ignore() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt)
             VALUES ('prm-1','ws-personale','usr-locale','T','b','private',1,
             datetime('now'),datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('tag-1','ws-personale','t',datetime('now'),datetime('now'))",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT OR IGNORE INTO PromptTags (PromptId, TagId) VALUES ('prm-1', 'tag-1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO PromptTags (PromptId, TagId) VALUES ('prm-1', 'tag-1')",
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM PromptTags", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "INSERT OR IGNORE non deve duplicare");
    }
}
