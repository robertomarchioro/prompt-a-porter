use serde::Serialize;
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

#[derive(Debug, Serialize, Clone)]
pub struct PromptRisultato {
    pub id: String,
    pub titolo: String,
    pub descrizione: String,
    pub body: String,
    pub visibilita: String,
    pub preferito: bool,
    pub uso_count: i64,
}

fn riga_a_prompt(row: &rusqlite::Row) -> rusqlite::Result<PromptRisultato> {
    Ok(PromptRisultato {
        id: row.get(0)?,
        titolo: row.get(1)?,
        descrizione: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        body: row.get(3)?,
        visibilita: row.get(4)?,
        preferito: row.get::<_, i64>(5)? != 0,
        uso_count: row.get(6)?,
    })
}

fn sanitizza_fts(query: &str) -> String {
    query
        .split_whitespace()
        .map(|w| {
            let pulito: String = w.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect();
            if pulito.is_empty() {
                String::new()
            } else {
                format!("{pulito}*")
            }
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[tauri::command]
pub fn prompt_cerca(
    query: String,
    state: State<'_, VaultState>,
) -> Result<Vec<PromptRisultato>, PapErrore> {
    state.with_conn(|conn| {
        let risultati = if query.trim().is_empty() {
            let mut stmt = conn.prepare(
                "SELECT Id, Title, Description, Body, Visibility, IsFavorite, UseCount
                 FROM Prompts
                 WHERE DeletedAt IS NULL
                 ORDER BY COALESCE(LastUsedAt, UpdatedAt) DESC
                 LIMIT 20",
            )?;
            stmt.query_map([], riga_a_prompt)?
                .filter_map(|r| r.ok())
                .collect()
        } else {
            let fts = sanitizza_fts(&query);
            if fts.is_empty() {
                return Ok(vec![]);
            }
            let mut stmt = conn.prepare(
                "SELECT f.PromptId, p.Title, p.Description, p.Body,
                        p.Visibility, p.IsFavorite, p.UseCount
                 FROM PromptsFts f
                 JOIN Prompts p ON f.PromptId = p.Id
                 WHERE PromptsFts MATCH ?1
                 AND p.DeletedAt IS NULL
                 ORDER BY rank
                 LIMIT 20",
            )?;
            stmt.query_map([&fts], riga_a_prompt)?
                .filter_map(|r| r.ok())
                .collect()
        };
        Ok(risultati)
    })
}
