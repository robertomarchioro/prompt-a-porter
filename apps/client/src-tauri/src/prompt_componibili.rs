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
        // `\d+` può eccedere i64 (es. version=99999999999999999999): in
        // overflow usiamo i64::MAX così la risoluzione fallisce con un
        // chiaro "versione non trovata" invece di mappare silenziosamente a 0
        // (che sembrerebbe una versione valida).
        let v: i64 = cap[1].parse().unwrap_or(i64::MAX);
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
                // Verifica il cap PRIMA di accodare l'espansione del child:
                // evita di gonfiare transitoriamente `out` fino a
                // MAX_OUTPUT_BYTES + espanso.len() (anti bomba di import).
                if out.len() + espanso.len() > MAX_OUTPUT_BYTES {
                    visitati.remove(prompt_id);
                    return Err(PapErrore::Generico(format!(
                        "Output di compilazione eccede {MAX_OUTPUT_BYTES} byte. Possibile bomba di import."
                    )));
                }
                out.push_str(&espanso);
            }
        }

        cursor = imp.byte_end;
    }
    // La coda dopo l'ultimo import va conteggiata nel cap: senza questo
    // check un body con coda molto grande dopo un singolo import aggira il
    // limite, perché l'append era post-loop e non verificato.
    if out.len() + (body.len() - cursor) > MAX_OUTPUT_BYTES {
        visitati.remove(prompt_id);
        return Err(PapErrore::Generico(format!(
            "Output di compilazione eccede {MAX_OUTPUT_BYTES} byte. Possibile bomba di import."
        )));
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

/// M5 PR-2: variante di `prompt_compila` che accetta il body inline
/// invece di leggerlo dal DB. Usata da AnteprimaTab per live preview
/// dell'espansione import senza attendere l'autosave del body editato.
///
/// `prompt_id` (opzionale) serve come "self id" per cycle detection:
/// se l'utente scrive `{{import "self-title"}}` nel proprio prompt e
/// passa il proprio id, il loop viene rilevato come ciclo. Se omesso,
/// usiamo un id sintetico `__inline_preview__`.
#[tauri::command]
pub fn prompt_compila_inline(
    body: String,
    prompt_id: Option<String>,
    state: State<'_, VaultState>,
) -> Result<String, PapErrore> {
    state.with_conn(|conn| {
        let id = prompt_id.as_deref().unwrap_or("__inline_preview__");
        let mut visitati: HashSet<String> = HashSet::new();
        compila_ricorsivo(conn, id, &body, &mut visitati, 0)
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

// ─── #303: dipendenze inverse (chi importa un dato prompt) ───────────

/// Un prompt vivo che importa un altro prompt via `{{import}}`.
#[derive(Debug, Clone, Serialize)]
pub struct Dipendente {
    pub id: String,
    pub titolo: String,
}

/// Lista dei prompt VIVI che importano `imported_id`. Fonda il warning di
/// cancellazione (#303: "questo prompt è importato da N altri") e l'avviso
/// di ripristino del cestino (#302). Esclude i genitori cancellati e
/// deduplica (un genitore che importa più volte conta una volta sola).
pub fn dipendenti_pure(
    conn: &Connection,
    imported_id: &str,
) -> Result<Vec<Dipendente>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT pr.Id, pr.Title
           FROM PromptImports pi
           JOIN Prompts pr ON pr.Id = pi.ParentPromptId
          WHERE pi.ImportedPromptId = ?1 AND pr.DeletedAt IS NULL
          ORDER BY pr.Title COLLATE NOCASE ASC",
    )?;
    let righe = stmt
        .query_map([imported_id], |r| {
            Ok(Dipendente {
                id: r.get(0)?,
                titolo: r.get(1)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(righe)
}

#[tauri::command]
pub fn prompt_dipendenti(
    id: String,
    state: State<'_, VaultState>,
) -> Result<Vec<Dipendente>, PapErrore> {
    state.with_conn(|conn| dipendenti_pure(conn, &id))
}

/// Rimuove ogni `{{import}}` che punta a `target_id` dal body di tutti i
/// prompt vivi che lo importano, in un'unica transazione. Prima di toccare
/// ciascun prompt crea uno snapshot di versione (la modifica è quindi
/// reversibile dalla cronologia). Ritorna il numero di prompt modificati.
///
/// Pensato per il flusso di cancellazione (#303): si chiama PRIMA di
/// `prompt_elimina(target_id)`, così non resta mai un istante con import
/// rotti. I token vengono rimossi per intero (incluso `with`/`version`) via
/// range byte, dal fondo verso l'inizio per non invalidare gli offset.
///
/// Nota: gli embedding dei prompt toccati restano "stale" finché il prossimo
/// salvataggio/backfill non li ricalcola (questo comando non ha accesso a
/// `EmbeddingsState`). La semantica testuale è comunque già aggiornata.
pub fn import_rimuovi_da_dipendenti_pure(
    conn: &Connection,
    target_id: &str,
) -> Result<usize, PapErrore> {
    let dipendenti = dipendenti_pure(conn, target_id)?;
    if dipendenti.is_empty() {
        return Ok(0);
    }

    let tx = conn.unchecked_transaction()?;
    let mut modificati = 0usize;
    for dip in &dipendenti {
        let body: String = tx.query_row(
            "SELECT Body FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
            [&dip.id],
            |r| r.get(0),
        )?;

        // Raccoglie i range dei soli token che risolvono a target_id.
        let mut ranges: Vec<(usize, usize)> = Vec::new();
        for imp in parse_imports(&body) {
            if resolve_path(&tx, &imp.path)?.as_deref() == Some(target_id) {
                ranges.push((imp.byte_start, imp.byte_end));
            }
        }
        if ranges.is_empty() {
            continue; // riga PromptImports stale: niente da togliere qui
        }

        // Snapshot dello stato corrente (pre-rimozione) → reversibile.
        crate::versioning::snapshot_versione(&tx, &dip.id, "usr-locale")?;

        // Rimuove dal fondo per mantenere validi gli offset precedenti.
        ranges.sort_by(|a, b| b.0.cmp(&a.0));
        let mut nuovo = body.clone();
        for (start, end) in ranges {
            nuovo.replace_range(start..end, "");
        }
        let nuovo = nuovo.trim().to_string();

        tx.execute(
            "UPDATE Prompts
             SET Body = ?1, Version = Version + 1, UpdatedAt = datetime('now'),
                 UpdatedByUserId = 'usr-locale'
             WHERE Id = ?2 AND DeletedAt IS NULL",
            params![nuovo, dip.id],
        )?;
        aggiorna_imports(&tx, &dip.id, &nuovo)?;
        crate::audit::registra(
            &tx,
            "prompt.import_rimosso",
            "Prompt",
            &dip.id,
            Some(target_id),
        );
        modificati += 1;
    }

    // FTS ricostruito una volta a fine batch (i body sono cambiati).
    crate::editor::ricostruisci_fts(&tx)?;
    tx.commit()?;
    Ok(modificati)
}

#[tauri::command]
pub fn import_rimuovi_da_dipendenti(
    target_id: String,
    state: State<'_, VaultState>,
) -> Result<usize, PapErrore> {
    state.with_conn(|conn| import_rimuovi_da_dipendenti_pure(conn, &target_id))
}

// ─── M4 PR-3: Intellisense autocomplete ────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct PromptSuggerito {
    pub id: String,
    pub titolo: String,
    /// Path della cartella (es. "/marketing/email") o "" se in root.
    /// Frontend lo usa come `detail` nel popover di completamento.
    pub folder_path: String,
}

/// Logica pura: cerca prompt il cui Title matcha il prefisso (case-insensitive),
/// ordinati per popolarita' (UseCount DESC, UpdatedAt DESC).
/// `escludi_id`: opzionale, tipicamente l'id del prompt corrente
/// nell'editor per evitare suggerimento self-import.
pub(crate) fn suggest_pure(
    conn: &Connection,
    prefix: &str,
    limit: usize,
    escludi_id: Option<&str>,
) -> Result<Vec<PromptSuggerito>, PapErrore> {
    let like_param = format!("{}%", prefix.trim());
    let escludi = escludi_id.unwrap_or("");
    let limit_i64 = limit as i64;
    let mut stmt = conn.prepare(
        "SELECT p.Id, p.Title, COALESCE(f.Path, '')
         FROM Prompts p
         LEFT JOIN Folders f ON f.Id = p.FolderId AND f.DeletedAt IS NULL
         WHERE p.DeletedAt IS NULL
           AND p.Title LIKE ?1 COLLATE NOCASE
           AND (?2 = '' OR p.Id != ?2)
         ORDER BY p.UseCount DESC, p.UpdatedAt DESC
         LIMIT ?3",
    )?;
    let rows: Vec<PromptSuggerito> = stmt
        .query_map(params![like_param, escludi, limit_i64], |r| {
            Ok(PromptSuggerito {
                id: r.get(0)?,
                titolo: r.get(1)?,
                folder_path: r.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

#[tauri::command]
pub fn prompt_suggest_intellisense(
    prefix: String,
    limit: Option<usize>,
    escludi_id: Option<String>,
    state: State<'_, VaultState>,
) -> Result<Vec<PromptSuggerito>, PapErrore> {
    let lim = limit.unwrap_or(20).clamp(1, 100);
    state.with_conn(|conn| suggest_pure(conn, &prefix, lim, escludi_id.as_deref()))
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
    fn cap_output_coda_grande_dopo_import_fallisce() {
        // Regressione: una coda > MAX_OUTPUT_BYTES dopo l'ultimo import
        // deve far fallire la compilazione. Prima del fix l'append della
        // coda era post-loop e non verificato → bypass del cap.
        let conn = db_test();
        inserisci_prompt(&conn, "prm-child", "child", "piccolo", None);
        let coda = "x".repeat(MAX_OUTPUT_BYTES + 10);
        let body = format!("{}{}", r#"{{import "child"}}"#, coda);
        let mut visitati = HashSet::new();
        let res = compila_ricorsivo(&conn, "prm-main", &body, &mut visitati, 0);
        assert!(res.is_err(), "coda oltre il cap deve fallire");
        assert!(res.unwrap_err().to_string().contains("eccede"));
    }

    #[test]
    fn cap_output_somma_espansioni_fallisce() {
        // Regressione: due import dello stesso prompt da ~600KB sommano a
        // 1.2MB > cap. Il check ora avviene PRIMA del push dell'espansione.
        let conn = db_test();
        let grande = "y".repeat(600_000);
        inserisci_prompt(&conn, "prm-big", "big", &grande, None);
        let body = r#"{{import "big"}} {{import "big"}}"#.to_string();
        let mut visitati = HashSet::new();
        let res = compila_ricorsivo(&conn, "prm-main2", &body, &mut visitati, 0);
        assert!(res.is_err(), "somma espansioni oltre il cap deve fallire");
        assert!(res.unwrap_err().to_string().contains("eccede"));
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

    // ─── #303: dipendenti_pure ─────────────────────────────────────

    #[test]
    fn dipendenti_vuoto_se_nessuno_importa() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-base", "Base", "corpo", None);
        assert!(dipendenti_pure(&conn, "prm-base").unwrap().is_empty());
    }

    #[test]
    fn dipendenti_elenca_genitori_vivi_ordinati() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-base", "Base", "corpo", None);
        inserisci_prompt(&conn, "prm-b", "Bravo", r#"{{import "Base"}}"#, None);
        inserisci_prompt(&conn, "prm-a", "Alfa", r#"{{import "Base"}}"#, None);
        aggiorna_imports(&conn, "prm-b", r#"{{import "Base"}}"#).unwrap();
        aggiorna_imports(&conn, "prm-a", r#"{{import "Base"}}"#).unwrap();
        let deps = dipendenti_pure(&conn, "prm-base").unwrap();
        assert_eq!(deps.len(), 2);
        // ORDER BY Title COLLATE NOCASE → Alfa prima di Bravo
        assert_eq!(deps[0].titolo, "Alfa");
        assert_eq!(deps[1].titolo, "Bravo");
    }

    #[test]
    fn dipendenti_esclude_genitori_cancellati() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-base", "Base", "corpo", None);
        inserisci_prompt(&conn, "prm-a", "Alfa", r#"{{import "Base"}}"#, None);
        aggiorna_imports(&conn, "prm-a", r#"{{import "Base"}}"#).unwrap();
        conn.execute(
            "UPDATE Prompts SET DeletedAt = datetime('now') WHERE Id = 'prm-a'",
            [],
        )
        .unwrap();
        assert!(
            dipendenti_pure(&conn, "prm-base").unwrap().is_empty(),
            "un genitore cancellato non è una dipendenza viva"
        );
    }

    #[test]
    fn dipendenti_dedup_genitore_con_doppio_import() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-base", "Base", "corpo", None);
        let body = r#"{{import "Base"}} e ancora {{import "Base"}}"#;
        inserisci_prompt(&conn, "prm-a", "Alfa", body, None);
        aggiorna_imports(&conn, "prm-a", body).unwrap();
        let deps = dipendenti_pure(&conn, "prm-base").unwrap();
        assert_eq!(deps.len(), 1, "stesso genitore con 2 import → una riga");
        assert_eq!(deps[0].id, "prm-a");
    }

    // ─── #303: import_rimuovi_da_dipendenti_pure ───────────────────

    fn versioni_di(conn: &Connection, id: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM PromptVersions WHERE PromptId = ?1",
            [id],
            |r| r.get(0),
        )
        .unwrap()
    }

    #[test]
    fn rimuovi_import_zero_se_nessun_dipendente() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-base", "Base", "corpo", None);
        assert_eq!(
            import_rimuovi_da_dipendenti_pure(&conn, "prm-base").unwrap(),
            0
        );
    }

    #[test]
    fn rimuovi_import_semplice_aggiorna_body_e_tabella() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-base", "Base", "corpo", None);
        let body = r#"prima {{import "Base"}} dopo"#;
        inserisci_prompt(&conn, "prm-a", "Alfa", body, None);
        aggiorna_imports(&conn, "prm-a", body).unwrap();

        let n = import_rimuovi_da_dipendenti_pure(&conn, "prm-base").unwrap();
        assert_eq!(n, 1);

        let nuovo: String = conn
            .query_row("SELECT Body FROM Prompts WHERE Id = 'prm-a'", [], |r| r.get(0))
            .unwrap();
        assert!(!nuovo.contains("import"), "il token deve sparire: {nuovo:?}");

        // PromptImports del genitore ripulita.
        let imp: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptImports WHERE ParentPromptId = 'prm-a'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(imp, 0);
        // Snapshot pre-rimozione creato (reversibile).
        assert!(versioni_di(&conn, "prm-a") >= 1);
    }

    #[test]
    fn rimuovi_import_con_modifiers_toglie_intero_token() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-base", "Base", "corpo", None);
        let body = r#"x {{import "Base" with tono=formale}} y"#;
        inserisci_prompt(&conn, "prm-a", "Alfa", body, None);
        aggiorna_imports(&conn, "prm-a", body).unwrap();

        import_rimuovi_da_dipendenti_pure(&conn, "prm-base").unwrap();
        let nuovo: String = conn
            .query_row("SELECT Body FROM Prompts WHERE Id = 'prm-a'", [], |r| r.get(0))
            .unwrap();
        assert!(!nuovo.contains("import"));
        assert!(!nuovo.contains("with"), "i modifiers spariscono col token");
    }

    #[test]
    fn rimuovi_import_multiplo_nello_stesso_body() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-base", "Base", "corpo", None);
        let body = r#"{{import "Base"}} mezzo {{import "Base"}}"#;
        inserisci_prompt(&conn, "prm-a", "Alfa", body, None);
        aggiorna_imports(&conn, "prm-a", body).unwrap();

        import_rimuovi_da_dipendenti_pure(&conn, "prm-base").unwrap();
        let nuovo: String = conn
            .query_row("SELECT Body FROM Prompts WHERE Id = 'prm-a'", [], |r| r.get(0))
            .unwrap();
        assert!(!nuovo.contains("import"), "tutte le occorrenze rimosse: {nuovo:?}");
    }

    #[test]
    fn rimuovi_import_due_genitori_conta_due() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-base", "Base", "corpo", None);
        inserisci_prompt(&conn, "prm-a", "Alfa", r#"{{import "Base"}}"#, None);
        inserisci_prompt(&conn, "prm-b", "Bravo", r#"{{import "Base"}}"#, None);
        aggiorna_imports(&conn, "prm-a", r#"{{import "Base"}}"#).unwrap();
        aggiorna_imports(&conn, "prm-b", r#"{{import "Base"}}"#).unwrap();

        assert_eq!(
            import_rimuovi_da_dipendenti_pure(&conn, "prm-base").unwrap(),
            2
        );
        assert!(dipendenti_pure(&conn, "prm-base").unwrap().is_empty());
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

    // ─── M4 PR-3: intellisense suggest_pure ────────────────────────

    #[test]
    fn suggest_filtra_per_prefisso_case_insensitive() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Marketing email", "body", None);
        inserisci_prompt(&conn, "prm-2", "marketing social", "body", None);
        inserisci_prompt(&conn, "prm-3", "Vendita prodotto", "body", None);

        let r = suggest_pure(&conn, "mark", 10, None).unwrap();
        assert_eq!(r.len(), 2, "atteso 2 match per 'mark', got {}", r.len());
        let titoli: Vec<&str> = r.iter().map(|p| p.titolo.as_str()).collect();
        assert!(titoli.contains(&"Marketing email"));
        assert!(titoli.contains(&"marketing social"));
    }

    #[test]
    fn suggest_ordina_per_use_count_desc() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Alfa A", "b", None);
        inserisci_prompt(&conn, "prm-2", "Alfa B", "b", None);
        conn.execute(
            "UPDATE Prompts SET UseCount = 5 WHERE Id = 'prm-2'",
            [],
        )
        .unwrap();

        let r = suggest_pure(&conn, "Alfa", 10, None).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].titolo, "Alfa B"); // UseCount maggiore prima
        assert_eq!(r[1].titolo, "Alfa A");
    }

    #[test]
    fn suggest_rispetta_limit() {
        let conn = db_test();
        for i in 0..5 {
            inserisci_prompt(&conn, &format!("p{i}"), &format!("X{i}"), "b", None);
        }
        let r = suggest_pure(&conn, "X", 3, None).unwrap();
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn suggest_esclude_id_specificato() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-self", "Self prompt", "b", None);
        inserisci_prompt(&conn, "prm-other", "Self other", "b", None);

        let r = suggest_pure(&conn, "Self", 10, Some("prm-self")).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "prm-other");
    }

    #[test]
    fn suggest_include_folder_path() {
        let conn = db_test();
        inserisci_folder(&conn, "fld-mkt", "Marketing", "/marketing");
        inserisci_prompt(&conn, "prm-1", "Email cold", "b", Some("fld-mkt"));

        let r = suggest_pure(&conn, "Email", 10, None).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].folder_path, "/marketing");
    }

    #[test]
    fn suggest_prompt_root_ha_folder_path_vuoto() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Senza cartella", "b", None);

        let r = suggest_pure(&conn, "Senza", 10, None).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].folder_path, "");
    }

    #[test]
    fn suggest_esclude_prompt_eliminati() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-live", "Live", "b", None);
        inserisci_prompt(&conn, "prm-del", "Live deleted", "b", None);
        conn.execute(
            "UPDATE Prompts SET DeletedAt = datetime('now') WHERE Id = 'prm-del'",
            [],
        )
        .unwrap();

        let r = suggest_pure(&conn, "Live", 10, None).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "prm-live");
    }

    // ─── M5 PR-2: compila_inline (body fornito esternamente) ───────

    #[test]
    fn compila_inline_espande_import_senza_db_lookup_del_parent() {
        let conn = db_test();
        // Solo child registrato; il "parent" e' il body inline passato.
        inserisci_prompt(&conn, "prm-child", "Saluto", "Ciao mondo", None);

        // Simulazione AnteprimaTab: body editato non ancora salvato
        let body_inline = r#"Premessa: {{import "Saluto"}}. Fine."#;
        let mut visitati = HashSet::new();
        let out = compila_ricorsivo(
            &conn,
            "__inline_preview__",
            body_inline,
            &mut visitati,
            0,
        )
        .unwrap();
        assert_eq!(out, "Premessa: Ciao mondo. Fine.");
    }

    #[test]
    fn compila_inline_cycle_detection_con_self_id() {
        let conn = db_test();
        // Prompt esistente "Padre" che importa se stesso (caso patologico
        // ma possibile durante editing). Inline simulate: l'utente edita
        // il body del prompt "Padre" aggiungendo {{import "Padre"}}.
        inserisci_prompt(&conn, "prm-padre", "Padre", "vecchio", None);

        let body_inline = r#"Test {{import "Padre"}} ciao"#;
        let mut visitati = HashSet::new();
        // Usiamo "prm-padre" come self id -> cycle detection lo rifiuta.
        let err = compila_ricorsivo(
            &conn,
            "prm-padre",
            body_inline,
            &mut visitati,
            0,
        )
        .unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Ciclo"), "atteso errore ciclo, got: {msg}");
    }

    #[test]
    fn compila_inline_supporta_with_e_version() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-tpl", "Tpl", "Ciao {{nome}}", None);
        crate::versioning::snapshot_versione(&conn, "prm-tpl", "usr-locale")
            .unwrap();
        // edit child -> v2
        conn.execute(
            "UPDATE Prompts SET Body = 'Body v2', Version = 2 WHERE Id = 'prm-tpl'",
            [],
        )
        .unwrap();
        crate::versioning::snapshot_versione(&conn, "prm-tpl", "usr-locale")
            .unwrap();

        let body_inline =
            r#"Risultato: {{import "Tpl" version=1 with nome=Mario}}"#;
        let mut visitati = HashSet::new();
        let out = compila_ricorsivo(
            &conn,
            "__inline_preview__",
            body_inline,
            &mut visitati,
            0,
        )
        .unwrap();
        assert_eq!(out, "Risultato: Ciao Mario");
    }

    // ─── Issue #293: compila_inline espande import nel body del modal ──
    //
    // Regressione: CompilaModal usava `dettaglio.body` grezzo (con
    // `{{import …}}` non espansi) sia per estrarre segnaposti sia per
    // produrre l'output finale. Il fix chiama `prompt_compila_inline`
    // sul body raw prima di derivare segnaposti e output, così gli import
    // vengono risolti esattamente come in AnteprimaTab.
    #[test]
    fn compila_inline_espande_import_per_modal_compila() {
        let conn = db_test();
        // Fragment importabile: contiene un segnaposto che l'utente
        // compilerà nella modale.
        inserisci_prompt(
            &conn,
            "prm-fragment",
            "IntroFragmento",
            "Sei un esperto di {{dominio}}.",
            None,
        );
        // Prompt principale con import + segnaposto locale.
        let body_raw = r#"{{import "IntroFragmento"}} Rispondi a: {{domanda}}"#;
        inserisci_prompt(&conn, "prm-main", "Principale", body_raw, None);

        // Simula la chiamata che CompilaModal.svelte ora esegue via
        // `prompt_compila_inline` prima di derivare segnaposti e output.
        let mut visitati = HashSet::new();
        let espanso =
            compila_ricorsivo(&conn, "prm-main", body_raw, &mut visitati, 0)
                .unwrap();

        // Il body espanso deve contenere il testo del fragment, non
        // la direttiva `{{import …}}` grezza.
        assert!(
            espanso.contains("Sei un esperto di {{dominio}}."),
            "body espanso deve contenere il testo del fragment: {espanso}"
        );
        assert!(
            !espanso.contains("{{import"),
            "body espanso non deve contenere direttive import non risolte: {espanso}"
        );
        // Il segnaposto {{domanda}} del prompt padre deve essere preservato
        // (verrà compilato dal frontend con i valori dell'utente).
        assert!(
            espanso.contains("{{domanda}}"),
            "segnaposto locale deve essere preservato: {espanso}"
        );
    }
}
