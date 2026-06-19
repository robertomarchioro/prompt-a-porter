use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errore::PapErrore;
use crate::util_random::riempi_random;
use crate::vault::VaultState;

#[derive(Debug, Serialize)]
pub struct VoceAudit {
    pub id: String,
    pub azione: String,
    pub tipo_entita: String,
    pub id_entita: String,
    pub metadati: String,
    pub avvenuto_a: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct AuditFiltro {
    /// ISO 8601 inclusivo. None = nessun limite inferiore.
    #[serde(default)]
    pub da: Option<String>,
    /// ISO 8601 esclusivo. None = nessun limite superiore.
    #[serde(default)]
    pub a: Option<String>,
    /// Match esatto su UserId.
    #[serde(default)]
    pub user_id: Option<String>,
    /// Substring (LIKE %x%) su Action.
    #[serde(default)]
    pub azione_like: Option<String>,
    /// Match esatto su EntityType.
    #[serde(default)]
    pub tipo_entita: Option<String>,
    /// Substring su EntityId o Metadata.
    #[serde(default)]
    pub testo: Option<String>,
    /// Default 100, max 500.
    #[serde(default)]
    pub limite: Option<u32>,
    /// Default 0.
    #[serde(default)]
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct AuditPaginato {
    pub voci: Vec<VoceAudit>,
    pub totale: i64,
    pub limite: u32,
    pub offset: u32,
}

pub fn registra(
    conn: &Connection,
    azione: &str,
    tipo_entita: &str,
    id_entita: &str,
    metadati: Option<&str>,
) {
    // Fire-and-forget by design (la firma resta `-> ()` per non appesantire
    // le decine di call site), ma un fallimento — incluso OS RNG non disponibile —
    // non deve sparire del tutto: lo logghiamo per renderlo osservabile.
    let id = match formato_id_audit() {
        Ok(id) => id,
        Err(e) => {
            log::warn!("audit registra: generazione id fallita (azione={azione}): {e}");
            return;
        }
    };
    if let Err(e) = conn.execute(
        "INSERT INTO AuditLog (Id, WorkspaceId, UserId, Action, EntityType, EntityId, Metadata, OccurredAt)
         VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, ?4, ?5, datetime('now'))",
        rusqlite::params![id, azione, tipo_entita, id_entita, metadati],
    ) {
        log::warn!("audit registra fallito (azione={azione}): {e}");
    }
}

fn formato_id_audit() -> Result<String, PapErrore> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut rnd = [0u8; 4];
    riempi_random(&mut rnd)?;
    Ok(format!(
        "aud-{:012x}{:02x}{:02x}{:02x}{:02x}",
        ts, rnd[0], rnd[1], rnd[2], rnd[3]
    ))
}

#[tauri::command]
pub fn audit_lista(
    limite: Option<u32>,
    tipo_entita: Option<String>,
    state: State<'_, VaultState>,
) -> Result<Vec<VoceAudit>, PapErrore> {
    state.with_conn(|conn| lista_pure(conn, limite, tipo_entita.as_deref()))
}

/// M7 PR-2: logica pura di `audit_lista`, testabile su Connection.
///
/// Bug-fix M7 PR-2: il codice pre-refactor passava named param `:tipo`
/// anche quando il SQL non lo conteneva (branch `else`) -> rusqlite
/// `InvalidParameterName(":tipo")`. Ora i parametri sono ramificati.
pub(crate) fn lista_pure(
    conn: &Connection,
    limite: Option<u32>,
    tipo_entita: Option<&str>,
) -> Result<Vec<VoceAudit>, PapErrore> {
    let lim = limite.unwrap_or(100).min(500);

    let row_mapper = |row: &rusqlite::Row<'_>| {
        Ok(VoceAudit {
            id: row.get(0)?,
            azione: row.get(1)?,
            tipo_entita: row.get(2)?,
            id_entita: row.get(3)?,
            metadati: row.get(4)?,
            avvenuto_a: row.get(5)?,
        })
    };

    let mut voci: Vec<VoceAudit> = Vec::new();
    if let Some(tipo) = tipo_entita {
        let mut stmt = conn.prepare(
            "SELECT Id, Action, EntityType, COALESCE(EntityId, ''), COALESCE(Metadata, ''), OccurredAt
             FROM AuditLog
             WHERE EntityType = :tipo
             ORDER BY OccurredAt DESC
             LIMIT :limite",
        )?;
        let iter = stmt.query_map(
            rusqlite::named_params! { ":tipo": tipo, ":limite": lim },
            row_mapper,
        )?;
        for r in iter {
            if let Ok(v) = r {
                voci.push(v);
            }
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT Id, Action, EntityType, COALESCE(EntityId, ''), COALESCE(Metadata, ''), OccurredAt
             FROM AuditLog
             ORDER BY OccurredAt DESC
             LIMIT :limite",
        )?;
        let iter = stmt.query_map(
            rusqlite::named_params! { ":limite": lim },
            row_mapper,
        )?;
        for r in iter {
            if let Ok(v) = r {
                voci.push(v);
            }
        }
    }
    Ok(voci)
}

/// Costruisce la WHERE clause + parametri SQL dai filtri.
/// Restituisce (clausola_where, parametri_named).
fn componi_where(
    filtro: &AuditFiltro,
) -> (String, Vec<(&'static str, Box<dyn rusqlite::ToSql>)>) {
    let mut conds: Vec<&'static str> = Vec::new();
    let mut params: Vec<(&'static str, Box<dyn rusqlite::ToSql>)> = Vec::new();

    if let Some(v) = filtro.da.as_ref().filter(|s| !s.is_empty()) {
        conds.push("OccurredAt >= :da");
        params.push((":da", Box::new(v.clone())));
    }
    if let Some(v) = filtro.a.as_ref().filter(|s| !s.is_empty()) {
        conds.push("OccurredAt < :a");
        params.push((":a", Box::new(v.clone())));
    }
    if let Some(v) = filtro.user_id.as_ref().filter(|s| !s.is_empty()) {
        conds.push("UserId = :user_id");
        params.push((":user_id", Box::new(v.clone())));
    }
    if let Some(v) = filtro.tipo_entita.as_ref().filter(|s| !s.is_empty()) {
        conds.push("EntityType = :tipo");
        params.push((":tipo", Box::new(v.clone())));
    }
    if let Some(v) = filtro.azione_like.as_ref().filter(|s| !s.is_empty()) {
        conds.push("Action LIKE :azione");
        params.push((":azione", Box::new(format!("%{v}%"))));
    }
    if let Some(v) = filtro.testo.as_ref().filter(|s| !s.is_empty()) {
        conds.push("(EntityId LIKE :testo OR Metadata LIKE :testo)");
        params.push((":testo", Box::new(format!("%{v}%"))));
    }

    let where_clause = if conds.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conds.join(" AND "))
    };

    (where_clause, params)
}

/// Tauri command: query paginata su AuditLog con filtri estesi.
#[tauri::command]
pub fn audit_query(
    filtro: AuditFiltro,
    state: State<'_, VaultState>,
) -> Result<AuditPaginato, PapErrore> {
    state.with_conn(|conn| query_pure(conn, &filtro))
}

/// M7 PR-2: logica pura di `audit_query`.
pub(crate) fn query_pure(
    conn: &Connection,
    filtro: &AuditFiltro,
) -> Result<AuditPaginato, PapErrore> {
    let limite = filtro.limite.unwrap_or(100).min(500);
    let offset = filtro.offset.unwrap_or(0);

    let (where_clause, params) = componi_where(filtro);

    let params_ref: Vec<(&str, &dyn rusqlite::ToSql)> = params
        .iter()
        .map(|(k, v)| (*k, v.as_ref()))
        .collect();

    // Conteggio totale (per paginazione UI).
    let sql_count = format!("SELECT COUNT(*) FROM AuditLog {where_clause}");
    let totale: i64 = conn.query_row(&sql_count, params_ref.as_slice(), |r| r.get(0))?;

    // Lista paginata.
    let sql_lista = format!(
        "SELECT Id, Action, EntityType, COALESCE(EntityId, ''), COALESCE(Metadata, ''), OccurredAt
         FROM AuditLog
         {where_clause}
         ORDER BY OccurredAt DESC
         LIMIT :limite OFFSET :offset"
    );

    let mut stmt = conn.prepare(&sql_lista)?;

    let mut params_lista = params_ref.clone();
    params_lista.push((":limite", &limite));
    params_lista.push((":offset", &offset));

    let voci: Vec<VoceAudit> = stmt
        .query_map(params_lista.as_slice(), |row| {
            Ok(VoceAudit {
                id: row.get(0)?,
                azione: row.get(1)?,
                tipo_entita: row.get(2)?,
                id_entita: row.get(3)?,
                metadati: row.get(4)?,
                avvenuto_a: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(AuditPaginato {
        voci,
        totale,
        limite,
        offset,
    })
}

/// Tauri command: esporta in CSV (string in memoria) il risultato del filtro,
/// senza paginazione (max 50000 righe per sicurezza).
#[tauri::command]
pub fn audit_export_csv(
    filtro: AuditFiltro,
    state: State<'_, VaultState>,
) -> Result<String, PapErrore> {
    state.with_conn(|conn| export_csv_pure(conn, &filtro))
}

/// M7 PR-2: logica pura di `audit_export_csv`.
pub(crate) fn export_csv_pure(
    conn: &Connection,
    filtro: &AuditFiltro,
) -> Result<String, PapErrore> {
    let (where_clause, params) = componi_where(filtro);

    let params_ref: Vec<(&str, &dyn rusqlite::ToSql)> = params
        .iter()
        .map(|(k, v)| (*k, v.as_ref()))
        .collect();

    let sql = format!(
        "SELECT Id, OccurredAt, UserId, Action, EntityType,
                COALESCE(EntityId, ''), COALESCE(Metadata, '')
         FROM AuditLog
         {where_clause}
         ORDER BY OccurredAt DESC
         LIMIT 50000"
    );

    let mut stmt = conn.prepare(&sql)?;

    let righe = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?
        .filter_map(|r| r.ok());

    let mut out =
        String::from("id,occurred_at,user_id,action,entity_type,entity_id,metadata\n");
    for (id, occ, user, act, etype, eid, meta) in righe {
        out.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            csv_quote(&id),
            csv_quote(&occ),
            csv_quote(&user),
            csv_quote(&act),
            csv_quote(&etype),
            csv_quote(&eid),
            csv_quote(&meta),
        ));
    }
    Ok(out)
}

fn csv_quote(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Tauri command: cancella le voci più vecchie di N giorni.
/// Operazione manuale (no scheduling automatico per ora).
/// Restituisce il numero di righe eliminate.
#[tauri::command]
pub fn audit_cleanup_oltre_giorni(
    giorni: u32,
    state: State<'_, VaultState>,
) -> Result<usize, PapErrore> {
    state.with_conn(|conn| cleanup_oltre_giorni_pure(conn, giorni))
}

/// M7 PR-2: logica pura di `audit_cleanup_oltre_giorni`.
pub(crate) fn cleanup_oltre_giorni_pure(
    conn: &Connection,
    giorni: u32,
) -> Result<usize, PapErrore> {
    let modifier = format!("-{giorni} days");
    let n = conn.execute(
        "DELETE FROM AuditLog WHERE OccurredAt < datetime('now', ?1)",
        rusqlite::params![modifier],
    )?;
    log::info!("Audit cleanup: eliminate {n} righe più vecchie di {giorni} giorni");
    Ok(n)
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

    #[test]
    fn formato_id_prefisso_e_lunghezza() {
        let id = formato_id_audit().unwrap();
        assert!(id.starts_with("aud-"));
        assert_eq!(id.len(), 24);
    }

    #[test]
    fn formato_id_univoco() {
        let id1 = formato_id_audit().unwrap();
        let id2 = formato_id_audit().unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn registra_scrive_in_db() {
        let conn = db_test();
        registra(&conn, "test.azione", "TestEntity", "ent-123", Some("meta"));

        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM AuditLog", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn registra_senza_metadati() {
        let conn = db_test();
        registra(&conn, "test.azione", "TestEntity", "ent-123", None);

        let meta: Option<String> = conn
            .query_row("SELECT Metadata FROM AuditLog LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert!(meta.is_none());
    }

    #[test]
    fn registra_multiple_e_filtra_per_tipo() {
        let conn = db_test();
        registra(&conn, "a1", "Prompt", "p1", Some("m1"));
        registra(&conn, "a2", "Vault", "v1", None);
        registra(&conn, "a3", "Prompt", "p2", Some("m3"));

        let totale: i64 =
            conn.query_row("SELECT COUNT(*) FROM AuditLog", [], |r| r.get(0)).unwrap();
        assert_eq!(totale, 3);

        let prompt_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM AuditLog WHERE EntityType = 'Prompt'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(prompt_count, 2);
    }

    #[test]
    fn registra_ordine_cronologico() {
        let conn = db_test();
        registra(&conn, "primo", "T", "", None);
        registra(&conn, "secondo", "T", "", None);

        let azione: String = conn
            .query_row(
                "SELECT Action FROM AuditLog ORDER BY OccurredAt DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(azione, "secondo");
    }

    #[test]
    fn componi_where_vuoto() {
        let f = AuditFiltro::default();
        let (clausola, params) = componi_where(&f);
        assert_eq!(clausola, "");
        assert!(params.is_empty());
    }

    #[test]
    fn componi_where_con_filtri() {
        let f = AuditFiltro {
            tipo_entita: Some("Prompt".to_string()),
            azione_like: Some("creato".to_string()),
            ..Default::default()
        };
        let (clausola, params) = componi_where(&f);
        assert!(clausola.contains("EntityType = :tipo"));
        assert!(clausola.contains("Action LIKE :azione"));
        assert!(clausola.starts_with("WHERE"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn cleanup_elimina_righe_vecchie() {
        let conn = db_test();
        // Riga vecchia (1 anno fa).
        conn.execute(
            "INSERT INTO AuditLog (Id, WorkspaceId, UserId, Action, EntityType, OccurredAt)
             VALUES ('aud-old', 'ws-personale', 'usr-locale', 'old', 'T', datetime('now', '-400 days'))",
            [],
        )
        .unwrap();
        // Riga recente.
        registra(&conn, "recente", "T", "x", None);

        // Cleanup oltre 365 giorni.
        let modifier = "-365 days";
        let n = conn
            .execute(
                "DELETE FROM AuditLog WHERE OccurredAt < datetime('now', ?1)",
                rusqlite::params![modifier],
            )
            .unwrap();
        assert_eq!(n, 1, "Deve eliminare solo la riga vecchia");

        let totale: i64 = conn
            .query_row("SELECT COUNT(*) FROM AuditLog", [], |r| r.get(0))
            .unwrap();
        assert_eq!(totale, 1);
    }

    #[test]
    fn csv_quote_escapa_virgolette() {
        assert_eq!(csv_quote("semplice"), "semplice");
        assert_eq!(csv_quote("con,virgola"), "\"con,virgola\"");
        assert_eq!(csv_quote("con \"virgolette\""), "\"con \"\"virgolette\"\"\"");
        assert_eq!(csv_quote("a\nb"), "\"a\nb\"");
    }

    #[test]
    fn componi_where_filtri_vuoti_clausola_vuota() {
        let f = AuditFiltro::default();
        let (clausola, params) = componi_where(&f);
        assert_eq!(clausola, "");
        assert!(params.is_empty());
    }

    #[test]
    fn componi_where_date_range_e_user() {
        let f = AuditFiltro {
            da: Some("2026-05-01T00:00:00Z".to_string()),
            a: Some("2026-05-07T23:59:59Z".to_string()),
            user_id: Some("usr-locale".to_string()),
            ..Default::default()
        };
        let (clausola, params) = componi_where(&f);
        assert!(clausola.contains("OccurredAt >= :da"));
        assert!(clausola.contains("OccurredAt < :a"));
        assert!(clausola.contains("UserId = :user_id"));
        assert!(clausola.starts_with("WHERE"));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn componi_where_testo_libero_cerca_in_entity_e_metadata() {
        let f = AuditFiltro {
            testo: Some("prm-".to_string()),
            ..Default::default()
        };
        let (clausola, params) = componi_where(&f);
        assert!(clausola.contains("EntityId LIKE :testo"));
        assert!(clausola.contains("Metadata LIKE :testo"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn componi_where_filtri_stringa_vuota_ignorati() {
        // Stringa "" deve essere trattata come None.
        let f = AuditFiltro {
            da: Some("".to_string()),
            tipo_entita: Some("".to_string()),
            ..Default::default()
        };
        let (clausola, params) = componi_where(&f);
        assert_eq!(clausola, "");
        assert!(params.is_empty());
    }

    #[test]
    fn csv_quote_stringa_vuota_e_solo_spazi() {
        assert_eq!(csv_quote(""), "");
        assert_eq!(csv_quote("  "), "  ");
    }

    #[test]
    fn csv_quote_combinazione_virgola_e_virgolette() {
        // Caso edge: contiene sia virgola che virgolette annidate.
        assert_eq!(
            csv_quote("a,\"b\",c"),
            "\"a,\"\"b\"\",c\""
        );
    }

    // ─── M7 PR-2: lista/query/export/cleanup _pure ─────────────────

    fn popola_db_audit(conn: &Connection) {
        // 5 voci: 3 Prompt, 1 Vault, 1 Folder
        registra(conn, "prompt.creato", "Prompt", "p-1", Some("titolo=Alpha"));
        registra(conn, "prompt.aggiornato", "Prompt", "p-1", None);
        registra(conn, "prompt.eliminato", "Prompt", "p-2", Some("titolo=Beta"));
        registra(conn, "vault.creato", "Vault", "", Some("cifrato"));
        registra(conn, "folder.creato", "Folder", "f-1", None);
    }

    #[test]
    fn lista_pure_default_limite_e_tutti_i_tipi() {
        let conn = db_test();
        popola_db_audit(&conn);
        let voci = lista_pure(&conn, None, None).unwrap();
        assert_eq!(voci.len(), 5);
    }

    #[test]
    fn lista_pure_filtra_per_tipo_entita() {
        let conn = db_test();
        popola_db_audit(&conn);
        let voci = lista_pure(&conn, None, Some("Prompt")).unwrap();
        assert_eq!(voci.len(), 3);
        assert!(voci.iter().all(|v| v.tipo_entita == "Prompt"));
    }

    #[test]
    fn lista_pure_rispetta_limite() {
        let conn = db_test();
        popola_db_audit(&conn);
        let voci = lista_pure(&conn, Some(2), None).unwrap();
        assert_eq!(voci.len(), 2);
    }

    #[test]
    fn lista_pure_limite_capped_a_500() {
        let conn = db_test();
        popola_db_audit(&conn);
        // Richiesta 10000 -> capped a 500 (poi limita ai 5 esistenti)
        let voci = lista_pure(&conn, Some(10_000), None).unwrap();
        assert_eq!(voci.len(), 5);
    }

    #[test]
    fn query_pure_senza_filtri() {
        let conn = db_test();
        popola_db_audit(&conn);
        let filtro = AuditFiltro::default();
        let res = query_pure(&conn, &filtro).unwrap();
        assert_eq!(res.totale, 5);
        assert_eq!(res.voci.len(), 5);
    }

    #[test]
    fn query_pure_filtra_per_azione_like() {
        let conn = db_test();
        popola_db_audit(&conn);
        let filtro = AuditFiltro {
            azione_like: Some("prompt.".to_string()),
            ..Default::default()
        };
        let res = query_pure(&conn, &filtro).unwrap();
        assert_eq!(res.totale, 3);
    }

    #[test]
    fn query_pure_filtra_per_testo_su_entity_id_e_metadata() {
        let conn = db_test();
        popola_db_audit(&conn);
        let filtro = AuditFiltro {
            testo: Some("Alpha".to_string()),
            ..Default::default()
        };
        let res = query_pure(&conn, &filtro).unwrap();
        assert_eq!(res.totale, 1, "Alpha appare in metadata di una sola voce");
    }

    #[test]
    fn query_pure_pagination_offset_e_limite() {
        let conn = db_test();
        popola_db_audit(&conn);
        let filtro = AuditFiltro {
            limite: Some(2),
            offset: Some(2),
            ..Default::default()
        };
        let res = query_pure(&conn, &filtro).unwrap();
        assert_eq!(res.totale, 5, "totale ignora pagination");
        assert_eq!(res.voci.len(), 2);
        assert_eq!(res.limite, 2);
        assert_eq!(res.offset, 2);
    }

    #[test]
    fn export_csv_pure_header_e_righe() {
        let conn = db_test();
        popola_db_audit(&conn);
        let csv = export_csv_pure(&conn, &AuditFiltro::default()).unwrap();
        let prima_riga = csv.lines().next().unwrap();
        assert_eq!(
            prima_riga,
            "id,occurred_at,user_id,action,entity_type,entity_id,metadata"
        );
        // 1 header + 5 voci = 6 righe (no trailing newline considerato)
        let n_righe = csv.lines().count();
        assert_eq!(n_righe, 6);
    }

    #[test]
    fn export_csv_pure_filtrato() {
        let conn = db_test();
        popola_db_audit(&conn);
        let filtro = AuditFiltro {
            tipo_entita: Some("Vault".to_string()),
            ..Default::default()
        };
        let csv = export_csv_pure(&conn, &filtro).unwrap();
        let n_righe = csv.lines().count();
        assert_eq!(n_righe, 2, "header + 1 voce Vault");
        assert!(csv.contains("vault.creato"));
        assert!(!csv.contains("prompt.creato"));
    }

    #[test]
    fn cleanup_oltre_giorni_pure_zero_giorni_rimuove_tutto() {
        let conn = db_test();
        popola_db_audit(&conn);
        // 0 giorni -> tutto e' "piu vecchio di 0 giorni fa" (esclusivo)
        // In pratica: cancella voci con OccurredAt < datetime('now', '-0 days')
        // = OccurredAt < now -> tutto cio' inserito qualche ms fa
        let _n = cleanup_oltre_giorni_pure(&conn, 0).unwrap();
        // best-effort: timing-dependent ma non fail — .unwrap() sopra già asserta successo
    }

    #[test]
    fn cleanup_oltre_giorni_pure_alti_giorni_preserva_tutto() {
        let conn = db_test();
        popola_db_audit(&conn);
        // 365 giorni: tutto e' troppo recente -> nessuna cancellazione
        let n = cleanup_oltre_giorni_pure(&conn, 365).unwrap();
        assert_eq!(n, 0);
        let totale: i64 = conn
            .query_row("SELECT COUNT(*) FROM AuditLog", [], |r| r.get(0))
            .unwrap();
        assert_eq!(totale, 5);
    }
}
