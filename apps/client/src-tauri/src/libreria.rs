use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

#[derive(Debug, Serialize, Clone)]
pub struct TagInfo {
    pub id: String,
    pub nome: String,
    pub colore: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PromptCard {
    pub id: String,
    pub titolo: String,
    pub descrizione: String,
    pub visibilita: String,
    pub preferito: bool,
    pub uso_count: i64,
    pub aggiornato_a: String,
    pub tags: Vec<TagInfo>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PromptDettaglio {
    pub id: String,
    pub titolo: String,
    pub descrizione: String,
    pub body: String,
    pub visibilita: String,
    pub target_model: String,
    pub folder_id: Option<String>,
    pub preferito: bool,
    pub uso_count: i64,
    pub creato_a: String,
    pub aggiornato_a: String,
    pub ultimo_uso: String,
    pub tags: Vec<TagInfo>,
}

#[derive(Debug, Serialize)]
pub struct ConteggiViste {
    pub tutti: i64,
    pub preferiti: i64,
    pub privati: i64,
    pub team: i64,
}

#[derive(Debug, Deserialize)]
pub struct FiltroLista {
    pub vista: String,
    pub tag_id: Option<String>,
    pub cerca: Option<String>,
    pub ordine: String,
    #[serde(default)]
    pub target_model: Option<String>,
    /// Filtra per cartella. `Some("__nessuna__")` → solo prompt senza cartella;
    /// `Some(id)` → prompt nel sottoalbero di quella cartella; `None` → no filtro.
    #[serde(default)]
    pub folder_id: Option<String>,
}

fn carica_tags(conn: &Connection, prompt_id: &str) -> Result<Vec<TagInfo>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT t.Id, t.Name, COALESCE(t.Color, '')
         FROM Tags t
         JOIN PromptTags pt ON pt.TagId = t.Id
         WHERE pt.PromptId = ?1 AND t.DeletedAt IS NULL",
    )?;
    let tags = stmt
        .query_map([prompt_id], |row| {
            Ok(TagInfo {
                id: row.get(0)?,
                nome: row.get(1)?,
                colore: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(tags)
}

fn riga_a_card(row: &rusqlite::Row) -> rusqlite::Result<PromptCard> {
    Ok(PromptCard {
        id: row.get(0)?,
        titolo: row.get(1)?,
        descrizione: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        visibilita: row.get(3)?,
        preferito: row.get::<_, i64>(4)? != 0,
        uso_count: row.get(5)?,
        aggiornato_a: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
        tags: vec![],
    })
}

/// Inserisce workspace "Personale" e utente locale se il DB è vuoto.
pub fn assicura_dati_base(conn: &Connection) -> Result<(), PapErrore> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM Workspaces", [], |r| r.get(0))?;
    if count == 0 {
        conn.execute(
            "INSERT INTO Workspaces (Id, Name, Type, CreatedAt, UpdatedAt)
             VALUES ('ws-personale', 'Personale', 'personal', datetime('now'), datetime('now'))",
            [],
        )?;
        conn.execute(
            "INSERT INTO Users (Id, WorkspaceId, DisplayName, Role, CreatedAt, UpdatedAt)
             VALUES ('usr-locale', 'ws-personale', 'Utente locale', 'Admin', datetime('now'), datetime('now'))",
            [],
        )?;
        log::info!("Dati base creati: workspace Personale + utente locale");
    }
    Ok(())
}

#[tauri::command]
pub fn libreria_conteggi(state: State<'_, VaultState>) -> Result<ConteggiViste, PapErrore> {
    state.with_conn(|conn| {
        let q = |cond: &str| -> Result<i64, PapErrore> {
            let sql = format!("SELECT COUNT(*) FROM Prompts WHERE DeletedAt IS NULL{cond}");
            Ok(conn.query_row(&sql, [], |r| r.get(0))?)
        };
        Ok(ConteggiViste {
            tutti: q("")?,
            preferiti: q(" AND IsFavorite = 1")?,
            privati: q(" AND Visibility = 'private'")?,
            team: q(" AND Visibility = 'workspace'")?,
        })
    })
}

#[tauri::command]
pub fn libreria_lista(
    filtro: FiltroLista,
    state: State<'_, VaultState>,
) -> Result<Vec<PromptCard>, PapErrore> {
    state.with_conn(|conn| {
        let cerca_param: Option<String> = filtro
            .cerca
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| format!("%{}%", s.trim()));

        let vista_cond = match filtro.vista.as_str() {
            "preferiti" => " AND p.IsFavorite = 1",
            "privati" => " AND p.Visibility = 'private'",
            "team" => " AND p.Visibility = 'workspace'",
            _ => "",
        };

        let ordine = match filtro.ordine.as_str() {
            "popolare" => "p.UseCount DESC, p.UpdatedAt DESC",
            "alfabetico" => "p.Title COLLATE NOCASE ASC",
            _ => "COALESCE(p.LastUsedAt, p.UpdatedAt) DESC",
        };

        let target_model_param: Option<String> = filtro
            .target_model
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .cloned();

        // Filtro cartella:
        // - None / vuoto / id non trovato → nessun filtro
        // - "__nessuna__" → solo root (FolderId IS NULL)
        // - id valido → sottoalbero (id stesso + tutti i discendenti via Path)
        let folder_solo_root = filtro.folder_id.as_deref() == Some("__nessuna__");
        let folder_path: Option<String> = match filtro.folder_id.as_deref() {
            None | Some("") | Some("__nessuna__") => None,
            Some(id) => conn
                .query_row(
                    "SELECT Path FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
                    [id],
                    |r| r.get(0),
                )
                .ok(),
        };
        let folder_path_prefix_like = folder_path.as_ref().map(|p| format!("{p}/%"));

        // SQL: tutti i named param sono sempre presenti; le condizioni
        // controllano l'attivazione via NULL check / boolean flag.
        let sql = format!(
            "SELECT p.Id, p.Title, p.Description, p.Visibility,
                    p.IsFavorite, p.UseCount, p.UpdatedAt
             FROM Prompts p
             LEFT JOIN PromptTags pt ON pt.PromptId = p.Id
             WHERE p.DeletedAt IS NULL{vista_cond}
               AND (:cerca IS NULL OR p.Title LIKE :cerca OR p.Description LIKE :cerca)
               AND (:tag_id IS NULL OR pt.TagId = :tag_id)
               AND (:target_model IS NULL OR p.TargetModel = :target_model)
               AND (NOT :folder_solo_root OR p.FolderId IS NULL)
               AND (
                   :folder_path IS NULL
                   OR p.FolderId IN (
                       SELECT Id FROM Folders
                       WHERE DeletedAt IS NULL
                         AND (Path = :folder_path OR Path LIKE :folder_prefix)
                   )
               )
             GROUP BY p.Id
             ORDER BY {ordine}
             LIMIT 100"
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut cards: Vec<PromptCard> = stmt
            .query_map(
                rusqlite::named_params! {
                    ":cerca": cerca_param,
                    ":tag_id": filtro.tag_id,
                    ":target_model": target_model_param,
                    ":folder_solo_root": folder_solo_root,
                    ":folder_path": folder_path,
                    ":folder_prefix": folder_path_prefix_like,
                },
                riga_a_card,
            )?
            .filter_map(|r| r.ok())
            .collect();

        for card in &mut cards {
            card.tags = carica_tags(conn, &card.id)?;
        }
        Ok(cards)
    })
}

#[tauri::command]
pub fn libreria_dettaglio(
    id: String,
    state: State<'_, VaultState>,
) -> Result<PromptDettaglio, PapErrore> {
    state.with_conn(|conn| {
        let mut det = conn.query_row(
            "SELECT Id, Title, Description, Body, Visibility, TargetModel,
                    FolderId, IsFavorite, UseCount, CreatedAt, UpdatedAt, LastUsedAt
             FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
            [&id],
            |row| {
                Ok(PromptDettaglio {
                    id: row.get(0)?,
                    titolo: row.get(1)?,
                    descrizione: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    body: row.get(3)?,
                    visibilita: row.get(4)?,
                    target_model: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                    folder_id: row.get::<_, Option<String>>(6)?,
                    preferito: row.get::<_, i64>(7)? != 0,
                    uso_count: row.get(8)?,
                    creato_a: row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                    aggiornato_a: row.get::<_, Option<String>>(10)?.unwrap_or_default(),
                    ultimo_uso: row.get::<_, Option<String>>(11)?.unwrap_or_default(),
                    tags: vec![],
                })
            },
        )?;
        det.tags = carica_tags(conn, &det.id)?;
        Ok(det)
    })
}

#[tauri::command]
pub fn libreria_toggle_preferito(
    id: String,
    state: State<'_, VaultState>,
) -> Result<bool, PapErrore> {
    state.with_conn(|conn| {
        let attuale: i64 = conn.query_row(
            "SELECT IsFavorite FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
            [&id],
            |r| r.get(0),
        )?;
        let nuovo = if attuale != 0 { 0 } else { 1 };
        conn.execute(
            "UPDATE Prompts SET IsFavorite = ?1, UpdatedAt = datetime('now') WHERE Id = ?2",
            rusqlite::params![nuovo, id],
        )?;
        let meta = if nuovo != 0 { "aggiunto" } else { "rimosso" };
        crate::audit::registra(conn, "prompt.preferito", "Prompt", &id, Some(meta));
        Ok(nuovo != 0)
    })
}

#[tauri::command]
pub fn libreria_tag_lista(state: State<'_, VaultState>) -> Result<Vec<TagInfo>, PapErrore> {
    state.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT Id, Name, COALESCE(Color, '')
             FROM Tags WHERE DeletedAt IS NULL
             ORDER BY Name COLLATE NOCASE ASC",
        )?;
        let tags = stmt
            .query_map([], |row| {
                Ok(TagInfo {
                    id: row.get(0)?,
                    nome: row.get(1)?,
                    colore: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(tags)
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn db_test() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        conn
    }

    #[test]
    fn assicura_dati_base_crea_workspace_e_utente() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();

        let ws: i64 =
            conn.query_row("SELECT COUNT(*) FROM Workspaces", [], |r| r.get(0)).unwrap();
        let usr: i64 = conn.query_row("SELECT COUNT(*) FROM Users", [], |r| r.get(0)).unwrap();
        assert_eq!(ws, 1);
        assert_eq!(usr, 1);
    }

    #[test]
    fn assicura_dati_base_idempotente() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        assicura_dati_base(&conn).unwrap();

        let ws: i64 =
            conn.query_row("SELECT COUNT(*) FROM Workspaces", [], |r| r.get(0)).unwrap();
        assert_eq!(ws, 1);
    }

    #[test]
    fn conteggi_vuoti() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts WHERE DeletedAt IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn carica_tags_vuoto() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();

        let tags = carica_tags(&conn, "prm-inesistente").unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn carica_tags_con_associazione() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt)
             VALUES ('prm-1', 'ws-personale', 'usr-locale', 'Test', 'b', 'private', 1,
             datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('tag-1', 'ws-personale', 'rust', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO PromptTags (PromptId, TagId) VALUES ('prm-1', 'tag-1')",
            [],
        )
        .unwrap();

        let tags = carica_tags(&conn, "prm-1").unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].nome, "rust");
    }

    #[test]
    fn toggle_preferito() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             IsFavorite, CreatedAt, UpdatedAt)
             VALUES ('prm-1', 'ws-personale', 'usr-locale', 'Test', 'b', 'private', 1, 0,
             datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let fav: i64 = conn
            .query_row(
                "SELECT IsFavorite FROM Prompts WHERE Id = 'prm-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fav, 0);

        conn.execute(
            "UPDATE Prompts SET IsFavorite = 1 WHERE Id = 'prm-1'",
            [],
        )
        .unwrap();
        let fav: i64 = conn
            .query_row(
                "SELECT IsFavorite FROM Prompts WHERE Id = 'prm-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fav, 1);

        conn.execute(
            "UPDATE Prompts SET IsFavorite = 0 WHERE Id = 'prm-1'",
            [],
        )
        .unwrap();
        let fav: i64 = conn
            .query_row(
                "SELECT IsFavorite FROM Prompts WHERE Id = 'prm-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fav, 0);
    }

    #[test]
    fn lista_filtra_eliminati() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt)
             VALUES ('prm-att', 'ws-personale', 'usr-locale', 'Attivo', 'b', 'private', 1,
             datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt, DeletedAt)
             VALUES ('prm-del', 'ws-personale', 'usr-locale', 'Eliminato', 'b', 'private', 1,
             datetime('now'), datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts WHERE DeletedAt IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
