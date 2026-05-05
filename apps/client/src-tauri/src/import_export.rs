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

/// Tauri command: esporta l'intero workspace come JSON serializzato.
#[tauri::command]
pub fn vault_export_json(state: State<'_, VaultState>) -> Result<String, PapErrore> {
    state.with_conn(|conn| {
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

        let mut stmt_prompts = conn.prepare(
            "SELECT Id, Title, Description, Body, Visibility, TargetModel,
                    IsFavorite, UseCount, LastUsedAt, Version, CreatedAt, UpdatedAt
             FROM Prompts
             WHERE DeletedAt IS NULL
             ORDER BY CreatedAt ASC",
        )?;
        let prompts_raw: Vec<(String, String, Option<String>, String, String,
                              Option<String>, i64, i64, Option<String>, i64,
                              String, String)> = stmt_prompts
            .query_map([], |r| {
                Ok((
                    r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
                    r.get(5)?, r.get::<_, i64>(6)?, r.get(7)?, r.get(8)?,
                    r.get(9)?, r.get(10)?, r.get(11)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

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
                folder_id: None,
                is_favorite: p.6 != 0,
                use_count: p.7,
                last_used_at: p.8,
                version: p.9,
                created_at: p.10,
                updated_at: p.11,
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

        let export = ExportV1 {
            schema_version: SCHEMA_VERSION,
            exported_at: ora_iso(),
            workspace,
            prompts,
            versions,
            tags,
            folders: Vec::new(),
        };

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

/// Tauri command: importa un export JSON applicando la modalità di gestione conflitti.
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

            match (esiste, modalita.as_str()) {
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

            let id_effettivo = match (esiste, modalita.as_str()) {
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
}
