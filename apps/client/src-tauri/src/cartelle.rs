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
}
