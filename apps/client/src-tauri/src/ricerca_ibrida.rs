// Modulo ricerca_ibrida — Fase 3 Step 3.
//
// Combina ricerca lessicale (FTS5) e ricerca semantica (sqlite-vec) via
// Reciprocal Rank Fusion pesata, per ottenere risultati che intercettano
// sia match testuali esatti sia "fratelli concettuali" del prompt cercato.
//
// Algoritmo:
//   score(d) = (1-alpha) * 1/(k_rrf + rank_lex(d)) + alpha * 1/(k_rrf + rank_sem(d))
//
// dove:
//   - rank_lex(d) = posizione 1-based di d nella top-K lessicale (FTS5);
//     +∞ se d non è in top-K lessicale → contributo 0.
//   - rank_sem(d) = analogo per top-K semantica (vec0).
//   - alpha ∈ [0,1] pesa semantico vs lessicale (0=solo FTS5, 1=solo vec0).
//   - k_rrf = costante (tipicamente 60, valore standard letteratura RRF).
//
// Fallback graceful: se Embeddings Session non è caricata (utente non ha
// abilitato la feature, modello non scaricato), `cerca_ibrida` ritorna
// solo i risultati FTS5 senza errore. Il punteggio semantico è 0 per tutti.

use rusqlite::Connection;
use serde::Serialize;
use tauri::State;

use crate::embeddings::{assicura_session_caricata, compute_embedding_opt, EmbeddingsState};
use crate::embeddings_store::search_nearest;
use crate::errore::PapErrore;
use crate::vault::VaultState;

/// Costante RRF standard. Più alto = ranking più piatto (meno premio ai top).
const K_RRF: f64 = 60.0;

/// Top-K interno tirato da ciascuna pipeline prima della fusione.
/// Più alto = più ricco il pool fuso, ma più lavoro inutile se l'utente
/// vuole solo top-N piccolo.
const POOL_SIZE: usize = 50;

#[derive(Debug, Serialize, Clone)]
pub struct RisultatoIbrido {
    pub id: String,
    pub titolo: String,
    pub descrizione: String,
    pub body: String,
    pub visibilita: String,
    pub preferito: bool,
    pub uso_count: i64,
    /// Punteggio combinato finale (più alto = più rilevante).
    pub score: f64,
    /// Posizione 1-based nella ranking lessicale, `None` se non presente.
    pub rank_lex: Option<usize>,
    /// Posizione 1-based nella ranking semantica, `None` se non presente.
    pub rank_sem: Option<usize>,
}

/// Fix #582: risposta di `prompt_cerca_ibrida` con un segnale esplicito sulla
/// disponibilità del modello di embedding, distinto da "nessun risultato".
///
/// Prima di questo fix, `cerca_semantica` scartava QUALUNQUE errore di
/// `assicura_session_caricata` con `let _ = ...` e degradava silenziosamente
/// a FTS5-only: dal punto di vista dell'utente, "il modello non è caricato"
/// e "nessuna corrispondenza trovata" producevano la stessa UI (zero
/// risultati semantici), senza modo di distinguerli senza aprire
/// Impostazioni.
#[derive(Debug, Serialize, Clone)]
pub struct RispostaRicercaIbrida {
    pub risultati: Vec<RisultatoIbrido>,
    /// `false` quando la ricerca è stata degradata a FTS5-only perché il
    /// modello di embedding non è (ancora) caricato — non scaricato,
    /// inizializzazione fallita, o droppato per idle-unload e non
    /// ricaricabile. Il frontend usa questo campo per mostrare un messaggio
    /// distinto da "nessun risultato" (vedi CommandPalette.svelte).
    pub modello_disponibile: bool,
}

fn sanitizza_fts(query: &str) -> String {
    query
        .split_whitespace()
        .map(|w| {
            let pulito: String = w
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if pulito.is_empty() {
                String::new()
            } else {
                format!("{pulito}*")
            }
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Top-K prompt id matching della query in FTS5, ordinati per rank ASC
/// (rank FTS5 più basso = più rilevante).
///
/// `pub` per il bench harness (Step 10 quality gate). Non parte dell'API
/// pubblica del crate per consumatori esterni.
pub fn cerca_lessicale(
    conn: &Connection,
    query: &str,
    k: usize,
) -> Result<Vec<String>, PapErrore> {
    let fts = sanitizza_fts(query);
    if fts.is_empty() {
        return Ok(vec![]);
    }
    let mut stmt = conn.prepare(
        "SELECT f.PromptId
         FROM PromptsFts f
         JOIN Prompts p ON f.PromptId = p.Id
         WHERE PromptsFts MATCH ?1 AND p.DeletedAt IS NULL
         ORDER BY rank
         LIMIT ?2",
    )?;
    let ids = stmt
        .query_map(rusqlite::params![&fts, k as i64], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(ids)
}

/// Traduce l'esito di `assicura_session_caricata` in un segnale booleano
/// esplicito per il chiamante, loggando (invece di scartare, fix #582)
/// l'eventuale errore.
///
/// Estratta come funzione pura, indipendente da `EmbeddingsState`/
/// `VaultState`, per poter testare sia il ramo positivo sia quello negativo
/// senza dover forzare un vero caricamento `ort` (che richiederebbe un
/// modello + libonnxruntime reali su disco — non riproducibile in un test
/// unitario). Copre esattamente il punto in cui, prima del fix, l'errore
/// veniva scartato con `let _ = ...`.
fn valuta_disponibilita_modello(esito: Result<bool, PapErrore>) -> bool {
    match esito {
        Ok(_) => true,
        Err(e) => {
            log::warn!("Ricerca semantica degradata a FTS5-only: {e}");
            false
        }
    }
}

/// Top-K prompt id più vicini all'embedding query in vec0, ordinati per
/// distance ASC. Vec vuoto se Session non disponibile o se vec0 non ha
/// embeddings.
///
/// v0.6.0 Step 2: prima del compute embedding, tenta un riload on-demand
/// della Session se è stata droppata dal timer idle-unload. Se il riload
/// fallisce (es. modello non scaricato), degrada a `(vec![], false)` — la
/// ricerca lessicale resta funzionante via FTS5, ma il chiamante riceve un
/// segnale esplicito (fix #582) invece di un semplice "zero risultati".
///
/// Ritorna `(id_prompt_semantici, modello_disponibile)`.
fn cerca_semantica(
    conn: &Connection,
    vault_state: &crate::vault::VaultState,
    rt_state: &EmbeddingsState,
    query: &str,
    k: usize,
) -> Result<(Vec<String>, bool), PapErrore> {
    // Fix #582: l'errore di riload NON viene più scartato con `let _ = ...`.
    // Prima di questo fix era impossibile, dal solo comportamento
    // dell'utente (zero risultati semantici) o dal log, capire se la causa
    // era "nessuna corrispondenza" o "il modello non è caricato" — la ricerca
    // ibrida degradava a FTS5-only senza lasciare traccia.
    let modello_disponibile =
        valuta_disponibilita_modello(assicura_session_caricata(rt_state, vault_state));

    if !modello_disponibile {
        return Ok((vec![], false));
    }

    let Some(query_emb) = compute_embedding_opt(rt_state, query)? else {
        // Non dovrebbe accadere (la Session è appena stata assicurata
        // caricata sopra), ma se succede trattiamolo comunque come "modello
        // non disponibile" per coerenza col segnale esposto al chiamante.
        return Ok((vec![], false));
    };
    let nearest = search_nearest(conn, &query_emb, k)?;
    Ok((nearest.into_iter().map(|(id, _dist)| id).collect(), true))
}

/// Reciprocal Rank Fusion pesata.
///
/// Restituisce `Vec<(id, score, rank_lex, rank_sem)>` ordinato per score DESC
/// (più alto prima).
///
/// `pub` per il bench harness (Step 10 quality gate).
pub fn rrf_fuse(
    lex: &[String],
    sem: &[String],
    alpha: f64,
    k_rrf: f64,
) -> Vec<(String, f64, Option<usize>, Option<usize>)> {
    use std::collections::HashMap;

    let mut score: HashMap<String, (f64, Option<usize>, Option<usize>)> = HashMap::new();

    for (i, id) in lex.iter().enumerate() {
        let rank = i + 1;
        let s = (1.0 - alpha) / (k_rrf + rank as f64);
        score
            .entry(id.clone())
            .and_modify(|(sc, rl, _)| {
                *sc += s;
                *rl = Some(rank);
            })
            .or_insert((s, Some(rank), None));
    }
    for (i, id) in sem.iter().enumerate() {
        let rank = i + 1;
        let s = alpha / (k_rrf + rank as f64);
        score
            .entry(id.clone())
            .and_modify(|(sc, _, rs)| {
                *sc += s;
                *rs = Some(rank);
            })
            .or_insert((s, None, Some(rank)));
    }

    let mut combined: Vec<_> = score
        .into_iter()
        .map(|(id, (s, rl, rs))| (id, s, rl, rs))
        .collect();
    combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    combined
}

/// Dettagli di un prompt come letti dalla riga SQL:
/// (titolo, descrizione, body, visibilità, preferito, conteggio usi).
type DettagliPrompt = (String, String, String, String, bool, i64);

/// Carica i dettagli di un singolo prompt per id. Ritorna `None` se non
/// trovato o eliminato.
fn carica_prompt(conn: &Connection, id: &str) -> Result<Option<DettagliPrompt>, PapErrore> {
    let r = conn.query_row(
        "SELECT Title, COALESCE(Description, ''), Body, Visibility, IsFavorite, UseCount
         FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
        [id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)? != 0,
                row.get::<_, i64>(5)?,
            ))
        },
    );
    match r {
        Ok(t) => Ok(Some(t)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(PapErrore::Db(e)),
    }
}

#[tauri::command]
pub fn prompt_cerca_ibrida(
    query: String,
    limit: Option<u32>,
    alpha: Option<f64>,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<RispostaRicercaIbrida, PapErrore> {
    let limit = limit.unwrap_or(20).min(100) as usize;
    let alpha = alpha.unwrap_or(0.5).clamp(0.0, 1.0);

    state.with_conn(|conn| {
        let q = query.trim();
        if q.is_empty() {
            // Coerente col ramo query non vuota sotto (`cerca_semantica`,
            // review post-merge PR #589 — minore): usa `assicura_session_caricata`
            // (tenta il riload on-demand se la Session è stata droppata
            // dal timer idle-unload) invece della sola lettura passiva
            // `session_caricata`. Senza questo, un modello idle-unloaded ma
            // ricaricabile risulterebbe `modello_disponibile: false` solo
            // perché la query era vuota — incoerente col comportamento a
            // query non vuota.
            return Ok(RispostaRicercaIbrida {
                risultati: vec![],
                modello_disponibile: valuta_disponibilita_modello(assicura_session_caricata(
                    &rt_state,
                    state.inner(),
                )),
            });
        }

        let lex_ids = cerca_lessicale(conn, q, POOL_SIZE)?;
        let (sem_ids, modello_disponibile) =
            cerca_semantica(conn, state.inner(), &rt_state, q, POOL_SIZE)?;

        let fused = rrf_fuse(&lex_ids, &sem_ids, alpha, K_RRF);

        let mut risultati: Vec<RisultatoIbrido> = Vec::with_capacity(limit);
        for (id, score, rank_lex, rank_sem) in fused.into_iter().take(limit) {
            if let Some((titolo, descrizione, body, visibilita, preferito, uso_count)) =
                carica_prompt(conn, &id)?
            {
                risultati.push(RisultatoIbrido {
                    id,
                    titolo,
                    descrizione,
                    body,
                    visibilita,
                    preferito,
                    uso_count,
                    score,
                    rank_lex,
                    rank_sem,
                });
            }
        }
        Ok(RispostaRicercaIbrida {
            risultati,
            modello_disponibile,
        })
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rrf_solo_lessicale_con_alpha_zero() {
        // alpha=0 → solo FTS5. La ranking è quella lessicale, score puro RRF.
        let lex = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let sem: Vec<String> = vec![];
        let fused = rrf_fuse(&lex, &sem, 0.0, 60.0);

        assert_eq!(fused.len(), 3);
        assert_eq!(fused[0].0, "a");
        assert_eq!(fused[1].0, "b");
        assert_eq!(fused[2].0, "c");
        // a è primo: score = 1/(60+1) ≈ 0.01639
        assert!((fused[0].1 - 1.0 / 61.0).abs() < 1e-9);
        assert_eq!(fused[0].2, Some(1)); // rank_lex
        assert_eq!(fused[0].3, None); // rank_sem
    }

    #[test]
    fn rrf_solo_semantica_con_alpha_uno() {
        let lex: Vec<String> = vec![];
        let sem = vec!["x".to_string(), "y".to_string()];
        let fused = rrf_fuse(&lex, &sem, 1.0, 60.0);

        assert_eq!(fused.len(), 2);
        assert_eq!(fused[0].0, "x");
        assert_eq!(fused[0].2, None); // rank_lex
        assert_eq!(fused[0].3, Some(1)); // rank_sem
    }

    #[test]
    fn rrf_combinata_con_overlap_premia_intersezione() {
        // 'b' compare in entrambe le ranking → score combinato > di chi
        // appare solo in una.
        let lex = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let sem = vec!["b".to_string(), "d".to_string(), "a".to_string()];
        let fused = rrf_fuse(&lex, &sem, 0.5, 60.0);

        let score_b = fused.iter().find(|x| x.0 == "b").unwrap().1;
        let score_c = fused.iter().find(|x| x.0 == "c").unwrap().1;
        let score_d = fused.iter().find(|x| x.0 == "d").unwrap().1;

        // b: rank lex 2 + rank sem 1 → score più alto di c (solo lex 3)
        // e di d (solo sem 2)
        assert!(score_b > score_c);
        assert!(score_b > score_d);
    }

    #[test]
    fn rrf_alpha_pesa_correttamente() {
        // Stesso item in entrambe le ranking, ma alpha diversi devono dare
        // peso proporzionale fra le due metà.
        let lex = vec!["x".to_string()];
        let sem = vec!["x".to_string()];

        let f0 = rrf_fuse(&lex, &sem, 0.0, 60.0);
        let f1 = rrf_fuse(&lex, &sem, 1.0, 60.0);
        let f5 = rrf_fuse(&lex, &sem, 0.5, 60.0);

        // alpha=0: solo lex contribuisce
        assert!((f0[0].1 - 1.0 / 61.0).abs() < 1e-9);
        // alpha=1: solo sem contribuisce
        assert!((f1[0].1 - 1.0 / 61.0).abs() < 1e-9);
        // alpha=0.5: contributo da entrambe, somma uguale a tutto-da-uno
        assert!((f5[0].1 - 1.0 / 61.0).abs() < 1e-9);
    }

    #[test]
    fn rrf_set_disgiunti_score_indipendenti() {
        let lex = vec!["a".to_string()];
        let sem = vec!["b".to_string()];
        let fused = rrf_fuse(&lex, &sem, 0.5, 60.0);

        assert_eq!(fused.len(), 2);
        let score_a = fused.iter().find(|x| x.0 == "a").unwrap().1;
        let score_b = fused.iter().find(|x| x.0 == "b").unwrap().1;

        // entrambi solo da una pipeline, alpha=0.5
        assert!((score_a - 0.5 / 61.0).abs() < 1e-9);
        assert!((score_b - 0.5 / 61.0).abs() < 1e-9);
    }

    #[test]
    fn sanitizza_fts_consistente_con_modulo_prompt() {
        // Sentinel: la sanitizzazione deve essere identica a quella di
        // prompt::sanitizza_fts (futuro refactor potrebbe estrarre helper
        // condiviso).
        assert_eq!(sanitizza_fts(""), "");
        assert_eq!(sanitizza_fts("hello world"), "hello* world*");
        assert_eq!(sanitizza_fts("hello! @world#"), "hello* world*");
    }

    // ─────────── Smoke test: fallback grace senza Session ───────────

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    #[test]
    fn cerca_semantica_senza_session_loaded_ritorna_vuoto_e_segnala_modello_non_disponibile() {
        // Quality gate Step 10 — grace degradation: se l'utente non ha
        // ancora abilitato/scaricato il modello, ricerca_ibrida deve
        // tornare risultati FTS-only senza errori.
        // Fix #582: l'errore di assicura_session_caricata NON viene più
        // scartato in silenzio — il chiamante riceve `modello_disponibile:
        // false` invece che una semplice lista vuota indistinguibile da
        // "nessuna corrispondenza".
        let conn = db_test();
        let dir = tempfile::tempdir().unwrap();
        let vault_state = crate::vault::VaultState::new(dir.path().to_path_buf());
        let rt_state = EmbeddingsState::new(); // session = None

        let (ids, modello_disponibile) =
            cerca_semantica(&conn, &vault_state, &rt_state, "qualunque query", 10).unwrap();
        assert!(
            ids.is_empty(),
            "senza Session, cerca_semantica ritorna vec vuoto, no errore"
        );
        assert!(
            !modello_disponibile,
            "il chiamante deve poter distinguere 'modello non caricato' da 'nessun risultato'"
        );
    }

    // ─── valuta_disponibilita_modello: ramo positivo + negativo (fix #582) ───
    //
    // `cerca_semantica` end-to-end non è testabile sul ramo POSITIVO senza un
    // vero caricamento `ort` (modello + libonnxruntime reali su disco): per
    // questo `valuta_disponibilita_modello` è estratta come funzione pura,
    // testabile con un `Result<bool, PapErrore>` sintetico sia sul ramo Ok
    // sia su quello Err — esattamente il punto che prima del fix scartava
    // l'errore con `let _ = ...`.
    //
    // Trappola nota del progetto: un test che verificasse solo il valore di
    // ritorno (`true`/`false`) non proverebbe che l'errore viene DAVVERO
    // loggato (a differenza di essere scartato in silenzio) — da qui
    // `testing_logger`.

    #[test]
    fn valuta_disponibilita_modello_ramo_positivo_true_e_nessun_warning() {
        testing_logger::setup();

        let disponibile = valuta_disponibilita_modello(Ok(true));
        assert!(disponibile, "Ok(_) deve tradursi in modello_disponibile = true");

        testing_logger::validate(|righe| {
            let warning_degradazione = righe
                .iter()
                .any(|r| r.level == log::Level::Warn && r.body.contains("degradata"));
            assert!(
                !warning_degradazione,
                "il ramo positivo non deve loggare una degradazione che non c'è stata"
            );
        });
    }

    #[test]
    fn valuta_disponibilita_modello_ramo_negativo_false_loggato_non_scartato() {
        testing_logger::setup();

        let esito: Result<bool, PapErrore> =
            Err(PapErrore::Generico("modello non scaricato (test)".into()));
        let disponibile = valuta_disponibilita_modello(esito);
        assert!(!disponibile, "Err(_) deve tradursi in modello_disponibile = false");

        testing_logger::validate(|righe| {
            let trovata = righe.iter().any(|r| {
                r.level == log::Level::Warn
                    && r.body.contains("degradata")
                    && r.body.contains("modello non scaricato (test)")
            });
            let corpi: Vec<&str> = righe.iter().map(|r| r.body.as_str()).collect();
            assert!(
                trovata,
                "l'errore deve finire nel log invece di essere scartato in silenzio: {corpi:?}"
            );
        });
    }
}
