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
    Migrazione {
        versione: 5,
        nome: "embeddings",
        sql: include_str!("../migrations/V005__embeddings.sql"),
    },
    Migrazione {
        versione: 6,
        nome: "tag_embeddings",
        sql: include_str!("../migrations/V006__tag_embeddings.sql"),
    },
    Migrazione {
        versione: 7,
        nome: "prompt_imports",
        sql: include_str!("../migrations/V007__prompt_imports.sql"),
    },
    Migrazione {
        versione: 8,
        nome: "prompt_goldens",
        sql: include_str!("../migrations/V008__prompt_goldens.sql"),
    },
    Migrazione {
        versione: 9,
        nome: "prompt_run_observations",
        sql: include_str!("../migrations/V009__prompt_run_observations.sql"),
    },
    Migrazione {
        versione: 10,
        nome: "provider_config",
        sql: include_str!("../migrations/V010__provider_config.sql"),
    },
    Migrazione {
        versione: 11,
        nome: "prompt_varianti",
        sql: include_str!("../migrations/V011__prompt_varianti.sql"),
    },
    Migrazione {
        versione: 12,
        nome: "prompt_fork",
        sql: include_str!("../migrations/V012__prompt_fork.sql"),
    },
    Migrazione {
        versione: 13,
        nome: "prompt_ratings",
        sql: include_str!("../migrations/V013__prompt_ratings.sql"),
    },
    Migrazione {
        versione: 14,
        nome: "sort_order",
        sql: include_str!("../migrations/V014__sort_order.sql"),
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
        // Necessario registrare l'auto-extension sqlite-vec PRIMA di aprire
        // la connessione, perché V005 crea una vec0 virtual table.
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        let n = esegui_migrazioni(&conn).unwrap();
        assert!(
            n >= 14,
            "Tutte le migrazioni devono essere applicate (almeno 14)"
        );
        assert_eq!(versione_corrente(&conn).unwrap(), 14);
    }

    #[test]
    fn migrazioni_idempotenti() {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        esegui_migrazioni(&conn).unwrap();
        let n = esegui_migrazioni(&conn).unwrap();
        assert_eq!(n, 0, "Nessuna migrazione deve essere ri-applicata");
    }

    #[test]
    fn tabelle_create_correttamente() {
        crate::embeddings_store::registra_auto_extension();
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

    #[test]
    fn v014_aggiunge_colonne_sortorder() {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        esegui_migrazioni(&conn).unwrap();

        let folders_cols: Vec<String> = conn
            .prepare("PRAGMA table_info(Folders)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(
            folders_cols.contains(&"SortOrder".to_string()),
            "Folders.SortOrder mancante. Colonne: {folders_cols:?}"
        );

        let prompts_cols: Vec<String> = conn
            .prepare("PRAGMA table_info(Prompts)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(
            prompts_cols.contains(&"SortOrder".to_string()),
            "Prompts.SortOrder mancante. Colonne: {prompts_cols:?}"
        );
    }

    #[test]
    fn v014_backfill_sortorder_stabile_per_creato_at() {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        esegui_migrazioni(&conn).unwrap();

        // Setup minimale: workspace + 3 folder con CreatedAt distinti
        conn.execute(
            "INSERT INTO Workspaces (Id, Name, Type, CreatedAt, UpdatedAt) VALUES ('ws1','test','personal','2026-01-01','2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path, CreatedAt, UpdatedAt) VALUES ('f1','ws1',NULL,'A','/A','2026-01-01','2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path, CreatedAt, UpdatedAt) VALUES ('f2','ws1',NULL,'B','/B','2026-01-02','2026-01-02')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path, CreatedAt, UpdatedAt) VALUES ('f3','ws1',NULL,'C','/C','2026-01-03','2026-01-03')",
            [],
        )
        .unwrap();

        // Re-applica il backfill V014 inline (idempotente: ricalcola SortOrder
        // sulla base del CreatedAt corrente, indipendente da run precedenti).
        conn.execute(
            "UPDATE Folders SET SortOrder = (
                SELECT COUNT(*) FROM Folders f2
                WHERE f2.WorkspaceId = Folders.WorkspaceId
                  AND COALESCE(f2.ParentFolderId, '') = COALESCE(Folders.ParentFolderId, '')
                  AND f2.CreatedAt < Folders.CreatedAt
            )",
            [],
        )
        .unwrap();

        let f1: i64 = conn
            .query_row("SELECT SortOrder FROM Folders WHERE Id='f1'", [], |r| r.get(0))
            .unwrap();
        let f2: i64 = conn
            .query_row("SELECT SortOrder FROM Folders WHERE Id='f2'", [], |r| r.get(0))
            .unwrap();
        let f3: i64 = conn
            .query_row("SELECT SortOrder FROM Folders WHERE Id='f3'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(f1, 0, "f1 (creata per prima) deve avere SortOrder=0");
        assert_eq!(f2, 1, "f2 (seconda) deve avere SortOrder=1");
        assert_eq!(f3, 2, "f3 (terza) deve avere SortOrder=2");
    }
}
