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
        if query.trim().is_empty() {
            let mut stmt = conn.prepare(
                "SELECT Id, Title, Description, Body, Visibility, IsFavorite, UseCount
                 FROM Prompts
                 WHERE DeletedAt IS NULL
                 ORDER BY COALESCE(LastUsedAt, UpdatedAt) DESC
                 LIMIT 20",
            )?;
            let risultati: Vec<PromptRisultato> = stmt
                .query_map([], riga_a_prompt)?
                .filter_map(|r| r.ok())
                .collect();
            Ok(risultati)
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
            let risultati: Vec<PromptRisultato> = stmt
                .query_map([&fts], riga_a_prompt)?
                .filter_map(|r| r.ok())
                .collect();
            Ok(risultati)
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sanitizza_stringa_vuota() {
        assert_eq!(sanitizza_fts(""), "");
    }

    #[test]
    fn sanitizza_spazi() {
        assert_eq!(sanitizza_fts("   "), "");
    }

    #[test]
    fn sanitizza_parola_singola() {
        assert_eq!(sanitizza_fts("hello"), "hello*");
    }

    #[test]
    fn sanitizza_parole_multiple() {
        assert_eq!(sanitizza_fts("hello world"), "hello* world*");
    }

    #[test]
    fn sanitizza_caratteri_speciali() {
        assert_eq!(sanitizza_fts("hello! @world#"), "hello* world*");
    }

    #[test]
    fn sanitizza_solo_speciali() {
        assert_eq!(sanitizza_fts("@#$%"), "");
    }

    #[test]
    fn sanitizza_underscore_preservato() {
        assert_eq!(sanitizza_fts("hello_world"), "hello_world*");
    }

    #[test]
    fn cerca_su_db_vuoto() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        crate::editor::ricostruisci_fts(&conn).unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT Id, Title, Description, Body, Visibility, IsFavorite, UseCount
                 FROM Prompts WHERE DeletedAt IS NULL ORDER BY UpdatedAt DESC LIMIT 20",
            )
            .unwrap();
        let risultati: Vec<PromptRisultato> = stmt
            .query_map([], riga_a_prompt)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(risultati.len(), 0);
    }

    #[test]
    fn cerca_fts_trova_prompt() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();

        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-1', 'ws-personale', 'usr-locale', 'Email marketing', 'Scrivi una email', 'private', 1, datetime('now'), datetime('now'))",
            [],
        ).unwrap();
        crate::editor::ricostruisci_fts(&conn).unwrap();

        let fts = sanitizza_fts("email");
        let mut stmt = conn
            .prepare(
                "SELECT f.PromptId, p.Title, p.Description, p.Body,
                        p.Visibility, p.IsFavorite, p.UseCount
                 FROM PromptsFts f JOIN Prompts p ON f.PromptId = p.Id
                 WHERE PromptsFts MATCH ?1 AND p.DeletedAt IS NULL LIMIT 20",
            )
            .unwrap();
        let risultati: Vec<PromptRisultato> = stmt
            .query_map([&fts], riga_a_prompt)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(risultati.len(), 1);
        assert_eq!(risultati[0].titolo, "Email marketing");
    }
}
