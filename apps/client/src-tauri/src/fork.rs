// Modulo fork — Fase 4 Step 5.
//
// Fork = clone indipendente di un prompt, con `ForkOfPromptId` che
// punta all'originale per tracciabilità. A differenza delle varianti
// (V011), il fork non ha un legame "intento condiviso" — è una copia
// privata dove l'utente sperimenta senza toccare l'originale.
//
// MVP scope (questa PR):
// - prompt_fork(prompt_id): duplica con suffisso "(fork)" e visibility
//   forzata a 'private'. Copia tag, snapshot v1, hook FTS/embedding/imports.
// - fork_info(prompt_id): se il prompt è a sua volta un fork, ritorna
//   info sull'originale (titolo + flag "eliminato"). Usato dalla UI
//   per mostrare il badge "Fork di...".
//
// Out of scope (sub-step futuri):
// - "Pull request leggera" dal fork verso l'originale (Step 6 approval)
// - Contatore "N fork attivi" lato originale per workspace team

use rusqlite::{params, Connection};
use serde::Serialize;
use tauri::State;

use crate::embeddings::EmbeddingsState;
use crate::errore::PapErrore;
use crate::vault::VaultState;

const USER_LOCALE: &str = "usr-locale";
const WORKSPACE_PERSONALE: &str = "ws-personale";

#[derive(Debug, Clone, Serialize)]
pub struct ForkOfInfo {
    pub original_id: String,
    pub original_titolo: Option<String>,
    pub original_eliminato: bool,
}

fn genera_id() -> String {
    use rand::RngCore;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut rnd = [0u8; 4];
    rand::rngs::OsRng.fill_bytes(&mut rnd);
    format!(
        "prm-{:012x}{:02x}{:02x}{:02x}{:02x}",
        ts, rnd[0], rnd[1], rnd[2], rnd[3]
    )
}

#[allow(clippy::type_complexity)]
fn carica_originale(
    conn: &Connection,
    src_id: &str,
) -> Result<
    (
        String,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
    ),
    PapErrore,
> {
    conn.query_row(
        "SELECT Title, Description, Body, TargetModel, FolderId
         FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
        [src_id],
        |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
            ))
        },
    )
    .map_err(|_| PapErrore::Generico(format!("Prompt '{src_id}' non trovato")))
}

fn copia_tags(conn: &Connection, src_id: &str, dst_id: &str) -> Result<(), PapErrore> {
    conn.execute(
        "INSERT OR IGNORE INTO PromptTags (PromptId, TagId)
         SELECT ?1, TagId FROM PromptTags WHERE PromptId = ?2",
        params![dst_id, src_id],
    )?;
    Ok(())
}

/// Logica pura di `prompt_fork`, testabile senza Tauri State.
pub(crate) fn fork_pure(conn: &Connection, prompt_id: &str) -> Result<String, PapErrore> {
    let (titolo, descrizione, body, target_model, folder_id) =
        carica_originale(conn, prompt_id)?;

    let id = genera_id();
    let titolo_fork = format!("{titolo} (fork)");

    // Visibility forzata a 'private': il fork è sempre per
    // sperimentazione locale. FolderId mantenuto se possibile (l'utente
    // può comunque spostarlo dopo).
    conn.execute(
        "INSERT INTO Prompts
            (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
             Visibility, TargetModel, FolderId, ForkOfPromptId, Version,
             CreatedAt, UpdatedAt)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'private', ?7, ?8, ?9, 1,
                 datetime('now'), datetime('now'))",
        params![
            id,
            WORKSPACE_PERSONALE,
            USER_LOCALE,
            titolo_fork,
            descrizione,
            body,
            target_model,
            folder_id,
            prompt_id,
        ],
    )?;

    copia_tags(conn, prompt_id, &id)?;
    crate::versioning::snapshot_versione(conn, &id, USER_LOCALE)?;
    crate::audit::registra(
        conn,
        "fork.creato",
        "Prompt",
        &id,
        Some(&format!("fork di {prompt_id}")),
    );
    log::info!("Fork creato: {id} (originale {prompt_id})");
    Ok(id)
}

/// Se `prompt_id` è un fork, ritorna info sull'originale. `None`
/// altrimenti. `original_eliminato = true` se l'originale è
/// soft-deleted (la tracciabilità sopravvive alla cancellazione).
pub(crate) fn info_pure(
    conn: &Connection,
    prompt_id: &str,
) -> Result<Option<ForkOfInfo>, PapErrore> {
    let original_id: Option<String> = conn
        .query_row(
            "SELECT ForkOfPromptId FROM Prompts WHERE Id = ?1",
            [prompt_id],
            |r| r.get(0),
        )
        .unwrap_or(None);

    let Some(original_id) = original_id else {
        return Ok(None);
    };

    let row: Option<(String, Option<String>)> = conn
        .query_row(
            "SELECT Title, DeletedAt FROM Prompts WHERE Id = ?1",
            [&original_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
        )
        .ok();

    let (titolo, eliminato) = match row {
        Some((t, deleted)) => (Some(t), deleted.is_some()),
        // Originale rimosso fisicamente (caso edge): tracciabilità
        // residua via solo l'id.
        None => (None, true),
    };

    Ok(Some(ForkOfInfo {
        original_id,
        original_titolo: titolo,
        original_eliminato: eliminato,
    }))
}

#[tauri::command]
pub fn prompt_fork(
    prompt_id: String,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<String, PapErrore> {
    let id = state.with_conn(|conn| fork_pure(conn, &prompt_id))?;

    // Hook FTS/embedding/imports fuori dalla transazione principale per
    // coerenza coi pattern di editor::prompt_crea.
    state.with_conn(|conn| {
        let body: String = conn.query_row(
            "SELECT Body FROM Prompts WHERE Id = ?1",
            [&id],
            |r| r.get(0),
        )?;
        crate::editor::ricostruisci_fts(conn)?;
        crate::editor::aggiorna_embedding(conn, &rt_state, &id, &body)?;
        crate::prompt_componibili::aggiorna_imports(conn, &id, &body)?;
        Ok(())
    })?;

    Ok(id)
}

#[tauri::command]
pub fn fork_info(
    prompt_id: String,
    state: State<'_, VaultState>,
) -> Result<Option<ForkOfInfo>, PapErrore> {
    state.with_conn(|conn| info_pure(conn, &prompt_id))
}

#[cfg(test)]
mod test {
    use super::*;

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        // Inserisci un prompt originale + 2 tag. Lo schema FK su
        // Workspaces forza l'uso di 'ws-personale' (l'unico creato
        // da assicura_dati_base in DB di test).
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
                Visibility, TargetModel, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-orig', 'ws-personale', 'usr-locale', 'Originale', 'desc',
                     'body originale', 'workspace', 'claude-opus', 1,
                     datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('t-1', 'ws-personale', 'tag1', datetime('now'), datetime('now')),
                    ('t-2', 'ws-personale', 'tag2', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO PromptTags (PromptId, TagId) VALUES ('prm-orig', 't-1'),
                    ('prm-orig', 't-2')",
            [],
        )
        .unwrap();
        crate::versioning::snapshot_versione(&conn, "prm-orig", "usr-locale").unwrap();
        conn
    }

    #[test]
    fn id_fork_inizia_con_prm() {
        assert!(genera_id().starts_with("prm-"));
    }

    #[test]
    fn fork_pure_imposta_fork_of_prompt_id() {
        let conn = db_test();
        let id = fork_pure(&conn, "prm-orig").unwrap();
        let fork_of: Option<String> = conn
            .query_row(
                "SELECT ForkOfPromptId FROM Prompts WHERE Id = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fork_of, Some("prm-orig".to_string()));
    }

    #[test]
    fn fork_pure_titolo_decorato_con_suffix() {
        let conn = db_test();
        let id = fork_pure(&conn, "prm-orig").unwrap();
        let titolo: String = conn
            .query_row("SELECT Title FROM Prompts WHERE Id = ?1", [&id], |r| r.get(0))
            .unwrap();
        assert_eq!(titolo, "Originale (fork)");
    }

    #[test]
    fn fork_pure_visibility_forzata_a_private() {
        let conn = db_test();
        let id = fork_pure(&conn, "prm-orig").unwrap();
        let vis: String = conn
            .query_row(
                "SELECT Visibility FROM Prompts WHERE Id = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(vis, "private", "Fork sempre privato a prescindere dall'originale");
    }

    #[test]
    fn fork_pure_workspace_personale_e_user_locale() {
        let conn = db_test();
        let id = fork_pure(&conn, "prm-orig").unwrap();
        let (ws, author): (String, String) = conn
            .query_row(
                "SELECT WorkspaceId, AuthorUserId FROM Prompts WHERE Id = ?1",
                [&id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(ws, "ws-personale");
        assert_eq!(author, "usr-locale");
    }

    #[test]
    fn fork_pure_eredita_body_e_target() {
        let conn = db_test();
        let id = fork_pure(&conn, "prm-orig").unwrap();
        let (body, target): (String, Option<String>) = conn
            .query_row(
                "SELECT Body, TargetModel FROM Prompts WHERE Id = ?1",
                [&id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(body, "body originale");
        assert_eq!(target, Some("claude-opus".to_string()));
    }

    #[test]
    fn fork_pure_eredita_tags() {
        let conn = db_test();
        let id = fork_pure(&conn, "prm-orig").unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptTags WHERE PromptId = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn fork_pure_snapshot_v1_in_promptversions() {
        let conn = db_test();
        let id = fork_pure(&conn, "prm-orig").unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptVersions WHERE PromptId = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(n >= 1);
    }

    #[test]
    fn fork_pure_originale_inesistente_e_errore() {
        let conn = db_test();
        let r = fork_pure(&conn, "prm-fantasma");
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("non trovato"));
    }

    #[test]
    fn fork_pure_audit_log_registra_evento() {
        let conn = db_test();
        let id = fork_pure(&conn, "prm-orig").unwrap();
        let azione: String = conn
            .query_row(
                "SELECT Action FROM AuditLog WHERE EntityType = 'Prompt' AND EntityId = ?1
                 ORDER BY OccurredAt DESC LIMIT 1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(azione, "fork.creato");
    }

    #[test]
    fn fork_di_fork_ricollega_al_secondo_originale() {
        // Edge case: fork di un fork. ForkOfPromptId del nipote punta
        // al fork (genitore diretto), non all'originale. Decisione di
        // policy semplice: una sola hop, no chain transitive.
        let conn = db_test();
        let id_a = fork_pure(&conn, "prm-orig").unwrap();
        let id_b = fork_pure(&conn, &id_a).unwrap();
        let fork_of: Option<String> = conn
            .query_row(
                "SELECT ForkOfPromptId FROM Prompts WHERE Id = ?1",
                [&id_b],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fork_of, Some(id_a));
    }

    // ─────────── Test info_pure ───────────

    #[test]
    fn info_pure_prompt_non_fork_e_none() {
        let conn = db_test();
        let r = info_pure(&conn, "prm-orig").unwrap();
        assert!(r.is_none());
    }

    #[test]
    fn info_pure_prompt_fork_ritorna_info() {
        let conn = db_test();
        let id = fork_pure(&conn, "prm-orig").unwrap();
        let info = info_pure(&conn, &id).unwrap().unwrap();
        assert_eq!(info.original_id, "prm-orig");
        assert_eq!(info.original_titolo.as_deref(), Some("Originale"));
        assert!(!info.original_eliminato);
    }

    #[test]
    fn info_pure_originale_eliminato_flag_true() {
        let conn = db_test();
        let id = fork_pure(&conn, "prm-orig").unwrap();
        // Soft-delete dell'originale.
        conn.execute(
            "UPDATE Prompts SET DeletedAt = datetime('now') WHERE Id = 'prm-orig'",
            [],
        )
        .unwrap();
        let info = info_pure(&conn, &id).unwrap().unwrap();
        assert_eq!(info.original_id, "prm-orig");
        assert_eq!(info.original_titolo.as_deref(), Some("Originale"));
        assert!(info.original_eliminato);
    }

    #[test]
    fn info_pure_originale_completamente_rimosso_titolo_none() {
        // Edge case: ForkOfPromptId punta a un id che non esiste in
        // Prompts (record corrotto, sync con dati incompleti). FK
        // disabilitate per il setup — la fn `info_pure` deve essere
        // resiliente a stato incoerente.
        let conn = db_test();
        let id = fork_pure(&conn, "prm-orig").unwrap();
        conn.execute("PRAGMA foreign_keys = OFF", []).unwrap();
        conn.execute(
            "UPDATE Prompts SET ForkOfPromptId = 'prm-never-existed' WHERE Id = ?1",
            [&id],
        )
        .unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        let info = info_pure(&conn, &id).unwrap().unwrap();
        assert_eq!(info.original_id, "prm-never-existed");
        assert!(info.original_titolo.is_none());
        assert!(info.original_eliminato);
    }

    #[test]
    fn info_pure_prompt_inesistente_e_none() {
        let conn = db_test();
        let r = info_pure(&conn, "prm-fantasma").unwrap();
        assert!(r.is_none());
    }
}
