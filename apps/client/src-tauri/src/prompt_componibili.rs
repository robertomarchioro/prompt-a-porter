// Modulo prompt_componibili — Fase 3 Step 8.
//
// Trasforma il vault da "lista di prompt" a sistema modulare di prompt
// componibili via sintassi `{{import "path"}}` nel body.
//
// Scope MVP (questa PR):
// - Parser estrae gli import dal body in ordine di apparizione
// - Resolver: path → prompt_id usando Folders.Path + Title (slug)
// - Compile ricorsivo con cycle detection (HashSet di id visitati) e
//   depth limit configurabile
// - Tabella PromptImports popolata ad ogni save (graph delle dipendenze
//   per future query "chi importa X")
// - Comando Tauri prompt_compila(id) → testo finale espanso
//
// Out of scope (follow-up):
// - Sintassi `with k=v` per variabili passate all'import
// - Sintassi `version=N` per pinning a versioni storiche
// - IMP001/002/003 lint rules in `linting.rs` (richiede accesso DB,
//   refactor del comando lint)

use regex::Regex;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::OnceLock;
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

/// Profondità massima di nidificazione import. Oltre questa soglia il
/// resolver ritorna errore — impedisce esplosioni da grafi profondi.
pub const MAX_DEPTH: usize = 5;

/// Limite di lunghezza output per evitare bombe di compilazione (analogo
/// a billion-laughs in XML). Se la compilazione produce più di tot byte,
/// fallisce.
const MAX_OUTPUT_BYTES: usize = 1_000_000;

#[derive(Debug, Clone, Serialize)]
pub struct ImportRef {
    pub path: String,
    /// Posizione 0-based nel body (ordine di apparizione).
    pub position: usize,
    /// Range byte nel body originale (inizio inclusivo, fine esclusiva).
    /// Usato dal compile per sostituire la stringa `{{import "..."}}`
    /// con il body risolto.
    pub byte_start: usize,
    pub byte_end: usize,
    /// M4 PR-1: variabili scopate passate via sintassi
    /// `{{import "x" with k=v, k2="value"}}`. Vec preserva l'ordine di
    /// dichiarazione; in caso di chiavi duplicate, vince l'ultima
    /// (semantica simile a CSS `last-rule-wins`).
    #[serde(default)]
    pub variables: Vec<(String, String)>,
    /// M4 PR-2: pinning a versione storica via sintassi
    /// `{{import "x" version=N}}`. Se `Some(N)`, compila_ricorsivo
    /// legge il body da `PromptVersions` invece che da `Prompts`,
    /// permettendo a un prompt di importare uno snapshot stabile
    /// del child anche dopo modifiche. `None` = ultima versione live.
    #[serde(default)]
    pub version: Option<i64>,
}

fn re_import() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        // M4 PR-2 refactor: il gruppo 2 cattura TUTTI i modifiers dopo
        // "path" (es. `version=3 with tono=formale`), parser esterno
        // `parse_modifiers` estrae version e with separatamente.
        // Vincolo sintattico: `version=N` deve precedere `with ...`
        // (se entrambi presenti). Documentato in roadmap M4.
        Regex::new(r#"\{\{\s*import\s+"([^"]+)"([^}]*?)\s*\}\}"#).unwrap()
    })
}

/// M4 PR-2: parse del blob modifiers per estrarre `version=N` e
/// `with k=v, ...` da `raw` (tutto cio' che sta fra `"path"` e `}}`).
///
/// Grammatica supportata (ordine fisso, version prima di with):
/// - `` (vuoto) -> (None, [])
/// - `version=N` -> (Some(N), [])
/// - `with k=v, k2=v2` -> (None, [(k,v), (k2,v2)])
/// - `version=N with k=v` -> (Some(N), [(k,v)])
///
/// NON supportato: `with k=v version=N` (with cattura sempre fino a fine).
fn parse_modifiers(raw: &str) -> (Option<i64>, Vec<(String, String)>) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (None, vec![]);
    }

    static RE_VER: OnceLock<Regex> = OnceLock::new();
    let re_ver = RE_VER.get_or_init(|| Regex::new(r"^version\s*=\s*(\d+)\b").unwrap());

    let (version, remaining) = if let Some(cap) = re_ver.captures(trimmed) {
        let v: i64 = cap[1].parse().unwrap_or(0);
        let consumed = cap.get(0).unwrap().end();
        (Some(v), trimmed[consumed..].trim_start())
    } else {
        (None, trimmed)
    };

    // Dopo eventuale version, cerco `with <...>` come prefisso del resto.
    let vars = if let Some(rest) = remaining
        .strip_prefix("with ")
        .or_else(|| remaining.strip_prefix("with\t"))
    {
        parse_with_clause(rest.trim())
    } else {
        vec![]
    };
    (version, vars)
}

/// Parse della clausola `with k=v, k2="value with spaces", ...` in
/// coppie (key, value). Supporta:
/// - identifier=value (no spaces)
/// - identifier="quoted string" (per value con spazi/virgole)
/// - separatore virgola fra coppie (spazi opzionali)
///
/// Token invalidi (no `=`, key non identifier) sono silenziosamente
/// ignorati per resilienza forward-compat. La validazione strict
/// e' delegata al linter (futura issue IMP004).
pub fn parse_with_clause(raw: &str) -> Vec<(String, String)> {
    static RE_KV: OnceLock<Regex> = OnceLock::new();
    let re = RE_KV.get_or_init(|| {
        Regex::new(r#"([A-Za-z_]\w*)\s*=\s*(?:"((?:[^"\\]|\\.)*)"|(\S+?))(?:\s*,\s*|\s*$)"#)
            .unwrap()
    });
    re.captures_iter(raw)
        .map(|cap| {
            let key = cap.get(1).unwrap().as_str().to_string();
            // group 2 = quoted value (senza quote), group 3 = unquoted.
            let value = cap
                .get(2)
                .map(|m| m.as_str().replace("\\\"", "\""))
                .or_else(|| cap.get(3).map(|m| m.as_str().to_string()))
                .unwrap_or_default();
            (key, value)
        })
        .collect()
}

/// Estrae tutti gli import dichiarati in un body, in ordine di apparizione.
pub fn parse_imports(body: &str) -> Vec<ImportRef> {
    re_import()
        .captures_iter(body)
        .enumerate()
        .map(|(i, cap)| {
            let m = cap.get(0).unwrap();
            let path = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let modifiers_raw = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let (version, variables) = parse_modifiers(modifiers_raw);
            ImportRef {
                path,
                position: i,
                byte_start: m.start(),
                byte_end: m.end(),
                variables,
                version,
            }
        })
        .collect()
}

/// M4 PR-1: applica le variabili scopate al body importato sostituendo
/// `{{nome}}` (segnaposti non-globali) con il rispettivo value.
/// Segnaposti senza match nella mappa restano intatti per essere
/// risolti dal compilatore frontend con i valori utente.
/// I segnaposti globali (`{{globale nome}}`) NON sono toccati: la
/// loro semantica e' "valore dal DB globale", non override per import.
fn applica_variabili_scoped(body: &str, vars: &[(String, String)]) -> String {
    if vars.is_empty() {
        return body.to_string();
    }
    static RE_SEG: OnceLock<Regex> = OnceLock::new();
    let re = RE_SEG.get_or_init(|| {
        // Stesso pattern di template.ts (RE_SEGNAPOSTO) ma in Rust regex
        // syntax. Cattura nome del segnaposto solo se NON globale.
        Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap()
    });
    re.replace_all(body, |caps: &regex::Captures| {
        let nome = &caps[1];
        // Skip parola riservata "globale" (e.g. malformed `{{globale}}`
        // senza spazio dopo non lo lasciamo passare come variabile).
        if nome == "globale" {
            return caps[0].to_string();
        }
        // Last-wins per chiavi duplicate
        for (k, v) in vars.iter().rev() {
            if k == nome {
                return v.clone();
            }
        }
        caps[0].to_string()
    })
    .into_owned()
}

/// Risolve un path a un prompt_id. Strategie in ordine:
/// 1. Path stile cartella: "marketing/email/cold" → match esatto
///    Folders.Path + Title come ultimo segmento
/// 2. Path è solo il titolo: match Prompts.Title (NOCASE) per prompt
///    a livello root
/// Ritorna `None` se nessun match.
pub fn resolve_path(conn: &Connection, path: &str) -> Result<Option<String>, PapErrore> {
    let pulito = path.trim().trim_start_matches('/');
    if pulito.is_empty() {
        return Ok(None);
    }

    if let Some(idx_slash) = pulito.rfind('/') {
        let folder_path = format!("/{}", &pulito[..idx_slash]);
        let titolo = &pulito[idx_slash + 1..];
        let mut stmt = conn.prepare(
            "SELECT p.Id FROM Prompts p
             JOIN Folders f ON p.FolderId = f.Id
             WHERE p.DeletedAt IS NULL
               AND f.DeletedAt IS NULL
               AND f.Path = ?1
               AND p.Title = ?2 COLLATE NOCASE
             LIMIT 1",
        )?;
        let id: Option<String> = stmt
            .query_row(params![folder_path, titolo], |r| r.get(0))
            .ok();
        if id.is_some() {
            return Ok(id);
        }
    }

    // Fallback: cercare per titolo (qualunque cartella, root preferito)
    let mut stmt = conn.prepare(
        "SELECT Id FROM Prompts
         WHERE DeletedAt IS NULL AND Title = ?1 COLLATE NOCASE
         ORDER BY (FolderId IS NULL) DESC, UpdatedAt DESC
         LIMIT 1",
    )?;
    let id: Option<String> = stmt.query_row(params![pulito], |r| r.get(0)).ok();
    Ok(id)
}

/// Compila ricorsivamente un body, sostituendo ogni `{{import "..."}}`
/// con il body del prompt importato (anch'esso compilato).
///
/// `visitati` traccia gli id già in fase di compilazione (per cycle
/// detection); aggiunto all'ingresso, rimosso all'uscita.
fn compila_ricorsivo(
    conn: &Connection,
    prompt_id: &str,
    body: &str,
    visitati: &mut HashSet<String>,
    depth: usize,
) -> Result<String, PapErrore> {
    if depth > MAX_DEPTH {
        return Err(PapErrore::Generico(format!(
            "Profondità massima di import superata ({MAX_DEPTH}). Probabile errore di design."
        )));
    }
    if !visitati.insert(prompt_id.to_string()) {
        return Err(PapErrore::Generico(format!(
            "Ciclo di import rilevato su {prompt_id}. Impossibile compilare."
        )));
    }

    let imports = parse_imports(body);
    if imports.is_empty() {
        visitati.remove(prompt_id);
        return Ok(body.to_string());
    }

    // Strategia: scorri il body in ordine, accumula in un buffer, e
    // ad ogni import sostituisci con il body risolto + compilato.
    let mut out = String::with_capacity(body.len());
    let mut cursor = 0usize;

    for imp in &imports {
        out.push_str(&body[cursor..imp.byte_start]);

        match resolve_path(conn, &imp.path)? {
            None => {
                visitati.remove(prompt_id);
                return Err(PapErrore::Generico(format!(
                    "Import non risolto: \"{}\"",
                    imp.path
                )));
            }
            Some(id_imp) => {
                // M4 PR-2: pinning a versione storica. Se `version=N`
                // specificato, leggi body da PromptVersions invece che
                // da Prompts (snapshot stabile, non segue le edit
                // successive del child).
                let body_imp: String = match imp.version {
                    Some(v) => conn
                        .query_row(
                            "SELECT Body FROM PromptVersions
                             WHERE PromptId = ?1 AND Version = ?2",
                            params![&id_imp, v],
                            |r| r.get(0),
                        )
                        .map_err(|_| {
                            visitati.remove(prompt_id);
                            PapErrore::Generico(format!(
                                "Versione {v} di '{}' non trovata in PromptVersions",
                                imp.path
                            ))
                        })?,
                    None => conn.query_row(
                        "SELECT Body FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
                        [&id_imp],
                        |r| r.get(0),
                    )?,
                };
                // M4 PR-1: applica variabili scopate `with k=v` PRIMA
                // della ricorsione: i segnaposti `{{k}}` del body
                // importato vengono pre-sostituiti con i value passati
                // dal chiamante. Gli import nested all'interno del body
                // importato vengono poi espansi normalmente.
                let body_pre_sostituito = applica_variabili_scoped(&body_imp, &imp.variables);
                let espanso = compila_ricorsivo(
                    conn,
                    &id_imp,
                    &body_pre_sostituito,
                    visitati,
                    depth + 1,
                )?;
                out.push_str(&espanso);
            }
        }

        if out.len() > MAX_OUTPUT_BYTES {
            visitati.remove(prompt_id);
            return Err(PapErrore::Generico(format!(
                "Output di compilazione eccede {MAX_OUTPUT_BYTES} byte. Possibile bomba di import."
            )));
        }

        cursor = imp.byte_end;
    }
    out.push_str(&body[cursor..]);
    visitati.remove(prompt_id);
    Ok(out)
}

#[tauri::command]
pub fn prompt_compila(
    id: String,
    state: State<'_, VaultState>,
) -> Result<String, PapErrore> {
    state.with_conn(|conn| {
        let body: String = conn.query_row(
            "SELECT Body FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
            [&id],
            |r| r.get(0),
        )?;
        let mut visitati: HashSet<String> = HashSet::new();
        compila_ricorsivo(conn, &id, &body, &mut visitati, 0)
    })
}

/// Preview di un prompt importato per hover/Ctrl+click nell'editor.
/// v0.7.0 Step 4: risolve il `path` dichiarato nel `{{import "path"}}`
/// e ritorna i metadati essenziali per il tooltip (id, titolo, body).
/// Il `body` non è espanso (quindi può contenere `{{import}}` interni).
#[derive(Debug, serde::Serialize)]
pub struct ImportPreview {
    pub id: String,
    pub titolo: String,
    pub body: String,
}

#[tauri::command]
pub fn prompt_resolve_import_preview(
    path: String,
    state: State<'_, VaultState>,
) -> Result<Option<ImportPreview>, PapErrore> {
    state.with_conn(|conn| resolve_import_preview_pure(conn, &path))
}

/// Helper testabile senza Tauri State.
pub(crate) fn resolve_import_preview_pure(
    conn: &Connection,
    path: &str,
) -> Result<Option<ImportPreview>, PapErrore> {
    let id_opt = resolve_path(conn, path)?;
    let Some(id) = id_opt else {
        return Ok(None);
    };
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT Title, Body FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
            [&id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();
    Ok(row.map(|(titolo, body)| ImportPreview { id, titolo, body }))
}

/// Popola/aggiorna la tabella `PromptImports` per il prompt dato.
/// Chiamato da editor::prompt_crea / prompt_aggiorna come hook.
pub fn aggiorna_imports(
    conn: &Connection,
    prompt_id: &str,
    body: &str,
) -> Result<(), PapErrore> {
    conn.execute(
        "DELETE FROM PromptImports WHERE ParentPromptId = ?1",
        [prompt_id],
    )?;
    let imports = parse_imports(body);
    for imp in &imports {
        let id_risolto: Option<String> = resolve_path(conn, &imp.path)?;
        conn.execute(
            "INSERT INTO PromptImports
                (ParentPromptId, Position, ImportedPath, ImportedPromptId)
             VALUES (?1, ?2, ?3, ?4)",
            params![prompt_id, imp.position as i64, imp.path, id_risolto],
        )?;
    }
    Ok(())
}

/// Cleanup: rimuove le righe di PromptImports per un prompt eliminato.
pub fn pulisci_imports(conn: &Connection, prompt_id: &str) -> Result<(), PapErrore> {
    conn.execute(
        "DELETE FROM PromptImports WHERE ParentPromptId = ?1",
        [prompt_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use rusqlite::params;

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    fn inserisci_prompt(
        conn: &Connection,
        id: &str,
        titolo: &str,
        body: &str,
        folder_id: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                Visibility, FolderId, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, 'private', ?4, 1,
                     datetime('now'), datetime('now'))",
            params![id, titolo, body, folder_id],
        )
        .unwrap();
    }

    fn inserisci_folder(conn: &Connection, id: &str, nome: &str, path: &str) {
        conn.execute(
            "INSERT INTO Folders (Id, WorkspaceId, Name, Path)
             VALUES (?1, 'ws-personale', ?2, ?3)",
            params![id, nome, path],
        )
        .unwrap();
    }

    #[test]
    fn parse_zero_import() {
        let imps = parse_imports("Niente import qui dentro.");
        assert_eq!(imps.len(), 0);
    }

    #[test]
    fn parse_singolo_import() {
        let body = r#"Prima di {{import "ruolo-esperto"}} dopo."#;
        let imps = parse_imports(body);
        assert_eq!(imps.len(), 1);
        assert_eq!(imps[0].path, "ruolo-esperto");
        assert_eq!(imps[0].position, 0);
    }

    #[test]
    fn parse_multipli_import_in_ordine() {
        let body = r#"{{import "a"}} text {{import "b/c"}} other {{import "d"}}"#;
        let imps = parse_imports(body);
        assert_eq!(imps.len(), 3);
        assert_eq!(imps[0].path, "a");
        assert_eq!(imps[1].path, "b/c");
        assert_eq!(imps[2].path, "d");
        assert!(imps[0].byte_start < imps[1].byte_start);
        assert!(imps[1].byte_start < imps[2].byte_start);
    }

    #[test]
    fn parse_spazi_extra_intorno_a_import() {
        let body = r#"{{   import   "spazi"   }}"#;
        let imps = parse_imports(body);
        assert_eq!(imps.len(), 1);
        assert_eq!(imps[0].path, "spazi");
    }

    #[test]
    fn resolve_path_titolo_root() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Cordiale", "body", None);
        let id = resolve_path(&conn, "Cordiale").unwrap();
        assert_eq!(id, Some("prm-1".to_string()));
    }

    #[test]
    fn resolve_path_titolo_case_insensitive() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Cordiale", "body", None);
        let id = resolve_path(&conn, "cordiale").unwrap();
        assert_eq!(id, Some("prm-1".to_string()));
    }

    #[test]
    fn resolve_path_cartella_e_titolo() {
        let conn = db_test();
        inserisci_folder(&conn, "fld-mkt", "marketing", "/marketing");
        inserisci_prompt(&conn, "prm-1", "email", "body email", Some("fld-mkt"));
        let id = resolve_path(&conn, "marketing/email").unwrap();
        assert_eq!(id, Some("prm-1".to_string()));
    }

    #[test]
    fn resolve_path_inesistente_ritorna_none() {
        let conn = db_test();
        let id = resolve_path(&conn, "non-esiste").unwrap();
        assert_eq!(id, None);
    }

    #[test]
    fn compila_senza_import_ritorna_body_invariato() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "x", "Niente import qui.", None);
        let mut visitati = HashSet::new();
        let out = compila_ricorsivo(&conn, "prm-1", "Niente import qui.", &mut visitati, 0).unwrap();
        assert_eq!(out, "Niente import qui.");
    }

    #[test]
    fn compila_singolo_import_inline() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Padre", "Inizio. {{import \"figlio\"}} Fine.", None);
        inserisci_prompt(&conn, "prm-2", "figlio", "BODY-FIGLIO", None);
        let body: String = conn
            .query_row("SELECT Body FROM Prompts WHERE Id = 'prm-1'", [], |r| r.get(0))
            .unwrap();
        let mut visitati = HashSet::new();
        let out = compila_ricorsivo(&conn, "prm-1", &body, &mut visitati, 0).unwrap();
        assert_eq!(out, "Inizio. BODY-FIGLIO Fine.");
    }

    #[test]
    fn compila_import_nidificati_ricorsivo() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "A", "A: {{import \"B\"}}", None);
        inserisci_prompt(&conn, "prm-2", "B", "B: {{import \"C\"}}", None);
        inserisci_prompt(&conn, "prm-3", "C", "FOGLIA", None);
        let body: String = conn
            .query_row("SELECT Body FROM Prompts WHERE Id = 'prm-1'", [], |r| r.get(0))
            .unwrap();
        let mut visitati = HashSet::new();
        let out = compila_ricorsivo(&conn, "prm-1", &body, &mut visitati, 0).unwrap();
        assert_eq!(out, "A: B: FOGLIA");
    }

    #[test]
    fn compila_ciclo_diretto_fallisce() {
        let conn = db_test();
        // A importa A — auto-loop diretto
        inserisci_prompt(&conn, "prm-1", "A", "self: {{import \"A\"}}", None);
        let body: String = conn
            .query_row("SELECT Body FROM Prompts WHERE Id = 'prm-1'", [], |r| r.get(0))
            .unwrap();
        let mut visitati = HashSet::new();
        let r = compila_ricorsivo(&conn, "prm-1", &body, &mut visitati, 0);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("Ciclo"));
    }

    #[test]
    fn compila_ciclo_indiretto_fallisce() {
        let conn = db_test();
        // A → B → C → A
        inserisci_prompt(&conn, "prm-a", "A", "{{import \"B\"}}", None);
        inserisci_prompt(&conn, "prm-b", "B", "{{import \"C\"}}", None);
        inserisci_prompt(&conn, "prm-c", "C", "{{import \"A\"}}", None);
        let body: String = conn
            .query_row("SELECT Body FROM Prompts WHERE Id = 'prm-a'", [], |r| r.get(0))
            .unwrap();
        let mut visitati = HashSet::new();
        let r = compila_ricorsivo(&conn, "prm-a", &body, &mut visitati, 0);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("Ciclo"));
    }

    #[test]
    fn compila_import_non_risolto_fallisce() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "x", "{{import \"non-esiste\"}}", None);
        let body: String = conn
            .query_row("SELECT Body FROM Prompts WHERE Id = 'prm-1'", [], |r| r.get(0))
            .unwrap();
        let mut visitati = HashSet::new();
        let r = compila_ricorsivo(&conn, "prm-1", &body, &mut visitati, 0);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("non risolto"));
    }

    #[test]
    fn aggiorna_imports_popola_tabella() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Padre", "{{import \"X\"}} {{import \"non-esiste\"}}", None);
        inserisci_prompt(&conn, "prm-2", "X", "body x", None);

        aggiorna_imports(&conn, "prm-1", "{{import \"X\"}} {{import \"non-esiste\"}}").unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptImports WHERE ParentPromptId = 'prm-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);

        let risolto: Option<String> = conn
            .query_row(
                "SELECT ImportedPromptId FROM PromptImports WHERE ParentPromptId = 'prm-1' AND Position = 0",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(risolto, Some("prm-2".to_string()));

        let non_risolto: Option<String> = conn
            .query_row(
                "SELECT ImportedPromptId FROM PromptImports WHERE ParentPromptId = 'prm-1' AND Position = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(non_risolto, None);
    }

    #[test]
    fn aggiorna_imports_idempotente_su_riinvocazione() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "P", "body", None);
        aggiorna_imports(&conn, "prm-1", "{{import \"X\"}}").unwrap();
        aggiorna_imports(&conn, "prm-1", "").unwrap(); // body senza più import → tabella ripulita
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptImports WHERE ParentPromptId = 'prm-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn pulisci_imports_per_eliminazione() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "P", "body", None);
        aggiorna_imports(&conn, "prm-1", "{{import \"X\"}}").unwrap();
        pulisci_imports(&conn, "prm-1").unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptImports WHERE ParentPromptId = 'prm-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 0);
    }

    // ─── M4 PR-1: sintassi `with k=v` ──────────────────────────────

    #[test]
    fn parse_with_clause_singola_kv() {
        let v = parse_with_clause("tono=formale");
        assert_eq!(v, vec![("tono".to_string(), "formale".to_string())]);
    }

    #[test]
    fn parse_with_clause_multipli_kv_separati_da_virgola() {
        let v = parse_with_clause("tono=formale, lingua=it, max=10");
        assert_eq!(
            v,
            vec![
                ("tono".to_string(), "formale".to_string()),
                ("lingua".to_string(), "it".to_string()),
                ("max".to_string(), "10".to_string()),
            ]
        );
    }

    #[test]
    fn parse_with_clause_value_quotato_con_spazi() {
        let v = parse_with_clause(r#"nome="Mario Rossi", ruolo=admin"#);
        assert_eq!(
            v,
            vec![
                ("nome".to_string(), "Mario Rossi".to_string()),
                ("ruolo".to_string(), "admin".to_string()),
            ]
        );
    }

    #[test]
    fn parse_with_clause_value_quotato_con_escape() {
        let v = parse_with_clause(r#"msg="Disse \"ciao\" a tutti""#);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, "msg");
        assert_eq!(v[0].1, r#"Disse "ciao" a tutti"#);
    }

    #[test]
    fn parse_imports_estrae_variables() {
        let body = r#"Prima {{import "x" with tono=formale, max=5}} dopo."#;
        let imps = parse_imports(body);
        assert_eq!(imps.len(), 1);
        assert_eq!(imps[0].path, "x");
        assert_eq!(
            imps[0].variables,
            vec![
                ("tono".to_string(), "formale".to_string()),
                ("max".to_string(), "5".to_string()),
            ]
        );
    }

    #[test]
    fn parse_imports_senza_with_ha_variables_vuoto() {
        let body = r#"{{import "x"}}"#;
        let imps = parse_imports(body);
        assert_eq!(imps.len(), 1);
        assert!(imps[0].variables.is_empty());
    }

    #[test]
    fn applica_variabili_sostituisce_segnaposti_match() {
        let out = applica_variabili_scoped(
            "Ciao {{nome}}, sei {{ruolo}}.",
            &[
                ("nome".to_string(), "Mario".to_string()),
                ("ruolo".to_string(), "admin".to_string()),
            ],
        );
        assert_eq!(out, "Ciao Mario, sei admin.");
    }

    #[test]
    fn applica_variabili_preserva_segnaposti_senza_match() {
        let out = applica_variabili_scoped(
            "Ciao {{nome}}, eta {{eta}}.",
            &[("nome".to_string(), "Mario".to_string())],
        );
        // eta non e' nelle variables -> resta inalterato per
        // compilazione frontend con valori utente
        assert_eq!(out, "Ciao Mario, eta {{eta}}.");
    }

    #[test]
    fn applica_variabili_non_tocca_segnaposti_globali() {
        let out = applica_variabili_scoped(
            "Autore {{globale autore}}, nome {{nome}}.",
            &[("nome".to_string(), "X".to_string())],
        );
        // globale resta intatto
        assert_eq!(out, "Autore {{globale autore}}, nome X.");
    }

    #[test]
    fn applica_variabili_chiavi_duplicate_last_wins() {
        let out = applica_variabili_scoped(
            "X={{k}}",
            &[
                ("k".to_string(), "primo".to_string()),
                ("k".to_string(), "secondo".to_string()),
            ],
        );
        assert_eq!(out, "X=secondo");
    }

    #[test]
    fn compila_con_import_with_vars_sostituisce_inline() {
        let conn = db_test();
        // child con segnaposti che verranno sovrascritti dal `with`
        inserisci_prompt(
            &conn,
            "prm-child",
            "Saluto",
            "Ciao {{nome}}, ruolo: {{ruolo}}",
            None,
        );
        // parent importa con variabili scopate (path = titolo root)
        let parent_body = r#"{{import "Saluto" with nome="Mario", ruolo=admin}}"#;
        inserisci_prompt(&conn, "prm-parent", "Padre", parent_body, None);

        let mut visitati = HashSet::new();
        let out = compila_ricorsivo(&conn, "prm-parent", parent_body, &mut visitati, 0)
            .unwrap();
        assert_eq!(out, "Ciao Mario, ruolo: admin");
    }

    #[test]
    fn compila_con_import_senza_with_lascia_segnaposti() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-child", "Saluto", "Ciao {{nome}}", None);
        let parent_body = r#"{{import "Saluto"}}"#;
        inserisci_prompt(&conn, "prm-parent", "Padre", parent_body, None);

        let mut visitati = HashSet::new();
        let out = compila_ricorsivo(&conn, "prm-parent", parent_body, &mut visitati, 0)
            .unwrap();
        // segnaposto resta -> sara' risolto dal frontend con valori utente
        assert_eq!(out, "Ciao {{nome}}");
    }

    // ─── M4 PR-2: sintassi `version=N` ─────────────────────────────

    #[test]
    fn parse_modifiers_vuoto_ritorna_none_vec_vuoto() {
        let (v, vars) = parse_modifiers("");
        assert!(v.is_none());
        assert!(vars.is_empty());
    }

    #[test]
    fn parse_modifiers_solo_version() {
        let (v, vars) = parse_modifiers("version=3");
        assert_eq!(v, Some(3));
        assert!(vars.is_empty());
    }

    #[test]
    fn parse_modifiers_solo_with() {
        let (v, vars) = parse_modifiers("with tono=formale");
        assert!(v.is_none());
        assert_eq!(vars, vec![("tono".to_string(), "formale".to_string())]);
    }

    #[test]
    fn parse_modifiers_version_e_with_combinati() {
        let (v, vars) = parse_modifiers(r#"version=5 with tono=formale, nome="Mario""#);
        assert_eq!(v, Some(5));
        assert_eq!(
            vars,
            vec![
                ("tono".to_string(), "formale".to_string()),
                ("nome".to_string(), "Mario".to_string()),
            ]
        );
    }

    #[test]
    fn parse_imports_estrae_version() {
        let body = r#"{{import "x" version=7}}"#;
        let imps = parse_imports(body);
        assert_eq!(imps.len(), 1);
        assert_eq!(imps[0].version, Some(7));
        assert!(imps[0].variables.is_empty());
    }

    #[test]
    fn parse_imports_version_e_with() {
        let body = r#"{{import "x" version=2 with k=v}}"#;
        let imps = parse_imports(body);
        assert_eq!(imps.len(), 1);
        assert_eq!(imps[0].version, Some(2));
        assert_eq!(imps[0].variables, vec![("k".to_string(), "v".to_string())]);
    }

    #[test]
    fn compila_con_import_version_legge_da_promptversions() {
        let conn = db_test();
        // crea child con body originale "v1" + snapshot versione 1
        inserisci_prompt(&conn, "prm-child", "Saluto", "Versione 1", None);
        crate::versioning::snapshot_versione(&conn, "prm-child", "usr-locale").unwrap();
        // simula edit: aggiorna body a "v2" + nuovo snapshot versione 2
        conn.execute(
            "UPDATE Prompts SET Body = 'Versione 2', Version = 2 WHERE Id = 'prm-child'",
            [],
        )
        .unwrap();
        crate::versioning::snapshot_versione(&conn, "prm-child", "usr-locale").unwrap();

        // parent importa con pinning a versione 1
        let parent_body = r#"{{import "Saluto" version=1}}"#;
        inserisci_prompt(&conn, "prm-parent", "Padre", parent_body, None);

        let mut visitati = HashSet::new();
        let out = compila_ricorsivo(&conn, "prm-parent", parent_body, &mut visitati, 0)
            .unwrap();
        // Deve leggere la versione 1 storica, NON la 2 corrente
        assert_eq!(out, "Versione 1");
    }

    #[test]
    fn compila_con_import_version_corrente_default() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-child", "Saluto", "Live body", None);
        crate::versioning::snapshot_versione(&conn, "prm-child", "usr-locale").unwrap();
        // Edit
        conn.execute(
            "UPDATE Prompts SET Body = 'Body modificato', Version = 2 WHERE Id = 'prm-child'",
            [],
        )
        .unwrap();

        // Import senza version -> deve leggere body live (corrente)
        let parent_body = r#"{{import "Saluto"}}"#;
        inserisci_prompt(&conn, "prm-parent", "Padre", parent_body, None);

        let mut visitati = HashSet::new();
        let out = compila_ricorsivo(&conn, "prm-parent", parent_body, &mut visitati, 0)
            .unwrap();
        assert_eq!(out, "Body modificato");
    }

    #[test]
    fn compila_con_import_version_inesistente_errore_chiaro() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-child", "Saluto", "Body", None);
        crate::versioning::snapshot_versione(&conn, "prm-child", "usr-locale").unwrap();

        // Import a version=99 che non esiste
        let parent_body = r#"{{import "Saluto" version=99}}"#;
        inserisci_prompt(&conn, "prm-parent", "Padre", parent_body, None);

        let mut visitati = HashSet::new();
        let err = compila_ricorsivo(&conn, "prm-parent", parent_body, &mut visitati, 0)
            .unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Versione 99"), "errore poco descrittivo: {msg}");
        assert!(msg.contains("Saluto"));
    }

    #[test]
    fn compila_con_import_version_e_with_combinati() {
        let conn = db_test();
        // child v1 con segnaposti
        inserisci_prompt(
            &conn,
            "prm-child",
            "Tpl",
            "Ciao {{nome}}, v1",
            None,
        );
        crate::versioning::snapshot_versione(&conn, "prm-child", "usr-locale").unwrap();
        // edit: v2 con body diverso
        conn.execute(
            "UPDATE Prompts SET Body = 'Ciao {{nome}}, v2', Version = 2 WHERE Id = 'prm-child'",
            [],
        )
        .unwrap();
        crate::versioning::snapshot_versione(&conn, "prm-child", "usr-locale").unwrap();

        let parent_body = r#"{{import "Tpl" version=1 with nome="Mario"}}"#;
        inserisci_prompt(&conn, "prm-parent", "Padre", parent_body, None);

        let mut visitati = HashSet::new();
        let out = compila_ricorsivo(&conn, "prm-parent", parent_body, &mut visitati, 0)
            .unwrap();
        // Pinning a v1 (body "Ciao {{nome}}, v1") + with sostituisce nome
        assert_eq!(out, "Ciao Mario, v1");
    }
}
