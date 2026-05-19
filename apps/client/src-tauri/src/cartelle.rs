// Gestione cartelle (organizzazione gerarchica dei prompt).
//
// I tag sono trasversali (un prompt → molti tag); le cartelle sono ubicazione
// canonica (un prompt → 1 cartella, opzionale, NULL = root). Modelli ortogonali.
//
// `Path` è denormalizzato per query efficienti tipo "tutti i prompt sotto
// /marketing": basta `WHERE FolderId IN (SELECT Id FROM Folders WHERE
// Path = '/marketing' OR Path LIKE '/marketing/%')`.
//
// Operazioni di sposta/rinomina richiedono recompute del path su tutti i
// discendenti, fatto in transazione.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

const WORKSPACE_ID: &str = "ws-personale";

#[derive(Debug, Serialize)]
pub struct Cartella {
    pub id: String,
    pub nome: String,
    pub path: String,
    pub parent_folder_id: Option<String>,
    pub conteggio_prompt: i64,
    pub creato_a: String,
    pub aggiornato_a: String,
}

#[derive(Debug, Deserialize)]
pub struct NuovaCartella {
    pub nome: String,
    #[serde(default)]
    pub parent_folder_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RinominaCartella {
    pub id: String,
    pub nuovo_nome: String,
}

#[derive(Debug, Deserialize)]
pub struct SpostaCartella {
    pub id: String,
    /// `None` → sposta a root del workspace.
    #[serde(default)]
    pub nuovo_parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpostaPrompt {
    pub prompt_id: String,
    /// `None` → toglie la cartella (prompt va a root).
    #[serde(default)]
    pub folder_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RiordinaFolder {
    pub id: String,
    pub new_sort: i64,
}

#[derive(Debug, Deserialize)]
pub struct RiordinaPrompt {
    pub prompt_id: String,
    pub new_sort: i64,
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
        "fld-{:012x}{:02x}{:02x}{:02x}{:02x}",
        ts, rnd[0], rnd[1], rnd[2], rnd[3]
    )
}

fn nome_valido(nome: &str) -> Result<String, PapErrore> {
    let trimmato = nome.trim();
    if trimmato.is_empty() {
        return Err(PapErrore::Generico("Nome cartella vuoto".into()));
    }
    if trimmato.contains('/') {
        return Err(PapErrore::Generico(
            "Il nome non può contenere '/'".into(),
        ));
    }
    if trimmato.len() > 100 {
        return Err(PapErrore::Generico("Nome troppo lungo (max 100 caratteri)".into()));
    }
    Ok(trimmato.to_string())
}

fn calcola_path(conn: &Connection, parent_id: Option<&str>, nome: &str) -> Result<String, PapErrore> {
    match parent_id {
        None => Ok(format!("/{nome}")),
        Some(pid) => {
            let parent_path: String = conn
                .query_row(
                    "SELECT Path FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
                    [pid],
                    |r| r.get(0),
                )
                .map_err(|_| PapErrore::Generico("Cartella parent non trovata".into()))?;
            Ok(format!("{parent_path}/{nome}"))
        }
    }
}

#[tauri::command]
pub fn folder_lista(state: State<'_, VaultState>) -> Result<Vec<Cartella>, PapErrore> {
    state.with_conn(lista_pure)
}

/// M7 PR-3: logica pura di `folder_lista`.
pub(crate) fn lista_pure(conn: &Connection) -> Result<Vec<Cartella>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT f.Id, f.Name, f.Path, f.ParentFolderId,
                (SELECT COUNT(*) FROM Prompts p
                 WHERE p.FolderId = f.Id AND p.DeletedAt IS NULL),
                f.CreatedAt, f.UpdatedAt
         FROM Folders f
         WHERE f.WorkspaceId = ?1 AND f.DeletedAt IS NULL
         ORDER BY f.Path COLLATE NOCASE ASC",
    )?;
    let cartelle = stmt
        .query_map([WORKSPACE_ID], |row| {
            Ok(Cartella {
                id: row.get(0)?,
                nome: row.get(1)?,
                path: row.get(2)?,
                parent_folder_id: row.get(3)?,
                conteggio_prompt: row.get(4)?,
                creato_a: row.get(5)?,
                aggiornato_a: row.get(6)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(cartelle)
}

#[tauri::command]
pub fn folder_crea(
    dati: NuovaCartella,
    state: State<'_, VaultState>,
) -> Result<String, PapErrore> {
    state.with_conn(|conn| crea_pure(conn, &dati))
}

/// M7 PR-3: logica pura di `folder_crea`.
pub(crate) fn crea_pure(
    conn: &Connection,
    dati: &NuovaCartella,
) -> Result<String, PapErrore> {
    let nome = nome_valido(&dati.nome)?;
    let path = calcola_path(conn, dati.parent_folder_id.as_deref(), &nome)?;
    let id = genera_id();

    conn.execute(
        "INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, WORKSPACE_ID, dati.parent_folder_id, nome, path],
    )
    .map_err(|e| {
        // Probabile violazione unique sibling name
        PapErrore::Generico(format!("Impossibile creare cartella: {e}"))
    })?;

    crate::audit::registra(conn, "folder.creato", "Folder", &id, Some(&path));
    log::info!("Cartella creata: {path} (id {id})");
    Ok(id)
}

fn rinomina_cascata(conn: &Connection, folder_id: &str, nuovo_nome: &str) -> Result<(), PapErrore> {
    // Path corrente della cartella che stiamo rinominando.
    let (vecchio_path, parent_id): (String, Option<String>) = conn.query_row(
        "SELECT Path, ParentFolderId FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
        [folder_id],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
    )?;

    let nuovo_path = match parent_id.as_deref() {
        None => format!("/{nuovo_nome}"),
        Some(pid) => {
            let parent_path: String = conn.query_row(
                "SELECT Path FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
                [pid],
                |r| r.get(0),
            )?;
            format!("{parent_path}/{nuovo_nome}")
        }
    };

    // Update self.
    conn.execute(
        "UPDATE Folders
         SET Name = ?1, Path = ?2, UpdatedAt = datetime('now')
         WHERE Id = ?3",
        params![nuovo_nome, nuovo_path, folder_id],
    )?;

    // Update discendenti: rimpiazza il prefisso del path.
    // Pattern: vecchio_path LIKE 'vecchio/%' → nuovo path = nuovo + suffisso.
    let prefisso_vecchio = format!("{vecchio_path}/");
    let prefisso_nuovo = format!("{nuovo_path}/");
    conn.execute(
        "UPDATE Folders
         SET Path = ?1 || SUBSTR(Path, LENGTH(?2) + 1),
             UpdatedAt = datetime('now')
         WHERE Path LIKE ?2 || '%' AND DeletedAt IS NULL",
        params![prefisso_nuovo, prefisso_vecchio],
    )?;

    Ok(())
}

#[tauri::command]
pub fn folder_rinomina(
    dati: RinominaCartella,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| rinomina_pure(conn, &dati))
}

/// M7 PR-3: logica pura di `folder_rinomina`.
pub(crate) fn rinomina_pure(
    conn: &Connection,
    dati: &RinominaCartella,
) -> Result<(), PapErrore> {
    let nuovo_nome = nome_valido(&dati.nuovo_nome)?;
    atomicamente(conn, |c| rinomina_cascata(c, &dati.id, &nuovo_nome))?;
    crate::audit::registra(conn, "folder.rinominato", "Folder", &dati.id, Some(&nuovo_nome));
    Ok(())
}

fn sposta_cascata(
    conn: &Connection,
    folder_id: &str,
    nuovo_parent_id: Option<&str>,
) -> Result<(), PapErrore> {
    // Stato corrente
    let (nome, vecchio_path): (String, String) = conn.query_row(
        "SELECT Name, Path FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
        [folder_id],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )?;

    // Anti-ciclo: il nuovo parent non può essere `folder_id` né uno dei suoi
    // discendenti. Discendente sse Path inizia con `vecchio_path/`.
    if let Some(pid) = nuovo_parent_id {
        if pid == folder_id {
            return Err(PapErrore::Generico(
                "Una cartella non può essere parent di sé stessa".into(),
            ));
        }
        let parent_path: String = conn.query_row(
            "SELECT Path FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
            [pid],
            |r| r.get(0),
        )?;
        if parent_path == vecchio_path || parent_path.starts_with(&format!("{vecchio_path}/")) {
            return Err(PapErrore::Generico(
                "Non puoi spostare una cartella dentro un suo discendente".into(),
            ));
        }
    }

    let nuovo_path = match nuovo_parent_id {
        None => format!("/{nome}"),
        Some(pid) => {
            let parent_path: String = conn.query_row(
                "SELECT Path FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
                [pid],
                |r| r.get(0),
            )?;
            format!("{parent_path}/{nome}")
        }
    };

    conn.execute(
        "UPDATE Folders
         SET ParentFolderId = ?1, Path = ?2, UpdatedAt = datetime('now')
         WHERE Id = ?3",
        params![nuovo_parent_id, nuovo_path, folder_id],
    )?;

    let prefisso_vecchio = format!("{vecchio_path}/");
    let prefisso_nuovo = format!("{nuovo_path}/");
    conn.execute(
        "UPDATE Folders
         SET Path = ?1 || SUBSTR(Path, LENGTH(?2) + 1),
             UpdatedAt = datetime('now')
         WHERE Path LIKE ?2 || '%' AND DeletedAt IS NULL",
        params![prefisso_nuovo, prefisso_vecchio],
    )?;
    Ok(())
}

#[tauri::command]
pub fn folder_sposta(
    dati: SpostaCartella,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| sposta_pure(conn, &dati))
}

/// M7 PR-3: logica pura di `folder_sposta`.
pub(crate) fn sposta_pure(
    conn: &Connection,
    dati: &SpostaCartella,
) -> Result<(), PapErrore> {
    atomicamente(conn, |c| {
        sposta_cascata(c, &dati.id, dati.nuovo_parent_id.as_deref())
    })?;
    crate::audit::registra(conn, "folder.spostato", "Folder", &dati.id, None);
    Ok(())
}

/// Re-pack atomico SortOrder dei siblings di una cartella.
/// Estratta come `pub(crate)` per testabilità diretta su `&Connection`.
pub(crate) fn riordina_folder_inner(
    conn: &Connection,
    id: &str,
    new_sort: i64,
) -> Result<(), PapErrore> {
    atomicamente(conn, |c| {
        let (workspace_id, parent_folder_id, current_sort): (String, Option<String>, i64) = c
            .query_row(
                "SELECT WorkspaceId, ParentFolderId, SortOrder
                 FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(|_| PapErrore::Generico(format!("Cartella {id} non trovata")))?;

        let sibling_count: i64 = c.query_row(
            "SELECT COUNT(*) FROM Folders
             WHERE WorkspaceId = ?1
               AND COALESCE(ParentFolderId, '') = COALESCE(?2, '')
               AND DeletedAt IS NULL
               AND Id != ?3",
            params![&workspace_id, &parent_folder_id, id],
            |r| r.get(0),
        )?;
        let target_sort = new_sort.clamp(0, sibling_count);

        if target_sort == current_sort {
            return Ok(()); // No-op: già al posto giusto
        }

        if target_sort < current_sort {
            // Spostiamo verso l'alto: shift +1 i siblings tra [target_sort, current_sort-1]
            c.execute(
                "UPDATE Folders SET SortOrder = SortOrder + 1
                 WHERE WorkspaceId = ?1
                   AND COALESCE(ParentFolderId, '') = COALESCE(?2, '')
                   AND DeletedAt IS NULL
                   AND Id != ?3
                   AND SortOrder >= ?4 AND SortOrder < ?5",
                params![&workspace_id, &parent_folder_id, id, target_sort, current_sort],
            )?;
        } else {
            // Spostiamo verso il basso: shift -1 i siblings tra [current_sort+1, target_sort]
            c.execute(
                "UPDATE Folders SET SortOrder = SortOrder - 1
                 WHERE WorkspaceId = ?1
                   AND COALESCE(ParentFolderId, '') = COALESCE(?2, '')
                   AND DeletedAt IS NULL
                   AND Id != ?3
                   AND SortOrder > ?4 AND SortOrder <= ?5",
                params![&workspace_id, &parent_folder_id, id, current_sort, target_sort],
            )?;
        }

        c.execute(
            "UPDATE Folders SET SortOrder = ?1, UpdatedAt = datetime('now') WHERE Id = ?2",
            params![target_sort, id],
        )?;
        Ok(())
    })
}

#[tauri::command]
pub fn folder_riordina(
    dati: RiordinaFolder,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        riordina_folder_inner(conn, &dati.id, dati.new_sort)?;
        crate::audit::registra(conn, "folder.riordinato", "Folder", &dati.id, None);
        log::info!("folder_riordina id={} new_sort={}", dati.id, dati.new_sort);
        Ok(())
    })
}

/// Re-pack atomico SortOrder dei siblings di un prompt.
pub(crate) fn riordina_prompt_inner(
    conn: &Connection,
    prompt_id: &str,
    new_sort: i64,
) -> Result<(), PapErrore> {
    atomicamente(conn, |c| {
        let (workspace_id, folder_id, current_sort): (String, Option<String>, i64) = c
            .query_row(
                "SELECT WorkspaceId, FolderId, SortOrder
                 FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
                params![prompt_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(|_| PapErrore::Generico(format!("Prompt {prompt_id} non trovato")))?;

        let sibling_count: i64 = c.query_row(
            "SELECT COUNT(*) FROM Prompts
             WHERE WorkspaceId = ?1
               AND COALESCE(FolderId, '') = COALESCE(?2, '')
               AND DeletedAt IS NULL
               AND Id != ?3",
            params![&workspace_id, &folder_id, prompt_id],
            |r| r.get(0),
        )?;
        let target_sort = new_sort.clamp(0, sibling_count);

        if target_sort == current_sort {
            return Ok(());
        }

        if target_sort < current_sort {
            c.execute(
                "UPDATE Prompts SET SortOrder = SortOrder + 1
                 WHERE WorkspaceId = ?1
                   AND COALESCE(FolderId, '') = COALESCE(?2, '')
                   AND DeletedAt IS NULL
                   AND Id != ?3
                   AND SortOrder >= ?4 AND SortOrder < ?5",
                params![&workspace_id, &folder_id, prompt_id, target_sort, current_sort],
            )?;
        } else {
            c.execute(
                "UPDATE Prompts SET SortOrder = SortOrder - 1
                 WHERE WorkspaceId = ?1
                   AND COALESCE(FolderId, '') = COALESCE(?2, '')
                   AND DeletedAt IS NULL
                   AND Id != ?3
                   AND SortOrder > ?4 AND SortOrder <= ?5",
                params![&workspace_id, &folder_id, prompt_id, current_sort, target_sort],
            )?;
        }

        c.execute(
            "UPDATE Prompts SET SortOrder = ?1, UpdatedAt = datetime('now') WHERE Id = ?2",
            params![target_sort, prompt_id],
        )?;
        Ok(())
    })
}

#[tauri::command]
pub fn prompt_riordina(
    dati: RiordinaPrompt,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        riordina_prompt_inner(conn, &dati.prompt_id, dati.new_sort)?;
        crate::audit::registra(conn, "prompt.riordinato", "Prompt", &dati.prompt_id, None);
        log::info!(
            "prompt_riordina id={} new_sort={}",
            dati.prompt_id,
            dati.new_sort
        );
        Ok(())
    })
}

#[tauri::command]
pub fn folder_elimina(id: String, state: State<'_, VaultState>) -> Result<(), PapErrore> {
    state.with_conn(|conn| elimina_pure(conn, &id))
}

/// M7 PR-3: logica pura di `folder_elimina`.
///
/// Soft-delete: marca DeletedAt sulla cartella e su tutti i discendenti.
/// I prompt dentro tornano a root (FolderId = NULL).
pub(crate) fn elimina_pure(conn: &Connection, id: &str) -> Result<(), PapErrore> {
    let path: String = conn.query_row(
        "SELECT Path FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
        [id],
        |r| r.get(0),
    )?;
    let prefisso = format!("{path}/");
    // Self + discendenti (tutti quelli con path che inizia con `path/`)
    conn.execute(
        "UPDATE Folders
         SET DeletedAt = datetime('now'), UpdatedAt = datetime('now')
         WHERE (Id = ?1 OR Path LIKE ?2 || '%') AND DeletedAt IS NULL",
        params![id, prefisso],
    )?;
    // I prompt dentro la cartella eliminata (e discendenti) tornano a root.
    conn.execute(
        "UPDATE Prompts
         SET FolderId = NULL, UpdatedAt = datetime('now')
         WHERE FolderId IN (
             SELECT Id FROM Folders
             WHERE (Id = ?1 OR Path LIKE ?2 || '%')
         )",
        params![id, prefisso],
    )?;
    crate::audit::registra(conn, "folder.eliminato", "Folder", id, Some(&path));
    log::info!("Cartella eliminata (soft): {path}");
    Ok(())
}

#[tauri::command]
pub fn prompt_sposta(dati: SpostaPrompt, state: State<'_, VaultState>) -> Result<(), PapErrore> {
    state.with_conn(|conn| prompt_sposta_pure(conn, &dati))
}

/// M7 PR-3: logica pura di `prompt_sposta`.
pub(crate) fn prompt_sposta_pure(
    conn: &Connection,
    dati: &SpostaPrompt,
) -> Result<(), PapErrore> {
    // Verifica esistenza cartella se specificata.
    if let Some(fid) = &dati.folder_id {
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
                [fid],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if !exists {
            return Err(PapErrore::Generico("Cartella destinazione non valida".into()));
        }
    }
    conn.execute(
        "UPDATE Prompts
         SET FolderId = ?1, UpdatedAt = datetime('now')
         WHERE Id = ?2 AND DeletedAt IS NULL",
        params![dati.folder_id, dati.prompt_id],
    )?;
    crate::audit::registra(
        conn,
        "prompt.spostato",
        "Prompt",
        &dati.prompt_id,
        dati.folder_id.as_deref(),
    );
    Ok(())
}

/// Esegue una closure dentro una transazione esplicita BEGIN/COMMIT.
/// `with_conn` espone `&Connection`, ma SQLite supporta BEGIN/COMMIT come
/// statement plain — non serve `&mut Connection` né `Transaction` di rusqlite.
/// In caso di errore, ROLLBACK best-effort.
fn atomicamente<F>(conn: &Connection, f: F) -> Result<(), PapErrore>
where
    F: FnOnce(&Connection) -> Result<(), PapErrore>,
{
    conn.execute_batch("BEGIN")?;
    match f(conn) {
        Ok(()) => {
            conn.execute_batch("COMMIT")?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
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

    fn crea(conn: &Connection, nome: &str, parent: Option<&str>) -> String {
        let id = genera_id();
        let path = calcola_path(conn, parent, nome).unwrap();
        conn.execute(
            "INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, WORKSPACE_ID, parent, nome, path],
        )
        .unwrap();
        id
    }

    #[test]
    fn nome_valido_casi() {
        assert!(nome_valido("Marketing").is_ok());
        assert_eq!(nome_valido("  spazio  ").unwrap(), "spazio");
        assert!(nome_valido("").is_err());
        assert!(nome_valido("   ").is_err());
        assert!(nome_valido("con/slash").is_err());
        assert!(nome_valido(&"x".repeat(101)).is_err());
    }

    #[test]
    fn calcola_path_root() {
        let conn = db_test();
        assert_eq!(calcola_path(&conn, None, "marketing").unwrap(), "/marketing");
    }

    #[test]
    fn calcola_path_nested() {
        let conn = db_test();
        let parent = crea(&conn, "marketing", None);
        let path = calcola_path(&conn, Some(&parent), "email").unwrap();
        assert_eq!(path, "/marketing/email");
    }

    #[test]
    fn rinomina_aggiorna_self_e_discendenti() {
        let conn = db_test();
        let mark = crea(&conn, "marketing", None);
        let email = crea(&conn, "email", Some(&mark));
        let _cold = crea(&conn, "cold", Some(&email));

        rinomina_cascata(&conn, &mark, "growth").unwrap();

        let path_mark: String = conn
            .query_row("SELECT Path FROM Folders WHERE Id = ?1", [&mark], |r| r.get(0))
            .unwrap();
        let path_email: String = conn
            .query_row("SELECT Path FROM Folders WHERE Id = ?1", [&email], |r| r.get(0))
            .unwrap();

        assert_eq!(path_mark, "/growth");
        assert_eq!(path_email, "/growth/email");
    }

    #[test]
    fn sposta_aggiorna_path_discendenti() {
        let conn = db_test();
        let mark = crea(&conn, "marketing", None);
        let email = crea(&conn, "email", Some(&mark));
        let archivio = crea(&conn, "archivio", None);

        sposta_cascata(&conn, &email, Some(&archivio)).unwrap();

        let path_email: String = conn
            .query_row("SELECT Path FROM Folders WHERE Id = ?1", [&email], |r| r.get(0))
            .unwrap();
        assert_eq!(path_email, "/archivio/email");
    }

    #[test]
    fn sposta_dentro_se_stessa_fallisce() {
        let conn = db_test();
        let f = crea(&conn, "f", None);
        assert!(sposta_cascata(&conn, &f, Some(&f)).is_err());
    }

    #[test]
    fn sposta_dentro_discendente_fallisce() {
        let conn = db_test();
        let mark = crea(&conn, "marketing", None);
        let email = crea(&conn, "email", Some(&mark));
        // Provo a spostare marketing dentro marketing/email (suo discendente).
        assert!(sposta_cascata(&conn, &mark, Some(&email)).is_err());
    }

    #[test]
    fn unique_sibling_name() {
        let conn = db_test();
        let _ = crea(&conn, "duplicato", None);
        // Stesso parent (root) + stesso nome → violazione UNIQUE.
        let path = calcola_path(&conn, None, "duplicato").unwrap();
        let result = conn.execute(
            "INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path)
             VALUES ('fld-x', ?1, NULL, 'duplicato', ?2)",
            params![WORKSPACE_ID, path],
        );
        assert!(result.is_err(), "Insert con nome duplicato deve fallire");
    }

    // ─────────── Stress test (Step 10 quality gate) ───────────

    /// Costruisce 20 catene lineari di profondità 5 → 100 cartelle totali,
    /// depth = 5 sul ramo più lungo. Ritorna un Vec<Vec<String>> con gli id
    /// delle cartelle, indicizzati [tree][level].
    fn costruisci_foresta_depth_5(conn: &Connection) -> Vec<Vec<String>> {
        const N_TREES: usize = 20;
        const DEPTH: usize = 5;
        let mut foresta = Vec::with_capacity(N_TREES);
        for t in 0..N_TREES {
            let mut catena = Vec::with_capacity(DEPTH);
            let mut parent: Option<String> = None;
            for l in 0..DEPTH {
                let nome = format!("t{t:02}-l{l}");
                let id = crea(conn, &nome, parent.as_deref());
                catena.push(id.clone());
                parent = Some(id);
            }
            foresta.push(catena);
        }
        foresta
    }

    /// Verifica l'invariante: per ogni cartella non eliminata,
    /// `Path` coincide con la concatenazione dei `Name` dei suoi antenati
    /// (incluso self). È la sentinella primaria contro inconsistenze
    /// `ParentFolderId` ↔ `Path` su sposta/rinomina di sotto-tree.
    fn verifica_invariante_path(conn: &Connection) {
        let mut stmt = conn
            .prepare(
                "SELECT Id, ParentFolderId, Name, Path FROM Folders
                 WHERE DeletedAt IS NULL",
            )
            .unwrap();
        let rows: Vec<(String, Option<String>, String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        for (id, parent_id, nome, path) in &rows {
            // Calcola path atteso risalendo lungo ParentFolderId.
            let atteso = match parent_id {
                None => format!("/{nome}"),
                Some(pid) => {
                    let parent_path: String = conn
                        .query_row(
                            "SELECT Path FROM Folders WHERE Id = ?1",
                            [pid],
                            |r| r.get(0),
                        )
                        .unwrap();
                    format!("{parent_path}/{nome}")
                }
            };
            assert_eq!(
                path, &atteso,
                "Path inconsistente per id={id}: stored={path}, atteso={atteso}"
            );
        }
    }

    #[test]
    fn foresta_100_cartelle_depth_5_invariante_path() {
        let conn = db_test();
        let foresta = costruisci_foresta_depth_5(&conn);
        assert_eq!(foresta.len() * foresta[0].len(), 100);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Folders WHERE DeletedAt IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 100);

        // Path delle foglie L4 ha 5 segmenti.
        let leaf = &foresta[0][4];
        let path_leaf: String = conn
            .query_row("SELECT Path FROM Folders WHERE Id = ?1", [leaf], |r| r.get(0))
            .unwrap();
        assert_eq!(path_leaf, "/t00-l0/t00-l1/t00-l2/t00-l3/t00-l4");

        verifica_invariante_path(&conn);
    }

    #[test]
    fn rinomina_root_aggiorna_tutta_la_catena() {
        let conn = db_test();
        let foresta = costruisci_foresta_depth_5(&conn);
        let root_t0 = &foresta[0][0];

        rinomina_cascata(&conn, root_t0, "rinominato").unwrap();

        // Tutta la catena deve ora avere Path /rinominato/...
        for (l, id) in foresta[0].iter().enumerate() {
            let path: String = conn
                .query_row("SELECT Path FROM Folders WHERE Id = ?1", [id], |r| r.get(0))
                .unwrap();
            let segmenti: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            assert_eq!(segmenti.len(), l + 1);
            assert_eq!(segmenti[0], "rinominato");
        }
        // L'invariante globale tiene su tutta la foresta.
        verifica_invariante_path(&conn);
    }

    #[test]
    fn sposta_sub_tree_profondo_aggiorna_path_discendenti() {
        let conn = db_test();
        let foresta = costruisci_foresta_depth_5(&conn);
        // Sposta il L2 di t00 (che ha 2 discendenti L3, L4) sotto t01-L0.
        let nodo_da_spostare = &foresta[0][2]; // /t00-l0/t00-l1/t00-l2
        let nuovo_parent = &foresta[1][0]; // /t01-l0

        sposta_cascata(&conn, nodo_da_spostare, Some(nuovo_parent)).unwrap();

        // Il nodo spostato ora vive sotto /t01-l0/t00-l2
        let path_spostato: String = conn
            .query_row(
                "SELECT Path FROM Folders WHERE Id = ?1",
                [nodo_da_spostare],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(path_spostato, "/t01-l0/t00-l2");

        // I suoi 2 discendenti L3, L4 anche.
        let l3 = &foresta[0][3];
        let l4 = &foresta[0][4];
        let path_l3: String = conn
            .query_row("SELECT Path FROM Folders WHERE Id = ?1", [l3], |r| r.get(0))
            .unwrap();
        let path_l4: String = conn
            .query_row("SELECT Path FROM Folders WHERE Id = ?1", [l4], |r| r.get(0))
            .unwrap();
        assert_eq!(path_l3, "/t01-l0/t00-l2/t00-l3");
        assert_eq!(path_l4, "/t01-l0/t00-l2/t00-l3/t00-l4");

        // Il restante tree t00 (l0, l1) NON deve essere stato toccato.
        let l0 = &foresta[0][0];
        let l1 = &foresta[0][1];
        let path_l0: String = conn
            .query_row("SELECT Path FROM Folders WHERE Id = ?1", [l0], |r| r.get(0))
            .unwrap();
        let path_l1: String = conn
            .query_row("SELECT Path FROM Folders WHERE Id = ?1", [l1], |r| r.get(0))
            .unwrap();
        assert_eq!(path_l0, "/t00-l0");
        assert_eq!(path_l1, "/t00-l0/t00-l1");

        verifica_invariante_path(&conn);
    }

    #[test]
    fn elimina_tree_marca_tutta_la_catena_deleted() {
        let conn = db_test();
        let foresta = costruisci_foresta_depth_5(&conn);
        // Soft-delete dell'intero tree 0 (la radice + 4 discendenti).
        let path_root_t0: String = conn
            .query_row(
                "SELECT Path FROM Folders WHERE Id = ?1",
                [&foresta[0][0]],
                |r| r.get(0),
            )
            .unwrap();
        let prefisso = format!("{path_root_t0}/");
        conn.execute(
            "UPDATE Folders
             SET DeletedAt = datetime('now')
             WHERE (Id = ?1 OR Path LIKE ?2 || '%') AND DeletedAt IS NULL",
            params![&foresta[0][0], prefisso],
        )
        .unwrap();

        let viventi: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Folders WHERE DeletedAt IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(viventi, 95, "100 - 5 (tree eliminato) = 95");

        // Sentinel: ogni id del tree 0 ha DeletedAt valorizzato.
        for id in &foresta[0] {
            let deleted: Option<String> = conn
                .query_row(
                    "SELECT DeletedAt FROM Folders WHERE Id = ?1",
                    [id],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(deleted.is_some(), "tree-0 nodo {id} non eliminato");
        }
    }

    #[test]
    fn sposta_dentro_discendente_profondo_fallisce() {
        let conn = db_test();
        let foresta = costruisci_foresta_depth_5(&conn);
        // L0 dentro L4 (suo discendente di 4 livelli) → deve fallire.
        let l0 = &foresta[0][0];
        let l4 = &foresta[0][4];
        assert!(
            sposta_cascata(&conn, l0, Some(l4)).is_err(),
            "sposta dentro discendente profondo deve fallire"
        );
        // Nessuna modifica intermedia: invariante intatta.
        verifica_invariante_path(&conn);
    }

    #[test]
    fn nomi_duplicati_fra_sotto_tree_diversi_consentiti() {
        // Lo schema UNIQUE è per (parent, name): nomi uguali sotto parent
        // diversi sono OK. Test sentinel: stesso nome "comune" sotto due
        // root diverse non viola il vincolo.
        let conn = db_test();
        let r1 = crea(&conn, "alpha", None);
        let r2 = crea(&conn, "beta", None);
        let _ = crea(&conn, "comune", Some(&r1));
        let _ = crea(&conn, "comune", Some(&r2));
        // Nessun panic — l'INSERT è andato.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Folders WHERE Name = 'comune'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    // ─────────────────── V014 — drag-reorder ───────────────────

    /// Crea una folder con SortOrder esplicito (per test riordino).
    fn crea_con_sort(conn: &Connection, nome: &str, parent: Option<&str>, sort: i64) -> String {
        let id = genera_id();
        let path = calcola_path(conn, parent, nome).unwrap();
        conn.execute(
            "INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path, SortOrder)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, WORKSPACE_ID, parent, nome, path, sort],
        )
        .unwrap();
        id
    }

    fn get_sort(conn: &Connection, id: &str) -> i64 {
        conn.query_row(
            "SELECT SortOrder FROM Folders WHERE Id = ?1",
            [id],
            |r| r.get(0),
        )
        .unwrap()
    }

    #[test]
    fn folder_riordina_no_op_se_stessa_posizione() {
        let conn = db_test();
        let f1 = crea_con_sort(&conn, "a", None, 0);
        let f2 = crea_con_sort(&conn, "b", None, 1);
        riordina_folder_inner(&conn, &f1, 0).unwrap();
        assert_eq!(get_sort(&conn, &f1), 0);
        assert_eq!(get_sort(&conn, &f2), 1);
    }

    #[test]
    fn folder_riordina_sposta_giu_shifta_siblings_su() {
        // f1=0, f2=1, f3=2. Sposta f1 a 2 → f2→0, f3→1, f1→2
        let conn = db_test();
        let f1 = crea_con_sort(&conn, "a", None, 0);
        let f2 = crea_con_sort(&conn, "b", None, 1);
        let f3 = crea_con_sort(&conn, "c", None, 2);
        riordina_folder_inner(&conn, &f1, 2).unwrap();
        assert_eq!(get_sort(&conn, &f2), 0, "f2 deve shiftare a 0");
        assert_eq!(get_sort(&conn, &f3), 1, "f3 deve shiftare a 1");
        assert_eq!(get_sort(&conn, &f1), 2, "f1 deve essere al target");
    }

    #[test]
    fn folder_riordina_sposta_su_shifta_siblings_giu() {
        // f1=0, f2=1, f3=2. Sposta f3 a 0 → f3→0, f1→1, f2→2
        let conn = db_test();
        let f1 = crea_con_sort(&conn, "a", None, 0);
        let f2 = crea_con_sort(&conn, "b", None, 1);
        let f3 = crea_con_sort(&conn, "c", None, 2);
        riordina_folder_inner(&conn, &f3, 0).unwrap();
        assert_eq!(get_sort(&conn, &f3), 0, "f3 al target");
        assert_eq!(get_sort(&conn, &f1), 1, "f1 shifta giù");
        assert_eq!(get_sort(&conn, &f2), 2, "f2 shifta giù");
    }

    #[test]
    fn folder_riordina_clamp_oltre_range() {
        // 3 sibling, sibling_count escludendo target = 2. new_sort=999 → 2.
        let conn = db_test();
        let f1 = crea_con_sort(&conn, "a", None, 0);
        let _f2 = crea_con_sort(&conn, "b", None, 1);
        let _f3 = crea_con_sort(&conn, "c", None, 2);
        riordina_folder_inner(&conn, &f1, 999).unwrap();
        assert_eq!(get_sort(&conn, &f1), 2, "clamp all'ultimo posto");
    }

    #[test]
    fn folder_riordina_id_inesistente_errore() {
        let conn = db_test();
        let res = riordina_folder_inner(&conn, "ghost", 0);
        assert!(matches!(res, Err(PapErrore::Generico(_))));
    }

    #[test]
    fn prompt_riordina_smoke() {
        // Pattern identico a folder_riordina; smoke test sufficiente.
        let conn = db_test();
        // Inseriamo 3 prompt fittizi nello stesso scope (FolderId NULL)
        // sfruttando lo schema esistente.
        for (i, id) in ["p1", "p2", "p3"].iter().enumerate() {
            conn.execute(
                "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                                      Visibility, CreatedAt, UpdatedAt, SortOrder, Version)
                 VALUES (?1, ?2, 'usr-locale', 'titolo', 'body',
                         'private', datetime('now'), datetime('now'), ?3, 1)",
                params![id, WORKSPACE_ID, i as i64],
            )
            .unwrap();
        }
        riordina_prompt_inner(&conn, "p1", 2).unwrap();
        let p1_sort: i64 = conn.query_row(
            "SELECT SortOrder FROM Prompts WHERE Id = 'p1'", [], |r| r.get(0)
        ).unwrap();
        let p2_sort: i64 = conn.query_row(
            "SELECT SortOrder FROM Prompts WHERE Id = 'p2'", [], |r| r.get(0)
        ).unwrap();
        let p3_sort: i64 = conn.query_row(
            "SELECT SortOrder FROM Prompts WHERE Id = 'p3'", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(p1_sort, 2);
        assert_eq!(p2_sort, 0);
        assert_eq!(p3_sort, 1);
    }

    // ─── M7 PR-3: copertura _pure wrapper command ─────────────────

    #[test]
    fn lista_pure_db_vuoto_ritorna_vec_vuoto() {
        let conn = db_test();
        let r = lista_pure(&conn).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn lista_pure_include_conteggio_prompt() {
        let conn = db_test();
        let fid = crea(&conn, "marketing", None);
        // inserisci 2 prompt nella cartella
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility,
                FolderId, Version, CreatedAt, UpdatedAt)
             VALUES ('p1', 'ws-personale', 'usr-locale', 'A', 'b', 'private', ?1, 1,
                     datetime('now'), datetime('now'))",
            params![fid],
        ).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility,
                FolderId, Version, CreatedAt, UpdatedAt)
             VALUES ('p2', 'ws-personale', 'usr-locale', 'B', 'b', 'private', ?1, 1,
                     datetime('now'), datetime('now'))",
            params![fid],
        ).unwrap();
        let r = lista_pure(&conn).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].conteggio_prompt, 2);
    }

    #[test]
    fn lista_pure_esclude_cartelle_eliminate() {
        let conn = db_test();
        let _vivo = crea(&conn, "vivo", None);
        let elim = crea(&conn, "eliminata", None);
        conn.execute(
            "UPDATE Folders SET DeletedAt = datetime('now') WHERE Id = ?1",
            params![elim],
        ).unwrap();
        let r = lista_pure(&conn).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].nome, "vivo");
    }

    #[test]
    fn crea_pure_happy_path() {
        let conn = db_test();
        let dati = NuovaCartella {
            nome: "ricerche".to_string(),
            parent_folder_id: None,
        };
        let id = crea_pure(&conn, &dati).unwrap();
        assert!(id.starts_with("fld-"));
        // Verifica path = /ricerche
        let path: String = conn.query_row(
            "SELECT Path FROM Folders WHERE Id = ?1",
            [&id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(path, "/ricerche");
    }

    #[test]
    fn crea_pure_nome_vuoto_errore() {
        let conn = db_test();
        let dati = NuovaCartella {
            nome: "".to_string(),
            parent_folder_id: None,
        };
        let r = crea_pure(&conn, &dati);
        assert!(r.is_err());
    }

    #[test]
    fn crea_pure_sibling_duplicato_errore() {
        let conn = db_test();
        let dati = NuovaCartella {
            nome: "dup".to_string(),
            parent_folder_id: None,
        };
        crea_pure(&conn, &dati).unwrap();
        let r = crea_pure(&conn, &dati);
        assert!(r.is_err(), "secondo crea con stesso nome+parent deve fallire UNIQUE");
    }

    #[test]
    fn rinomina_pure_happy_path() {
        let conn = db_test();
        let id = crea(&conn, "vecchio", None);
        let dati = RinominaCartella {
            id: id.clone(),
            nuovo_nome: "nuovo".to_string(),
        };
        rinomina_pure(&conn, &dati).unwrap();
        let (nome, path): (String, String) = conn.query_row(
            "SELECT Name, Path FROM Folders WHERE Id = ?1",
            [&id], |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap();
        assert_eq!(nome, "nuovo");
        assert_eq!(path, "/nuovo");
    }

    #[test]
    fn rinomina_pure_nome_invalido_errore() {
        let conn = db_test();
        let id = crea(&conn, "x", None);
        let dati = RinominaCartella {
            id,
            nuovo_nome: "".to_string(),
        };
        let r = rinomina_pure(&conn, &dati);
        assert!(r.is_err());
    }

    #[test]
    fn sposta_pure_aggiorna_parent_e_path() {
        let conn = db_test();
        let parent = crea(&conn, "padre", None);
        let child = crea(&conn, "figlio", None);
        let dati = SpostaCartella {
            id: child.clone(),
            nuovo_parent_id: Some(parent.clone()),
        };
        sposta_pure(&conn, &dati).unwrap();
        let (parent_id, path): (Option<String>, String) = conn.query_row(
            "SELECT ParentFolderId, Path FROM Folders WHERE Id = ?1",
            [&child], |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap();
        assert_eq!(parent_id.as_deref(), Some(parent.as_str()));
        assert_eq!(path, "/padre/figlio");
    }

    #[test]
    fn sposta_pure_dentro_se_stessa_errore() {
        let conn = db_test();
        let id = crea(&conn, "x", None);
        let dati = SpostaCartella {
            id: id.clone(),
            nuovo_parent_id: Some(id),
        };
        let r = sposta_pure(&conn, &dati);
        assert!(r.is_err());
    }

    #[test]
    fn elimina_pure_soft_delete_cartella_e_prompt_a_root() {
        let conn = db_test();
        let fid = crea(&conn, "tomb", None);
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility,
                FolderId, Version, CreatedAt, UpdatedAt)
             VALUES ('p1', 'ws-personale', 'usr-locale', 'P', 'b', 'private', ?1, 1,
                     datetime('now'), datetime('now'))",
            params![fid],
        ).unwrap();

        elimina_pure(&conn, &fid).unwrap();

        // Folder DeletedAt valorizzato
        let del: Option<String> = conn.query_row(
            "SELECT DeletedAt FROM Folders WHERE Id = ?1",
            [&fid], |r| r.get(0)
        ).unwrap();
        assert!(del.is_some());

        // Prompt tornato a root (FolderId NULL)
        let p_folder: Option<String> = conn.query_row(
            "SELECT FolderId FROM Prompts WHERE Id = 'p1'", [], |r| r.get(0)
        ).unwrap();
        assert!(p_folder.is_none());
    }

    #[test]
    fn elimina_pure_cartella_inesistente_errore() {
        let conn = db_test();
        let r = elimina_pure(&conn, "fld-fantasma");
        assert!(r.is_err());
    }

    #[test]
    fn prompt_sposta_pure_a_folder_valida() {
        let conn = db_test();
        let fid = crea(&conn, "dest", None);
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility,
                Version, CreatedAt, UpdatedAt)
             VALUES ('p1', 'ws-personale', 'usr-locale', 'P', 'b', 'private', 1,
                     datetime('now'), datetime('now'))",
            [],
        ).unwrap();
        let dati = SpostaPrompt {
            prompt_id: "p1".to_string(),
            folder_id: Some(fid.clone()),
        };
        prompt_sposta_pure(&conn, &dati).unwrap();
        let pf: Option<String> = conn.query_row(
            "SELECT FolderId FROM Prompts WHERE Id = 'p1'", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(pf.as_deref(), Some(fid.as_str()));
    }

    #[test]
    fn prompt_sposta_pure_a_root() {
        let conn = db_test();
        let fid = crea(&conn, "src", None);
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility,
                FolderId, Version, CreatedAt, UpdatedAt)
             VALUES ('p1', 'ws-personale', 'usr-locale', 'P', 'b', 'private', ?1, 1,
                     datetime('now'), datetime('now'))",
            params![fid],
        ).unwrap();
        let dati = SpostaPrompt {
            prompt_id: "p1".to_string(),
            folder_id: None,
        };
        prompt_sposta_pure(&conn, &dati).unwrap();
        let pf: Option<String> = conn.query_row(
            "SELECT FolderId FROM Prompts WHERE Id = 'p1'", [], |r| r.get(0)
        ).unwrap();
        assert!(pf.is_none());
    }

    #[test]
    fn prompt_sposta_pure_folder_inesistente_errore() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility,
                Version, CreatedAt, UpdatedAt)
             VALUES ('p1', 'ws-personale', 'usr-locale', 'P', 'b', 'private', 1,
                     datetime('now'), datetime('now'))",
            [],
        ).unwrap();
        let dati = SpostaPrompt {
            prompt_id: "p1".to_string(),
            folder_id: Some("fld-fantasma".to_string()),
        };
        let r = prompt_sposta_pure(&conn, &dati);
        assert!(r.is_err());
    }
}
