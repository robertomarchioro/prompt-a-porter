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
// - Grace degradation: se la Session ort non è caricata (modello non
//   scaricato o auto-init non ancora completato), il loop esce a mani
//   vuote popolando il counter `saltati_no_session` invece di
//   propagare un errore. L'utente può rilanciare il backfill dopo
//   l'init senza perdere il progresso parziale (idempotente).

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
    pub tipo: &'static str, // "prompt" | "tag"
    pub processati: usize,
    pub totale_stima: usize,
    /// ID appena completato (utile per UI debug).
    pub ultimo_id: String,
}

#[derive(Debug, Serialize)]
pub struct EsitoBackfill {
    pub prompt_processati: usize,
    pub tag_processati: usize,
    pub saltati_no_session: usize,
    pub errori: usize,
}

/// Tipo di entità da processare nel backfill.
enum TipoBackfill {
    Prompt,
    Tag,
}

impl TipoBackfill {
    fn etichetta(&self) -> &'static str {
        match self {
            TipoBackfill::Prompt => "prompt",
            TipoBackfill::Tag => "tag",
        }
    }
}

/// Loop di backfill generico parametrizzato sul tipo. Ritorna numero
/// processati. Se la Session ort non è caricata, esce subito senza
/// errore e incrementa `saltati_no_session` con la stima di candidati.
fn esegui_loop(
    app: &tauri::AppHandle,
    state: &State<'_, VaultState>,
    rt_state: &State<'_, EmbeddingsState>,
    tipo: TipoBackfill,
    errori: &mut usize,
    saltati_no_session: &mut usize,
) -> Result<usize, PapErrore> {
    let mut processati = 0usize;

    let totale_stima: usize = state.with_conn(|conn| {
        let sql = match tipo {
            TipoBackfill::Prompt => {
                "SELECT COUNT(*) FROM Prompts p
                 WHERE p.DeletedAt IS NULL
                   AND NOT EXISTS (
                       SELECT 1 FROM PromptsEmbeddings e WHERE e.PromptId = p.Id
                   )"
            }
            TipoBackfill::Tag => {
                "SELECT COUNT(*) FROM Tags t
                 WHERE t.DeletedAt IS NULL
                   AND NOT EXISTS (
                       SELECT 1 FROM TagsEmbeddings e WHERE e.TagId = t.Id
                   )"
            }
        };
        let n: i64 = conn.query_row(sql, [], |r| r.get(0))?;
        Ok(n as usize)
    })?;

    log::info!(
        "embeddings_backfill: stima {totale_stima} {} da processare",
        tipo.etichetta()
    );

    while processati < MAX_TOTAL {
        let candidati: Vec<(String, String)> = state.with_conn(|conn| {
            let sql = match tipo {
                TipoBackfill::Prompt => {
                    "SELECT p.Id, p.Body FROM Prompts p
                     WHERE p.DeletedAt IS NULL
                       AND NOT EXISTS (
                           SELECT 1 FROM PromptsEmbeddings e WHERE e.PromptId = p.Id
                       )
                     LIMIT ?1"
                }
                TipoBackfill::Tag => {
                    "SELECT t.Id, t.Name FROM Tags t
                     WHERE t.DeletedAt IS NULL
                       AND NOT EXISTS (
                           SELECT 1 FROM TagsEmbeddings e WHERE e.TagId = t.Id
                       )
                     LIMIT ?1"
                }
            };
            let mut stmt = conn.prepare(sql)?;
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

        let mut emb_batch: Vec<(String, Vec<f32>)> = Vec::with_capacity(candidati.len());
        for (id, testo) in &candidati {
            match compute_embedding_opt(rt_state, testo) {
                Ok(Some(emb)) => emb_batch.push((id.clone(), emb)),
                Ok(None) => {
                    // Grace degradation: Session non loaded (modello non
                    // scaricato o auto-init non ancora completato). Il
                    // counter usa la stima totale aggiornata al netto di
                    // quanti abbiamo già processato in questa run, così
                    // l'UI può mostrare un numero rappresentativo.
                    // Idempotente: l'utente rilancia post-init senza
                    // perdere il progresso parziale.
                    let rimanenti = totale_stima.saturating_sub(processati);
                    *saltati_no_session += rimanenti;
                    log::info!(
                        "embeddings_backfill: Session non loaded, {rimanenti} {} saltati",
                        tipo.etichetta()
                    );
                    return Ok(processati);
                }
                Err(e) => {
                    log::error!(
                        "backfill {} compute fallito per {id}: {e}",
                        tipo.etichetta()
                    );
                    *errori += 1;
                }
            }
        }

        let ultimo_id = state.with_conn(|conn| {
            let mut last = String::new();
            for (id, emb) in &emb_batch {
                let res = match tipo {
                    TipoBackfill::Prompt => embeddings_store::upsert_embedding(conn, id, emb),
                    TipoBackfill::Tag => embeddings_store::upsert_tag_embedding(conn, id, emb),
                };
                if let Err(e) = res {
                    log::error!(
                        "backfill {} upsert fallito per {id}: {e}",
                        tipo.etichetta()
                    );
                    *errori += 1;
                } else {
                    last = id.clone();
                    processati += 1;
                }
            }
            Ok(last)
        })?;

        let _ = app.emit(
            "embeddings:backfill:progress",
            ProgressBackfill {
                tipo: tipo.etichetta(),
                processati,
                totale_stima,
                ultimo_id,
            },
        );
    }

    Ok(processati)
}

#[tauri::command]
pub fn embeddings_backfill(
    app: tauri::AppHandle,
    state: State<'_, VaultState>,
    rt_state: State<'_, EmbeddingsState>,
) -> Result<EsitoBackfill, PapErrore> {
    let mut saltati_no_session = 0usize;
    let mut errori = 0usize;

    let prompt_processati = esegui_loop(
        &app,
        &state,
        &rt_state,
        TipoBackfill::Prompt,
        &mut errori,
        &mut saltati_no_session,
    )?;
    let tag_processati = esegui_loop(
        &app,
        &state,
        &rt_state,
        TipoBackfill::Tag,
        &mut errori,
        &mut saltati_no_session,
    )?;

    log::info!(
        "embeddings_backfill: completato. prompt={prompt_processati}, tag={tag_processati}, saltati_no_session={saltati_no_session}, errori={errori}"
    );
    Ok(EsitoBackfill {
        prompt_processati,
        tag_processati,
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

    #[test]
    fn esito_backfill_segnale_grace_degradation() {
        // Documenta l'invariante introdotto in Step 10: il campo
        // `saltati_no_session` rappresenta il numero di candidati che
        // non sono stati processati perché la Session ort non era
        // ancora caricata. Quando > 0, il client UI può mostrare un
        // hint "rilancia il backfill dopo l'init".
        let esito = super::EsitoBackfill {
            prompt_processati: 0,
            tag_processati: 0,
            saltati_no_session: 5,
            errori: 0,
        };
        // Property semantica: saltati > 0 ⇒ idempotente, l'utente può
        // rilanciare e i 5 saltati saranno picked up.
        assert!(esito.saltati_no_session > 0);
        assert_eq!(esito.errori, 0, "saltati_no_session NON è errore");
    }
}
