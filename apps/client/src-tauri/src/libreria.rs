use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

#[derive(Debug, Serialize, Clone)]
pub struct TagInfo {
    pub id: String,
    pub nome: String,
    pub colore: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PromptCard {
    pub id: String,
    pub titolo: String,
    pub descrizione: String,
    pub visibilita: String,
    pub preferito: bool,
    pub uso_count: i64,
    pub aggiornato_a: String,
    pub tags: Vec<TagInfo>,
    /// Issue #141: per la modalità densità "anteprima" la card mostra
    /// le prime righe del Body. Tronchiamo a 800 char server-side per
    /// limitare il payload (lista max 100 card → max ~80 KB extra).
    /// Il client poi ri-applica `righePreview` via CSS `-webkit-line-clamp`.
    pub body_preview: String,
    /// Voto medio degli ultimi 90 giorni (stessa finestra dell'ordinamento
    /// "Migliori"/qualita), in `[-1, 1]`. `None` se nessun voto nella
    /// finestra. Mostrato in lista al posto del conteggio usi quando si
    /// ordina per qualita.
    pub rating_medio: Option<f64>,
    /// #403/#412: id del prompt principale se questa card è una variante,
    /// `None` se è un principale. Il frontend lo usa per il rientro + il
    /// connettore "↳" nella lista. Va esposto qui (non solo nel dettaglio),
    /// altrimenti l'evidenziazione varianti in lista non compare.
    pub parent_prompt_id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PromptDettaglio {
    pub id: String,
    pub titolo: String,
    pub descrizione: String,
    pub body: String,
    pub visibilita: String,
    pub target_model: String,
    pub folder_id: Option<String>,
    pub preferito: bool,
    pub uso_count: i64,
    pub creato_a: String,
    pub aggiornato_a: String,
    pub ultimo_uso: String,
    pub tags: Vec<TagInfo>,
    /// M3 PR-5: id del prompt principale se questo è una variante,
    /// `None` se è il prompt principale stesso. Usato dal frontend per
    /// mostrare il bottone "Promuovi a principale" solo sulle varianti
    /// e per listare correttamente le sister anche partendo da una
    /// variante.
    pub parent_prompt_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConteggiViste {
    pub tutti: i64,
    pub preferiti: i64,
    pub privati: i64,
    pub team: i64,
    /// Prompt nel cestino (soft-deleted). Filtra `DeletedAt IS NOT NULL`,
    /// l'opposto di tutte le altre viste. (#302)
    pub cestino: i64,
}

#[derive(Debug, Deserialize)]
pub struct FiltroLista {
    pub vista: String,
    pub tag_id: Option<String>,
    pub cerca: Option<String>,
    pub ordine: String,
    #[serde(default)]
    pub target_model: Option<String>,
    /// Filtra per cartella. `Some("__nessuna__")` → solo prompt senza cartella;
    /// `Some(id)` → prompt nel sottoalbero di quella cartella; `None` → no filtro.
    #[serde(default)]
    pub folder_id: Option<String>,
}

fn carica_tags(conn: &Connection, prompt_id: &str) -> Result<Vec<TagInfo>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT t.Id, t.Name, COALESCE(t.Color, '')
         FROM Tags t
         JOIN PromptTags pt ON pt.TagId = t.Id
         WHERE pt.PromptId = ?1 AND t.DeletedAt IS NULL",
    )?;
    let tags = stmt
        .query_map([prompt_id], |row| {
            Ok(TagInfo {
                id: row.get(0)?,
                nome: row.get(1)?,
                colore: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(tags)
}

fn riga_a_card(row: &rusqlite::Row) -> rusqlite::Result<PromptCard> {
    Ok(PromptCard {
        id: row.get(0)?,
        titolo: row.get(1)?,
        descrizione: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        visibilita: row.get(3)?,
        preferito: row.get::<_, i64>(4)? != 0,
        uso_count: row.get(5)?,
        aggiornato_a: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
        tags: vec![],
        body_preview: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
        rating_medio: row.get::<_, Option<f64>>(8)?,
        parent_prompt_id: row.get::<_, Option<String>>(9)?,
    })
}

/// Inserisce workspace "Personale" e utente locale se il DB è vuoto.
pub fn assicura_dati_base(conn: &Connection) -> Result<(), PapErrore> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM Workspaces", [], |r| r.get(0))?;
    if count == 0 {
        conn.execute(
            "INSERT INTO Workspaces (Id, Name, Type, CreatedAt, UpdatedAt)
             VALUES ('ws-personale', 'Personale', 'personal', datetime('now'), datetime('now'))",
            [],
        )?;
        conn.execute(
            "INSERT INTO Users (Id, WorkspaceId, DisplayName, Role, CreatedAt, UpdatedAt)
             VALUES ('usr-locale', 'ws-personale', 'Utente locale', 'Admin', datetime('now'), datetime('now'))",
            [],
        )?;
        log::info!("Dati base creati: workspace Personale + utente locale");
    }
    Ok(())
}

pub fn conteggi_pure(conn: &Connection) -> Result<ConteggiViste, PapErrore> {
    let q = |cond: &str| -> Result<i64, PapErrore> {
        let sql = format!("SELECT COUNT(*) FROM Prompts WHERE DeletedAt IS NULL{cond}");
        Ok(conn.query_row(&sql, [], |r| r.get(0))?)
    };
    let cestino: i64 = conn.query_row(
        "SELECT COUNT(*) FROM Prompts WHERE DeletedAt IS NOT NULL",
        [],
        |r| r.get(0),
    )?;
    Ok(ConteggiViste {
        tutti: q("")?,
        preferiti: q(" AND IsFavorite = 1")?,
        privati: q(" AND Visibility = 'private'")?,
        team: q(" AND Visibility = 'workspace'")?,
        cestino,
    })
}

#[tauri::command]
pub fn libreria_conteggi(state: State<'_, VaultState>) -> Result<ConteggiViste, PapErrore> {
    state.with_conn(conteggi_pure)
}

pub fn lista_pure(
    conn: &Connection,
    filtro: &FiltroLista,
) -> Result<Vec<PromptCard>, PapErrore> {
    let cerca_param: Option<String> = filtro
            .cerca
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| format!("%{}%", s.trim()));

        let vista_cond = match filtro.vista.as_str() {
            "preferiti" => " AND p.IsFavorite = 1",
            "privati" => " AND p.Visibility = 'private'",
            "team" => " AND p.Visibility = 'workspace'",
            _ => "",
        };

        let ordine = match filtro.ordine.as_str() {
            "popolare" => "p.UseCount DESC, p.UpdatedAt DESC",
            "alfabetico" => "p.Title COLLATE NOCASE ASC",
            // Migliori: rating medio degli ultimi 90 giorni (decrescente).
            // Prompt senza rating finiscono in fondo (COALESCE = -2,
            // sotto al minimo possibile -1). A parità di media, vince chi
            // ha più voti nella stessa finestra (un 1.0 su 10 voti batte un
            // 1.0 su 1 voto). Tie-breaker finali su UseCount + UpdatedAt.
            "qualita" => {
                "COALESCE(
                    (SELECT AVG(CAST(r.Rating AS REAL))
                     FROM PromptRatings r
                     WHERE r.PromptId = p.Id
                       AND r.CreatedAt >= datetime('now', '-90 days')),
                    -2
                 ) DESC,
                 (SELECT COUNT(*)
                  FROM PromptRatings r
                  WHERE r.PromptId = p.Id
                    AND r.CreatedAt >= datetime('now', '-90 days')
                 ) DESC, p.UseCount DESC, p.UpdatedAt DESC"
            }
            _ => "COALESCE(p.LastUsedAt, p.UpdatedAt) DESC",
        };

        let target_model_param: Option<String> = filtro
            .target_model
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .cloned();

        // Filtro cartella:
        // - None / vuoto / id non trovato → nessun filtro
        // - "__nessuna__" → solo root (FolderId IS NULL)
        // - id valido → sottoalbero (id stesso + tutti i discendenti via Path)
        let folder_solo_root = filtro.folder_id.as_deref() == Some("__nessuna__");
        let folder_path: Option<String> = match filtro.folder_id.as_deref() {
            None | Some("") | Some("__nessuna__") => None,
            Some(id) => conn
                .query_row(
                    "SELECT Path FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
                    [id],
                    |r| r.get(0),
                )
                .ok(),
        };
        let folder_path_prefix_like = folder_path.as_ref().map(|p| format!("{p}/%"));

        // SQL: tutti i named param sono sempre presenti; le condizioni
        // controllano l'attivazione via NULL check / boolean flag.
        let sql = format!(
            "SELECT p.Id, p.Title, p.Description, p.Visibility,
                    p.IsFavorite, p.UseCount, p.UpdatedAt,
                    SUBSTR(COALESCE(p.Body, ''), 1, 800) AS body_preview,
                    (SELECT AVG(CAST(r.Rating AS REAL))
                     FROM PromptRatings r
                     WHERE r.PromptId = p.Id
                       AND r.CreatedAt >= datetime('now', '-90 days')) AS rating_medio,
                    p.ParentPromptId AS parent_prompt_id
             FROM Prompts p
             LEFT JOIN PromptTags pt ON pt.PromptId = p.Id
             WHERE p.DeletedAt IS NULL{vista_cond}
               AND (:cerca IS NULL OR p.Title LIKE :cerca OR p.Description LIKE :cerca)
               AND (:tag_id IS NULL OR pt.TagId = :tag_id)
               AND (:target_model IS NULL OR p.TargetModel = :target_model)
               AND (NOT :folder_solo_root OR p.FolderId IS NULL)
               AND (
                   :folder_path IS NULL
                   OR p.FolderId IN (
                       SELECT Id FROM Folders
                       WHERE DeletedAt IS NULL
                         AND (Path = :folder_path OR Path LIKE :folder_prefix)
                   )
               )
             GROUP BY p.Id
             ORDER BY {ordine}
             LIMIT 100"
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut cards: Vec<PromptCard> = stmt
            .query_map(
                rusqlite::named_params! {
                    ":cerca": cerca_param,
                    ":tag_id": filtro.tag_id,
                    ":target_model": target_model_param,
                    ":folder_solo_root": folder_solo_root,
                    ":folder_path": folder_path,
                    ":folder_prefix": folder_path_prefix_like,
                },
                riga_a_card,
            )?
            .filter_map(|r| r.ok())
            .collect();

    for card in &mut cards {
        card.tags = carica_tags(conn, &card.id)?;
    }
    Ok(cards)
}

#[tauri::command]
pub fn libreria_lista(
    filtro: FiltroLista,
    state: State<'_, VaultState>,
) -> Result<Vec<PromptCard>, PapErrore> {
    state.with_conn(|conn| lista_pure(conn, &filtro))
}

pub fn dettaglio_pure(conn: &Connection, id: &str) -> Result<PromptDettaglio, PapErrore> {
    let mut det = conn.query_row(
        "SELECT Id, Title, Description, Body, Visibility, TargetModel,
                FolderId, IsFavorite, UseCount, CreatedAt, UpdatedAt, LastUsedAt,
                ParentPromptId
         FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
        [id],
        |row| {
            Ok(PromptDettaglio {
                id: row.get(0)?,
                titolo: row.get(1)?,
                descrizione: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                body: row.get(3)?,
                visibilita: row.get(4)?,
                target_model: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                folder_id: row.get::<_, Option<String>>(6)?,
                preferito: row.get::<_, i64>(7)? != 0,
                uso_count: row.get(8)?,
                creato_a: row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                aggiornato_a: row.get::<_, Option<String>>(10)?.unwrap_or_default(),
                ultimo_uso: row.get::<_, Option<String>>(11)?.unwrap_or_default(),
                tags: vec![],
                parent_prompt_id: row.get::<_, Option<String>>(12)?,
            })
        },
    )?;
    det.tags = carica_tags(conn, &det.id)?;
    Ok(det)
}

#[tauri::command]
pub fn libreria_dettaglio(
    id: String,
    state: State<'_, VaultState>,
) -> Result<PromptDettaglio, PapErrore> {
    state.with_conn(|conn| dettaglio_pure(conn, &id))
}

pub fn toggle_preferito_pure(conn: &Connection, id: &str) -> Result<bool, PapErrore> {
    let attuale: i64 = conn.query_row(
        "SELECT IsFavorite FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
        [id],
        |r| r.get(0),
    )?;
    let nuovo = if attuale != 0 { 0 } else { 1 };
    conn.execute(
        "UPDATE Prompts SET IsFavorite = ?1, UpdatedAt = datetime('now') WHERE Id = ?2",
        rusqlite::params![nuovo, id],
    )?;
    let meta = if nuovo != 0 { "aggiunto" } else { "rimosso" };
    crate::audit::registra(conn, "prompt.preferito", "Prompt", id, Some(meta));
    Ok(nuovo != 0)
}

#[tauri::command]
pub fn libreria_toggle_preferito(
    id: String,
    state: State<'_, VaultState>,
) -> Result<bool, PapErrore> {
    state.with_conn(|conn| toggle_preferito_pure(conn, &id))
}

pub fn tag_lista_pure(conn: &Connection) -> Result<Vec<TagInfo>, PapErrore> {
    let mut stmt = conn.prepare(
        "SELECT Id, Name, COALESCE(Color, '')
         FROM Tags WHERE DeletedAt IS NULL
         ORDER BY Name COLLATE NOCASE ASC",
    )?;
    let tags = stmt
        .query_map([], |row| {
            Ok(TagInfo {
                id: row.get(0)?,
                nome: row.get(1)?,
                colore: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(tags)
}

#[tauri::command]
pub fn libreria_tag_lista(state: State<'_, VaultState>) -> Result<Vec<TagInfo>, PapErrore> {
    state.with_conn(tag_lista_pure)
}

#[cfg(test)]
mod test {
    use super::*;

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        conn
    }

    #[test]
    fn assicura_dati_base_crea_workspace_e_utente() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();

        let ws: i64 =
            conn.query_row("SELECT COUNT(*) FROM Workspaces", [], |r| r.get(0)).unwrap();
        let usr: i64 = conn.query_row("SELECT COUNT(*) FROM Users", [], |r| r.get(0)).unwrap();
        assert_eq!(ws, 1);
        assert_eq!(usr, 1);
    }

    #[test]
    fn assicura_dati_base_idempotente() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        assicura_dati_base(&conn).unwrap();

        let ws: i64 =
            conn.query_row("SELECT COUNT(*) FROM Workspaces", [], |r| r.get(0)).unwrap();
        assert_eq!(ws, 1);
    }

    #[test]
    fn conteggi_vuoti() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts WHERE DeletedAt IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn carica_tags_vuoto() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();

        let tags = carica_tags(&conn, "prm-inesistente").unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn carica_tags_con_associazione() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt)
             VALUES ('prm-1', 'ws-personale', 'usr-locale', 'Test', 'b', 'private', 1,
             datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('tag-1', 'ws-personale', 'rust', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO PromptTags (PromptId, TagId) VALUES ('prm-1', 'tag-1')",
            [],
        )
        .unwrap();

        let tags = carica_tags(&conn, "prm-1").unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].nome, "rust");
    }

    #[test]
    fn toggle_preferito() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             IsFavorite, CreatedAt, UpdatedAt)
             VALUES ('prm-1', 'ws-personale', 'usr-locale', 'Test', 'b', 'private', 1, 0,
             datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let fav: i64 = conn
            .query_row(
                "SELECT IsFavorite FROM Prompts WHERE Id = 'prm-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fav, 0);

        conn.execute(
            "UPDATE Prompts SET IsFavorite = 1 WHERE Id = 'prm-1'",
            [],
        )
        .unwrap();
        let fav: i64 = conn
            .query_row(
                "SELECT IsFavorite FROM Prompts WHERE Id = 'prm-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fav, 1);

        conn.execute(
            "UPDATE Prompts SET IsFavorite = 0 WHERE Id = 'prm-1'",
            [],
        )
        .unwrap();
        let fav: i64 = conn
            .query_row(
                "SELECT IsFavorite FROM Prompts WHERE Id = 'prm-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fav, 0);
    }

    #[test]
    fn lista_filtra_eliminati() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt)
             VALUES ('prm-att', 'ws-personale', 'usr-locale', 'Attivo', 'b', 'private', 1,
             datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt, DeletedAt)
             VALUES ('prm-del', 'ws-personale', 'usr-locale', 'Eliminato', 'b', 'private', 1,
             datetime('now'), datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts WHERE DeletedAt IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn ordine_qualita_ordina_per_rating_medio_no_rated_in_fondo() {
        // Step 5 v0.5.0: ordine "qualita" usa AVG(Rating) ultimi 90 giorni.
        // Prompt senza rating finiscono in fondo (COALESCE -2).
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();

        // 3 prompt con rating diversi.
        // prm-top: 2 rating positivi → media 1.0
        // prm-mid: rating misti −1, +1 → media 0.0
        // prm-no:  nessun rating → COALESCE -2
        for (id, titolo) in [
            ("prm-top", "Top"),
            ("prm-mid", "Mid"),
            ("prm-no", "NoRating"),
        ] {
            conn.execute(
                "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                 Visibility, Version, CreatedAt, UpdatedAt)
                 VALUES (?1, 'ws-personale', 'usr-locale', ?2, 'body', 'private', 1,
                 datetime('now'), datetime('now'))",
                rusqlite::params![id, titolo],
            )
            .unwrap();
        }

        // Rating per prm-top (+1, +1).
        for (i, voto) in [1i64, 1i64].iter().enumerate() {
            conn.execute(
                "INSERT INTO PromptRatings (Id, PromptId, UserId, Rating, CreatedAt)
                 VALUES (?1, 'prm-top', 'usr-locale', ?2, datetime('now'))",
                rusqlite::params![format!("rtg-top-{}", i), voto],
            )
            .unwrap();
        }
        // Rating per prm-mid (-1, +1).
        for (i, voto) in [-1i64, 1i64].iter().enumerate() {
            conn.execute(
                "INSERT INTO PromptRatings (Id, PromptId, UserId, Rating, CreatedAt)
                 VALUES (?1, 'prm-mid', 'usr-locale', ?2, datetime('now'))",
                rusqlite::params![format!("rtg-mid-{}", i), voto],
            )
            .unwrap();
        }

        // Esegue la stessa query SQL del ramo "qualita" per verificare l'ordine.
        let mut stmt = conn
            .prepare(
                "SELECT p.Id FROM Prompts p
                 WHERE p.DeletedAt IS NULL
                 GROUP BY p.Id
                 ORDER BY COALESCE(
                     (SELECT AVG(CAST(r.Rating AS REAL))
                      FROM PromptRatings r
                      WHERE r.PromptId = p.Id
                        AND r.CreatedAt >= datetime('now', '-90 days')),
                     -2
                 ) DESC,
                 (SELECT COUNT(*)
                  FROM PromptRatings r
                  WHERE r.PromptId = p.Id
                    AND r.CreatedAt >= datetime('now', '-90 days')
                 ) DESC, p.UseCount DESC, p.UpdatedAt DESC",
            )
            .unwrap();
        let ids: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(ids, vec!["prm-top", "prm-mid", "prm-no"]);
    }

    #[test]
    fn ordine_qualita_a_parita_di_media_vince_piu_voti() {
        // A parità di rating medio, l'ordine "qualita" mette davanti chi ha
        // più voti nella finestra di 90 giorni (più consenso = più affidabile).
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();

        // Due prompt con media identica (+1.0) ma cardinalità diversa:
        // prm-molti: 3 voti +1 → media 1.0, 3 voti
        // prm-pochi: 1 voto  +1 → media 1.0, 1 voto
        // UpdatedAt uguale così l'unico discriminante è il conteggio voti.
        for (id, titolo) in [("prm-molti", "Molti"), ("prm-pochi", "Pochi")] {
            conn.execute(
                "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                 Visibility, Version, CreatedAt, UpdatedAt)
                 VALUES (?1, 'ws-personale', 'usr-locale', ?2, 'body', 'private', 1,
                 datetime('now'), datetime('now'))",
                rusqlite::params![id, titolo],
            )
            .unwrap();
        }

        for i in 0..3 {
            conn.execute(
                "INSERT INTO PromptRatings (Id, PromptId, UserId, Rating, CreatedAt)
                 VALUES (?1, 'prm-molti', 'usr-locale', 1, datetime('now'))",
                rusqlite::params![format!("rtg-molti-{}", i)],
            )
            .unwrap();
        }
        conn.execute(
            "INSERT INTO PromptRatings (Id, PromptId, UserId, Rating, CreatedAt)
             VALUES ('rtg-pochi-0', 'prm-pochi', 'usr-locale', 1, datetime('now'))",
            [],
        )
        .unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT p.Id FROM Prompts p
                 WHERE p.DeletedAt IS NULL
                 GROUP BY p.Id
                 ORDER BY COALESCE(
                     (SELECT AVG(CAST(r.Rating AS REAL))
                      FROM PromptRatings r
                      WHERE r.PromptId = p.Id
                        AND r.CreatedAt >= datetime('now', '-90 days')),
                     -2
                 ) DESC,
                 (SELECT COUNT(*)
                  FROM PromptRatings r
                  WHERE r.PromptId = p.Id
                    AND r.CreatedAt >= datetime('now', '-90 days')
                 ) DESC, p.UseCount DESC, p.UpdatedAt DESC",
            )
            .unwrap();
        let ids: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(ids, vec!["prm-molti", "prm-pochi"]);
    }

    #[test]
    fn ordine_qualita_esclude_rating_oltre_90_giorni() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();

        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
             Visibility, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-vecchio', 'ws-personale', 'usr-locale', 'Vecchio', 'b',
             'private', 1, datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        // Rating fuori dalla finestra (180 giorni fa) — deve essere ignorato.
        conn.execute(
            "INSERT INTO PromptRatings (Id, PromptId, UserId, Rating, CreatedAt)
             VALUES ('rtg-old', 'prm-vecchio', 'usr-locale', 1, datetime('now', '-180 days'))",
            [],
        )
        .unwrap();

        let media: Option<f64> = conn
            .query_row(
                "SELECT AVG(CAST(r.Rating AS REAL))
                 FROM PromptRatings r
                 WHERE r.PromptId = 'prm-vecchio'
                   AND r.CreatedAt >= datetime('now', '-90 days')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        // Nessun rating dentro la finestra → AVG ritorna NULL.
        assert!(media.is_none());
    }

    fn inserisci_prompt(
        conn: &Connection,
        id: &str,
        titolo: &str,
        visibility: &str,
        is_favorite: i64,
        target_model: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
             Visibility, Version, IsFavorite, TargetModel, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, 'body', ?3, 1, ?4, ?5,
             datetime('now'), datetime('now'))",
            rusqlite::params![id, titolo, visibility, is_favorite, target_model],
        )
        .unwrap();
    }

    #[test]
    fn filtro_vista_preferiti_solo_favorite() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "prm-fav", "Fav", "private", 1, None);
        inserisci_prompt(&conn, "prm-no-fav", "NoFav", "private", 0, None);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts p
                 WHERE p.DeletedAt IS NULL AND p.IsFavorite = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn filtro_vista_privati_e_team() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "prm-priv", "Priv", "private", 0, None);
        inserisci_prompt(&conn, "prm-team", "Team", "workspace", 0, None);

        let count_priv: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts WHERE Visibility = 'private'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let count_team: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts WHERE Visibility = 'workspace'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count_priv, 1);
        assert_eq!(count_team, 1);
    }

    #[test]
    fn filtro_target_model_specifico() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "prm-claude", "Claude", "private", 0, Some("claude-sonnet-4-6"));
        inserisci_prompt(&conn, "prm-gpt", "GPT", "private", 0, Some("gpt-4o"));
        inserisci_prompt(&conn, "prm-no-model", "NoModel", "private", 0, None);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts WHERE TargetModel = ?1",
                rusqlite::params!["claude-sonnet-4-6"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Modello inesistente → 0 match.
        let count_null: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts WHERE TargetModel = ?1",
                rusqlite::params!["nessuno"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count_null, 0);
    }

    #[test]
    fn filtro_testo_match_title_e_description() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
             Visibility, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-1', 'ws-personale', 'usr-locale', 'Reclami', 'gestione clienti', 'b',
             'private', 1, datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Description, Body,
             Visibility, Version, CreatedAt, UpdatedAt)
             VALUES ('prm-2', 'ws-personale', 'usr-locale', 'Codice', 'review programmatica', 'b',
             'private', 1, datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        // Testo "clienti" matcha solo prm-1 via Description.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts
                 WHERE Title LIKE ?1 OR Description LIKE ?1",
                rusqlite::params!["%clienti%"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    fn filtro_default(vista: &str) -> FiltroLista {
        FiltroLista {
            vista: vista.to_string(),
            tag_id: None,
            cerca: None,
            ordine: "recente".to_string(),
            target_model: None,
            folder_id: None,
        }
    }

    #[test]
    fn conteggi_pure_vuoto() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        let c = conteggi_pure(&conn).unwrap();
        assert_eq!(c.tutti, 0);
        assert_eq!(c.preferiti, 0);
        assert_eq!(c.privati, 0);
        assert_eq!(c.team, 0);
        assert_eq!(c.cestino, 0);
    }

    #[test]
    fn conteggi_pure_cestino_conta_solo_cancellati() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "vivo", "Vivo", "private", 0, None);
        inserisci_prompt(&conn, "morto", "Morto", "private", 0, None);
        conn.execute(
            "UPDATE Prompts SET DeletedAt = datetime('now') WHERE Id = 'morto'",
            [],
        )
        .unwrap();
        let c = conteggi_pure(&conn).unwrap();
        assert_eq!(c.tutti, 1, "le viste vive escludono i cancellati");
        assert_eq!(c.cestino, 1, "il cestino conta solo i cancellati");
    }

    #[test]
    fn conteggi_pure_separa_per_visibilita_e_preferiti() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "a", "A", "private", 1, None);
        inserisci_prompt(&conn, "b", "B", "private", 0, None);
        inserisci_prompt(&conn, "c", "C", "workspace", 1, None);
        let c = conteggi_pure(&conn).unwrap();
        assert_eq!(c.tutti, 3);
        assert_eq!(c.preferiti, 2);
        assert_eq!(c.privati, 2);
        assert_eq!(c.team, 1);
    }

    #[test]
    fn lista_pure_default_ritorna_tutti_attivi() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "p1", "Uno", "private", 0, None);
        inserisci_prompt(&conn, "p2", "Due", "workspace", 0, None);
        let cards = lista_pure(&conn, &filtro_default("tutti")).unwrap();
        assert_eq!(cards.len(), 2);
    }

    #[test]
    fn lista_pure_espone_rating_medio_finestra_90gg() {
        // La lista restituisce il voto medio degli ultimi 90 giorni così che
        // il frontend possa mostrarlo al posto del conteggio usi quando si
        // ordina per "Migliori". Stessa finestra/semantica dell'ORDER BY.
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "p-voti", "Con voti", "private", 0, None);
        inserisci_prompt(&conn, "p-senza", "Senza voti", "private", 0, None);
        inserisci_prompt(&conn, "p-vecchio", "Voto vecchio", "private", 0, None);

        // p-voti: +1, +1, 0 → media 0.6667 nella finestra
        for (i, r) in [1, 1, 0].iter().enumerate() {
            conn.execute(
                "INSERT INTO PromptRatings (Id, PromptId, UserId, Rating, CreatedAt)
                 VALUES (?1, 'p-voti', 'usr-locale', ?2, datetime('now'))",
                rusqlite::params![format!("rtg-{i}"), r],
            )
            .unwrap();
        }
        // p-vecchio: un voto oltre i 90 giorni → fuori finestra → None
        conn.execute(
            "INSERT INTO PromptRatings (Id, PromptId, UserId, Rating, CreatedAt)
             VALUES ('rtg-old', 'p-vecchio', 'usr-locale', 1, datetime('now', '-100 days'))",
            [],
        )
        .unwrap();

        let cards = lista_pure(&conn, &filtro_default("tutti")).unwrap();
        let trova = |id: &str| cards.iter().find(|c| c.id == id).unwrap();

        let media = trova("p-voti").rating_medio.expect("media presente");
        assert!((media - 2.0 / 3.0).abs() < 1e-6, "media attesa ~0.667, vista {media}");
        assert!(
            trova("p-senza").rating_medio.is_none(),
            "nessun voto → None"
        );
        assert!(
            trova("p-vecchio").rating_medio.is_none(),
            "voto oltre 90 giorni escluso dalla finestra → None"
        );
    }

    #[test]
    fn lista_pure_espone_parent_prompt_id_per_varianti() {
        // #412: il connettore varianti in lista (rientro + "↳") legge
        // parent_prompt_id dalla query lista, non solo dal dettaglio. Verifica
        // che la card della variante riporti l'id del principale e che il
        // principale resti None.
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "p-principale", "Principale", "private", 0, None);
        inserisci_prompt(&conn, "p-variante", "Variante", "private", 0, None);
        conn.execute(
            "UPDATE Prompts SET ParentPromptId = 'p-principale' WHERE Id = 'p-variante'",
            [],
        )
        .unwrap();

        let cards = lista_pure(&conn, &filtro_default("tutti")).unwrap();
        let trova = |id: &str| cards.iter().find(|c| c.id == id).unwrap();
        assert_eq!(
            trova("p-variante").parent_prompt_id.as_deref(),
            Some("p-principale"),
            "la variante espone l'id del principale"
        );
        assert!(
            trova("p-principale").parent_prompt_id.is_none(),
            "il principale non ha parent"
        );
    }

    #[test]
    fn lista_pure_vista_preferiti_filtra() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "p1", "Fav", "private", 1, None);
        inserisci_prompt(&conn, "p2", "NoFav", "private", 0, None);
        let cards = lista_pure(&conn, &filtro_default("preferiti")).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "p1");
    }

    #[test]
    fn lista_pure_cerca_match_title() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "p1", "Reclami clienti", "private", 0, None);
        inserisci_prompt(&conn, "p2", "Altro", "private", 0, None);
        let mut f = filtro_default("tutti");
        f.cerca = Some("clienti".to_string());
        let cards = lista_pure(&conn, &f).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "p1");
    }

    #[test]
    fn lista_pure_target_model_filtra() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "p1", "Claude", "private", 0, Some("claude-sonnet"));
        inserisci_prompt(&conn, "p2", "GPT", "private", 0, Some("gpt-4"));
        let mut f = filtro_default("tutti");
        f.target_model = Some("claude-sonnet".to_string());
        let cards = lista_pure(&conn, &f).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "p1");
    }

    #[test]
    fn lista_pure_body_preview_troncato_a_800() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        let body = "x".repeat(2000);
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
             Visibility, Version, CreatedAt, UpdatedAt)
             VALUES ('p1', 'ws-personale', 'usr-locale', 'Lungo', ?1,
             'private', 1, datetime('now'), datetime('now'))",
            rusqlite::params![body],
        )
        .unwrap();
        let cards = lista_pure(&conn, &filtro_default("tutti")).unwrap();
        assert_eq!(cards[0].body_preview.len(), 800);
    }

    #[test]
    fn dettaglio_pure_ritorna_prompt_e_tag() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version,
             CreatedAt, UpdatedAt)
             VALUES ('p1', 'ws-personale', 'usr-locale', 'Hello', 'world', 'private', 1,
             datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('t1', 'ws-personale', 'rust', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO PromptTags (PromptId, TagId) VALUES ('p1', 't1')",
            [],
        )
        .unwrap();
        let det = dettaglio_pure(&conn, "p1").unwrap();
        assert_eq!(det.id, "p1");
        assert_eq!(det.titolo, "Hello");
        assert_eq!(det.body, "world");
        assert_eq!(det.tags.len(), 1);
        assert_eq!(det.tags[0].nome, "rust");
    }

    #[test]
    fn dettaglio_pure_prompt_inesistente_errore() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        assert!(dettaglio_pure(&conn, "non-esiste").is_err());
    }

    #[test]
    fn toggle_preferito_pure_inverte_stato() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        inserisci_prompt(&conn, "p1", "T", "private", 0, None);
        let nuovo = toggle_preferito_pure(&conn, "p1").unwrap();
        assert!(nuovo);
        let nuovo = toggle_preferito_pure(&conn, "p1").unwrap();
        assert!(!nuovo);
    }

    #[test]
    fn tag_lista_pure_ordina_e_filtra_eliminati() {
        let conn = db_test();
        assicura_dati_base(&conn).unwrap();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, CreatedAt, UpdatedAt)
             VALUES ('t1', 'ws-personale', 'zebra', datetime('now'), datetime('now')),
                    ('t2', 'ws-personale', 'alpha', datetime('now'), datetime('now')),
                    ('t3', 'ws-personale', 'cancel', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE Tags SET DeletedAt = datetime('now') WHERE Id = 't3'",
            [],
        )
        .unwrap();
        let tags = tag_lista_pure(&conn).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].nome, "alpha");
        assert_eq!(tags[1].nome, "zebra");
    }
}
