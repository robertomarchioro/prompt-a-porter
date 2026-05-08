use rand::RngCore;
use rusqlite::Connection;
use serde::Serialize;
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

/// Limite di versioni storiche mantenute per ciascun prompt.
/// Oltre questa soglia, le versioni più vecchie vengono cancellate (rolling delete).
const LIMITE_VERSIONI_PER_PROMPT: i64 = 100;

#[derive(Debug, Serialize, Clone)]
pub struct VersioneStorica {
    pub id: String,
    pub prompt_id: String,
    pub version: i64,
    pub titolo: String,
    pub descrizione: Option<String>,
    pub body: String,
    pub visibilita: Option<String>,
    pub target_model: Option<String>,
    pub creato_a: String,
    pub creato_da_user_id: String,
    /// V014: nome dell'autore della versione (JOIN su Users.DisplayName).
    pub autore_display_name: String,
    /// V014: email dell'autore (NULL se non impostata). Usato lato UI per
    /// generare avatar deterministici (hash SHA1+HSL).
    pub autore_email: Option<String>,
}

fn genera_version_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut rnd = [0u8; 4];
    rand::rngs::OsRng.fill_bytes(&mut rnd);
    format!(
        "pv-{:012x}{:02x}{:02x}{:02x}{:02x}",
        ts, rnd[0], rnd[1], rnd[2], rnd[3]
    )
}

/// Inserisce uno snapshot completo della versione corrente del prompt in PromptVersions.
/// Usato sia da `prompt_crea` (snapshot v1 iniziale) sia da `prompt_aggiorna` (snapshot
/// pre-update, prima che `Version` venga incrementata).
///
/// Il caller è responsabile del coordinamento con UPDATE/INSERT su Prompts.
pub(crate) fn snapshot_versione(
    conn: &Connection,
    prompt_id: &str,
    user_id: &str,
) -> Result<(), PapErrore> {
    // Legge lo stato corrente del prompt.
    let (version, title, description, body, visibility, target_model): (
        i64,
        String,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
    ) = conn.query_row(
        "SELECT Version, Title, Description, Body, Visibility, TargetModel
         FROM Prompts
         WHERE Id = ?1 AND DeletedAt IS NULL",
        [prompt_id],
        |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        },
    )?;

    // Insert OR IGNORE per idempotenza: se la (PromptId, Version) esiste già
    // (es. backfill v1 della migration V002), non duplica.
    let id = genera_version_id();
    conn.execute(
        "INSERT OR IGNORE INTO PromptVersions
            (Id, PromptId, Version, Title, Description, Body, Visibility, TargetModel,
             CreatedAt, CreatedByUserId)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), ?9)",
        rusqlite::params![
            id,
            prompt_id,
            version,
            title,
            description,
            body,
            visibility,
            target_model,
            user_id
        ],
    )?;

    // Rolling delete: mantiene solo le ultime LIMITE_VERSIONI_PER_PROMPT versioni.
    pulisci_versioni_eccedenti(conn, prompt_id)?;

    Ok(())
}

/// Cancella le versioni più vecchie di un prompt se superano la soglia configurata.
fn pulisci_versioni_eccedenti(conn: &Connection, prompt_id: &str) -> Result<(), PapErrore> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM PromptVersions WHERE PromptId = ?1",
        [prompt_id],
        |r| r.get(0),
    )?;

    if count <= LIMITE_VERSIONI_PER_PROMPT {
        return Ok(());
    }

    let da_cancellare = count - LIMITE_VERSIONI_PER_PROMPT;
    conn.execute(
        "DELETE FROM PromptVersions
         WHERE Id IN (
             SELECT Id FROM PromptVersions
             WHERE PromptId = ?1
             ORDER BY Version ASC
             LIMIT ?2
         )",
        rusqlite::params![prompt_id, da_cancellare],
    )?;

    Ok(())
}

/// Helper interno per `prompt_get_history`. Estratto come `pub(crate)` per
/// testabilità diretta su `&Connection` senza passare dallo State Tauri.
pub(crate) fn prompt_get_history_inner(
    conn: &Connection,
    prompt_id: &str,
) -> Result<Vec<VersioneStorica>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT pv.Id, pv.PromptId, pv.Version, pv.Title, pv.Description, pv.Body,
                pv.Visibility, pv.TargetModel, pv.CreatedAt, pv.CreatedByUserId,
                u.DisplayName, u.Email
         FROM PromptVersions pv
         JOIN Users u ON u.Id = pv.CreatedByUserId
         WHERE pv.PromptId = ?1
         ORDER BY pv.Version DESC",
    )?;
    let versioni: Vec<VersioneStorica> = stmt
        .query_map([prompt_id], |row| {
            Ok(VersioneStorica {
                id: row.get(0)?,
                prompt_id: row.get(1)?,
                version: row.get(2)?,
                titolo: row.get(3)?,
                descrizione: row.get(4)?,
                body: row.get(5)?,
                visibilita: row.get(6)?,
                target_model: row.get(7)?,
                creato_a: row.get(8)?,
                creato_da_user_id: row.get(9)?,
                autore_display_name: row.get(10)?,
                autore_email: row.get(11)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(versioni)
}

/// Tauri command: ritorna la storia completa delle versioni di un prompt,
/// ordinate dalla più recente alla più vecchia.
#[tauri::command]
pub fn prompt_get_history(
    prompt_id: String,
    state: State<'_, VaultState>,
) -> Result<Vec<VersioneStorica>, PapErrore> {
    state.with_conn(|conn| prompt_get_history_inner(conn, &prompt_id))
}

/// Tauri command: applica una versione storica come nuova testa del prompt.
/// Rollback "soft": copia i campi della versione target su Prompts (Version+1),
/// crea snapshot della nuova testa in PromptVersions. La storia precedente
/// resta consultabile.
#[tauri::command]
pub fn prompt_rollback(
    prompt_id: String,
    target_version: i64,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        // Legge la versione target.
        let (title, description, body, visibility, target_model): (
            String,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
        ) = conn.query_row(
            "SELECT Title, Description, Body, Visibility, TargetModel
             FROM PromptVersions
             WHERE PromptId = ?1 AND Version = ?2",
            rusqlite::params![&prompt_id, target_version],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                ))
            },
        )?;

        // Snapshot della testa corrente prima dell'overwrite.
        snapshot_versione(conn, &prompt_id, "usr-locale")?;

        // Applica i campi della versione target come nuova testa.
        // Visibility ha CHECK constraint sui valori 'private'/'workspace',
        // default a 'private' se la versione storica aveva NULL.
        let visibility_eff = visibility.as_deref().unwrap_or("private");

        conn.execute(
            "UPDATE Prompts
             SET Title = ?1, Description = ?2, Body = ?3, Visibility = ?4,
                 TargetModel = ?5,
                 Version = Version + 1,
                 UpdatedAt = datetime('now'),
                 UpdatedByUserId = 'usr-locale'
             WHERE Id = ?6 AND DeletedAt IS NULL",
            rusqlite::params![
                title,
                description,
                body,
                visibility_eff,
                target_model,
                prompt_id
            ],
        )?;

        // Snapshot della nuova testa (rollback completato).
        snapshot_versione(conn, &prompt_id, "usr-locale")?;

        crate::audit::registra(
            conn,
            "prompt.rollback",
            "Prompt",
            &prompt_id,
            Some(&format!("rollback alla v{target_version}")),
        );

        log::info!("Prompt {prompt_id} rollback alla versione {target_version}");
        Ok(())
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    fn crea_prompt_test(conn: &Connection, id: &str, body: &str) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
             Visibility, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', 'titolo', 'descrizione', ?2,
             'private', 1, datetime('now'), datetime('now'))",
            rusqlite::params![id, body],
        )
        .unwrap();
    }

    #[test]
    fn snapshot_inserisce_versione() {
        let conn = db_test();
        crea_prompt_test(&conn, "prm-v1", "body iniziale");

        snapshot_versione(&conn, "prm-v1", "usr-locale").unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptVersions WHERE PromptId = 'prm-v1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn snapshot_idempotente_per_stessa_version() {
        let conn = db_test();
        crea_prompt_test(&conn, "prm-v2", "body");

        snapshot_versione(&conn, "prm-v2", "usr-locale").unwrap();
        snapshot_versione(&conn, "prm-v2", "usr-locale").unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptVersions WHERE PromptId = 'prm-v2'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Stesso (PromptId, Version) non deve duplicare");
    }

    #[test]
    fn history_ritorna_versioni_ordinate() {
        let conn = db_test();
        crea_prompt_test(&conn, "prm-v3", "v1");

        // Simula 3 versioni successive.
        for v in 1..=3 {
            conn.execute(
                "UPDATE Prompts SET Version = ?1, Body = ?2 WHERE Id = 'prm-v3'",
                rusqlite::params![v, format!("v{}", v)],
            )
            .unwrap();
            snapshot_versione(&conn, "prm-v3", "usr-locale").unwrap();
        }

        let mut stmt = conn
            .prepare(
                "SELECT Version FROM PromptVersions
                 WHERE PromptId = 'prm-v3' ORDER BY Version DESC",
            )
            .unwrap();
        let versions: Vec<i64> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(versions, vec![3, 2, 1]);
    }

    #[test]
    fn rollback_applica_versione_target_come_nuova_testa() {
        let conn = db_test();
        crea_prompt_test(&conn, "prm-r1", "body originale v1");

        // Snapshot v1
        snapshot_versione(&conn, "prm-r1", "usr-locale").unwrap();

        // Simula update -> v2
        conn.execute(
            "UPDATE Prompts SET Version = 2, Body = 'body modificato v2' WHERE Id = 'prm-r1'",
            [],
        )
        .unwrap();
        snapshot_versione(&conn, "prm-r1", "usr-locale").unwrap();

        // Logica rollback inline (emula prompt_rollback senza Tauri State).
        let (title, body): (String, String) = conn
            .query_row(
                "SELECT Title, Body FROM PromptVersions
                 WHERE PromptId = 'prm-r1' AND Version = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();

        snapshot_versione(&conn, "prm-r1", "usr-locale").unwrap();
        conn.execute(
            "UPDATE Prompts SET Title = ?1, Body = ?2, Version = Version + 1
             WHERE Id = 'prm-r1'",
            rusqlite::params![title, body],
        )
        .unwrap();
        snapshot_versione(&conn, "prm-r1", "usr-locale").unwrap();

        // Verifica: la testa ora ha il body della v1.
        let body_corrente: String = conn
            .query_row("SELECT Body FROM Prompts WHERE Id = 'prm-r1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(body_corrente, "body originale v1");

        // Verifica: la storia mantiene v1, v2, v3.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptVersions WHERE PromptId = 'prm-r1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 3, "v1 + v2 + v3 (post-rollback)");
    }

    #[test]
    fn rolling_delete_oltre_soglia() {
        let conn = db_test();
        crea_prompt_test(&conn, "prm-rd", "body");

        // Inserisce 105 versioni manualmente.
        for v in 1..=105 {
            let id = format!("pv-rd-{:03}", v);
            conn.execute(
                "INSERT INTO PromptVersions
                    (Id, PromptId, Version, Title, Body, CreatedAt, CreatedByUserId)
                 VALUES (?1, 'prm-rd', ?2, 'titolo', 'body', datetime('now'), 'usr-locale')",
                rusqlite::params![id, v],
            )
            .unwrap();
        }

        // Trigger rolling delete tramite snapshot (incrementa Version a 106 e poi pulisce).
        conn.execute("UPDATE Prompts SET Version = 106 WHERE Id = 'prm-rd'", [])
            .unwrap();
        snapshot_versione(&conn, "prm-rd", "usr-locale").unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptVersions WHERE PromptId = 'prm-rd'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            count, LIMITE_VERSIONI_PER_PROMPT,
            "Rolling delete deve mantenere solo {LIMITE_VERSIONI_PER_PROMPT} versioni"
        );

        // Versioni 1-6 cancellate, restano 7-106.
        let min_version: i64 = conn
            .query_row(
                "SELECT MIN(Version) FROM PromptVersions WHERE PromptId = 'prm-rd'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(min_version, 7);
    }

    #[test]
    fn prompt_get_history_include_autore() {
        // V014: il cmd deve restituire DisplayName + Email dell'autore via
        // JOIN Users.
        let conn = db_test();
        // Aggiorna l'utente di default con un'email per testare il path Some(_).
        conn.execute(
            "UPDATE Users SET Email = 'roberto@example.com' WHERE Id = 'usr-locale'",
            [],
        )
        .unwrap();
        crea_prompt_test(&conn, "prm-aut", "body");
        snapshot_versione(&conn, "prm-aut", "usr-locale").unwrap();

        let versioni = prompt_get_history_inner(&conn, "prm-aut").unwrap();
        assert_eq!(versioni.len(), 1);
        assert_eq!(versioni[0].autore_display_name, "Utente locale");
        assert_eq!(
            versioni[0].autore_email,
            Some("roberto@example.com".to_string())
        );
    }

    #[test]
    fn prompt_get_history_email_none_se_non_settata() {
        // Default user post `assicura_dati_base` ha Email NULL.
        let conn = db_test();
        crea_prompt_test(&conn, "prm-noemail", "body");
        snapshot_versione(&conn, "prm-noemail", "usr-locale").unwrap();

        let versioni = prompt_get_history_inner(&conn, "prm-noemail").unwrap();
        assert_eq!(versioni.len(), 1);
        assert_eq!(versioni[0].autore_email, None);
    }
}
