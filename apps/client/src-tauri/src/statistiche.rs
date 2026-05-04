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
        })
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn db_test() -> Connection {
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
}
