//! Issue #159: storage per segnaposti globali.
//!
//! Tabella `GlobalPlaceholders(Name PK, Value, UpdatedAt)` (V015).
//! 3 cmd Tauri esposti:
//! - `globale_placeholder_lista` — ritorna tutti gli (name, value)
//!   ordinati per name. Usato da ImpostazioniModal (tabella editabile)
//!   e CompilaModal (pre-fill valore default).
//! - `globale_placeholder_aggiorna` — UPSERT (name, value).
//! - `globale_placeholder_elimina` — DELETE per name.

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

#[derive(Debug, Serialize, Clone)]
pub struct PlaceholderGlobale {
    pub name: String,
    pub value: String,
    pub updated_at: String,
}

#[tauri::command]
pub fn globale_placeholder_lista(
    state: State<'_, VaultState>,
) -> Result<Vec<PlaceholderGlobale>, PapErrore> {
    state.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT Name, Value, UpdatedAt FROM GlobalPlaceholders \
             ORDER BY Name COLLATE NOCASE ASC",
        )?;
        let righe: Vec<PlaceholderGlobale> = stmt
            .query_map([], |r| {
                Ok(PlaceholderGlobale {
                    name: r.get(0)?,
                    value: r.get(1)?,
                    updated_at: r.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(righe)
    })
}

#[derive(Debug, Deserialize)]
pub struct AggiornaPlaceholder {
    pub name: String,
    pub value: String,
}

#[tauri::command]
pub fn globale_placeholder_aggiorna(
    dati: AggiornaPlaceholder,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    let nome = dati.name.trim();
    if nome.is_empty() {
        return Err(PapErrore::Generico(
            "Nome segnaposto globale vuoto".to_string(),
        ));
    }
    state.with_conn(|conn| {
        conn.execute(
            "INSERT INTO GlobalPlaceholders (Name, Value, UpdatedAt) \
             VALUES (?1, ?2, datetime('now')) \
             ON CONFLICT(Name) DO UPDATE SET \
                Value = excluded.Value, \
                UpdatedAt = datetime('now')",
            params![nome, dati.value],
        )?;
        Ok(())
    })
}

#[tauri::command]
pub fn globale_placeholder_elimina(
    name: String,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        conn.execute(
            "DELETE FROM GlobalPlaceholders WHERE Name = ?1",
            params![name],
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod test {
    use rusqlite::{params, Connection};

    fn db_temp() -> Connection {
        // sqlite-vec è auto-extension globale (vedi embeddings_store).
        // Va registrata prima di Connection::open_in_memory altrimenti
        // V005 (embeddings) fallisce con "no such module: vec0".
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        conn
    }

    #[test]
    fn upsert_inserisce_nuovo() {
        let conn = db_temp();
        conn.execute(
            "INSERT INTO GlobalPlaceholders (Name, Value, UpdatedAt) \
             VALUES (?1, ?2, datetime('now'))",
            params!["autore", "Pippo"],
        )
        .unwrap();
        let v: String = conn
            .query_row(
                "SELECT Value FROM GlobalPlaceholders WHERE Name = 'autore'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(v, "Pippo");
    }

    #[test]
    fn upsert_aggiorna_esistente() {
        let conn = db_temp();
        conn.execute(
            "INSERT INTO GlobalPlaceholders (Name, Value, UpdatedAt) \
             VALUES (?1, ?2, datetime('now'))",
            params!["autore", "Pippo"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO GlobalPlaceholders (Name, Value, UpdatedAt) \
             VALUES (?1, ?2, datetime('now')) \
             ON CONFLICT(Name) DO UPDATE SET Value = excluded.Value",
            params!["autore", "Topolino"],
        )
        .unwrap();
        let v: String = conn
            .query_row(
                "SELECT Value FROM GlobalPlaceholders WHERE Name = 'autore'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(v, "Topolino");
    }

    #[test]
    fn delete_rimuove() {
        let conn = db_temp();
        conn.execute(
            "INSERT INTO GlobalPlaceholders (Name, Value, UpdatedAt) \
             VALUES ('autore', 'Pippo', datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "DELETE FROM GlobalPlaceholders WHERE Name = ?1",
            params!["autore"],
        )
        .unwrap();
        let conta: i64 = conn
            .query_row("SELECT COUNT(*) FROM GlobalPlaceholders", [], |r| r.get(0))
            .unwrap();
        assert_eq!(conta, 0);
    }
}
