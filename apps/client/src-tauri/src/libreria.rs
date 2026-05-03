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

        let sql = format!(
            "SELECT p.Id, p.Title, p.Description, p.Visibility,
                    p.IsFavorite, p.UseCount, p.UpdatedAt
             FROM Prompts p
             LEFT JOIN PromptTags pt ON pt.PromptId = p.Id
             WHERE p.DeletedAt IS NULL{vista_cond}
               AND (:cerca IS NULL OR p.Title LIKE :cerca OR p.Description LIKE :cerca)
               AND (:tag_id IS NULL OR pt.TagId = :tag_id)
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
                    IsFavorite, UseCount, CreatedAt, UpdatedAt, LastUsedAt
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
                    preferito: row.get::<_, i64>(6)? != 0,
                    uso_count: row.get(7)?,
                    creato_a: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                    aggiornato_a: row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                    ultimo_uso: row.get::<_, Option<String>>(10)?.unwrap_or_default(),
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
