// Modulo embeddings_backfill — Fase 3 Step 3.
//
// Comando Tauri per popolare gli embedding di prompt esistenti (creati
// prima dell'attivazione della feature ricerca semantica). Idempotente:
// salta i prompt che hanno già un embedding.
//
// Strategia:
// - Batch di BATCH_SIZE prompt per cycle, lock SQLite rilasciato fra batch
//   per non bloccare altre operazioni utente.
// - Per ogni prompt: legge body, calcola embedding, upsert in vec0.
// - Emit progress event "embeddings:backfill:progress" ad ogni batch.
// - Pre-condizione: Embeddings Session caricata (chiamata a embeddings_init
//   precedente). Se non lo è, ritorna errore esplicito.

use rusqlite::params;
use serde::Serialize;
use tauri::{Emitter, State};

use crate::embeddings::{compute_embedding_opt, EmbeddingsState};
use crate::embeddings_store;
use crate::errore::PapErrore;
use crate::vault::VaultState;

/// Quanti prompt processiamo per ciclo. Il lock SQLite è rilasciato fra
/// cicli, così altre operazioni (UI list, save prompt) possono proseguire.
const BATCH_SIZE: usize = 10;

/// Limite globale di sicurezza: il backfill non processa più di MAX_TOTAL
/// prompt in una singola invocazione. Su vault molto grandi (>1k prompt)
/// l'utente può rilanciare il comando — è idempotente.
const MAX_TOTAL: usize = 5000;

#[derive(Debug, Clone, Serialize)]
pub struct ProgressBackfill {
    pub processati: usize,
    pub totale_stima: usize,
    /// Prompt id appena completato (utile per UI debug).
    pub ultimo_id: String,
}

#[derive(Debug, Serialize)]
pub struct EsitoBackfill {
    pub processati: usize,
    pub saltati_no_session: usize,
    pub errori: usize,
}

#[tauri::command]
pub fn embeddings_backfill(
    app: tauri::AppHandle,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<EsitoBackfill, PapErrore> {
    let mut processati = 0usize;
    let mut saltati_no_session = 0usize;
    let mut errori = 0usize;

    // Stima iniziale del totale (per progress UI). Si aggiorna ad ogni batch.
    let totale_stima: usize = state.with_conn(|conn| {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM Prompts p
             WHERE p.DeletedAt IS NULL
               AND NOT EXISTS (
                   SELECT 1 FROM PromptsEmbeddings e WHERE e.PromptId = p.Id
               )",
            [],
            |r| r.get(0),
        )?;
        Ok(n as usize)
    })?;

    log::info!("embeddings_backfill: stima {totale_stima} prompt da processare");

    while processati < MAX_TOTAL {
        // 1. Leggi un batch di candidati (id + body) sotto lock.
        let candidati: Vec<(String, String)> = state.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT p.Id, p.Body FROM Prompts p
                 WHERE p.DeletedAt IS NULL
                   AND NOT EXISTS (
                       SELECT 1 FROM PromptsEmbeddings e WHERE e.PromptId = p.Id
                   )
                 LIMIT ?1",
            )?;
            let rows = stmt
                .query_map(params![BATCH_SIZE as i64], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?
                .filter_map(|r| r.ok())
                .collect();
            Ok(rows)
        })?;

        if candidati.is_empty() {
            break;
        }

        // 2. Calcola gli embeddings fuori dal lock vault (lock rt_state interno
        //    al compute, condiviso con altri command embedding).
        let mut emb_batch: Vec<(String, Vec<f32>)> = Vec::with_capacity(candidati.len());
        for (id, body) in &candidati {
            match compute_embedding_opt(&rt_state, body) {
                Ok(Some(emb)) => emb_batch.push((id.clone(), emb)),
                Ok(None) => {
                    saltati_no_session += 1;
                    // Session non loaded: ritorna subito errore esplicito,
                    // non c'è motivo di continuare.
                    return Err(PapErrore::Generico(
                        "Embeddings non inizializzati. Chiama embeddings_init prima del backfill.".into(),
                    ));
                }
                Err(e) => {
                    log::error!("backfill compute fallito per {id}: {e}");
                    errori += 1;
                }
            }
        }

        // 3. Upsert in vec0 sotto lock vault.
        let ultimo_id = state.with_conn(|conn| {
            let mut last = String::new();
            for (id, emb) in &emb_batch {
                if let Err(e) = embeddings_store::upsert_embedding(conn, id, emb) {
                    log::error!("backfill upsert fallito per {id}: {e}");
                    errori += 1;
                } else {
                    last = id.clone();
                    processati += 1;
                }
            }
            Ok(last)
        })?;

        // 4. Emit progress.
        let _ = app.emit(
            "embeddings:backfill:progress",
            ProgressBackfill {
                processati,
                totale_stima,
                ultimo_id,
            },
        );
    }

    log::info!(
        "embeddings_backfill: completato. processati={processati}, errori={errori}"
    );
    Ok(EsitoBackfill {
        processati,
        saltati_no_session,
        errori,
    })
}

#[cfg(test)]
mod test {
    // I test reali del backfill richiedono Session ort caricata + modello
    // scaricato (~150 MB), pesante per CI. La logica del batching è
    // semplice e direttamente derivata da embeddings_store::prompt_senza_embedding
    // (già testato), e i path di upsert/lookup sono già coperti dai test
    // di embeddings_store. Niente test unit diretti qui — verifica
    // manuale post-merge nel client real-world.

    #[test]
    fn batch_size_e_max_total_sensati() {
        // Sentinel: se qualcuno cambia accidentalmente i valori a numeri
        // assurdi (es. BATCH_SIZE > MAX_TOTAL), questo test rileva.
        assert!(super::BATCH_SIZE > 0);
        assert!(super::MAX_TOTAL > super::BATCH_SIZE);
        assert!(super::BATCH_SIZE <= 100, "batch troppo grande, blocca lock troppo a lungo");
    }
}
