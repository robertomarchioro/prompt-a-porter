// Modulo varianti — Fase 4 Step 1.
//
// Una "variante" è un prompt con stesso intento del Parent ma testo
// alternativo. UseCount, rating, versioning sono indipendenti, così
// emerge naturalmente la formulazione migliore.
//
// MVP scope (questa PR):
// - varianti_lista(parent_id): lista varianti attive di un prompt
// - prompt_crea_variante(parent_id, etichetta?): duplica il parent con
//   ParentPromptId valorizzato, suggerendo "B"/"C"/... se etichetta omessa
// - Hook FTS/embedding/imports applicati anche alle varianti (sono
//   prompt a tutti gli effetti)
//
// Out of scope (sub-step futuri):
// - prompt_promuovi_variante: scambia main ↔ variant
// - UI pannello "Confronto varianti" multicolonna
// - Statistiche aggregate per gruppo di varianti

use rusqlite::{params, Connection};
use serde::Serialize;
use tauri::State;

use crate::embeddings::EmbeddingsState;
use crate::errore::PapErrore;
use crate::vault::VaultState;

const USER_LOCALE: &str = "usr-locale";

#[derive(Debug, Clone, Serialize)]
pub struct VariantInfo {
    pub id: String,
    pub parent_prompt_id: String,
    pub variant_label: String,
    pub titolo: String,
    pub body: String,
    pub uso_count: i64,
    pub creato_a: String,
    pub aggiornato_a: String,
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
        "prm-{:012x}{:02x}{:02x}{:02x}{:02x}",
        ts, rnd[0], rnd[1], rnd[2], rnd[3]
    )
}

fn etichette_usate(conn: &Connection, parent_id: &str) -> Result<Vec<String>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT VariantLabel FROM Prompts
         WHERE ParentPromptId = ?1 AND DeletedAt IS NULL AND VariantLabel IS NOT NULL",
    )?;
    let rows: Vec<String> = stmt
        .query_map([parent_id], |r| r.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Suggerisce la prossima etichetta libera ("B", "C", "D", ...).
/// "A" è riservata al prompt principale per convenzione, anche se non
/// viene mai memorizzata (il principale ha `VariantLabel = NULL`).
fn prossima_etichetta(usate: &[String]) -> String {
    for c in b'B'..=b'Z' {
        let candidato = (c as char).to_string();
        if !usate.iter().any(|u| u.eq_ignore_ascii_case(&candidato)) {
            return candidato;
        }
    }
    // Oltre 25 varianti — fallback con suffisso numerico.
    format!("V{}", usate.len() + 1)
}

#[allow(clippy::type_complexity)]
fn carica_parent(
    conn: &Connection,
    parent_id: &str,
) -> Result<
    (
        String,
        Option<String>,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    PapErrore,
> {
    conn.query_row(
        "SELECT Title, Description, Body, Visibility, TargetModel, FolderId, ParentPromptId
         FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
        [parent_id],
        |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        },
    )
    .map_err(|_| PapErrore::Generico(format!("Prompt parent '{parent_id}' non trovato")))
}

fn copia_tags(conn: &Connection, src_id: &str, dst_id: &str) -> Result<(), PapErrore> {
    conn.execute(
        "INSERT OR IGNORE INTO PromptTags (PromptId, TagId)
         SELECT ?1, TagId FROM PromptTags WHERE PromptId = ?2",
        params![dst_id, src_id],
    )?;
    Ok(())
}

/// Logica pura di `prompt_crea_variante`, testabile senza Tauri State.
pub(crate) fn crea_variante_pure(
    conn: &Connection,
    parent_id: &str,
    etichetta: Option<&str>,
) -> Result<String, PapErrore> {
    let (titolo, descrizione, body, visibility, target_model, folder_id, parent_di_parent) =
        carica_parent(conn, parent_id)?;

    // Se l'utente passa il `parent_id` di una variante, ri-aggancia il
    // grandparent come parent reale per evitare gerarchie a 2+ livelli
    // (le varianti sono "sorelle" del principale, non nipoti).
    let effective_parent = parent_di_parent.unwrap_or_else(|| parent_id.to_string());

    let usate = etichette_usate(conn, &effective_parent)?;
    let label = match etichetta.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(custom) => {
            if usate.iter().any(|u| u.eq_ignore_ascii_case(custom)) {
                return Err(PapErrore::Generico(format!(
                    "Etichetta '{custom}' già usata per questo prompt"
                )));
            }
            custom.to_string()
        }
        None => prossima_etichetta(&usate),
    };

    let id = genera_id();
    let titolo_variante = format!("{titolo} ({label})");

    conn.execute(
        "INSERT INTO Prompts
            (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
             Visibility, TargetModel, FolderId, ParentPromptId, VariantLabel,
             IsVariant, Version, CreatedAt, UpdatedAt)
         VALUES (?1, 'ws-personale', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, 1,
                 datetime('now'), datetime('now'))",
        params![
            id,
            USER_LOCALE,
            titolo_variante,
            descrizione,
            body,
            visibility,
            target_model,
            folder_id,
            effective_parent,
            label,
        ],
    )?;

    copia_tags(conn, &effective_parent, &id)?;
    crate::versioning::snapshot_versione(conn, &id, USER_LOCALE)?;
    crate::audit::registra(
        conn,
        "variante.creata",
        "Prompt",
        &id,
        Some(&format!("variante {label} di {effective_parent}")),
    );
    log::info!("Variante creata: {id} (parent {effective_parent}, label {label})");
    Ok(id)
}

pub(crate) fn lista_pure(
    conn: &Connection,
    parent_id: &str,
) -> Result<Vec<VariantInfo>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT Id, ParentPromptId, COALESCE(VariantLabel, ''), Title, Body,
                COALESCE(UseCount, 0), CreatedAt, UpdatedAt
         FROM Prompts
         WHERE ParentPromptId = ?1 AND DeletedAt IS NULL
         ORDER BY VariantLabel ASC",
    )?;
    let rows: Vec<VariantInfo> = stmt
        .query_map([parent_id], |r| {
            Ok(VariantInfo {
                id: r.get(0)?,
                parent_prompt_id: r.get(1)?,
                variant_label: r.get(2)?,
                titolo: r.get(3)?,
                body: r.get(4)?,
                uso_count: r.get(5)?,
                creato_a: r.get(6)?,
                aggiornato_a: r.get(7)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

#[tauri::command]
pub fn prompt_crea_variante(
    parent_id: String,
    etichetta: Option<String>,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<String, PapErrore> {
    let id = state.with_conn(|conn| crea_variante_pure(conn, &parent_id, etichetta.as_deref()))?;

    // Hook FTS/embedding/imports fuori dalla transazione principale per
    // coerenza coi pattern di editor::prompt_crea.
    state.with_conn(|conn| {
        let body: String = conn.query_row(
            "SELECT Body FROM Prompts WHERE Id = ?1",
            [&id],
            |r| r.get(0),
        )?;
        crate::editor::ricostruisci_fts(conn)?;
        crate::editor::aggiorna_embedding(conn, &rt_state, &id, &body)?;
        crate::prompt_componibili::aggiorna_imports(conn, &id, &body)?;
        Ok(())
    })?;

    Ok(id)
}

#[tauri::command]
pub fn varianti_lista(
    parent_id: String,
    state: State<'_, VaultState>,
) -> Result<Vec<VariantInfo>, PapErrore> {
    state.with_conn(|conn| lista_pure(conn, &parent_id))
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
        // Inserisci un prompt parent + 2 tag.
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
                Visibility, TargetModel, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-parent', 'ws-personale', 'usr-locale', 'Padre', 'desc',
                     'body originale', 'private', 'claude-sonnet', 1,
                     datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('t-1', 'ws-personale', 'tag1', datetime('now'), datetime('now')),
                    ('t-2', 'ws-personale', 'tag2', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO PromptTags (PromptId, TagId) VALUES ('prm-parent', 't-1'),
                    ('prm-parent', 't-2')",
            [],
        )
        .unwrap();
        // Snapshot iniziale per soddisfare check su PromptVersions.
        crate::versioning::snapshot_versione(&conn, "prm-parent", "usr-locale").unwrap();
        conn
    }

    #[test]
    fn prossima_etichetta_b_se_nessuna_usata() {
        assert_eq!(prossima_etichetta(&[]), "B");
    }

    #[test]
    fn prossima_etichetta_salta_quelle_usate() {
        let usate = vec!["B".to_string(), "C".to_string()];
        assert_eq!(prossima_etichetta(&usate), "D");
    }

    #[test]
    fn prossima_etichetta_case_insensitive() {
        let usate = vec!["b".to_string()];
        assert_eq!(prossima_etichetta(&usate), "C");
    }

    #[test]
    fn crea_variante_default_etichetta_b() {
        let conn = db_test();
        let id = crea_variante_pure(&conn, "prm-parent", None).unwrap();
        let (label, parent, is_variant): (Option<String>, Option<String>, i64) = conn
            .query_row(
                "SELECT VariantLabel, ParentPromptId, IsVariant FROM Prompts WHERE Id = ?1",
                [&id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(label, Some("B".to_string()));
        assert_eq!(parent, Some("prm-parent".to_string()));
        assert_eq!(is_variant, 1);
    }

    #[test]
    fn crea_variante_con_etichetta_custom() {
        let conn = db_test();
        let id = crea_variante_pure(&conn, "prm-parent", Some("Formal")).unwrap();
        let label: Option<String> = conn
            .query_row("SELECT VariantLabel FROM Prompts WHERE Id = ?1", [&id], |r| r.get(0))
            .unwrap();
        assert_eq!(label, Some("Formal".to_string()));
    }

    #[test]
    fn crea_variante_titolo_decorato_con_label() {
        let conn = db_test();
        let id = crea_variante_pure(&conn, "prm-parent", Some("B")).unwrap();
        let titolo: String = conn
            .query_row("SELECT Title FROM Prompts WHERE Id = ?1", [&id], |r| r.get(0))
            .unwrap();
        assert_eq!(titolo, "Padre (B)");
    }

    #[test]
    fn crea_variante_eredita_body_e_target() {
        let conn = db_test();
        let id = crea_variante_pure(&conn, "prm-parent", None).unwrap();
        let (body, target): (String, Option<String>) = conn
            .query_row(
                "SELECT Body, TargetModel FROM Prompts WHERE Id = ?1",
                [&id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(body, "body originale");
        assert_eq!(target, Some("claude-sonnet".to_string()));
    }

    #[test]
    fn crea_variante_eredita_tags() {
        let conn = db_test();
        let id = crea_variante_pure(&conn, "prm-parent", None).unwrap();
        let n_tag: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptTags WHERE PromptId = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_tag, 2);
    }

    #[test]
    fn crea_variante_etichetta_duplicata_e_errore() {
        let conn = db_test();
        crea_variante_pure(&conn, "prm-parent", Some("B")).unwrap();
        let r = crea_variante_pure(&conn, "prm-parent", Some("B"));
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("già usata"));
    }

    #[test]
    fn crea_variante_di_variante_riaggancia_grandparent() {
        let conn = db_test();
        let id_b = crea_variante_pure(&conn, "prm-parent", Some("B")).unwrap();
        // Crea variante della variante: deve risalire al parent reale.
        let id_c = crea_variante_pure(&conn, &id_b, Some("C")).unwrap();
        let parent: Option<String> = conn
            .query_row("SELECT ParentPromptId FROM Prompts WHERE Id = ?1", [&id_c], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(parent, Some("prm-parent".to_string()));
    }

    #[test]
    fn crea_variante_parent_inesistente_e_errore() {
        let conn = db_test();
        let r = crea_variante_pure(&conn, "prm-fantasma", None);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("non trovato"));
    }

    #[test]
    fn crea_variante_snapshot_v1_in_promptversions() {
        let conn = db_test();
        let id = crea_variante_pure(&conn, "prm-parent", None).unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM PromptVersions WHERE PromptId = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(n >= 1, "PromptVersions deve avere almeno v1 della variante");
    }

    #[test]
    fn lista_pure_ordina_per_variant_label() {
        let conn = db_test();
        crea_variante_pure(&conn, "prm-parent", Some("D")).unwrap();
        crea_variante_pure(&conn, "prm-parent", Some("B")).unwrap();
        crea_variante_pure(&conn, "prm-parent", Some("C")).unwrap();
        let l = lista_pure(&conn, "prm-parent").unwrap();
        let labels: Vec<String> = l.iter().map(|v| v.variant_label.clone()).collect();
        assert_eq!(labels, vec!["B", "C", "D"]);
    }

    #[test]
    fn lista_pure_db_senza_varianti() {
        let conn = db_test();
        let l = lista_pure(&conn, "prm-parent").unwrap();
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn lista_pure_esclude_varianti_eliminate() {
        let conn = db_test();
        let id_b = crea_variante_pure(&conn, "prm-parent", Some("B")).unwrap();
        crea_variante_pure(&conn, "prm-parent", Some("C")).unwrap();
        // Soft-delete della variante B.
        conn.execute(
            "UPDATE Prompts SET DeletedAt = datetime('now') WHERE Id = ?1",
            params![id_b],
        )
        .unwrap();
        let l = lista_pure(&conn, "prm-parent").unwrap();
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].variant_label, "C");
    }

    #[test]
    fn audit_log_registra_creazione_variante() {
        let conn = db_test();
        let id = crea_variante_pure(&conn, "prm-parent", None).unwrap();
        let azione: String = conn
            .query_row(
                "SELECT Action FROM AuditLog WHERE EntityType = 'Prompt' AND EntityId = ?1
                 ORDER BY OccurredAt DESC LIMIT 1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(azione, "variante.creata");
    }
}
