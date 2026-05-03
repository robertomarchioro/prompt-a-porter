use rusqlite::Connection;
use serde::Serialize;
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

#[derive(Debug, Serialize)]
pub struct VoceAudit {
    pub id: String,
    pub azione: String,
    pub tipo_entita: String,
    pub id_entita: String,
    pub metadati: String,
    pub avvenuto_a: String,
}

pub fn registra(
    conn: &Connection,
    azione: &str,
    tipo_entita: &str,
    id_entita: &str,
    metadati: Option<&str>,
) {
    let id = formato_id_audit();
    let _ = conn.execute(
        "INSERT INTO AuditLog (Id, WorkspaceId, UserId, Action, EntityType, EntityId, Metadata, OccurredAt)
         VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, ?4, ?5, datetime('now'))",
        rusqlite::params![id, azione, tipo_entita, id_entita, metadati],
    );
}

fn formato_id_audit() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut rnd = [0u8; 4];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut rnd);
    format!(
        "aud-{:012x}{:02x}{:02x}{:02x}{:02x}",
        ts, rnd[0], rnd[1], rnd[2], rnd[3]
    )
}

#[tauri::command]
pub fn audit_lista(
    limite: Option<u32>,
    tipo_entita: Option<String>,
    state: State<'_, VaultState>,
) -> Result<Vec<VoceAudit>, PapErrore> {
    state.with_conn(|conn| {
        let lim = limite.unwrap_or(100).min(500);

        let sql = if tipo_entita.is_some() {
            "SELECT Id, Action, EntityType, COALESCE(EntityId, ''), COALESCE(Metadata, ''), OccurredAt
             FROM AuditLog
             WHERE EntityType = :tipo
             ORDER BY OccurredAt DESC
             LIMIT :limite"
        } else {
            "SELECT Id, Action, EntityType, COALESCE(EntityId, ''), COALESCE(Metadata, ''), OccurredAt
             FROM AuditLog
             ORDER BY OccurredAt DESC
             LIMIT :limite"
        };

        let mut stmt = conn.prepare(sql)?;
        let voci = stmt
            .query_map(
                rusqlite::named_params! {
                    ":tipo": tipo_entita,
                    ":limite": lim,
                },
                |row| {
                    Ok(VoceAudit {
                        id: row.get(0)?,
                        azione: row.get(1)?,
                        tipo_entita: row.get(2)?,
                        id_entita: row.get(3)?,
                        metadati: row.get(4)?,
                        avvenuto_a: row.get(5)?,
                    })
                },
            )?
            .filter_map(|r| r.ok())
            .collect();
        Ok(voci)
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
    fn formato_id_prefisso_e_lunghezza() {
        let id = formato_id_audit();
        assert!(id.starts_with("aud-"));
        assert_eq!(id.len(), 24);
    }

    #[test]
    fn formato_id_univoco() {
        let id1 = formato_id_audit();
        let id2 = formato_id_audit();
        assert_ne!(id1, id2);
    }

    #[test]
    fn registra_scrive_in_db() {
        let conn = db_test();
        registra(&conn, "test.azione", "TestEntity", "ent-123", Some("meta"));

        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM AuditLog", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn registra_senza_metadati() {
        let conn = db_test();
        registra(&conn, "test.azione", "TestEntity", "ent-123", None);

        let meta: Option<String> = conn
            .query_row("SELECT Metadata FROM AuditLog LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert!(meta.is_none());
    }

    #[test]
    fn registra_multiple_e_filtra_per_tipo() {
        let conn = db_test();
        registra(&conn, "a1", "Prompt", "p1", Some("m1"));
        registra(&conn, "a2", "Vault", "v1", None);
        registra(&conn, "a3", "Prompt", "p2", Some("m3"));

        let totale: i64 =
            conn.query_row("SELECT COUNT(*) FROM AuditLog", [], |r| r.get(0)).unwrap();
        assert_eq!(totale, 3);

        let prompt_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM AuditLog WHERE EntityType = 'Prompt'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(prompt_count, 2);
    }

    #[test]
    fn registra_ordine_cronologico() {
        let conn = db_test();
        registra(&conn, "primo", "T", "", None);
        registra(&conn, "secondo", "T", "", None);

        let azione: String = conn
            .query_row(
                "SELECT Action FROM AuditLog ORDER BY OccurredAt DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(azione, "secondo");
    }
}
