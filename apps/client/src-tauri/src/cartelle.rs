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
    state.with_conn(|conn| {
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
    })
}

#[tauri::command]
pub fn folder_crea(
    dati: NuovaCartella,
    state: State<'_, VaultState>,
) -> Result<String, PapErrore> {
    state.with_conn(|conn| {
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
    })
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
    state.with_conn(|conn| {
        let nuovo_nome = nome_valido(&dati.nuovo_nome)?;
        atomicamente(conn, |c| rinomina_cascata(c, &dati.id, &nuovo_nome))?;
        crate::audit::registra(conn, "folder.rinominato", "Folder", &dati.id, Some(&nuovo_nome));
        Ok(())
    })
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
    state.with_conn(|conn| {
        atomicamente(conn, |c| {
            sposta_cascata(c, &dati.id, dati.nuovo_parent_id.as_deref())
        })?;
        crate::audit::registra(conn, "folder.spostato", "Folder", &dati.id, None);
        Ok(())
    })
}

#[tauri::command]
pub fn folder_elimina(id: String, state: State<'_, VaultState>) -> Result<(), PapErrore> {
    // Soft-delete: marca DeletedAt sulla cartella e su tutti i discendenti.
    // I prompt dentro restano (con FolderId puntante a una cartella deleted),
    // ma il filtro libreria_lista li mostrerà comunque (non filtra per Folders.DeletedAt).
    // Nota: per semplicità v0.2.1 NON spostiamo i prompt a root — l'UX di
    // "elimina cartella" dovrà avvisare. Si può raffinare in PR successiva.
    state.with_conn(|conn| {
        let path: String = conn.query_row(
            "SELECT Path FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
            [&id],
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
        crate::audit::registra(conn, "folder.eliminato", "Folder", &id, Some(&path));
        log::info!("Cartella eliminata (soft): {path}");
        Ok(())
    })
}

#[tauri::command]
pub fn prompt_sposta(dati: SpostaPrompt, state: State<'_, VaultState>) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
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
    })
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
}
