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
