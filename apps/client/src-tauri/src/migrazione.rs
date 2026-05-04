use rusqlite::Connection;

use crate::errore::PapErrore;

/// Ogni migrazione ha un numero di versione, un nome e l'SQL da eseguire.
struct Migrazione {
    versione: u32,
    nome: &'static str,
    sql: &'static str,
}

/// Elenco migrazioni embedded nel binario. Aggiungere qui le future migrazioni.
static MIGRAZIONI: &[Migrazione] = &[
    Migrazione {
        versione: 1,
        nome: "schema_iniziale",
        sql: include_str!("../migrations/V001__schema_iniziale.sql"),
    },
    Migrazione {
        versione: 2,
        nome: "versioning_completo",
        sql: include_str!("../migrations/V002__versioning_completo.sql"),
    },
    Migrazione {
        versione: 3,
        nome: "indici_audit",
        sql: include_str!("../migrations/V003__indici_audit.sql"),
    },
    Migrazione {
        versione: 4,
        nome: "cartelle",
        sql: include_str!("../migrations/V004__cartelle.sql"),
    },
];

/// Crea la tabella di tracking se non esiste.
fn crea_tabella_migrazioni(conn: &Connection) -> Result<(), PapErrore> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _Migrazioni (
            Versione    INTEGER PRIMARY KEY,
            Nome        TEXT NOT NULL,
            ApplicataA  TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;
    Ok(())
}

/// Restituisce la versione più alta già applicata, o 0 se nessuna.
fn versione_corrente(conn: &Connection) -> Result<u32, PapErrore> {
    let versione: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(Versione), 0) FROM _Migrazioni",
            [],
            |row| row.get(0),
        )?;
    Ok(versione)
}

/// Esegue tutte le migrazioni pendenti in ordine.
/// Restituisce il numero di migrazioni applicate.
pub fn esegui_migrazioni(conn: &Connection) -> Result<u32, PapErrore> {
    crea_tabella_migrazioni(conn)?;
    let corrente = versione_corrente(conn)?;
    let mut applicate = 0u32;

    for m in MIGRAZIONI {
        if m.versione <= corrente {
            continue;
        }

        log::info!(
            "Migrazione V{:03} ({}) in corso…",
            m.versione,
            m.nome
        );

        conn.execute_batch(m.sql).map_err(|e| {
            PapErrore::Migrazione(format!(
                "V{:03} ({}): {e}",
                m.versione, m.nome
            ))
        })?;

        conn.execute(
            "INSERT INTO _Migrazioni (Versione, Nome) VALUES (?1, ?2)",
            rusqlite::params![m.versione, m.nome],
        )?;

        log::info!("Migrazione V{:03} applicata.", m.versione);
        applicate += 1;
    }

    Ok(applicate)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn migrazioni_su_db_nuovo() {
        let conn = Connection::open_in_memory().unwrap();
        let n = esegui_migrazioni(&conn).unwrap();
        assert!(n >= 4, "Tutte le migrazioni devono essere applicate (almeno 4)");
        assert_eq!(versione_corrente(&conn).unwrap(), 4);
    }

    #[test]
    fn migrazioni_idempotenti() {
        let conn = Connection::open_in_memory().unwrap();
        esegui_migrazioni(&conn).unwrap();
        let n = esegui_migrazioni(&conn).unwrap();
        assert_eq!(n, 0, "Nessuna migrazione deve essere ri-applicata");
    }

    #[test]
    fn tabelle_create_correttamente() {
        let conn = Connection::open_in_memory().unwrap();
        esegui_migrazioni(&conn).unwrap();

        let tabelle: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        let attese = [
            "AuditLog",
            "Folders",
            "PromptTags",
            "PromptVersions",
            "Prompts",
            "SyncMeta",
            "Tags",
            "Users",
            "Workspaces",
            "_Migrazioni",
        ];

        for t in &attese {
            assert!(
                tabelle.contains(&t.to_string()),
                "Tabella {t} mancante. Trovate: {tabelle:?}"
            );
        }
    }
}
