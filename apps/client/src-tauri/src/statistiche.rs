// Statistiche aggregate sul vault. Tutto client-side, zero dati escono.
//
// Scope v0.2.1: aggregazioni che NON dipendono da Fase 3 step ancora da fare
// (no embeddings, no linting, no folders, no imports). Da estendere quando
// Step 5/7/8 di Fase 3 atterrano.

use rusqlite::Connection;
use serde::Serialize;
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

#[derive(Debug, Serialize)]
pub struct Statistiche {
    pub totali: Totali,
    pub top_usati: Vec<PromptUsato>,
    pub non_usati: Vec<PromptInattivo>,
    pub per_tag: Vec<DistribuzioneTag>,
    pub per_target_model: Vec<DistribuzioneStringa>,
    pub per_visibilita: Vec<DistribuzioneStringa>,
    /// v0.6.0 Step 4: top prompt importati da altri prompt via
    /// `{{import "path"}}`. Riusa `idx_imports_imported` come grafo
    /// inverso. Vuoto se nessun import risolto.
    pub top_importati: Vec<PromptImportato>,
    /// v0.6.0 Step 4: percentuale di prompt senza issue + breakdown
    /// per categoria. Lint body-only (no IMP), aggregato.
    pub lint_health: LintHealth,
    /// V014: media char-count del Body dei prompt attivi divisa per 4
    /// (proxy token cl100k). 0 se nessun prompt attivo. UI mostra come
    /// "~N token medi". Sostituibile in futuro con tokenizer reale.
    pub token_medi: i64,
}

#[derive(Debug, Serialize)]
pub struct Totali {
    pub prompt_attivi: i64,
    pub prompt_eliminati: i64,
    pub tag_attivi: i64,
    pub creati_ultimi_30g: i64,
    pub aggiornati_ultimi_30g: i64,
    pub totale_versioni: i64,
}

#[derive(Debug, Serialize)]
pub struct PromptUsato {
    pub id: String,
    pub titolo: String,
    pub uso_count: i64,
    pub ultimo_uso: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PromptInattivo {
    pub id: String,
    pub titolo: String,
    pub aggiornato_a: String,
    pub giorni_inattivo: i64,
}

#[derive(Debug, Serialize)]
pub struct DistribuzioneTag {
    pub id: String,
    pub nome: String,
    pub colore: String,
    pub conteggio: i64,
}

#[derive(Debug, Serialize)]
pub struct DistribuzioneStringa {
    pub valore: String,
    pub conteggio: i64,
}

/// Top prompt importati da altri (grafo inverso di `PromptImports`).
/// `conteggio_importatori` = numero distinto di prompt che importano questo.
#[derive(Debug, Serialize)]
pub struct PromptImportato {
    pub id: String,
    pub titolo: String,
    pub conteggio_importatori: i64,
}

/// Salute del linter sul vault: % prompt senza issue + categorie.
/// Lint body-only (le regole IMP* richiedono il grafo, le saltiamo per
/// performance — la stima è comunque utile come "health score").
#[derive(Debug, Serialize)]
pub struct LintHealth {
    pub totale_prompt: i64,
    pub prompt_senza_issue: i64,
    /// Percentuale 0-100 con 1 decimale (`prompt_senza_issue / totale * 100`).
    /// `0.0` se totale_prompt == 0 (no prompt → no health da misurare).
    pub percentuale_health: f64,
    /// Top categorie issue (LEN, PH, PII, STY) ordinate per conteggio
    /// decrescente. Categoria estratta dal prefisso del code (es. `PH001`
    /// → `PH`). Massimo 5 entry per evitare rumore.
    pub top_categorie: Vec<DistribuzioneStringa>,
}

fn calcola_totali(conn: &Connection) -> Result<Totali, PapErrore> {
    let prompt_attivi: i64 =
        conn.query_row("SELECT COUNT(*) FROM Prompts WHERE DeletedAt IS NULL", [], |r| r.get(0))?;
    let prompt_eliminati: i64 =
        conn.query_row("SELECT COUNT(*) FROM Prompts WHERE DeletedAt IS NOT NULL", [], |r| r.get(0))?;
    let tag_attivi: i64 =
        conn.query_row("SELECT COUNT(*) FROM Tags WHERE DeletedAt IS NULL", [], |r| r.get(0))?;
    let creati_ultimi_30g: i64 = conn.query_row(
        "SELECT COUNT(*) FROM Prompts
         WHERE DeletedAt IS NULL AND CreatedAt >= datetime('now', '-30 days')",
        [],
        |r| r.get(0),
    )?;
    let aggiornati_ultimi_30g: i64 = conn.query_row(
        "SELECT COUNT(*) FROM Prompts
         WHERE DeletedAt IS NULL AND UpdatedAt >= datetime('now', '-30 days')",
        [],
        |r| r.get(0),
    )?;
    let totale_versioni: i64 =
        conn.query_row("SELECT COUNT(*) FROM PromptVersions", [], |r| r.get(0))?;

    Ok(Totali {
        prompt_attivi,
        prompt_eliminati,
        tag_attivi,
        creati_ultimi_30g,
        aggiornati_ultimi_30g,
        totale_versioni,
    })
}

fn top_usati(conn: &Connection) -> Result<Vec<PromptUsato>, PapErrore> {
    // Top 10 prompt per UseCount con almeno un uso registrato negli ultimi 30 giorni.
    let mut stmt = conn.prepare(
        "SELECT Id, Title, UseCount, LastUsedAt
         FROM Prompts
         WHERE DeletedAt IS NULL
           AND UseCount > 0
           AND (LastUsedAt IS NULL OR LastUsedAt >= datetime('now', '-30 days'))
         ORDER BY UseCount DESC, LastUsedAt DESC
         LIMIT 10",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(PromptUsato {
                id: row.get(0)?,
                titolo: row.get(1)?,
                uso_count: row.get(2)?,
                ultimo_uso: row.get::<_, Option<String>>(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

fn non_usati(conn: &Connection) -> Result<Vec<PromptInattivo>, PapErrore> {
    // Prompt che non sono usati o aggiornati da > 90 giorni — candidati a cleanup.
    let mut stmt = conn.prepare(
        "SELECT Id, Title, UpdatedAt,
                CAST((julianday('now') - julianday(COALESCE(LastUsedAt, UpdatedAt))) AS INTEGER) AS giorni
         FROM Prompts
         WHERE DeletedAt IS NULL
           AND COALESCE(LastUsedAt, UpdatedAt) < datetime('now', '-90 days')
         ORDER BY giorni DESC
         LIMIT 20",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(PromptInattivo {
                id: row.get(0)?,
                titolo: row.get(1)?,
                aggiornato_a: row.get(2)?,
                giorni_inattivo: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

fn distribuzione_tag(conn: &Connection) -> Result<Vec<DistribuzioneTag>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT t.Id, t.Name, COALESCE(t.Color, ''), COUNT(pt.PromptId) AS conteggio
         FROM Tags t
         LEFT JOIN PromptTags pt ON pt.TagId = t.Id
         LEFT JOIN Prompts p ON p.Id = pt.PromptId AND p.DeletedAt IS NULL
         WHERE t.DeletedAt IS NULL
         GROUP BY t.Id, t.Name, t.Color
         HAVING conteggio > 0
         ORDER BY conteggio DESC, t.Name COLLATE NOCASE
         LIMIT 20",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(DistribuzioneTag {
                id: row.get(0)?,
                nome: row.get(1)?,
                colore: row.get(2)?,
                conteggio: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

fn distribuzione_target_model(conn: &Connection) -> Result<Vec<DistribuzioneStringa>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(NULLIF(TRIM(TargetModel), ''), '(non specificato)') AS valore,
                COUNT(*) AS conteggio
         FROM Prompts
         WHERE DeletedAt IS NULL
         GROUP BY valore
         ORDER BY conteggio DESC, valore",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(DistribuzioneStringa {
                valore: row.get(0)?,
                conteggio: row.get(1)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

fn distribuzione_visibilita(conn: &Connection) -> Result<Vec<DistribuzioneStringa>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT Visibility, COUNT(*) AS conteggio
         FROM Prompts
         WHERE DeletedAt IS NULL
         GROUP BY Visibility
         ORDER BY conteggio DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(DistribuzioneStringa {
                valore: row.get(0)?,
                conteggio: row.get(1)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Top prompt importati da altri (v0.6.0 Step 4).
/// Riusa l'indice `idx_imports_imported` per il grafo inverso.
/// LIMIT 10 per la dashboard.
fn top_importati(conn: &Connection) -> Result<Vec<PromptImportato>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT p.Id, p.Title, COUNT(DISTINCT pi.ParentPromptId) AS conteggio
         FROM Prompts p
         JOIN PromptImports pi ON pi.ImportedPromptId = p.Id
         WHERE p.DeletedAt IS NULL
         GROUP BY p.Id, p.Title
         HAVING conteggio > 0
         ORDER BY conteggio DESC, p.Title COLLATE NOCASE
         LIMIT 10",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(PromptImportato {
                id: row.get(0)?,
                titolo: row.get(1)?,
                conteggio_importatori: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Calcola lint health body-only su tutti i prompt attivi.
/// Estrae la categoria dal prefisso del code (es. `PH001` → `PH`).
/// Top 5 categorie ordinate per conteggio decrescente.
pub(crate) fn calcola_lint_health(conn: &Connection) -> Result<LintHealth, PapErrore> {
    let bodies: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT Body FROM Prompts WHERE DeletedAt IS NULL",
        )?;
        let mut rows = stmt.query([])?;
        let mut acc: Vec<String> = Vec::new();
        while let Some(row) = rows.next()? {
            acc.push(row.get::<_, String>(0)?);
        }
        acc
    };

    let totale = bodies.len() as i64;
    let mut senza_issue: i64 = 0;
    let mut per_categoria: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();

    for body in &bodies {
        let issues = crate::linting::analizza(body);
        if issues.is_empty() {
            senza_issue += 1;
        }
        for issue in &issues {
            // Estrae prefisso lettere (es. "PH001" → "PH", "LEN002" → "LEN").
            let prefisso: String = issue
                .code
                .chars()
                .take_while(|c| c.is_ascii_alphabetic())
                .collect();
            if !prefisso.is_empty() {
                *per_categoria.entry(prefisso).or_insert(0) += 1;
            }
        }
    }

    let percentuale = if totale > 0 {
        ((senza_issue as f64 / totale as f64) * 1000.0).round() / 10.0
    } else {
        0.0
    };

    let mut top: Vec<DistribuzioneStringa> = per_categoria
        .into_iter()
        .map(|(valore, conteggio)| DistribuzioneStringa { valore, conteggio })
        .collect();
    top.sort_by(|a, b| b.conteggio.cmp(&a.conteggio).then(a.valore.cmp(&b.valore)));
    top.truncate(5);

    Ok(LintHealth {
        totale_prompt: totale,
        prompt_senza_issue: senza_issue,
        percentuale_health: percentuale,
        top_categorie: top,
    })
}

/// V014: media char-count del Body dei prompt attivi divisa per 4
/// (proxy heuristic OpenAI cl100k: 1 token ≈ 4 chars). `LENGTH(Body)`
/// in SQLite è char-count UTF-8 (NON byte-count): consistente per body
/// con caratteri multi-byte. 0 se nessun prompt attivo.
pub(crate) fn calcola_token_medi(conn: &Connection) -> Result<i64, PapErrore> {
    let avg_chars: f64 = conn.query_row(
        "SELECT COALESCE(AVG(CAST(LENGTH(Body) AS REAL)), 0.0)
         FROM Prompts WHERE DeletedAt IS NULL",
        [],
        |r| r.get(0),
    )?;
    Ok((avg_chars / 4.0).round() as i64)
}

#[tauri::command]
pub fn statistiche_query(state: State<'_, VaultState>) -> Result<Statistiche, PapErrore> {
    state.with_conn(|conn| {
        Ok(Statistiche {
            totali: calcola_totali(conn)?,
            top_usati: top_usati(conn)?,
            non_usati: non_usati(conn)?,
            per_tag: distribuzione_tag(conn)?,
            per_target_model: distribuzione_target_model(conn)?,
            per_visibilita: distribuzione_visibilita(conn)?,
            top_importati: top_importati(conn)?,
            lint_health: calcola_lint_health(conn)?,
            token_medi: calcola_token_medi(conn)?,
        })
    })
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

    fn inserisci_prompt(
        conn: &Connection,
        id: &str,
        titolo: &str,
        visibilita: &str,
        target: Option<&str>,
        uso_count: i64,
        last_used_offset_giorni: Option<i64>,
        updated_offset_giorni: i64,
    ) {
        let last_used = last_used_offset_giorni
            .map(|d| format!("datetime('now', '-{} days')", d))
            .unwrap_or_else(|| "NULL".into());
        let updated = format!("datetime('now', '-{} days')", updated_offset_giorni);
        let target_sql = target
            .map(|t| format!("'{}'", t))
            .unwrap_or_else(|| "NULL".into());
        let sql = format!(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                Visibility, TargetModel, IsFavorite, UseCount, LastUsedAt,
                Version, CreatedAt, UpdatedAt)
             VALUES ('{id}', 'ws-personale', 'usr-locale', '{titolo}', 'b',
                '{visibilita}', {target_sql}, 0, {uso_count}, {last_used}, 1,
                {updated}, {updated})"
        );
        conn.execute(&sql, []).unwrap();
    }

    #[test]
    fn totali_su_db_vuoto() {
        let conn = db_test();
        let t = calcola_totali(&conn).unwrap();
        assert_eq!(t.prompt_attivi, 0);
        assert_eq!(t.prompt_eliminati, 0);
        assert_eq!(t.tag_attivi, 0);
        assert_eq!(t.creati_ultimi_30g, 0);
    }

    #[test]
    fn totali_con_dati() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Recente", "private", Some("claude-opus"), 5, Some(1), 1);
        inserisci_prompt(&conn, "prm-2", "Vecchio", "workspace", None, 0, None, 100);
        let t = calcola_totali(&conn).unwrap();
        assert_eq!(t.prompt_attivi, 2);
        assert_eq!(t.creati_ultimi_30g, 1);
        assert_eq!(t.aggiornati_ultimi_30g, 1);
    }

    #[test]
    fn top_usati_ordina_per_uso_count() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "A", "private", None, 10, Some(1), 1);
        inserisci_prompt(&conn, "prm-2", "B", "private", None, 50, Some(2), 2);
        inserisci_prompt(&conn, "prm-3", "C", "private", None, 0, None, 5);
        let top = top_usati(&conn).unwrap();
        assert_eq!(top.len(), 2, "esclude prompt con uso_count = 0");
        assert_eq!(top[0].id, "prm-2");
        assert_eq!(top[0].uso_count, 50);
        assert_eq!(top[1].id, "prm-1");
    }

    #[test]
    fn non_usati_solo_oltre_90_giorni() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-recente", "R", "private", None, 0, None, 30);
        inserisci_prompt(&conn, "prm-vecchio", "V", "private", None, 0, None, 120);
        let inattivi = non_usati(&conn).unwrap();
        assert_eq!(inattivi.len(), 1);
        assert_eq!(inattivi[0].id, "prm-vecchio");
        assert!(inattivi[0].giorni_inattivo >= 119);
    }

    #[test]
    fn distribuzione_target_model_etichetta_vuoti() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "A", "private", Some("claude-opus"), 0, None, 1);
        inserisci_prompt(&conn, "prm-2", "B", "private", Some("claude-opus"), 0, None, 1);
        inserisci_prompt(&conn, "prm-3", "C", "private", None, 0, None, 1);
        let dist = distribuzione_target_model(&conn).unwrap();
        assert_eq!(dist.len(), 2);
        assert_eq!(dist[0].valore, "claude-opus");
        assert_eq!(dist[0].conteggio, 2);
        assert_eq!(dist[1].valore, "(non specificato)");
        assert_eq!(dist[1].conteggio, 1);
    }

    #[test]
    fn distribuzione_visibilita() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "A", "private", None, 0, None, 1);
        inserisci_prompt(&conn, "prm-2", "B", "private", None, 0, None, 1);
        inserisci_prompt(&conn, "prm-3", "C", "workspace", None, 0, None, 1);
        let dist = super::distribuzione_visibilita(&conn).unwrap();
        assert_eq!(dist.len(), 2);
        assert_eq!(dist[0].valore, "private");
        assert_eq!(dist[0].conteggio, 2);
    }

    // ─────────── v0.6.0 Step 4: top_importati ───────────

    fn inserisci_import(
        conn: &Connection,
        parent_id: &str,
        position: i64,
        imported_path: &str,
        imported_id: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO PromptImports (ParentPromptId, Position, ImportedPath, ImportedPromptId)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![parent_id, position, imported_path, imported_id],
        )
        .unwrap();
    }

    #[test]
    fn top_importati_grafo_inverso_conta_distinct_parent() {
        let conn = db_test();
        // 1 prompt target + 2 prompt che lo importano → conteggio = 2.
        inserisci_prompt(&conn, "prm-target", "Target", "private", None, 0, None, 1);
        inserisci_prompt(&conn, "prm-a", "A", "private", None, 0, None, 1);
        inserisci_prompt(&conn, "prm-b", "B", "private", None, 0, None, 1);
        inserisci_import(&conn, "prm-a", 0, "target", Some("prm-target"));
        inserisci_import(&conn, "prm-b", 0, "target", Some("prm-target"));
        // prm-a importa target una seconda volta in altra posizione →
        // COUNT(DISTINCT ParentPromptId) deve restare 2, non 3.
        inserisci_import(&conn, "prm-a", 1, "target-bis", Some("prm-target"));

        let top = super::top_importati(&conn).unwrap();
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].id, "prm-target");
        assert_eq!(top[0].conteggio_importatori, 2);
    }

    #[test]
    fn top_importati_skippa_target_eliminati() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility,
             Version, CreatedAt, UpdatedAt, DeletedAt)
             VALUES ('prm-elim', 'ws-personale', 'usr-locale', 'Eliminato', 'b', 'private', 1,
             datetime('now'), datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        inserisci_prompt(&conn, "prm-a", "A", "private", None, 0, None, 1);
        inserisci_import(&conn, "prm-a", 0, "elim", Some("prm-elim"));

        let top = super::top_importati(&conn).unwrap();
        assert!(top.is_empty(), "Il prompt eliminato non deve apparire");
    }

    #[test]
    fn top_importati_vuoto_se_nessun_import() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "Solo", "private", None, 0, None, 1);
        let top = super::top_importati(&conn).unwrap();
        assert!(top.is_empty());
    }

    #[test]
    fn top_importati_ignora_path_non_risolti() {
        // ImportedPromptId = NULL → JOIN escludente, no contributo al conteggio.
        let conn = db_test();
        inserisci_prompt(&conn, "prm-a", "A", "private", None, 0, None, 1);
        inserisci_import(&conn, "prm-a", 0, "non-esistente", None);
        let top = super::top_importati(&conn).unwrap();
        assert!(top.is_empty());
    }

    // ─────────── v0.6.0 Step 4: calcola_lint_health ───────────

    fn inserisci_prompt_con_body(conn: &Connection, id: &str, body: &str) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility,
             Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', 'T', ?2, 'private', 1,
             datetime('now'), datetime('now'))",
            rusqlite::params![id, body],
        )
        .unwrap();
    }

    #[test]
    fn lint_health_vuoto_quando_nessun_prompt() {
        let conn = db_test();
        let h = super::calcola_lint_health(&conn).unwrap();
        assert_eq!(h.totale_prompt, 0);
        assert_eq!(h.prompt_senza_issue, 0);
        assert_eq!(h.percentuale_health, 0.0);
        assert!(h.top_categorie.is_empty());
    }

    #[test]
    fn lint_health_tutti_puliti_100_percent() {
        let conn = db_test();
        // Body sopra LEN_MIN_BODY (30 char) e ben formati → 0 issue.
        inserisci_prompt_con_body(
            &conn,
            "prm-1",
            "Riformula il testo seguente in un tono cortese e professionale.",
        );
        inserisci_prompt_con_body(
            &conn,
            "prm-2",
            "Traduci la frase seguente in italiano mantenendo il senso originale.",
        );

        let h = super::calcola_lint_health(&conn).unwrap();
        assert_eq!(h.totale_prompt, 2);
        assert_eq!(h.prompt_senza_issue, 2);
        assert_eq!(h.percentuale_health, 100.0);
        assert!(h.top_categorie.is_empty());
    }

    #[test]
    fn lint_health_categorizza_per_prefisso() {
        let conn = db_test();
        // Email PII → PII001 (categoria PII), body abbastanza lungo per evitare LEN002.
        inserisci_prompt_con_body(
            &conn,
            "prm-pii",
            "Contatta mario@example.com per ulteriori informazioni in merito.",
        );
        // Body pulito sopra 30 char.
        inserisci_prompt_con_body(
            &conn,
            "prm-ok",
            "Riformula il testo in tono formale e professionale.",
        );

        let h = super::calcola_lint_health(&conn).unwrap();
        assert_eq!(h.totale_prompt, 2);
        assert_eq!(h.prompt_senza_issue, 1);
        assert_eq!(h.percentuale_health, 50.0);
        assert!(h.top_categorie.iter().any(|c| c.valore == "PII"));
    }

    #[test]
    fn token_medi_proxy_chars_diviso_4() {
        // 3 prompt: body length 16, 32, 48 chars → AVG=32 → token_medi=8.
        let conn = db_test();
        inserisci_prompt_con_body(&conn, "p1", &"a".repeat(16));
        inserisci_prompt_con_body(&conn, "p2", &"b".repeat(32));
        inserisci_prompt_con_body(&conn, "p3", &"c".repeat(48));

        assert_eq!(super::calcola_token_medi(&conn).unwrap(), 8);
    }

    #[test]
    fn token_medi_zero_se_nessun_prompt() {
        let conn = db_test();
        assert_eq!(super::calcola_token_medi(&conn).unwrap(), 0);
    }

    #[test]
    fn token_medi_ignora_prompt_eliminati() {
        // Prompt soft-deleted non contribuisce alla media.
        let conn = db_test();
        inserisci_prompt_con_body(&conn, "p-attivo", &"x".repeat(40));
        inserisci_prompt_con_body(&conn, "p-deleted", &"y".repeat(400));
        conn.execute(
            "UPDATE Prompts SET DeletedAt = datetime('now') WHERE Id = 'p-deleted'",
            [],
        )
        .unwrap();

        // AVG = 40, /4 = 10
        assert_eq!(super::calcola_token_medi(&conn).unwrap(), 10);
    }
}
