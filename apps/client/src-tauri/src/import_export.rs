use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

/// Versione corrente del formato di export. Vedi docs/utente/formato-export-json.md.
const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportV1 {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "exportedAt")]
    pub exported_at: String,
    pub workspace: WorkspaceMeta,
    pub prompts: Vec<PromptExport>,
    pub versions: Vec<VersioneExport>,
    pub tags: Vec<TagExport>,
    #[serde(default)]
    pub folders: Vec<FolderExport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceMeta {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub workspace_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptExport {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    pub visibility: String,
    pub target_model: Option<String>,
    pub folder_id: Option<String>,
    pub is_favorite: bool,
    pub use_count: i64,
    pub last_used_at: Option<String>,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tag_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersioneExport {
    pub id: String,
    pub prompt_id: String,
    pub version: i64,
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    pub visibility: Option<String>,
    pub target_model: Option<String>,
    pub created_at: String,
    pub created_by_user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagExport {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderExport {
    pub id: String,
    pub parent_folder_id: Option<String>,
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ImportReport {
    pub nuovi: u32,
    pub aggiornati: u32,
    pub conflitti: u32,
    pub errori: Vec<String>,
}

fn ora_iso() -> String {
    // Formato ISO 8601 UTC senza chrono.
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let secs_in_day = 86400u64;
    let h = (t / 3600) % 24;
    let m = (t / 60) % 60;
    let s = t % 60;
    let days_since_epoch = (t / secs_in_day) as i64;
    let (y, mo, d) = days_to_ymd(days_since_epoch);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn days_to_ymd(mut days: i64) -> (i64, u32, u32) {
    // Howard Hinnant date algorithm. days = giorni dal 1970-01-01.
    days += 719468;
    let era = if days >= 0 { days / 146097 } else { (days - 146096) / 146097 };
    let doe = (days - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn valida_modalita(s: &str) -> Result<(), PapErrore> {
    match s {
        "skip" | "overwrite" | "rename" => Ok(()),
        altro => Err(PapErrore::Migrazione(format!(
            "Modalità import non valida: '{altro}' (attesi: skip|overwrite|rename)"
        ))),
    }
}

/// Helper "pure" (no Tauri State) per costruire l'`ExportV1` da una
/// connection. Esposto pub(crate) per testabilità diretta senza dover
/// montare uno State Tauri. v0.7.0 Step 1 refactor.
///
/// Wrapper che chiama `export_pure_filter(conn, None)` → esporta tutto.
pub(crate) fn export_pure(conn: &Connection) -> Result<ExportV1, PapErrore> {
    export_pure_filter(conn, None)
}

/// Variante con filtro per `folder_id`: se `Some(id)`, esporta solo i
/// prompt del sotto-albero della cartella (matching su `Folders.Path`
/// con LIKE prefix). Se `None`, esporta tutti i prompt attivi.
/// v0.7.0 Step 2.
///
/// Nota: le `folders` nel payload restano `Vec::new()` per coerenza
/// con il flusso di import attuale che non le gestisce. Il roundtrip
/// folders è scope v0.8.
pub(crate) fn export_pure_filter(
    conn: &Connection,
    folder_id: Option<&str>,
) -> Result<ExportV1, PapErrore> {
    let workspace: WorkspaceMeta = conn.query_row(
        "SELECT Id, Name, Type FROM Workspaces LIMIT 1",
        [],
        |r| {
            Ok(WorkspaceMeta {
                id: r.get(0)?,
                name: r.get(1)?,
                workspace_type: r.get(2)?,
            })
        },
    )?;

    // Costruzione query Prompts. Se folder_id specificato, risolvi il
    // Path della folder e filtra i prompt nel sotto-albero (path esatto
    // OR path LIKE "<folder_path>/%").
    type PromptRaw = (String, String, Option<String>, String, String,
                     Option<String>, Option<String>, i64, i64,
                     Option<String>, i64, String, String);

    fn collect_row(r: &rusqlite::Row) -> rusqlite::Result<PromptRaw> {
        Ok((
            r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
            r.get(5)?, r.get(6)?, r.get::<_, i64>(7)?, r.get(8)?,
            r.get(9)?, r.get(10)?, r.get(11)?, r.get(12)?,
        ))
    }

    let prompts_raw: Vec<PromptRaw> = if let Some(fid) = folder_id {
        let folder_path: String = conn.query_row(
            "SELECT Path FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
            [fid],
            |r| r.get(0),
        ).map_err(|e| PapErrore::Generico(format!("Cartella '{fid}' non trovata: {e}")))?;
        let prefix_like = format!("{folder_path}/%");

        let mut stmt = conn.prepare(
            "SELECT p.Id, p.Title, p.Description, p.Body, p.Visibility, p.TargetModel,
                    p.FolderId, p.IsFavorite, p.UseCount, p.LastUsedAt, p.Version,
                    p.CreatedAt, p.UpdatedAt
             FROM Prompts p
             JOIN Folders f ON f.Id = p.FolderId
             WHERE p.DeletedAt IS NULL
               AND f.DeletedAt IS NULL
               AND (f.Path = ?1 OR f.Path LIKE ?2)
             ORDER BY p.CreatedAt ASC",
        )?;
        let mut rows = stmt.query(rusqlite::params![folder_path, prefix_like])?;
        let mut acc: Vec<PromptRaw> = Vec::new();
        while let Some(row) = rows.next()? {
            acc.push(collect_row(row)?);
        }
        acc
    } else {
        let mut stmt = conn.prepare(
            "SELECT Id, Title, Description, Body, Visibility, TargetModel,
                    FolderId, IsFavorite, UseCount, LastUsedAt, Version,
                    CreatedAt, UpdatedAt
             FROM Prompts
             WHERE DeletedAt IS NULL
             ORDER BY CreatedAt ASC",
        )?;
        let mut rows = stmt.query([])?;
        let mut acc: Vec<PromptRaw> = Vec::new();
        while let Some(row) = rows.next()? {
            acc.push(collect_row(row)?);
        }
        acc
    };

    let mut prompts: Vec<PromptExport> = Vec::with_capacity(prompts_raw.len());
    for p in prompts_raw {
        let mut stmt_tags = conn.prepare(
            "SELECT TagId FROM PromptTags WHERE PromptId = ?1 ORDER BY TagId ASC",
        )?;
        let tag_ids: Vec<String> = stmt_tags
            .query_map([&p.0], |r| r.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        prompts.push(PromptExport {
            id: p.0,
            title: p.1,
            description: p.2,
            body: p.3,
            visibility: p.4,
            target_model: p.5,
            folder_id: p.6,
            is_favorite: p.7 != 0,
            use_count: p.8,
            last_used_at: p.9,
            version: p.10,
            created_at: p.11,
            updated_at: p.12,
            tag_ids,
        });
    }

    let mut stmt_v = conn.prepare(
        "SELECT Id, PromptId, Version, Title, Description, Body, Visibility,
                TargetModel, CreatedAt, CreatedByUserId
         FROM PromptVersions
         ORDER BY PromptId ASC, Version ASC",
    )?;
    let versions: Vec<VersioneExport> = stmt_v
        .query_map([], |r| {
            Ok(VersioneExport {
                id: r.get(0)?,
                prompt_id: r.get(1)?,
                version: r.get(2)?,
                title: r.get(3)?,
                description: r.get(4)?,
                body: r.get(5)?,
                visibility: r.get(6)?,
                target_model: r.get(7)?,
                created_at: r.get(8)?,
                created_by_user_id: r.get(9)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut stmt_t = conn.prepare(
        "SELECT Id, Name, Color, CreatedAt FROM Tags
         WHERE DeletedAt IS NULL
         ORDER BY CreatedAt ASC",
    )?;
    let tags: Vec<TagExport> = stmt_t
        .query_map([], |r| {
            Ok(TagExport {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                created_at: r.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(ExportV1 {
        schema_version: SCHEMA_VERSION,
        exported_at: ora_iso(),
        workspace,
        prompts,
        versions,
        tags,
        folders: Vec::new(),
    })
}

/// Tauri command: esporta l'intero workspace come JSON serializzato.
#[tauri::command]
pub fn vault_export_json(state: State<'_, VaultState>) -> Result<String, PapErrore> {
    state.with_conn(|conn| {
        let export = export_pure(conn)?;
        let json = serde_json::to_string_pretty(&export)?;

        crate::audit::registra(
            conn,
            "vault.exported",
            "Vault",
            "",
            Some(&format!(
                "{} prompts, {} versioni, {} tag",
                export.prompts.len(),
                export.versions.len(),
                export.tags.len()
            )),
        );

        Ok(json)
    })
}

/// Tauri command: esporta solo i prompt del sotto-albero della cartella
/// indicata (incluse sotto-cartelle). v0.7.0 Step 2.
#[tauri::command]
pub fn vault_export_folder_json(
    folder_id: String,
    state: State<'_, VaultState>,
) -> Result<String, PapErrore> {
    state.with_conn(|conn| {
        let export = export_pure_filter(conn, Some(&folder_id))?;
        let json = serde_json::to_string_pretty(&export)?;

        crate::audit::registra(
            conn,
            "vault.exported.folder",
            "Folder",
            &folder_id,
            Some(&format!(
                "{} prompts (sotto-albero), {} tag",
                export.prompts.len(),
                export.tags.len()
            )),
        );

        Ok(json)
    })
}

/// Helper "pure" (no Tauri State) per applicare un `ExportV1` parsato
/// sulla connection con la modalità di gestione conflitti specificata.
/// Riusabile dai test e dal Tauri command `vault_import_json`.
/// v0.7.0 Step 1 refactor.
///
/// `modalita` valida: `"skip"`, `"overwrite"`, `"rename"`. Validare
/// prima di chiamare (vedi `valida_modalita`).
pub(crate) fn import_pure(
    conn: &Connection,
    export: &ExportV1,
    modalita: &str,
) -> Result<ImportReport, PapErrore> {
    let mut report = ImportReport {
        nuovi: 0,
        aggiornati: 0,
        conflitti: 0,
        errori: Vec::new(),
    };

    // Tags prima.
    for tag in &export.tags {
        let esiste: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM Tags WHERE Id = ?1)",
                [&tag.id],
                |r| r.get(0),
            )
            .unwrap_or(false);

        match (esiste, modalita) {
            (false, _) => {
                if let Err(e) = conn.execute(
                    "INSERT INTO Tags (Id, WorkspaceId, Name, Color, CreatedAt, UpdatedAt)
                     VALUES (?1, 'ws-personale', ?2, ?3, ?4, ?4)",
                    rusqlite::params![tag.id, tag.name, tag.color, tag.created_at],
                ) {
                    report.errori.push(format!("Tag {}: {}", tag.id, e));
                } else {
                    report.nuovi += 1;
                }
            }
            (true, "skip") | (true, "rename") => {
                report.conflitti += 1;
            }
            (true, "overwrite") => {
                if let Err(e) = conn.execute(
                    "UPDATE Tags SET Name = ?1, Color = ?2, UpdatedAt = datetime('now')
                     WHERE Id = ?3",
                    rusqlite::params![tag.name, tag.color, tag.id],
                ) {
                    report.errori.push(format!("Tag {}: {}", tag.id, e));
                } else {
                    report.aggiornati += 1;
                }
            }
            _ => unreachable!("modalita validata in ingresso"),
        }
    }

    // Prompts.
    for prompt in &export.prompts {
        let esiste: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM Prompts WHERE Id = ?1)",
                [&prompt.id],
                |r| r.get(0),
            )
            .unwrap_or(false);

        let id_effettivo = match (esiste, modalita) {
            (false, _) => prompt.id.clone(),
            (true, "skip") => {
                report.conflitti += 1;
                continue;
            }
            (true, "overwrite") => prompt.id.clone(),
            (true, "rename") => format!("{}-imp", prompt.id),
            _ => unreachable!(),
        };

        let title = if id_effettivo == prompt.id {
            prompt.title.clone()
        } else {
            format!("{} (importato)", prompt.title)
        };

        let esito = if esiste && modalita == "overwrite" {
            conn.execute(
                "UPDATE Prompts SET Title = ?1, Description = ?2, Body = ?3,
                        Visibility = ?4, TargetModel = ?5, IsFavorite = ?6,
                        UseCount = ?7, LastUsedAt = ?8, Version = ?9,
                        UpdatedAt = datetime('now')
                 WHERE Id = ?10",
                rusqlite::params![
                    title,
                    prompt.description,
                    prompt.body,
                    prompt.visibility,
                    prompt.target_model,
                    prompt.is_favorite as i64,
                    prompt.use_count,
                    prompt.last_used_at,
                    prompt.version,
                    prompt.id
                ],
            )
            .map(|_| "agg")
        } else {
            conn.execute(
                "INSERT INTO Prompts
                    (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
                     Visibility, TargetModel, IsFavorite, UseCount, LastUsedAt,
                     Version, CreatedAt, UpdatedAt)
                 VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, ?4, ?5, ?6,
                         ?7, ?8, ?9, ?10, ?11, ?12)",
                rusqlite::params![
                    id_effettivo,
                    title,
                    prompt.description,
                    prompt.body,
                    prompt.visibility,
                    prompt.target_model,
                    prompt.is_favorite as i64,
                    prompt.use_count,
                    prompt.last_used_at,
                    prompt.version,
                    prompt.created_at,
                    prompt.updated_at
                ],
            )
            .map(|_| "new")
        };

        match esito {
            Ok("new") => report.nuovi += 1,
            Ok("agg") => report.aggiornati += 1,
            Err(e) => {
                report.errori.push(format!("Prompt {}: {}", prompt.id, e));
                continue;
            }
            _ => continue,
        }

        // Riassocia tag.
        let _ = conn.execute(
            "DELETE FROM PromptTags WHERE PromptId = ?1",
            [&id_effettivo],
        );
        for tag_id in &prompt.tag_ids {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO PromptTags (PromptId, TagId) VALUES (?1, ?2)",
                rusqlite::params![id_effettivo, tag_id],
            );
        }
    }

    // Versioni storiche (best-effort, INSERT OR IGNORE per UNIQUE constraint).
    for ver in &export.versions {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO PromptVersions
                (Id, PromptId, Version, Title, Description, Body, Visibility,
                 TargetModel, CreatedAt, CreatedByUserId)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                ver.id,
                ver.prompt_id,
                ver.version,
                ver.title,
                ver.description,
                ver.body,
                ver.visibility,
                ver.target_model,
                ver.created_at,
                ver.created_by_user_id,
            ],
        );
    }

    crate::editor::ricostruisci_fts(conn)?;

    Ok(report)
}

/// Tauri command: importa un export JSON applicando la modalità di
/// gestione conflitti. Wrapper di `import_pure` con parsing JSON,
/// validazione modalità + schemaVersion, audit log.
#[tauri::command]
pub fn vault_import_json(
    json: String,
    modalita: String,
    state: State<'_, VaultState>,
) -> Result<ImportReport, PapErrore> {
    valida_modalita(&modalita)?;

    let export: ExportV1 = serde_json::from_str(&json)
        .map_err(|e| PapErrore::Migrazione(format!("JSON non valido: {e}")))?;

    if export.schema_version > SCHEMA_VERSION {
        return Err(PapErrore::Migrazione(format!(
            "schemaVersion {} non supportata (max supportato: {})",
            export.schema_version, SCHEMA_VERSION
        )));
    }

    state.with_conn(|conn| {
        let report = import_pure(conn, &export, &modalita)?;

        crate::audit::registra(
            conn,
            "vault.imported",
            "Vault",
            "",
            Some(&format!(
                "modalita={} nuovi={} aggiornati={} conflitti={} errori={}",
                modalita,
                report.nuovi,
                report.aggiornati,
                report.conflitti,
                report.errori.len()
            )),
        );

        Ok(report)
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ora_iso_formato() {
        let s = ora_iso();
        assert!(s.ends_with('Z'));
        assert_eq!(s.len(), 20);
        assert_eq!(s.chars().nth(4), Some('-'));
        assert!(s.contains('T'));
    }

    #[test]
    fn modalita_valida() {
        assert!(valida_modalita("skip").is_ok());
        assert!(valida_modalita("overwrite").is_ok());
        assert!(valida_modalita("rename").is_ok());
        assert!(valida_modalita("delete").is_err());
        assert!(valida_modalita("").is_err());
    }

    #[test]
    fn schema_version_corrente_e_uno() {
        assert_eq!(SCHEMA_VERSION, 1);
    }

    #[test]
    fn export_struct_serializza_camelcase_topkeys() {
        let e = ExportV1 {
            schema_version: 1,
            exported_at: "2026-05-03T18:00:00Z".to_string(),
            workspace: WorkspaceMeta {
                id: "ws".into(),
                name: "Test".into(),
                workspace_type: "personal".into(),
            },
            prompts: vec![],
            versions: vec![],
            tags: vec![],
            folders: vec![],
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"schemaVersion\":1"));
        assert!(json.contains("\"exportedAt\":\"2026-05-03T18:00:00Z\""));
        assert!(json.contains("\"type\":\"personal\""));
    }

    #[test]
    fn round_trip_serialize_deserialize() {
        let originale = ExportV1 {
            schema_version: 1,
            exported_at: "2026-05-03T18:00:00Z".to_string(),
            workspace: WorkspaceMeta {
                id: "ws".into(),
                name: "Test".into(),
                workspace_type: "personal".into(),
            },
            prompts: vec![PromptExport {
                id: "prm-1".into(),
                title: "Titolo".into(),
                description: Some("desc".into()),
                body: "body con {{var}}".into(),
                visibility: "private".into(),
                target_model: None,
                folder_id: None,
                is_favorite: true,
                use_count: 5,
                last_used_at: None,
                version: 2,
                created_at: "2026-01-01T00:00:00Z".into(),
                updated_at: "2026-01-02T00:00:00Z".into(),
                tag_ids: vec!["tag-a".into(), "tag-b".into()],
            }],
            versions: vec![],
            tags: vec![],
            folders: vec![],
        };
        let json = serde_json::to_string(&originale).unwrap();
        let parsed: ExportV1 = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.prompts.len(), 1);
        assert_eq!(parsed.prompts[0].id, "prm-1");
        assert_eq!(parsed.prompts[0].tag_ids, vec!["tag-a", "tag-b"]);
        assert!(parsed.prompts[0].is_favorite);
    }

    #[test]
    fn schema_version_futura_rilevabile() {
        let json = r#"{"schemaVersion":99,"exportedAt":"2026-01-01T00:00:00Z","workspace":{"id":"x","name":"x","type":"personal"},"prompts":[],"versions":[],"tags":[]}"#;
        let parsed: ExportV1 = serde_json::from_str(json).unwrap();
        assert!(parsed.schema_version > SCHEMA_VERSION);
    }

    // ─────────── v0.7.0 Step 1: export_pure / import_pure ───────────

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    fn inserisci_prompt(conn: &Connection, id: &str, titolo: &str, body: &str) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility,
             Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, 'private', 1,
             datetime('now'), datetime('now'))",
            rusqlite::params![id, titolo, body],
        )
        .unwrap();
    }

    #[test]
    fn export_pure_db_solo_dati_base_zero_prompt() {
        let conn = db_test();
        let exp = export_pure(&conn).unwrap();
        assert_eq!(exp.schema_version, SCHEMA_VERSION);
        assert_eq!(exp.workspace.id, "ws-personale");
        assert!(exp.prompts.is_empty());
        assert!(exp.tags.is_empty());
        assert!(exp.versions.is_empty());
    }

    #[test]
    fn export_pure_include_prompt_attivi() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-a", "Alfa", "body alfa");
        inserisci_prompt(&conn, "prm-b", "Bravo", "body bravo");
        let exp = export_pure(&conn).unwrap();
        assert_eq!(exp.prompts.len(), 2);
        let ids: Vec<&str> = exp.prompts.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"prm-a"));
        assert!(ids.contains(&"prm-b"));
    }

    #[test]
    fn export_pure_esclude_prompt_eliminati() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-attivo", "Attivo", "body");
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility,
             Version, CreatedAt, UpdatedAt, DeletedAt)
             VALUES ('prm-elim', 'ws-personale', 'usr-locale', 'Eliminato', 'b', 'private', 1,
             datetime('now'), datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        let exp = export_pure(&conn).unwrap();
        assert_eq!(exp.prompts.len(), 1);
        assert_eq!(exp.prompts[0].id, "prm-attivo");
    }

    #[test]
    fn export_pure_include_tag_associati_al_prompt() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Test", "body");
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('tag-x', 'ws-personale', 'x', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO PromptTags (PromptId, TagId) VALUES ('prm-1', 'tag-x')",
            [],
        )
        .unwrap();
        let exp = export_pure(&conn).unwrap();
        assert_eq!(exp.prompts[0].tag_ids, vec!["tag-x"]);
        assert_eq!(exp.tags.len(), 1);
        assert_eq!(exp.tags[0].id, "tag-x");
    }

    fn payload_minimo(prompt_id: &str, titolo: &str) -> ExportV1 {
        ExportV1 {
            schema_version: 1,
            exported_at: "2026-05-07T00:00:00Z".to_string(),
            workspace: WorkspaceMeta {
                id: "ws-personale".into(),
                name: "Personale".into(),
                workspace_type: "personal".into(),
            },
            prompts: vec![PromptExport {
                id: prompt_id.into(),
                title: titolo.into(),
                description: None,
                body: "body".into(),
                visibility: "private".into(),
                target_model: None,
                folder_id: None,
                is_favorite: false,
                use_count: 0,
                last_used_at: None,
                version: 1,
                created_at: "2026-05-07T00:00:00Z".into(),
                updated_at: "2026-05-07T00:00:00Z".into(),
                tag_ids: vec![],
            }],
            versions: vec![],
            tags: vec![],
            folders: vec![],
        }
    }

    #[test]
    fn import_pure_db_vuoto_inserisce_prompt_come_nuovo() {
        let conn = db_test();
        let payload = payload_minimo("prm-new", "Nuovo");
        let report = import_pure(&conn, &payload, "skip").unwrap();
        assert_eq!(report.nuovi, 1);
        assert_eq!(report.aggiornati, 0);
        assert_eq!(report.conflitti, 0);
        assert!(report.errori.is_empty());

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM Prompts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn import_pure_modalita_skip_su_id_esistente_incrementa_conflitti() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Originale", "body");
        let payload = payload_minimo("prm-1", "Da-import");
        let report = import_pure(&conn, &payload, "skip").unwrap();
        assert_eq!(report.nuovi, 0);
        assert_eq!(report.conflitti, 1);

        // Verifica che il titolo originale sia preservato.
        let title: String = conn
            .query_row("SELECT Title FROM Prompts WHERE Id = 'prm-1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(title, "Originale");
    }

    #[test]
    fn import_pure_modalita_overwrite_aggiorna_record() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Originale", "body-vecchio");
        let payload = payload_minimo("prm-1", "Aggiornato");
        let report = import_pure(&conn, &payload, "overwrite").unwrap();
        assert_eq!(report.nuovi, 0);
        assert_eq!(report.aggiornati, 1);
        assert_eq!(report.conflitti, 0);

        let title: String = conn
            .query_row("SELECT Title FROM Prompts WHERE Id = 'prm-1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(title, "Aggiornato");
    }

    #[test]
    fn import_pure_modalita_rename_crea_id_suffix_imp() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Originale", "body");
        let payload = payload_minimo("prm-1", "Da-rinominare");
        let report = import_pure(&conn, &payload, "rename").unwrap();
        // Tag conflict → conteggio nuovi sul prompt rinominato.
        assert_eq!(report.nuovi, 1);

        // Verifica che esista il record con id rinominato.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts WHERE Id = 'prm-1-imp'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Originale preservato.
        let title_orig: String = conn
            .query_row("SELECT Title FROM Prompts WHERE Id = 'prm-1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(title_orig, "Originale");
    }

    #[test]
    fn import_pure_round_trip_export_pure() {
        // Inserisci dati, esporta, importa su DB pulito, esporta di nuovo,
        // verifica equivalenza essenziale (id prompt e tag).
        let conn1 = db_test();
        inserisci_prompt(&conn1, "prm-1", "Test", "body");
        let exp1 = export_pure(&conn1).unwrap();

        let conn2 = db_test();
        let report = import_pure(&conn2, &exp1, "skip").unwrap();
        assert_eq!(report.nuovi, 1);

        let exp2 = export_pure(&conn2).unwrap();
        assert_eq!(exp2.prompts.len(), 1);
        assert_eq!(exp2.prompts[0].id, "prm-1");
        assert_eq!(exp2.prompts[0].title, "Test");
    }

    // ─────────── v0.7.0 Step 2: export_pure_filter (cartella) ───────────

    fn inserisci_folder(conn: &Connection, id: &str, name: &str, path: &str, parent: Option<&str>) {
        conn.execute(
            "INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', ?2, ?3, ?4, datetime('now'), datetime('now'))",
            rusqlite::params![id, parent, name, path],
        )
        .unwrap();
    }

    fn inserisci_prompt_in_folder(conn: &Connection, id: &str, titolo: &str, folder_id: Option<&str>) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility,
             Version, FolderId, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, 'body', 'private', 1,
             ?3, datetime('now'), datetime('now'))",
            rusqlite::params![id, titolo, folder_id],
        )
        .unwrap();
    }

    #[test]
    fn export_pure_filter_none_equivale_a_export_pure() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "A", "body");
        let a = export_pure(&conn).unwrap();
        let b = export_pure_filter(&conn, None).unwrap();
        assert_eq!(a.prompts.len(), b.prompts.len());
        assert_eq!(a.prompts[0].id, b.prompts[0].id);
    }

    #[test]
    fn export_pure_filter_folder_inesistente_ritorna_errore() {
        let conn = db_test();
        let r = export_pure_filter(&conn, Some("fld-fantasma"));
        assert!(r.is_err());
        let msg = format!("{:?}", r.unwrap_err());
        assert!(msg.contains("non trovata"));
    }

    #[test]
    fn export_pure_filter_solo_prompt_della_cartella() {
        let conn = db_test();
        inserisci_folder(&conn, "fld-mkt", "marketing", "/marketing", None);
        inserisci_folder(&conn, "fld-dev", "dev", "/dev", None);
        inserisci_prompt_in_folder(&conn, "prm-mkt", "Marketing", Some("fld-mkt"));
        inserisci_prompt_in_folder(&conn, "prm-dev", "Dev", Some("fld-dev"));
        inserisci_prompt_in_folder(&conn, "prm-root", "Root", None);

        let exp = export_pure_filter(&conn, Some("fld-mkt")).unwrap();
        assert_eq!(exp.prompts.len(), 1);
        assert_eq!(exp.prompts[0].id, "prm-mkt");
        assert_eq!(exp.prompts[0].folder_id.as_deref(), Some("fld-mkt"));
    }

    #[test]
    fn export_pure_filter_include_sotto_cartelle() {
        let conn = db_test();
        // /marketing
        // /marketing/email
        // /marketing/social
        inserisci_folder(&conn, "fld-mkt", "marketing", "/marketing", None);
        inserisci_folder(&conn, "fld-email", "email", "/marketing/email", Some("fld-mkt"));
        inserisci_folder(&conn, "fld-social", "social", "/marketing/social", Some("fld-mkt"));

        inserisci_prompt_in_folder(&conn, "prm-top", "Top", Some("fld-mkt"));
        inserisci_prompt_in_folder(&conn, "prm-email", "Email", Some("fld-email"));
        inserisci_prompt_in_folder(&conn, "prm-social", "Social", Some("fld-social"));

        let exp = export_pure_filter(&conn, Some("fld-mkt")).unwrap();
        assert_eq!(exp.prompts.len(), 3, "Atteso il prompt root + 2 sub-folder");
        let ids: Vec<&str> = exp.prompts.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"prm-top"));
        assert!(ids.contains(&"prm-email"));
        assert!(ids.contains(&"prm-social"));
    }

    #[test]
    fn export_pure_popola_folder_id_dal_db() {
        let conn = db_test();
        inserisci_folder(&conn, "fld-x", "x", "/x", None);
        inserisci_prompt_in_folder(&conn, "prm-1", "Test", Some("fld-x"));

        let exp = export_pure(&conn).unwrap();
        assert_eq!(exp.prompts.len(), 1);
        assert_eq!(exp.prompts[0].folder_id.as_deref(), Some("fld-x"));
    }

    #[test]
    fn import_pure_tag_skip_su_id_esistente() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('tag-x', 'ws-personale', 'OriginaleNome', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let mut payload = payload_minimo("prm-x", "x");
        payload.tags = vec![TagExport {
            id: "tag-x".into(),
            name: "NuovoNome".into(),
            color: None,
            created_at: "2026-05-07T00:00:00Z".into(),
        }];

        let report = import_pure(&conn, &payload, "skip").unwrap();
        assert_eq!(report.conflitti, 1);

        let nome: String = conn
            .query_row("SELECT Name FROM Tags WHERE Id = 'tag-x'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(nome, "OriginaleNome", "skip non deve modificare tag esistente");
    }
}
