// Modulo cestino — recupero ed eliminazione definitiva dei prompt cancellati.
//
// La cancellazione (`editor::prompt_elimina`) è un soft-delete: imposta
// `DeletedAt` senza rimuovere la riga. Questo modulo espone il cestino sopra
// a quel meccanismo:
//   - `cestino_lista`           → i prompt soft-deleted (con quanti li importano)
//   - `prompt_ripristina`       → riporta in vita un prompt (DeletedAt = NULL)
//   - `prompt_elimina_definitivo` → DELETE fisico con cascade manuale
//   - `cestino_svuota`          → purge fisico di tutto il cestino
//
// Issue #302. Blueprint: docs/roadmap/cestino-e-cancellazione-import.md
//
// Nota cascade (verificato su schema V001-V015): `PRAGMA foreign_keys = ON` e
// solo `PromptTags` ha `ON DELETE CASCADE`. Tutte le altre relazioni verso
// `Prompts(Id)` sono FK semplici e vanno gestite a mano prima del DELETE
// finale, altrimenti la cancellazione fisica fallisce (vedi §3.1bis del
// blueprint). I `PromptTags` invece sopravvivono al soft-delete (vengono
// preservati apposta, così il ripristino è completo) e si svuotano da soli
// via cascade al purge.

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

// ─────────── Tipi pubblici ───────────

#[derive(Debug, Clone, Serialize)]
pub struct PromptCancellato {
    pub id: String,
    pub titolo: String,
    /// Timestamp ISO di cancellazione (colonna `DeletedAt`).
    pub eliminato_il: String,
    /// Quanti prompt VIVI importano ancora questo prompt cancellato: se > 0,
    /// ripristinarlo ricuce import oggi rotti (IMP001) altrove.
    pub importato_da: usize,
}

// ─────────── Logica pura (testabile su &Connection) ───────────

/// Lista dei prompt nel cestino, dal più recente. Include il conteggio dei
/// dipendenti vivi (informativo per il ripristino).
pub fn cestino_lista_pure(conn: &Connection) -> Result<Vec<PromptCancellato>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT p.Id, p.Title, p.DeletedAt,
                (SELECT COUNT(DISTINCT pi.ParentPromptId)
                   FROM PromptImports pi
                   JOIN Prompts pr ON pr.Id = pi.ParentPromptId
                  WHERE pi.ImportedPromptId = p.Id AND pr.DeletedAt IS NULL) AS importato_da
         FROM Prompts p
         WHERE p.DeletedAt IS NOT NULL
         ORDER BY p.DeletedAt DESC",
    )?;
    let righe = stmt
        .query_map([], |r| {
            let importato_da: i64 = r.get(3)?;
            Ok(PromptCancellato {
                id: r.get(0)?,
                titolo: r.get(1)?,
                eliminato_il: r.get(2)?,
                importato_da: importato_da.max(0) as usize,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(righe)
}

/// Ripristina un prompt dal cestino. Ritorna `true` se qualcosa è stato
/// ripristinato, `false` se l'id non era nel cestino (no-op idempotente).
///
/// Oltre ad azzerare `DeletedAt`, ricostruisce gli import in uscita
/// (`aggiorna_imports`, che il soft-delete aveva ripulito) e re-indicizza
/// l'FTS. I tag sopravvivono al soft-delete, quindi tornano automaticamente.
pub fn ripristina_pure(conn: &Connection, id: &str) -> Result<bool, PapErrore> {
    let body: Option<String> = conn
        .query_row(
            "SELECT Body FROM Prompts WHERE Id = ?1 AND DeletedAt IS NOT NULL",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(body) = body else {
        return Ok(false);
    };

    conn.execute(
        "UPDATE Prompts SET DeletedAt = NULL, UpdatedAt = datetime('now') WHERE Id = ?1",
        [id],
    )?;
    // Ricostruisce le righe PromptImports in uscita dal body ripristinato.
    crate::prompt_componibili::aggiorna_imports(conn, id, &body)?;
    // Re-indicizza l'FTS (il prompt cancellato ne era escluso).
    crate::editor::ricostruisci_fts(conn)?;
    crate::audit::registra(conn, "prompt.ripristinato", "Prompt", id, None);
    log::info!("Prompt ripristinato dal cestino: {id}");
    Ok(true)
}

/// Eliminazione fisica di un prompt che si trova nel cestino. Ritorna `true`
/// se eliminato, `false` se l'id non era nel cestino (guard: mai su prompt
/// vivi). Cascade manuale in transazione unica.
///
/// Le varianti/fork che puntavano a questo prompt vengono "promosse" a prompt
/// indipendenti (riferimento azzerato) invece di essere cancellate: niente
/// perdita di dati a cascata non richiesta.
pub fn elimina_definitivo_pure(conn: &Connection, id: &str) -> Result<bool, PapErrore> {
    let nel_cestino = conn
        .query_row(
            "SELECT 1 FROM Prompts WHERE Id = ?1 AND DeletedAt IS NOT NULL",
            [id],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if !nel_cestino {
        return Ok(false);
    }

    let tx = conn.unchecked_transaction()?;
    // Figli con FK semplice (no cascade) → cancellare prima della riga Prompts.
    tx.execute("DELETE FROM PromptVersions WHERE PromptId = ?1", [id])?;
    tx.execute(
        "DELETE FROM PromptImports WHERE ParentPromptId = ?1 OR ImportedPromptId = ?1",
        [id],
    )?;
    tx.execute("DELETE FROM PromptRatings WHERE PromptId = ?1", [id])?;
    tx.execute("DELETE FROM PromptGoldens WHERE PromptId = ?1", [id])?;
    // Self-ref: promuovi varianti/fork orfani a prompt indipendenti.
    tx.execute(
        "UPDATE Prompts SET ParentPromptId = NULL, IsVariant = 0, VariantLabel = NULL
         WHERE ParentPromptId = ?1",
        [id],
    )?;
    tx.execute(
        "UPDATE Prompts SET ForkOfPromptId = NULL WHERE ForkOfPromptId = ?1",
        [id],
    )?;
    // DELETE finale (guard ridondante su DeletedAt): cascade su PromptTags.
    tx.execute(
        "DELETE FROM Prompts WHERE Id = ?1 AND DeletedAt IS NOT NULL",
        [id],
    )?;
    tx.commit()?;

    crate::audit::registra(conn, "prompt.eliminato_definitivo", "Prompt", id, None);
    log::info!("Prompt eliminato definitivamente: {id}");
    Ok(true)
}

/// Svuota il cestino: purge fisico di ogni prompt cancellato. Ritorna il
/// numero di prompt effettivamente eliminati.
pub fn svuota_pure(conn: &Connection) -> Result<usize, PapErrore> {
    let ids: Vec<String> = {
        let mut stmt =
            conn.prepare("SELECT Id FROM Prompts WHERE DeletedAt IS NOT NULL")?;
        let raccolti: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();
        raccolti
    };
    let mut n = 0usize;
    for id in ids {
        if elimina_definitivo_pure(conn, &id)? {
            n += 1;
        }
    }
    Ok(n)
}

// ─────────── Comandi Tauri ───────────

#[tauri::command]
pub fn cestino_lista(
    state: State<'_, VaultState>,
) -> Result<Vec<PromptCancellato>, PapErrore> {
    state.with_conn(cestino_lista_pure)
}

#[tauri::command]
pub fn prompt_ripristina(id: String, state: State<'_, VaultState>) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        ripristina_pure(conn, &id)?;
        Ok(())
    })
}

#[tauri::command]
pub fn prompt_elimina_definitivo(
    id: String,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        elimina_definitivo_pure(conn, &id)?;
        Ok(())
    })
}

#[tauri::command]
pub fn cestino_svuota(state: State<'_, VaultState>) -> Result<usize, PapErrore> {
    state.with_conn(svuota_pure)
}

// ─────────── Test ───────────

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

    fn inserisci(conn: &Connection, id: &str, titolo: &str, body: &str) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                Visibility, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, 'private', 1,
                     datetime('now'), datetime('now'))",
            params![id, titolo, body],
        )
        .unwrap();
    }

    fn soft_delete(conn: &Connection, id: &str) {
        conn.execute(
            "UPDATE Prompts SET DeletedAt = datetime('now') WHERE Id = ?1",
            [id],
        )
        .unwrap();
    }

    fn esiste_vivo(conn: &Connection, id: &str) -> bool {
        conn.query_row(
            "SELECT 1 FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
            [id],
            |_| Ok(()),
        )
        .optional()
        .unwrap()
        .is_some()
    }

    #[test]
    fn lista_solo_cancellati_ordinati_per_data() {
        let conn = db_test();
        inserisci(&conn, "prm-vivo", "Vivo", "corpo");
        inserisci(&conn, "prm-x", "Vecchio", "corpo");
        inserisci(&conn, "prm-y", "Nuovo", "corpo");
        // x cancellato prima, y dopo
        conn.execute(
            "UPDATE Prompts SET DeletedAt = '2026-01-01T00:00:00Z' WHERE Id = 'prm-x'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE Prompts SET DeletedAt = '2026-02-01T00:00:00Z' WHERE Id = 'prm-y'",
            [],
        )
        .unwrap();

        let lista = cestino_lista_pure(&conn).unwrap();
        assert_eq!(lista.len(), 2, "il prompt vivo non deve comparire");
        // ordine DeletedAt DESC → y (feb) prima di x (gen)
        assert_eq!(lista[0].id, "prm-y");
        assert_eq!(lista[1].id, "prm-x");
    }

    #[test]
    fn lista_conta_dipendenti_vivi() {
        let conn = db_test();
        inserisci(&conn, "prm-base", "Base", "corpo base");
        inserisci(&conn, "prm-uno", "Uno", r#"{{import "Base"}}"#);
        crate::prompt_componibili::aggiorna_imports(&conn, "prm-uno", r#"{{import "Base"}}"#)
            .unwrap();
        soft_delete(&conn, "prm-base");

        let lista = cestino_lista_pure(&conn).unwrap();
        let base = lista.iter().find(|p| p.id == "prm-base").unwrap();
        assert_eq!(base.importato_da, 1);
    }

    #[test]
    fn ripristina_riporta_in_vita() {
        let conn = db_test();
        inserisci(&conn, "prm-r", "Da ripristinare", "corpo");
        soft_delete(&conn, "prm-r");
        assert!(!esiste_vivo(&conn, "prm-r"));

        let ok = ripristina_pure(&conn, "prm-r").unwrap();
        assert!(ok);
        assert!(esiste_vivo(&conn, "prm-r"));
        // non più nel cestino
        assert!(cestino_lista_pure(&conn).unwrap().is_empty());
    }

    #[test]
    fn ripristina_id_inesistente_o_vivo_e_no_op() {
        let conn = db_test();
        inserisci(&conn, "prm-vivo", "Vivo", "corpo");
        assert!(!ripristina_pure(&conn, "prm-vivo").unwrap(), "prompt vivo: no-op");
        assert!(!ripristina_pure(&conn, "prm-ghost").unwrap(), "id inesistente: no-op");
    }

    #[test]
    fn ripristina_preserva_i_tag() {
        let conn = db_test();
        inserisci(&conn, "prm-t", "Con tag", "corpo");
        // Crea tag + associazione (i tag sopravvivono al soft-delete).
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('tag-1', 'ws-personale', 'lavoro', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO PromptTags (PromptId, TagId) VALUES ('prm-t', 'tag-1')",
            [],
        )
        .unwrap();
        soft_delete(&conn, "prm-t");

        ripristina_pure(&conn, "prm-t").unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptTags WHERE PromptId = 'prm-t'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1, "il tag deve sopravvivere al ciclo cancella→ripristina");
    }

    #[test]
    fn ripristina_ricuce_import_rotto() {
        let conn = db_test();
        inserisci(&conn, "prm-base", "Base", "corpo base");
        inserisci(&conn, "prm-uno", "Uno", r#"{{import "Base"}}"#);
        crate::prompt_componibili::aggiorna_imports(&conn, "prm-uno", r#"{{import "Base"}}"#)
            .unwrap();
        soft_delete(&conn, "prm-base");

        // ripristino → la riga PromptImports di prm-uno torna risolvibile
        ripristina_pure(&conn, "prm-base").unwrap();
        assert!(esiste_vivo(&conn, "prm-base"));
    }

    #[test]
    fn elimina_definitivo_rifiuta_prompt_vivo() {
        let conn = db_test();
        inserisci(&conn, "prm-vivo", "Vivo", "corpo");
        assert!(!elimina_definitivo_pure(&conn, "prm-vivo").unwrap());
        assert!(esiste_vivo(&conn, "prm-vivo"), "un prompt vivo non va mai purgato");
    }

    #[test]
    fn elimina_definitivo_rimuove_riga_e_figli() {
        let conn = db_test();
        inserisci(&conn, "prm-p", "Purge", r#"{{import "Base"}}"#);
        inserisci(&conn, "prm-base", "Base", "corpo");
        // versione + import + rating + golden
        conn.execute(
            "INSERT INTO PromptVersions (Id, PromptId, Version, Title, Body, CreatedAt, CreatedByUserId)
             VALUES ('ver-1', 'prm-p', 1, 'Purge', 'corpo', datetime('now'), 'usr-locale')",
            [],
        )
        .unwrap();
        crate::prompt_componibili::aggiorna_imports(&conn, "prm-p", r#"{{import "Base"}}"#)
            .unwrap();
        soft_delete(&conn, "prm-p");

        let ok = elimina_definitivo_pure(&conn, "prm-p").unwrap();
        assert!(ok);
        // riga sparita
        let righe: i64 = conn
            .query_row("SELECT COUNT(*) FROM Prompts WHERE Id = 'prm-p'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(righe, 0);
        // figli spariti
        let ver: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptVersions WHERE PromptId = 'prm-p'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(ver, 0);
        let imp: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptImports WHERE ParentPromptId = 'prm-p'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(imp, 0);
    }

    #[test]
    fn elimina_definitivo_promuove_varianti_orfane() {
        let conn = db_test();
        inserisci(&conn, "prm-main", "Main", "corpo");
        inserisci(&conn, "prm-var", "Var", "corpo variante");
        conn.execute(
            "UPDATE Prompts SET ParentPromptId = 'prm-main', IsVariant = 1, VariantLabel = 'b'
             WHERE Id = 'prm-var'",
            [],
        )
        .unwrap();
        soft_delete(&conn, "prm-main");

        elimina_definitivo_pure(&conn, "prm-main").unwrap();
        // la variante sopravvive, promossa a indipendente
        let (parent, is_var): (Option<String>, i64) = conn
            .query_row(
                "SELECT ParentPromptId, IsVariant FROM Prompts WHERE Id = 'prm-var'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(parent.is_none());
        assert_eq!(is_var, 0);
    }

    #[test]
    fn svuota_purga_tutto_il_cestino() {
        let conn = db_test();
        inserisci(&conn, "prm-vivo", "Vivo", "corpo");
        inserisci(&conn, "prm-a", "A", "corpo");
        inserisci(&conn, "prm-b", "B", "corpo");
        soft_delete(&conn, "prm-a");
        soft_delete(&conn, "prm-b");

        let n = svuota_pure(&conn).unwrap();
        assert_eq!(n, 2);
        assert!(cestino_lista_pure(&conn).unwrap().is_empty());
        assert!(esiste_vivo(&conn, "prm-vivo"), "il prompt vivo resta intatto");
    }
}
