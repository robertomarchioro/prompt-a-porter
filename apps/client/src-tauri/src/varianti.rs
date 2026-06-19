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
// - prompt_promuovi_variante(variant_id): swap main <-> variant (M3 PR-5)
//
// Out of scope (sub-step futuri):
// - Migration backreference su promozione: gli `{{import "id-vecchia-main"}}`
//   restano puntati alla vecchia main (ora variante). Una migration
//   automatica che riscrive tutti gli import nei prompt terzi richiede
//   decisione semantica (silenzioso vs notifica utente) -> backlog M3.x.
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

fn genera_id() -> Result<String, PapErrore> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut rnd = [0u8; 4];
    crate::util_random::riempi_random(&mut rnd)?;
    Ok(format!(
        "prm-{:012x}{:02x}{:02x}{:02x}{:02x}",
        ts, rnd[0], rnd[1], rnd[2], rnd[3]
    ))
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

    let id = genera_id()?;
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

/// Logica pura di `prompt_promuovi_variante` (M3 PR-5).
///
/// Swap main <-> variant:
/// 1. `variant_id` deve essere una variante (ParentPromptId != NULL),
///    altrimenti errore "non e' una variante".
/// 2. `old_main` = `variant.ParentPromptId`.
/// 3. Tutte le altre sister della variante (escluso variant_id) vengono
///    ri-parentate al nuovo main: `SET ParentPromptId = variant_id
///    WHERE ParentPromptId = old_main AND Id != variant_id`.
/// 4. `old_main`: diventa variante del nuovo main con label assegnata
///    come prossima libera fra le sister post-swap (a/b/c...). La sua
///    `VariantLabel` precedente era NULL (era main).
/// 5. `variant_id`: diventa main: `ParentPromptId = NULL, IsVariant = 0,
///    VariantLabel = NULL`.
///
/// Backreference NON migrati: gli `{{import "old_main_id"}}` continuano
/// a puntare alla vecchia main (ora variante). Backlog M3.x per migration
/// automatica con scelta semantica (vedi header modulo).
pub(crate) fn promuovi_pure(
    conn: &Connection,
    variant_id: &str,
) -> Result<(), PapErrore> {
    // 1. Verifica che sia una variante (ParentPromptId != NULL)
    let parent_id: Option<String> = conn
        .query_row(
            "SELECT ParentPromptId FROM Prompts
             WHERE Id = ?1 AND DeletedAt IS NULL",
            [variant_id],
            |r| r.get(0),
        )
        .map_err(|_| PapErrore::Generico(format!("Prompt '{variant_id}' non trovato")))?;
    let old_main_id = parent_id.ok_or_else(|| {
        PapErrore::Generico(format!(
            "Prompt '{variant_id}' non e' una variante (e' gia' il principale)"
        ))
    })?;

    // I tre UPDATE che seguono sono uno swap atomico: un errore a metà
    // lascerebbe i prompt in stato corrotto (half-swap). BEGIN/COMMIT, con
    // ROLLBACK best-effort su errore. (SQLite accetta BEGIN/COMMIT come
    // statement plain, quindi basta &Connection.)
    conn.execute_batch("BEGIN")?;
    let res = (|| -> Result<(), PapErrore> {
        // 3. Ri-aggancia le sister al nuovo main (variant_id).
        //    Esclude variant_id stesso (sta passando a main, non variante).
        conn.execute(
            "UPDATE Prompts
             SET ParentPromptId = ?1
             WHERE ParentPromptId = ?2 AND Id != ?1 AND DeletedAt IS NULL",
            params![variant_id, old_main_id],
        )?;

        // 4a. Calcola label per ex-main: prossima libera fra le sister
        //     attuali (dopo lo step 3). Le sister ora includono tutti i
        //     fratelli + l'ex-main che sta per diventare variante. La
        //     variante promossa NON e' piu' sister (sta per essere main).
        let usate_dopo_swap = etichette_usate(conn, variant_id)?;
        let label_old_main = prossima_etichetta(&usate_dopo_swap);

        // 4b. Old main: diventa variante.
        conn.execute(
            "UPDATE Prompts
             SET ParentPromptId = ?1, IsVariant = 1, VariantLabel = ?2,
                 UpdatedAt = datetime('now')
             WHERE Id = ?3 AND DeletedAt IS NULL",
            params![variant_id, label_old_main, old_main_id],
        )?;

        // 5. Variante promossa: diventa main.
        conn.execute(
            "UPDATE Prompts
             SET ParentPromptId = NULL, IsVariant = 0, VariantLabel = NULL,
                 UpdatedAt = datetime('now')
             WHERE Id = ?1 AND DeletedAt IS NULL",
            params![variant_id],
        )?;
        Ok(())
    })();

    match res {
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

#[tauri::command]
pub fn prompt_promuovi_variante(
    variant_id: String,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| promuovi_pure(conn, &variant_id))?;
    // FTS: body invariato per entrambi i prompt -> non serve ricostruire.
    // Embedding: idem (l'embedding e' funzione del body, non della
    // relazione parent/variant). Nessun hook post-swap necessario.
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

    // ─────────── Stress test (Step 9 quality gate Fase 4) ───────────

    #[test]
    fn stress_25_varianti_riempie_tutte_le_lettere_b_z() {
        // Sentinel: con etichetta=None il generatore deve produrre B,
        // C, ..., Z (25 lettere). Esaurito Z, l'auto-generate deve
        // passare al fallback "V<N>".
        let conn = db_test();
        for _ in 0..25 {
            crea_variante_pure(&conn, "prm-parent", None).unwrap();
        }
        let l = lista_pure(&conn, "prm-parent").unwrap();
        assert_eq!(l.len(), 25);
        let labels: std::collections::HashSet<String> =
            l.iter().map(|v| v.variant_label.clone()).collect();
        // Tutte le lettere B-Z presenti, nessuna A (riservata al principale).
        for c in b'B'..=b'Z' {
            let lbl = (c as char).to_string();
            assert!(labels.contains(&lbl), "etichetta '{lbl}' mancante");
        }
        assert!(!labels.contains(&"A".to_string()));
    }

    #[test]
    fn stress_oltre_25_varianti_fallback_vn() {
        let conn = db_test();
        for _ in 0..25 {
            crea_variante_pure(&conn, "prm-parent", None).unwrap();
        }
        // 26ª variante: nessuna lettera B-Z disponibile → "V<N>".
        let id_extra = crea_variante_pure(&conn, "prm-parent", None).unwrap();
        let label: Option<String> = conn
            .query_row(
                "SELECT VariantLabel FROM Prompts WHERE Id = ?1",
                [&id_extra],
                |r| r.get(0),
            )
            .unwrap();
        // Format è "V<N>" con N = numero usate (25) + 1 = 26.
        assert!(label.unwrap().starts_with('V'));
    }

    #[test]
    fn stress_lista_grande_resta_ordinata() {
        // 100 varianti con etichette custom ordinabili lessicograficamente.
        let conn = db_test();
        // Genero etichette pad-zero ("L001"..."L100") così l'ORDER BY
        // ASC alfabetico è prevedibile.
        for i in 0..100 {
            let label = format!("L{:03}", i);
            crea_variante_pure(&conn, "prm-parent", Some(&label)).unwrap();
        }
        let l = lista_pure(&conn, "prm-parent").unwrap();
        assert_eq!(l.len(), 100);
        // Verifica ordinamento monotono crescente.
        for i in 1..l.len() {
            assert!(
                l[i - 1].variant_label <= l[i].variant_label,
                "ordinamento non monotono fra {} e {}",
                l[i - 1].variant_label,
                l[i].variant_label
            );
        }
    }

    // ─── M3 PR-5: promuovi_pure ────────────────────────────────────

    fn stato_prompt(conn: &Connection, id: &str) -> (Option<String>, i64, Option<String>) {
        conn.query_row(
            "SELECT ParentPromptId, IsVariant, VariantLabel FROM Prompts WHERE Id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap()
    }

    #[test]
    fn promuovi_swap_main_variante_senza_sister() {
        let conn = db_test();
        // crea variante "B" del main
        let v_id = crea_variante_pure(&conn, "prm-parent", Some("B")).unwrap();

        promuovi_pure(&conn, &v_id).unwrap();

        // V e' nuovo main
        let (parent, is_var, label) = stato_prompt(&conn, &v_id);
        assert!(parent.is_none(), "variante promossa deve avere ParentPromptId NULL");
        assert_eq!(is_var, 0);
        assert!(label.is_none());

        // prm-parent e' ora variante con label "B" (prossima libera, no sister)
        let (parent, is_var, label) = stato_prompt(&conn, "prm-parent");
        assert_eq!(parent.as_deref(), Some(v_id.as_str()));
        assert_eq!(is_var, 1);
        assert_eq!(label.as_deref(), Some("B"));
    }

    #[test]
    fn promuovi_swap_con_sister_rep_arenta() {
        let conn = db_test();
        // Main prm-parent + 3 varianti B, C, D
        let v_b = crea_variante_pure(&conn, "prm-parent", Some("B")).unwrap();
        let v_c = crea_variante_pure(&conn, "prm-parent", Some("C")).unwrap();
        let v_d = crea_variante_pure(&conn, "prm-parent", Some("D")).unwrap();

        // Promuovi C -> diventa main; B/D devono diventare sister di C;
        // prm-parent diventa variante di C con label "B" (prima libera
        // fra sister attuali: B, D usate -> B viene riusato? No,
        // prossima_etichetta scorre B..Z saltando le usate. Sister di
        // C dopo swap = {prm-parent (in arrivo), v_b, v_d}. Labels
        // gia' usate fra v_b/v_d = "B", "D". prossima_etichetta
        // restituisce "C" (la prima libera fra B/C/D... salta B,
        // restituisce C). Quindi prm-parent prende "C".
        promuovi_pure(&conn, &v_c).unwrap();

        // C e' main
        let (parent, is_var, label) = stato_prompt(&conn, &v_c);
        assert!(parent.is_none());
        assert_eq!(is_var, 0);
        assert!(label.is_none());

        // B e D ora puntano a C come parent
        let (parent_b, is_var_b, label_b) = stato_prompt(&conn, &v_b);
        assert_eq!(parent_b.as_deref(), Some(v_c.as_str()));
        assert_eq!(is_var_b, 1);
        assert_eq!(label_b.as_deref(), Some("B"));

        let (parent_d, is_var_d, label_d) = stato_prompt(&conn, &v_d);
        assert_eq!(parent_d.as_deref(), Some(v_c.as_str()));
        assert_eq!(is_var_d, 1);
        assert_eq!(label_d.as_deref(), Some("D"));

        // prm-parent ora variante di C; label C (prima libera saltando B/D)
        let (parent_p, is_var_p, label_p) = stato_prompt(&conn, "prm-parent");
        assert_eq!(parent_p.as_deref(), Some(v_c.as_str()));
        assert_eq!(is_var_p, 1);
        assert_eq!(label_p.as_deref(), Some("C"));
    }

    #[test]
    fn promuovi_errore_se_main() {
        let conn = db_test();
        let err = promuovi_pure(&conn, "prm-parent").unwrap_err();
        assert!(format!("{err}").contains("non e' una variante"));
    }

    #[test]
    fn promuovi_errore_se_inesistente() {
        let conn = db_test();
        let err = promuovi_pure(&conn, "prm-non-esiste").unwrap_err();
        assert!(format!("{err}").contains("non trovato"));
    }

    #[test]
    fn promuovi_preserva_body_e_titolo() {
        let conn = db_test();
        let v_id = crea_variante_pure(&conn, "prm-parent", Some("B")).unwrap();
        // Modifica body della variante per distinguerla
        conn.execute(
            "UPDATE Prompts SET Body = 'body variante personalizzato',
             Title = 'Titolo variante' WHERE Id = ?1",
            [&v_id],
        )
        .unwrap();

        promuovi_pure(&conn, &v_id).unwrap();

        // Body/Title della variante (ora main) preservati
        let (body, title): (String, String) = conn
            .query_row(
                "SELECT Body, Title FROM Prompts WHERE Id = ?1",
                [&v_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(body, "body variante personalizzato");
        assert_eq!(title, "Titolo variante");

        // Body/Title del vecchio main (ora variante) preservati
        let (body, title): (String, String) = conn
            .query_row(
                "SELECT Body, Title FROM Prompts WHERE Id = 'prm-parent'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(body, "body originale");
        assert_eq!(title, "Padre");
    }
}
