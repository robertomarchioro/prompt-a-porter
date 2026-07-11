use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

/// Fix #455: unico workspace supportato oggi (coerente con `ws-personale`
/// usato in tutto il resto del backend, vedi `libreria::assicura_dati_base`).
/// `SyncMeta` è per-workspace (PK `WorkspaceId`) in previsione di un
/// eventuale multi-workspace futuro, ma finché esiste un solo workspace
/// personale il token vive tutto in questa riga.
const WORKSPACE_ID_DEFAULT: &str = "ws-personale";

/// Fix #455 (security review MEDIUM): legge il `sync_token` dal vault
/// cifrato (tabella `SyncMeta`, schema V001, mai usata prima d'ora) invece
/// che da `preferenze.json` in chiaro. `None` se non è mai stato
/// configurato nessun token, o se il valore salvato è una stringa vuota.
pub(crate) fn sync_token_carica_pure(conn: &Connection) -> Result<Option<String>, PapErrore> {
    let token: Option<Option<String>> = conn
        .query_row(
            "SELECT LastSyncToken FROM SyncMeta WHERE WorkspaceId = ?1",
            [WORKSPACE_ID_DEFAULT],
            |r| r.get::<_, Option<String>>(0),
        )
        .optional()?;
    Ok(token.flatten().filter(|t| !t.is_empty()))
}

/// Fix #455: salva/aggiorna il `sync_token` nel vault cifrato. Upsert sulla
/// riga `SyncMeta` del workspace personale — non esiste ancora nessun
/// comando che la crei, quindi il primo salvataggio la inserisce.
/// `token` vuoto è un caso d'uso legittimo: rappresenta un logout esplicito
/// (ripulisce il valore salvato senza eliminare la riga).
pub(crate) fn sync_token_salva_pure(conn: &Connection, token: &str) -> Result<(), PapErrore> {
    conn.execute(
        "INSERT INTO SyncMeta (WorkspaceId, LastSyncToken) VALUES (?1, ?2)
         ON CONFLICT(WorkspaceId) DO UPDATE SET LastSyncToken = excluded.LastSyncToken",
        rusqlite::params![WORKSPACE_ID_DEFAULT, token],
    )?;
    Ok(())
}

/// Fix #455 (review HIGH-2): comando Tauri DEDICATO al `sync_token`,
/// disaccoppiato dal round-trip generico `preferenze_carica`/`preferenze_salva`
/// — un salvataggio di preferenze non correlate (tema, editor, debug-log,
/// ...) non deve più richiedere il vault aperto solo perché portava con sé
/// anche il token. Include la migrazione one-shot del token legacy da un
/// `preferenze.json` pre-fix (comportamento invariato, solo rilocato qui).
/// Richiede il vault aperto: `PapErrore::VaultChiuso` altrimenti (comando
/// pensato per funzionare solo a vault sbloccato, come da spec).
#[tauri::command]
pub fn sync_token_carica(
    state: State<'_, crate::preferenze::PreferenzeState>,
    vault: State<'_, VaultState>,
) -> Result<Option<String>, PapErrore> {
    sync_token_carica_impl(&state, &vault)
}

/// Logica testabile di `sync_token_carica` (pattern `_impl` usato altrove,
/// es. `vault::vault_cifrato_impl`).
pub(crate) fn sync_token_carica_impl(
    pref_state: &crate::preferenze::PreferenzeState,
    vault: &VaultState,
) -> Result<Option<String>, PapErrore> {
    let path = pref_state.file_path();
    if let Some(token_legacy) = crate::preferenze::estrai_token_legacy(&path) {
        // Migrazione one-shot: propaga VaultChiuso pulito se il vault non è
        // aperto, SENZA toccare il file — il token resta nel file legacy
        // (unico posto dove esiste) e si ritenta al prossimo load a vault
        // aperto, invece di rischiare di perderlo.
        vault.with_conn(|conn| sync_token_salva_pure(conn, &token_legacy))?;
        crate::preferenze::rimuovi_token_legacy(pref_state.data_dir())?;
        return Ok(Some(token_legacy));
    }
    vault.with_conn(sync_token_carica_pure)
}

/// Fix #455 (review HIGH-2): comando Tauri dedicato al salvataggio del
/// `sync_token` — richiede il vault aperto (nessun caso "best-effort" qui:
/// a differenza del vecchio `preferenze_salva`, questo comando esiste SOLO
/// per il token, quindi l'errore pulito `VaultChiuso` è sempre corretto).
#[tauri::command]
pub fn sync_token_salva(token: String, state: State<'_, VaultState>) -> Result<(), PapErrore> {
    state.with_conn(|conn| sync_token_salva_pure(conn, &token))
}

#[derive(Debug, Deserialize)]
pub struct SyncDelta {
    pub prompts: Vec<SyncPrompt>,
    pub tags: Vec<SyncTag>,
    #[serde(rename = "promptTags")]
    pub prompt_tags: Vec<SyncPromptTag>,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPrompt {
    pub id: String,
    pub workspace_id: String,
    pub author_user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    pub visibility: String,
    pub target_model: Option<String>,
    pub is_favorite: i32,
    pub use_count: i32,
    pub last_used_at: Option<String>,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
    pub updated_by_user_id: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncTag {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPromptTag {
    pub prompt_id: String,
    pub tag_id: String,
}

#[tauri::command]
pub fn sync_applica_delta(delta: SyncDelta, state: State<'_, VaultState>) -> Result<u32, PapErrore> {
    state.with_conn(|conn| {
        let mut applicati: u32 = 0;

        for tag in &delta.tags {
            // EXISTS invece di COUNT(*); l'errore DB viene propagato (`?`)
            // invece di essere mascherato in `false` da `unwrap_or`.
            let esiste: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM Tags WHERE Id = ?1)",
                [&tag.id],
                |r| r.get::<_, i32>(0),
            )? > 0;

            if esiste {
                conn.execute(
                    "UPDATE Tags SET Name = ?1, Color = ?2, UpdatedAt = ?3, DeletedAt = ?4
                     WHERE Id = ?5 AND UpdatedAt < ?3",
                    rusqlite::params![tag.name, tag.color, tag.updated_at, tag.deleted_at, tag.id],
                )?;
            } else {
                conn.execute(
                    "INSERT OR IGNORE INTO Tags (Id, WorkspaceId, Name, Color, CreatedAt, UpdatedAt, DeletedAt)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        tag.id,
                        tag.workspace_id,
                        tag.name,
                        tag.color,
                        tag.created_at,
                        tag.updated_at,
                        tag.deleted_at
                    ],
                )?;
            }
            applicati += 1;
        }

        for p in &delta.prompts {
            // Trust boundary: il delta arriva dal server di sync. Validiamo
            // `visibility` contro il set ammesso PRIMA di scrivere: un record
            // malformato viene saltato (con log) invece di far abortire
            // l'intero delta sul CHECK del DB. (workspace_id/author_user_id
            // non sono ancora validati: multi-workspace non implementato.)
            if !matches!(p.visibility.as_str(), "private" | "workspace") {
                log::warn!(
                    "sync: prompt {} scartato, visibility non valida: {:?}",
                    p.id,
                    p.visibility
                );
                continue;
            }

            // EXISTS invece di COUNT(*); errore DB propagato invece di
            // mascherato in `false`.
            let esiste: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM Prompts WHERE Id = ?1)",
                [&p.id],
                |r| r.get::<_, i32>(0),
            )? > 0;

            if esiste {
                conn.execute(
                    "UPDATE Prompts SET Title = ?1, Description = ?2, Body = ?3,
                     Visibility = ?4, TargetModel = ?5, Version = ?6,
                     UpdatedAt = ?7, UpdatedByUserId = ?8, DeletedAt = ?9
                     WHERE Id = ?10 AND UpdatedAt < ?7",
                    rusqlite::params![
                        p.title,
                        p.description,
                        p.body,
                        p.visibility,
                        p.target_model,
                        p.version,
                        p.updated_at,
                        p.updated_by_user_id,
                        p.deleted_at,
                        p.id
                    ],
                )?;
            } else {
                conn.execute(
                    "INSERT OR IGNORE INTO Prompts
                     (Id, WorkspaceId, AuthorUserId, Title, Description, Body, Visibility,
                      TargetModel, IsFavorite, UseCount, LastUsedAt, Version,
                      CreatedAt, UpdatedAt, UpdatedByUserId, DeletedAt)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                    rusqlite::params![
                        p.id,
                        p.workspace_id,
                        p.author_user_id,
                        p.title,
                        p.description,
                        p.body,
                        p.visibility,
                        p.target_model,
                        p.is_favorite,
                        p.use_count,
                        p.last_used_at,
                        p.version,
                        p.created_at,
                        p.updated_at,
                        p.updated_by_user_id,
                        p.deleted_at
                    ],
                )?;
            }
            applicati += 1;
        }

        for pt in &delta.prompt_tags {
            conn.execute(
                "INSERT OR IGNORE INTO PromptTags (PromptId, TagId) VALUES (?1, ?2)",
                rusqlite::params![pt.prompt_id, pt.tag_id],
            )?;
        }

        crate::editor::ricostruisci_fts(conn)?;
        crate::audit::registra(conn, "sync.delta_applicato", "Sync", "", Some(&format!("{applicati} entità")));

        Ok(applicati)
    })
}

#[cfg(test)]
mod test {
    fn db_test() -> rusqlite::Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    #[test]
    fn inserisci_tag_nuovo() {
        let conn = db_test();
        conn.execute(
            "INSERT OR IGNORE INTO Tags (Id, WorkspaceId, Name, Color, CreatedAt, UpdatedAt)
             VALUES ('tag-s1', 'ws-personale', 'synced', '#ff0000', '2024-01-01', '2024-01-01')",
            [],
        )
        .unwrap();

        let name: String = conn
            .query_row(
                "SELECT Name FROM Tags WHERE Id = 'tag-s1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(name, "synced");
    }

    #[test]
    fn inserisci_prompt_nuovo() {
        let conn = db_test();
        conn.execute(
            "INSERT OR IGNORE INTO Prompts
             (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-s1','ws-personale','usr-locale','Synced','body','workspace',1,
             '2024-01-01','2024-01-01')",
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM Prompts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn aggiorna_solo_se_piu_recente() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('tag-1', 'ws-personale', 'originale', '2024-01-01', '2024-01-01 12:00:00')",
            [],
        )
        .unwrap();

        let changed = conn
            .execute(
                "UPDATE Tags SET Name = 'vecchio' WHERE Id = 'tag-1' AND UpdatedAt < '2024-01-01 10:00:00'",
                [],
            )
            .unwrap();
        assert_eq!(changed, 0);

        let name: String = conn
            .query_row("SELECT Name FROM Tags WHERE Id = 'tag-1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "originale");

        let changed = conn
            .execute(
                "UPDATE Tags SET Name = 'nuovo' WHERE Id = 'tag-1' AND UpdatedAt < '2024-01-02 00:00:00'",
                [],
            )
            .unwrap();
        assert_eq!(changed, 1);

        let name: String = conn
            .query_row("SELECT Name FROM Tags WHERE Id = 'tag-1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "nuovo");
    }

    #[test]
    fn prompt_tag_insert_or_ignore() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt)
             VALUES ('prm-1','ws-personale','usr-locale','T','b','private',1,
             datetime('now'),datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('tag-1','ws-personale','t',datetime('now'),datetime('now'))",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT OR IGNORE INTO PromptTags (PromptId, TagId) VALUES ('prm-1', 'tag-1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO PromptTags (PromptId, TagId) VALUES ('prm-1', 'tag-1')",
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM PromptTags", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "INSERT OR IGNORE non deve duplicare");
    }

    // ─────────── Fix #455: sync_token nel vault (SyncMeta) ───────────

    #[test]
    fn sync_token_carica_pure_nessun_token_torna_none() {
        let conn = db_test();
        assert_eq!(super::sync_token_carica_pure(&conn).unwrap(), None);
    }

    #[test]
    fn sync_token_salva_e_carica_pure_roundtrip() {
        let conn = db_test();
        super::sync_token_salva_pure(&conn, "tok-abc123").unwrap();
        assert_eq!(
            super::sync_token_carica_pure(&conn).unwrap(),
            Some("tok-abc123".to_string())
        );
    }

    #[test]
    fn sync_token_salva_pure_upsert_sovrascrive() {
        let conn = db_test();
        super::sync_token_salva_pure(&conn, "vecchio").unwrap();
        super::sync_token_salva_pure(&conn, "nuovo").unwrap();
        assert_eq!(
            super::sync_token_carica_pure(&conn).unwrap(),
            Some("nuovo".to_string())
        );
        let righe: i64 = conn
            .query_row("SELECT COUNT(*) FROM SyncMeta", [], |r| r.get(0))
            .unwrap();
        assert_eq!(righe, 1, "upsert non deve duplicare la riga");
    }

    #[test]
    fn sync_token_salva_pure_stringa_vuota_pulisce() {
        let conn = db_test();
        super::sync_token_salva_pure(&conn, "tok-da-cancellare").unwrap();
        super::sync_token_salva_pure(&conn, "").unwrap();
        assert_eq!(super::sync_token_carica_pure(&conn).unwrap(), None);
    }

    #[test]
    fn sync_token_salva_pure_scrive_sul_workspace_personale() {
        let conn = db_test();
        super::sync_token_salva_pure(&conn, "tok-riservato").unwrap();
        let workspace_id: String = conn
            .query_row(
                "SELECT WorkspaceId FROM SyncMeta WHERE LastSyncToken = 'tok-riservato'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(workspace_id, "ws-personale");
    }

    // ─────── Fix #455 (HIGH-2): sync_token_carica_impl (comando dedicato) ───────

    use crate::errore::PapErrore;
    use crate::preferenze::PreferenzeState;
    use crate::vault::VaultState;

    /// Vault temporaneo NON cifrato ma già aperto (niente password richiesta
    /// nei test): basta ad esercitare `with_conn`/`SyncMeta`, che non
    /// dipendono dalla cifratura.
    fn vault_temp_aperto() -> (tempfile::TempDir, VaultState) {
        crate::embeddings_store::registra_auto_extension();
        let dir = tempfile::tempdir().unwrap();
        let state = VaultState::new(dir.path().to_path_buf());
        crate::vault::vault_crea_aperto_impl(&state).unwrap();
        (dir, state)
    }

    #[test]
    fn sync_token_carica_impl_nessun_file_nessun_token_vault_aperto() {
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let (_vdir, vault) = vault_temp_aperto();

        assert_eq!(super::sync_token_carica_impl(&pref_state, &vault).unwrap(), None);
    }

    #[test]
    fn sync_token_carica_impl_vault_chiuso_senza_legacy_e_errore_pulito() {
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let vault = VaultState::new(dir.path().join("vault-non-aperto"));

        let r = super::sync_token_carica_impl(&pref_state, &vault);
        assert!(matches!(r, Err(PapErrore::VaultChiuso)));
    }

    #[test]
    fn sync_token_carica_impl_ritorna_il_valore_gia_salvato_nel_vault() {
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let (_vdir, vault) = vault_temp_aperto();

        vault
            .with_conn(|conn| super::sync_token_salva_pure(conn, "tok-dal-vault"))
            .unwrap();

        assert_eq!(
            super::sync_token_carica_impl(&pref_state, &vault).unwrap(),
            Some("tok-dal-vault".to_string())
        );
    }

    #[test]
    fn sync_token_carica_impl_migra_token_legacy_da_preferenze_json() {
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let (_vdir, vault) = vault_temp_aperto();

        // Simula un preferenze.json scritto da una build precedente al
        // fix #455, con il token ancora in chiaro.
        std::fs::write(
            pref_state.file_path(),
            r#"{"sync_token":"tok-legacy-in-chiaro"}"#,
        )
        .unwrap();

        let token = super::sync_token_carica_impl(&pref_state, &vault).unwrap();
        assert_eq!(token, Some("tok-legacy-in-chiaro".to_string()));

        // Il file deve essere ripulito...
        let json = std::fs::read_to_string(pref_state.file_path()).unwrap();
        assert!(!json.contains("tok-legacy-in-chiaro"));

        // ...e il token vive ora nel vault.
        let dal_vault = vault.with_conn(super::sync_token_carica_pure).unwrap();
        assert_eq!(dal_vault, Some("tok-legacy-in-chiaro".to_string()));
    }

    #[test]
    fn sync_token_carica_impl_migrazione_con_vault_chiuso_e_errore_pulito_file_intatto() {
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let vault = VaultState::new(dir.path().join("vault-non-aperto"));

        std::fs::write(
            pref_state.file_path(),
            r#"{"sync_token":"tok-legacy-in-chiaro"}"#,
        )
        .unwrap();

        let r = super::sync_token_carica_impl(&pref_state, &vault);
        assert!(matches!(r, Err(PapErrore::VaultChiuso)));

        // Il file NON deve essere toccato: il token resta l'unica copia
        // esistente, si ritenta la migrazione al prossimo load a vault aperto.
        let json = std::fs::read_to_string(pref_state.file_path()).unwrap();
        assert!(json.contains("tok-legacy-in-chiaro"));
    }

    #[test]
    fn sync_token_salva_impl_via_comando_richiede_vault_aperto() {
        let dir = tempfile::tempdir().unwrap();
        let vault = VaultState::new(dir.path().join("vault-non-aperto"));
        let r = vault.with_conn(|conn| super::sync_token_salva_pure(conn, "x"));
        assert!(matches!(r, Err(PapErrore::VaultChiuso)));
    }
}
