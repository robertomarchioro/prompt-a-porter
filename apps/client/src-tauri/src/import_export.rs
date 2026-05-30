use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::embeddings::EmbeddingsState;
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

/// Esporta un singolo prompt in formato Markdown con YAML front-matter.
/// Front-matter compatibile con Jekyll/Hugo, include la lista degli
/// `imports` parsati dal body (riproducibilità).
///
/// v0.7.0 Step 6.
pub(crate) fn prompt_export_markdown_pure(
    conn: &Connection,
    prompt_id: &str,
) -> Result<String, PapErrore> {
    #[derive(Debug)]
    struct PromptRow {
        title: String,
        description: Option<String>,
        body: String,
        visibility: String,
        target_model: Option<String>,
        version: i64,
        created_at: String,
        updated_at: String,
    }

    let p = conn.query_row(
        "SELECT Title, Description, Body, Visibility, TargetModel, Version, CreatedAt, UpdatedAt
         FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
        [prompt_id],
        |r| {
            Ok(PromptRow {
                title: r.get(0)?,
                description: r.get(1)?,
                body: r.get(2)?,
                visibility: r.get(3)?,
                target_model: r.get(4)?,
                version: r.get(5)?,
                created_at: r.get(6)?,
                updated_at: r.get(7)?,
            })
        },
    ).map_err(|e| PapErrore::Generico(format!("Prompt '{prompt_id}' non trovato: {e}")))?;

    let imports = crate::prompt_componibili::parse_imports(&p.body);

    let mut fm = String::new();
    fm.push_str("---\n");
    fm.push_str(&format!("title: {}\n", yaml_quote(&p.title)));
    if let Some(d) = &p.description {
        if !d.trim().is_empty() {
            fm.push_str(&format!("description: {}\n", yaml_quote(d)));
        }
    }
    if let Some(m) = &p.target_model {
        if !m.trim().is_empty() {
            fm.push_str(&format!("target_model: {}\n", yaml_quote(m)));
        }
    }
    fm.push_str(&format!("visibility: {}\n", p.visibility));
    fm.push_str(&format!("version: {}\n", p.version));
    fm.push_str(&format!("created_at: {}\n", p.created_at));
    fm.push_str(&format!("updated_at: {}\n", p.updated_at));
    if !imports.is_empty() {
        fm.push_str("imports:\n");
        for imp in &imports {
            fm.push_str(&format!("  - {}\n", yaml_quote(&imp.path)));
        }
    }
    fm.push_str("---\n\n");
    fm.push_str(&p.body);

    Ok(fm)
}

/// Helper: emette una stringa YAML quotata in modo sicuro.
/// Usa double-quote + escape minimal: backslash e double-quote.
/// Sicuro per la maggior parte dei casi (nessun YAML anchor o tag);
/// per casi pathologic (newline embed, controlli) consigliato Jinja
/// block scalar, ma scope contenuto: usiamo escape minimale.
fn yaml_quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Tauri command: esporta un prompt in formato Markdown con
/// YAML front-matter. v0.7.0 Step 6.
#[tauri::command]
pub fn prompt_export_markdown(
    prompt_id: String,
    state: State<'_, VaultState>,
) -> Result<String, PapErrore> {
    state.with_conn(|conn| {
        let md = prompt_export_markdown_pure(conn, &prompt_id)?;
        crate::audit::registra(
            conn,
            "prompt.exported.markdown",
            "Prompt",
            &prompt_id,
            None,
        );
        Ok(md)
    })
}

// ─── M6 PR-1: Markdown import (singolo file) ───────────────────────

/// Risultato del parsing front-matter + body di un file `.md`.
/// Tutti i campi sono best-effort: front-matter assente o malformato
/// produce default sensati (titolo da fallback chiamante, visibility
/// "private", body intero).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownImport {
    pub titolo: String,
    pub descrizione: String,
    pub body: String,
    pub target_model: String,
    pub visibility: String,
}

impl Default for MarkdownImport {
    fn default() -> Self {
        Self {
            titolo: String::new(),
            descrizione: String::new(),
            body: String::new(),
            target_model: String::new(),
            visibility: "private".to_string(),
        }
    }
}

/// Parser puro del front-matter YAML semplificato + body markdown.
///
/// Grammatica supportata (subset coerente col formato emesso da
/// `prompt_export_markdown_pure`):
/// - Front-matter delimitato da `---` su righe dedicate
/// - Coppie `key: value` (value opzionalmente quoted con `"..."`)
/// - Liste `key:\n  - item\n  - item` (ignorate per M6 PR-1: i list
///   item come `imports` vengono ri-derivati dal body al re-save)
///
/// Compatibilità Obsidian/Foam: solo le chiavi note (`title`,
/// `description`, `body`, `target_model`, `visibility`) vengono lette;
/// chiavi sconosciute sono ignorate silentemente.
///
/// Se il front-matter è assente o malformato, il body intero viene
/// usato e i metadati restano default (chiamante deve fornire titolo
/// fallback, es. nome file).
pub fn parse_markdown_frontmatter(testo: &str) -> MarkdownImport {
    let mut out = MarkdownImport::default();
    let testo = testo.trim_start_matches('\u{FEFF}'); // strip BOM

    // Front-matter presente solo se inizia con `---\n` (o `---\r\n`)
    let inizio_fm = testo.strip_prefix("---\n").or_else(|| testo.strip_prefix("---\r\n"));
    let Some(after_open) = inizio_fm else {
        // No front-matter -> tutto e' body
        out.body = testo.to_string();
        return out;
    };

    // Cerca chiusura `\n---\n` o `\n---\r\n` o `\n---` a fine file
    let fm_end_marker = find_fm_close(after_open);
    let Some((fm_block, body)) = fm_end_marker else {
        // Front-matter non chiuso -> trattalo come body
        out.body = testo.to_string();
        return out;
    };

    // Parse riga per riga
    for line in fm_block.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("  - ") {
            // Skip silenzioso (item di lista, ignorato per ora)
            continue;
        }
        let Some(colon) = trimmed.find(':') else {
            continue;
        };
        let key = trimmed[..colon].trim();
        let raw_value = trimmed[colon + 1..].trim();
        // Se value vuoto, e' header di lista -> skip (poi sotto-righe
        // `  - ...` saranno ignorate sopra)
        if raw_value.is_empty() {
            continue;
        }
        let value = unquote_yaml(raw_value);
        match key {
            "title" => out.titolo = value,
            "description" => out.descrizione = value,
            "target_model" => out.target_model = value,
            "visibility" => out.visibility = value,
            _ => {} // ignora chiavi sconosciute
        }
    }

    out.body = body.trim_start_matches('\n').to_string();
    if out.visibility.is_empty() {
        out.visibility = "private".to_string();
    }
    out
}

/// Cerca la chiusura del front-matter (`---` su riga dedicata) e
/// ritorna `(fm_content_senza_delimitatori, body_dopo_delimitatore)`.
fn find_fm_close(s: &str) -> Option<(String, String)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Cerca un newline seguito da "---"
        if let Some(rel) = s[i..].find("\n---") {
            let pos = i + rel;
            let after_marker = pos + 4; // skip "\n---"
            // Verifica che sia una riga dedicata: dopo "---" o EOF o \n o \r\n
            let is_eol = after_marker >= bytes.len()
                || bytes[after_marker] == b'\n'
                || (bytes[after_marker] == b'\r'
                    && after_marker + 1 < bytes.len()
                    && bytes[after_marker + 1] == b'\n');
            if is_eol {
                let fm = s[..pos].to_string();
                let body_start = if after_marker >= bytes.len() {
                    bytes.len()
                } else if bytes[after_marker] == b'\r' {
                    after_marker + 2
                } else {
                    after_marker + 1
                };
                let body = s[body_start..].to_string();
                return Some((fm, body));
            }
            i = pos + 1;
        } else {
            return None;
        }
    }
    None
}

/// Helper: rimuove eventuali quote di apertura/chiusura da un value
/// YAML (`"..."` o `'...'`) e unescape `\"` / `\\`.
fn unquote_yaml(s: &str) -> String {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
    {
        let inner = &trimmed[1..trimmed.len() - 1];
        return inner.replace("\\\"", "\"").replace("\\\\", "\\");
    }
    trimmed.to_string()
}

/// Tauri command: importa un singolo file `.md` come nuovo prompt.
/// `testo` è il contenuto del file (frontend lo legge via fs API).
/// `nome_file` è usato come fallback titolo se front-matter assente.
/// Ritorna l'id del prompt creato.
#[tauri::command]
pub fn prompt_import_markdown(
    testo: String,
    nome_file: Option<String>,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<String, PapErrore> {
    let mut parsed = parse_markdown_frontmatter(&testo);
    // Fallback titolo: nome file senza estensione, o "Importato"
    if parsed.titolo.trim().is_empty() {
        parsed.titolo = nome_file
            .as_deref()
            .map(|n| n.trim_end_matches(".md").trim_end_matches(".markdown").to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Importato".to_string());
    }
    let body = parsed.body.trim().to_string();
    let visibility = match parsed.visibility.as_str() {
        "workspace" => "workspace",
        _ => "private",
    };

    let nuovo_id = format!("prm-{}", crate::editor::genera_id());
    state.with_conn(|conn| {
        conn.execute(
            "INSERT INTO Prompts
                (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
                 Visibility, TargetModel, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, ?4, ?5, ?6, 1,
                     datetime('now'), datetime('now'))",
            rusqlite::params![
                nuovo_id,
                parsed.titolo.trim(),
                parsed.descrizione.trim(),
                body.as_str(),
                visibility,
                if parsed.target_model.trim().is_empty() {
                    None
                } else {
                    Some(parsed.target_model.trim())
                },
            ],
        )?;
        crate::versioning::snapshot_versione(conn, &nuovo_id, "usr-locale")?;
        crate::editor::ricostruisci_fts(conn)?;
        crate::editor::aggiorna_embedding(conn, &rt_state, &nuovo_id, &body)?;
        crate::prompt_componibili::aggiorna_imports(conn, &nuovo_id, &body)?;
        crate::audit::registra(
            conn,
            "prompt.imported.markdown",
            "Prompt",
            &nuovo_id,
            Some(parsed.titolo.trim()),
        );
        Ok(())
    })?;
    log::info!("Prompt importato da markdown: {nuovo_id}");
    Ok(nuovo_id)
}

// ─── M6 PR-2: Markdown import bulk (directory ricorsiva) ───────────

/// Limite profondità ricorsione per evitare loop simbolici / strutture
/// patologiche. Coerente con il limite import nidificati (5 livelli).
pub const BULK_MAX_DEPTH: usize = 5;

/// Limite numero file processati per chiamata (protezione contro
/// directory enormi che bloccherebbero la UI per minuti).
pub const BULK_MAX_FILES: usize = 1000;

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportFileSuccess {
    pub nome_file: String,
    pub id: String,
    pub titolo: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportFileError {
    pub nome_file: String,
    pub errore: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkImportReport {
    pub totale_trovati: usize,
    pub importati: Vec<BulkImportFileSuccess>,
    pub falliti: Vec<BulkImportFileError>,
    /// True se la scansione si è fermata per limit (vedi BULK_MAX_FILES).
    pub truncated: bool,
}

/// Walker ricorsivo: raccoglie path dei file `.md`/`.markdown` sotto
/// `root`, in ordine alfabetico stabile per riproducibilità.
/// Salta link simbolici e directory nascoste (prefisso `.`).
fn walk_md_files(
    root: &std::path::Path,
    depth: usize,
    accum: &mut Vec<std::path::PathBuf>,
) -> std::io::Result<()> {
    if depth > BULK_MAX_DEPTH || accum.len() >= BULK_MAX_FILES {
        return Ok(());
    }
    let mut entries: Vec<_> = std::fs::read_dir(root)?
        .filter_map(|e| e.ok())
        .collect();
    // Ordine alfabetico per output deterministico
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        if accum.len() >= BULK_MAX_FILES {
            break;
        }
        let path = entry.path();
        let file_name = entry.file_name();
        let name_lossy = file_name.to_string_lossy();
        // Salta dir/file nascosti (Obsidian: .obsidian/, .git/, ecc.)
        if name_lossy.starts_with('.') {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            let _ = walk_md_files(&path, depth + 1, accum);
        } else if metadata.is_file() {
            let lower = name_lossy.to_lowercase();
            if lower.ends_with(".md") || lower.ends_with(".markdown") {
                accum.push(path);
            }
        }
    }
    Ok(())
}

/// Tauri command: importa tutti i file `.md`/`.markdown` da una
/// directory ricorsivamente (max 5 livelli, max 1000 file per call).
///
/// Best-effort: ogni file viene importato individualmente; gli errori
/// per-file (parsing, INSERT, ecc.) sono raccolti nel report senza
/// interrompere il batch. La directory viene scansionata in ordine
/// alfabetico per output deterministico.
///
/// Skipped silently: file/dir nascosti (prefisso `.`, es. `.obsidian/`,
/// `.git/`), link simbolici (loop protection).
#[tauri::command]
pub fn vault_import_markdown_bulk(
    directory: String,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<BulkImportReport, PapErrore> {
    let root = std::path::PathBuf::from(&directory);
    if !root.exists() {
        return Err(PapErrore::Generico(format!(
            "Directory non esistente: {directory}"
        )));
    }
    if !root.is_dir() {
        return Err(PapErrore::Generico(format!(
            "Path non e' una directory: {directory}"
        )));
    }

    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    walk_md_files(&root, 0, &mut paths).map_err(|e| {
        PapErrore::Generico(format!("Errore scansione directory: {e}"))
    })?;
    let totale = paths.len();
    let truncated = totale >= BULK_MAX_FILES;

    // Singolo `with_conn` esterno: il loop sui file gira dentro la
    // chiusura -> `rt_state` viene catturato per reference, no clone.
    // L'errore singolo (es. INSERT fail) non interrompe il batch:
    // accumula in `falliti` e continua.
    let (importati, falliti) = state.with_conn(|conn| {
        let mut ok: Vec<BulkImportFileSuccess> = Vec::new();
        let mut ko: Vec<BulkImportFileError> = Vec::new();

        for path in &paths {
            let nome_file = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());

            let testo = match std::fs::read_to_string(path) {
                Ok(t) => t,
                Err(e) => {
                    ko.push(BulkImportFileError {
                        nome_file,
                        errore: format!("read_to_string fallita: {e}"),
                    });
                    continue;
                }
            };

            let mut parsed = parse_markdown_frontmatter(&testo);
            if parsed.titolo.trim().is_empty() {
                parsed.titolo = nome_file
                    .trim_end_matches(".md")
                    .trim_end_matches(".markdown")
                    .to_string();
                if parsed.titolo.trim().is_empty() {
                    parsed.titolo = "Importato".to_string();
                }
            }
            let body = parsed.body.trim().to_string();
            let visibility = match parsed.visibility.as_str() {
                "workspace" => "workspace",
                _ => "private",
            };
            let target_opt: Option<&str> = if parsed.target_model.trim().is_empty() {
                None
            } else {
                Some(parsed.target_model.trim())
            };

            let nuovo_id = format!("prm-{}", crate::editor::genera_id());
            let exec: Result<(), PapErrore> = (|| {
                conn.execute(
                    "INSERT INTO Prompts
                        (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
                         Visibility, TargetModel, Version, CreatedAt, UpdatedAt)
                     VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, ?4, ?5, ?6, 1,
                             datetime('now'), datetime('now'))",
                    rusqlite::params![
                        nuovo_id,
                        parsed.titolo.trim(),
                        parsed.descrizione.trim(),
                        body.as_str(),
                        visibility,
                        target_opt,
                    ],
                )?;
                crate::versioning::snapshot_versione(conn, &nuovo_id, "usr-locale")?;
                crate::editor::aggiorna_embedding(conn, &rt_state, &nuovo_id, &body)?;
                crate::prompt_componibili::aggiorna_imports(conn, &nuovo_id, &body)?;
                Ok(())
            })();

            match exec {
                Ok(()) => ok.push(BulkImportFileSuccess {
                    nome_file,
                    id: nuovo_id,
                    titolo: parsed.titolo.trim().to_string(),
                }),
                Err(e) => ko.push(BulkImportFileError {
                    nome_file,
                    errore: format!("{e}"),
                }),
            }
        }

        // FTS rebuild una volta sola a fine batch (evita N rebuild costosi).
        // Errore loggato: un FTS stantio rende la ricerca incompleta, ma non
        // deve far fallire un import altrimenti riuscito.
        if let Err(e) = crate::editor::ricostruisci_fts(conn) {
            log::error!("ricostruisci_fts fallita dopo bulk import markdown: {e}");
        }
        crate::audit::registra(
            conn,
            "vault.imported.markdown.bulk",
            "Vault",
            "bulk",
            Some(&format!(
                "importati={}, falliti={}, truncated={}",
                ok.len(),
                ko.len(),
                truncated
            )),
        );
        Ok((ok, ko))
    })?;

    log::info!(
        "Bulk markdown import completato: {}/{} importati, {} falliti{}",
        importati.len(),
        totale,
        falliti.len(),
        if truncated { " (TRUNCATED)" } else { "" }
    );

    Ok(BulkImportReport {
        totale_trovati: totale,
        importati,
        falliti,
        truncated,
    })
}

// ─── M6 PR-3: Markdown export bulk -> zip ──────────────────────────

/// Sanitize titolo per uso come filename: rimuovi separatori path
/// (`/`, `\`) e caratteri non sicuri su Windows (`<>:"|?*`), poi trim.
/// Stringa vuota dopo sanitize -> usa fallback `prompt`.
fn sanitize_filename(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| {
            if matches!(
                c,
                '/' | '\\'
                    | '<'
                    | '>'
                    | ':'
                    | '"'
                    | '|'
                    | '?'
                    | '*'
                    | '\0'
                    | '\n'
                    | '\r'
                    | '\t'
            ) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').to_string();
    if trimmed.is_empty() {
        "prompt".to_string()
    } else {
        trimmed
    }
}

/// Sanitize folder path: ogni segmento separato da `/` viene
/// sanitizzato individualmente, segmenti vuoti scartati.
fn sanitize_folder_path(path: &str) -> String {
    path.trim_start_matches('/')
        .split('/')
        .filter(|seg| !seg.is_empty())
        .map(sanitize_filename)
        .collect::<Vec<_>>()
        .join("/")
}

/// Logica pura: scrive un archivio zip in `buf` contenente tutti i
/// prompt del vault (filtrati opzionalmente per folder root).
///
/// Layout zip:
/// ```text
/// <folder_path>/<title>.md
/// ```
/// Per prompt in root il file e' alla radice dello zip.
/// Per titoli duplicati nella stessa cartella, viene suffissato
/// `-{id-short}` per disambiguare.
pub(crate) fn export_markdown_zip_pure(
    conn: &Connection,
    folder_id_root: Option<&str>,
    buf: &mut std::io::Cursor<Vec<u8>>,
) -> Result<usize, PapErrore> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    // Risolvi folder root path se passato (per filtrare sottoalbero).
    let folder_root_path: Option<String> = match folder_id_root {
        Some(id) if !id.trim().is_empty() => conn
            .query_row(
                "SELECT Path FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
                [id],
                |r| r.get::<_, String>(0),
            )
            .ok(),
        _ => None,
    };

    // SELECT prompt + folder path. Filtro:
    // - se folder_root_path settato: FolderId == root OR Path LIKE root/%
    // - altrimenti: tutto il vault
    let sql = "
        SELECT p.Id, p.Title, COALESCE(f.Path, '')
        FROM Prompts p
        LEFT JOIN Folders f ON f.Id = p.FolderId AND f.DeletedAt IS NULL
        WHERE p.DeletedAt IS NULL
          AND (:root IS NULL OR f.Path = :root OR f.Path LIKE :root_prefix)
        ORDER BY f.Path, p.Title COLLATE NOCASE
    ";
    let root_param: Option<String> = folder_root_path.clone();
    let root_prefix_param: Option<String> = folder_root_path
        .as_ref()
        .map(|p| format!("{p}/%"));

    let mut stmt = conn.prepare(sql)?;
    let rows: Vec<(String, String, String)> = stmt
        .query_map(
            rusqlite::named_params! {
                ":root": root_param,
                ":root_prefix": root_prefix_param,
            },
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            },
        )?
        .filter_map(|r| r.ok())
        .collect();

    let mut zipw = zip::ZipWriter::new(buf);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Detect collisioni nome file all'interno dello stesso folder
    let mut nomi_per_folder: std::collections::HashMap<String, std::collections::HashSet<String>> =
        std::collections::HashMap::new();
    let mut esportati = 0usize;

    for (id, titolo, folder_path) in &rows {
        let md = match prompt_export_markdown_pure(conn, id) {
            Ok(s) => s,
            Err(_) => continue, // skip silenzioso prompt rotti
        };

        let folder_zip = sanitize_folder_path(folder_path);
        let titolo_safe = sanitize_filename(titolo);
        let base_filename = format!("{titolo_safe}.md");
        let nomi_in_folder = nomi_per_folder
            .entry(folder_zip.clone())
            .or_default();
        let filename = if nomi_in_folder.contains(&base_filename) {
            // Collisione: suffix id-short
            let id_short = id.strip_prefix("prm-").unwrap_or(id);
            let suffix_len = id_short.len().min(8);
            format!("{titolo_safe}-{}.md", &id_short[..suffix_len])
        } else {
            base_filename
        };
        nomi_in_folder.insert(filename.clone());

        let full_path = if folder_zip.is_empty() {
            filename
        } else {
            format!("{folder_zip}/{filename}")
        };

        zipw.start_file(&full_path, options)
            .map_err(|e| PapErrore::Generico(format!("zip start_file: {e}")))?;
        zipw.write_all(md.as_bytes())
            .map_err(|e| PapErrore::Generico(format!("zip write_all: {e}")))?;
        esportati += 1;
    }

    zipw.finish()
        .map_err(|e| PapErrore::Generico(format!("zip finish: {e}")))?;
    Ok(esportati)
}

#[derive(Debug, Clone, Serialize)]
pub struct ZipExportResult {
    /// Bytes del file zip serializzato. Frontend lo salva via fs API.
    pub bytes: Vec<u8>,
    pub totale_esportati: usize,
}

/// Tauri command: esporta tutti i prompt del vault (o di una cartella
/// + discendenti) come archivio zip contenente file .md con
/// front-matter YAML. Il frontend riceve i bytes e li salva tramite
/// `dialog.save` + `fs.writeFile`.
#[tauri::command]
pub fn vault_export_markdown_zip(
    folder_id: Option<String>,
    state: State<'_, VaultState>,
) -> Result<ZipExportResult, PapErrore> {
    let mut buf = std::io::Cursor::new(Vec::new());
    let totale = state.with_conn(|conn| {
        let n = export_markdown_zip_pure(conn, folder_id.as_deref(), &mut buf)?;
        crate::audit::registra(
            conn,
            "vault.exported.markdown.zip",
            "Vault",
            folder_id.as_deref().unwrap_or("vault"),
            Some(&format!("totale_esportati={n}")),
        );
        Ok::<usize, PapErrore>(n)
    })?;
    log::info!("Vault esportato in zip markdown: {totale} prompt");
    Ok(ZipExportResult {
        bytes: buf.into_inner(),
        totale_esportati: totale,
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
            (_, altro) => {
                return Err(PapErrore::Generico(format!(
                    "Modalità import non gestita: {altro}"
                )))
            }
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
            (_, altro) => {
                return Err(PapErrore::Generico(format!(
                    "Modalità import non gestita: {altro}"
                )))
            }
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

        // Riassocia tag. Gli errori non bloccano l'import ma sono riportati
        // (prima venivano scartati silenziosamente -> dati tag incompleti).
        if let Err(e) = conn.execute(
            "DELETE FROM PromptTags WHERE PromptId = ?1",
            [&id_effettivo],
        ) {
            report
                .errori
                .push(format!("PromptTags delete {id_effettivo}: {e}"));
        }
        for tag_id in &prompt.tag_ids {
            if let Err(e) = conn.execute(
                "INSERT OR IGNORE INTO PromptTags (PromptId, TagId) VALUES (?1, ?2)",
                rusqlite::params![id_effettivo, tag_id],
            ) {
                report
                    .errori
                    .push(format!("PromptTag {id_effettivo}/{tag_id}: {e}"));
            }
        }
    }

    // Versioni storiche (INSERT OR IGNORE per UNIQUE constraint). Errori
    // riportati invece di scartati silenziosamente.
    for ver in &export.versions {
        if let Err(e) = conn.execute(
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
        ) {
            report.errori.push(format!("Versione {}: {e}", ver.id));
        }
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

    // ─────────── v0.7.0 Step 6: prompt_export_markdown ───────────

    #[test]
    fn yaml_quote_escapa_double_quote_e_backslash() {
        assert_eq!(super::yaml_quote("semplice"), "\"semplice\"");
        assert_eq!(super::yaml_quote("con \"virgolette\""), "\"con \\\"virgolette\\\"\"");
        assert_eq!(super::yaml_quote("path\\back"), "\"path\\\\back\"");
    }

    #[test]
    fn export_markdown_prompt_inesistente_errore() {
        let conn = db_test();
        let r = super::prompt_export_markdown_pure(&conn, "prm-fantasma");
        assert!(r.is_err());
    }

    #[test]
    fn export_markdown_genera_front_matter_minimo() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Marketing Email", "Riformula in tono formale.");
        let md = super::prompt_export_markdown_pure(&conn, "prm-1").unwrap();
        assert!(md.starts_with("---\n"));
        assert!(md.contains("title: \"Marketing Email\""));
        assert!(md.contains("visibility: private"));
        assert!(md.contains("version: 1"));
        // Body separato dal front-matter da \n\n.
        assert!(md.contains("---\n\nRiformula in tono formale."));
    }

    #[test]
    fn export_markdown_include_imports_dal_body() {
        let conn = db_test();
        inserisci_prompt(
            &conn,
            "prm-comp",
            "Composto",
            r#"Body con {{import "header"}} e {{import "footer/std"}}."#,
        );
        let md = super::prompt_export_markdown_pure(&conn, "prm-comp").unwrap();
        assert!(md.contains("imports:"));
        assert!(md.contains("  - \"header\""));
        assert!(md.contains("  - \"footer/std\""));
    }

    #[test]
    fn export_markdown_no_imports_se_body_pulito() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-pulito", "Pulito", "Body senza alcun import.");
        let md = super::prompt_export_markdown_pure(&conn, "prm-pulito").unwrap();
        assert!(!md.contains("imports:"));
    }

    #[test]
    fn export_markdown_skippa_description_vuota() {
        let conn = db_test();
        // inserisci_prompt non setta Description, NULL → skip.
        inserisci_prompt(&conn, "prm-1", "Title", "Body abbastanza lungo per il test.");
        let md = super::prompt_export_markdown_pure(&conn, "prm-1").unwrap();
        assert!(!md.contains("description:"));
    }

    // ─── M6 PR-1: parse_markdown_frontmatter ───────────────────────

    #[test]
    fn parse_md_no_frontmatter_tutto_body() {
        let r = super::parse_markdown_frontmatter("Solo body senza front-matter");
        assert_eq!(r.body, "Solo body senza front-matter");
        assert!(r.titolo.is_empty());
        assert_eq!(r.visibility, "private");
    }

    #[test]
    fn parse_md_frontmatter_completo() {
        let testo = "---\ntitle: \"Mio Prompt\"\ndescription: \"Una descrizione\"\ntarget_model: \"claude-sonnet-4-6\"\nvisibility: private\nversion: 3\n---\n\nQuesto è il body.";
        let r = super::parse_markdown_frontmatter(testo);
        assert_eq!(r.titolo, "Mio Prompt");
        assert_eq!(r.descrizione, "Una descrizione");
        assert_eq!(r.target_model, "claude-sonnet-4-6");
        assert_eq!(r.visibility, "private");
        assert_eq!(r.body, "Questo è il body.");
    }

    #[test]
    fn parse_md_frontmatter_value_non_quotato() {
        let testo = "---\ntitle: SenzaQuote\nvisibility: workspace\n---\nBody";
        let r = super::parse_markdown_frontmatter(testo);
        assert_eq!(r.titolo, "SenzaQuote");
        assert_eq!(r.visibility, "workspace");
        assert_eq!(r.body, "Body");
    }

    #[test]
    fn parse_md_frontmatter_non_chiuso_diventa_body() {
        let testo = "---\ntitle: \"Test\"\n\nQuesto manca del separatore di chiusura.";
        let r = super::parse_markdown_frontmatter(testo);
        assert!(r.titolo.is_empty());
        assert_eq!(r.body, testo);
    }

    #[test]
    fn parse_md_strip_bom() {
        let testo = "\u{FEFF}---\ntitle: \"Con BOM\"\n---\nBody";
        let r = super::parse_markdown_frontmatter(testo);
        assert_eq!(r.titolo, "Con BOM");
    }

    #[test]
    fn parse_md_lista_imports_ignorata() {
        let testo = "---\ntitle: \"X\"\nimports:\n  - foo\n  - bar/baz\n---\nbody";
        let r = super::parse_markdown_frontmatter(testo);
        assert_eq!(r.titolo, "X");
        assert_eq!(r.body, "body");
        // Lista imports skip silenzioso (la re-derivazione avviene
        // al save via prompt_componibili::aggiorna_imports).
    }

    #[test]
    fn parse_md_chiavi_obsidian_ignorate() {
        let testo = "---\ntitle: \"Obsidian\"\ntags: [foo, bar]\naliases: [x]\n---\nbody";
        let r = super::parse_markdown_frontmatter(testo);
        assert_eq!(r.titolo, "Obsidian");
        assert_eq!(r.body, "body");
        // tags/aliases sono ignorate silentemente (non rompono import)
    }

    #[test]
    fn parse_md_value_con_escape() {
        let testo = "---\ntitle: \"Con \\\"quote\\\" dentro\"\n---\nbody";
        let r = super::parse_markdown_frontmatter(testo);
        assert_eq!(r.titolo, "Con \"quote\" dentro");
    }

    #[test]
    fn parse_md_crlf_line_endings() {
        let testo = "---\r\ntitle: \"Windows\"\r\n---\r\nbody";
        let r = super::parse_markdown_frontmatter(testo);
        assert_eq!(r.titolo, "Windows");
        assert_eq!(r.body, "body");
    }

    #[test]
    fn parse_md_visibility_default_private() {
        let testo = "---\ntitle: \"X\"\n---\nbody";
        let r = super::parse_markdown_frontmatter(testo);
        assert_eq!(r.visibility, "private");
    }

    // ─── M6 PR-2: walk_md_files (puro filesystem walker) ───────────

    fn tmp_dir_test() -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix("pap-bulk-md-")
            .tempdir()
            .unwrap()
    }

    fn write_file(dir: &std::path::Path, nome: &str, contenuto: &str) {
        let p = dir.join(nome);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, contenuto).unwrap();
    }

    #[test]
    fn walk_raccoglie_md_root_e_sottocartelle() {
        let dir = tmp_dir_test();
        write_file(dir.path(), "a.md", "body a");
        write_file(dir.path(), "sub/b.md", "body b");
        write_file(dir.path(), "sub/nested/c.markdown", "body c");
        write_file(dir.path(), "ignored.txt", "non md");

        let mut acc = Vec::new();
        super::walk_md_files(dir.path(), 0, &mut acc).unwrap();
        assert_eq!(acc.len(), 3);
        // Ordine alfabetico stabile (a, sub/b, sub/nested/c)
        let nomi: Vec<String> = acc
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(nomi.contains(&"a.md".to_string()));
        assert!(nomi.contains(&"b.md".to_string()));
        assert!(nomi.contains(&"c.markdown".to_string()));
    }

    #[test]
    fn walk_salta_dir_nascoste() {
        let dir = tmp_dir_test();
        write_file(dir.path(), ".obsidian/config.md", "skip me");
        write_file(dir.path(), ".git/objects/x.md", "skip me");
        write_file(dir.path(), "visibile.md", "keep");

        let mut acc = Vec::new();
        super::walk_md_files(dir.path(), 0, &mut acc).unwrap();
        assert_eq!(acc.len(), 1);
        assert!(acc[0].ends_with("visibile.md"));
    }

    #[test]
    fn walk_salta_file_non_md() {
        let dir = tmp_dir_test();
        write_file(dir.path(), "doc.txt", "no");
        write_file(dir.path(), "img.png", "binary");
        write_file(dir.path(), "valid.md", "yes");
        write_file(dir.path(), "valid.MD", "case-insensitive");

        let mut acc = Vec::new();
        super::walk_md_files(dir.path(), 0, &mut acc).unwrap();
        assert_eq!(acc.len(), 2);
    }

    #[test]
    fn walk_rispetta_max_depth() {
        let dir = tmp_dir_test();
        // Crea 7 livelli; MAX_DEPTH=5 -> taglia dopo
        let mut nested = dir.path().to_path_buf();
        for i in 0..7 {
            nested.push(format!("d{i}"));
        }
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("deep.md"), "too deep").unwrap();
        // E uno a profondita' accettabile (depth=2)
        write_file(dir.path(), "d0/d1/ok.md", "shallow");

        let mut acc = Vec::new();
        super::walk_md_files(dir.path(), 0, &mut acc).unwrap();
        let names: Vec<String> = acc
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(names.contains(&"ok.md".to_string()));
        assert!(!names.contains(&"deep.md".to_string()), "deep.md non doveva passare MAX_DEPTH=5");
    }

    // ─── M6 PR-3: export bulk -> zip ───────────────────────────────

    #[test]
    fn sanitize_filename_rimuove_caratteri_non_sicuri() {
        assert_eq!(super::sanitize_filename("Hello World"), "Hello World");
        assert_eq!(super::sanitize_filename("foo/bar"), "foo_bar");
        assert_eq!(super::sanitize_filename("a<b>c:d\"e|f?g*h"), "a_b_c_d_e_f_g_h");
        assert_eq!(super::sanitize_filename("/leading"), "_leading");
        assert_eq!(super::sanitize_filename(""), "prompt");
        assert_eq!(super::sanitize_filename("   "), "prompt");
        assert_eq!(super::sanitize_filename("..."), "prompt");
    }

    #[test]
    fn sanitize_folder_path_normale() {
        assert_eq!(super::sanitize_folder_path(""), "");
        assert_eq!(super::sanitize_folder_path("/"), "");
        assert_eq!(super::sanitize_folder_path("/marketing"), "marketing");
        assert_eq!(
            super::sanitize_folder_path("/marketing/email"),
            "marketing/email"
        );
        // Segmenti con caratteri non sicuri sanitizzati
        assert_eq!(super::sanitize_folder_path("/a:b/c?d"), "a_b/c_d");
    }

    fn unzip_entries(bytes: &[u8]) -> Vec<(String, String)> {
        let mut ar = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let mut out = Vec::new();
        for i in 0..ar.len() {
            let mut f = ar.by_index(i).unwrap();
            let name = f.name().to_string();
            let mut content = String::new();
            use std::io::Read;
            f.read_to_string(&mut content).unwrap();
            out.push((name, content));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    #[test]
    fn export_zip_vault_vuoto_ritorna_zip_valido_vuoto() {
        let conn = db_test();
        let mut buf = std::io::Cursor::new(Vec::new());
        let n = super::export_markdown_zip_pure(&conn, None, &mut buf).unwrap();
        assert_eq!(n, 0);
        // Zip vuoto e' comunque valido
        let entries = unzip_entries(&buf.into_inner());
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn export_zip_include_tutti_i_prompt_root() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Alpha", "body alpha");
        inserisci_prompt(&conn, "prm-2", "Beta", "body beta");

        let mut buf = std::io::Cursor::new(Vec::new());
        let n = super::export_markdown_zip_pure(&conn, None, &mut buf).unwrap();
        assert_eq!(n, 2);
        let entries = unzip_entries(&buf.into_inner());
        let names: Vec<String> = entries.iter().map(|(n, _)| n.clone()).collect();
        assert!(names.contains(&"Alpha.md".to_string()));
        assert!(names.contains(&"Beta.md".to_string()));
        // Contenuto include front-matter
        let alpha = entries.iter().find(|(n, _)| n == "Alpha.md").unwrap();
        assert!(alpha.1.starts_with("---\n"));
        assert!(alpha.1.contains("title: \"Alpha\""));
        assert!(alpha.1.contains("body alpha"));
    }

    #[test]
    fn export_zip_layout_con_folder() {
        let conn = db_test();
        inserisci_folder(&conn, "fld-mkt", "Marketing", "/marketing", None);
        inserisci_folder(&conn, "fld-mkt-mail", "Email", "/marketing/email", Some("fld-mkt"));

        // 1 prompt root + 1 in /marketing + 1 in /marketing/email
        inserisci_prompt(&conn, "prm-root", "RootP", "body root");
        // Insert con FolderId
        conn.execute(
            "UPDATE Prompts SET FolderId = 'fld-mkt' WHERE Id = 'prm-root'",
            [],
        )
        .ok();
        // Reinserisci due con folder
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, FolderId, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-m', 'ws-personale', 'usr-locale', 'Mkt', 'body mkt', 'private', 'fld-mkt', 1, datetime('now'), datetime('now'))",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, FolderId, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-me', 'ws-personale', 'usr-locale', 'EmailP', 'body email', 'private', 'fld-mkt-mail', 1, datetime('now'), datetime('now'))",
            [],
        ).unwrap();

        let mut buf = std::io::Cursor::new(Vec::new());
        let n = super::export_markdown_zip_pure(&conn, None, &mut buf).unwrap();
        assert_eq!(n, 3);
        let entries = unzip_entries(&buf.into_inner());
        let names: Vec<String> = entries.iter().map(|(n, _)| n.clone()).collect();
        // RootP era riassegnato a fld-mkt
        assert!(names.contains(&"marketing/RootP.md".to_string()));
        assert!(names.contains(&"marketing/Mkt.md".to_string()));
        assert!(names.contains(&"marketing/email/EmailP.md".to_string()));
    }

    #[test]
    fn export_zip_filtro_per_folder_id_root_include_discendenti() {
        let conn = db_test();
        inserisci_folder(&conn, "fld-mkt", "Marketing", "/marketing", None);
        inserisci_folder(&conn, "fld-vendita", "Vendita", "/vendita", None);
        inserisci_folder(&conn, "fld-mkt-mail", "Email", "/marketing/email", Some("fld-mkt"));

        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, FolderId, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-m', 'ws-personale', 'usr-locale', 'Mkt', 'b', 'private', 'fld-mkt', 1, datetime('now'), datetime('now'))",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, FolderId, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-me', 'ws-personale', 'usr-locale', 'EmailP', 'b', 'private', 'fld-mkt-mail', 1, datetime('now'), datetime('now'))",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, FolderId, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-v', 'ws-personale', 'usr-locale', 'Vend', 'b', 'private', 'fld-vendita', 1, datetime('now'), datetime('now'))",
            [],
        ).unwrap();

        let mut buf = std::io::Cursor::new(Vec::new());
        // Filtra solo subtree /marketing -> Mkt + EmailP, NO Vend
        let n =
            super::export_markdown_zip_pure(&conn, Some("fld-mkt"), &mut buf).unwrap();
        assert_eq!(n, 2);
        let entries = unzip_entries(&buf.into_inner());
        let names: Vec<String> = entries.iter().map(|(n, _)| n.clone()).collect();
        assert!(names.iter().any(|n| n.contains("Mkt.md")));
        assert!(names.iter().any(|n| n.contains("EmailP.md")));
        assert!(!names.iter().any(|n| n.contains("Vend.md")));
    }

    #[test]
    fn export_zip_collisione_titoli_disambigua_con_id() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-abc12345", "Duplicato", "uno");
        inserisci_prompt(&conn, "prm-def67890", "Duplicato", "due");

        let mut buf = std::io::Cursor::new(Vec::new());
        let n = super::export_markdown_zip_pure(&conn, None, &mut buf).unwrap();
        assert_eq!(n, 2);
        let entries = unzip_entries(&buf.into_inner());
        let names: Vec<String> = entries.iter().map(|(n, _)| n.clone()).collect();
        // Entrambi presenti, uno con suffisso id-short
        assert!(names.contains(&"Duplicato.md".to_string()));
        assert!(names.iter().any(|n| n.starts_with("Duplicato-")), "atteso suffisso id, got {names:?}");
    }
}
