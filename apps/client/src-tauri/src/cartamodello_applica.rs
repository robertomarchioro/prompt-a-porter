//! Cartamodello — applicazione della proposta confermata dall'utente.
//!
//! Un'unica transazione: crea i moduli nuovi (nella cartella `Moduli`
//! accanto agli originali), riscrive gli import verso i moduli riusati, poi
//! salva il corpo ricomposto di ogni originale come **nuova versione**
//! (l'originale resta in cronologia, ripristinabile). Se un passo fallisce,
//! nulla viene scritto.
//!
//! La richiesta arriva dal frontend dopo la revisione, ma il vault può essere
//! cambiato dall'analisi (il lock viene rilasciato durante la chiamata al
//! provider) e il frontend non è fidato per costruzione: **tutto viene
//! ri-verificato qui** prima di scrivere — originali intatti, titoli dei
//! moduli liberi, import risolvibili (blueprint §6b).

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::cartamodello::TipoModulo;
use crate::cartamodello_verifica::rinomina_import;
use crate::embeddings::EmbeddingsState;
use crate::errore::PapErrore;
use crate::linting::{analizza, Severita};
use crate::prompt_componibili::{parse_imports, resolve_path};

/// Nome della sottocartella in cui finiscono i moduli (D6).
pub const NOME_CARTELLA_MODULI: &str = "Moduli";
/// Tetti sulla richiesta, allineati a quelli dell'analisi: il frontend
/// costruisce la richiesta da una proposta già limitata, ma il comando non
/// deve fidarsi (review sicurezza PR-2).
const MAX_MODULI_RICHIESTA: usize = crate::cartamodello::MAX_MODULI_RISPOSTA;
const MAX_PROMPT_RICHIESTA: usize = crate::cartamodello::MAX_PROMPT_PER_LOTTO;

// ─────────────────────────────── richiesta ───────────────────────────────

/// Cosa fare di un modulo proposto, deciso dall'utente in revisione.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "tipo", rename_all = "lowercase")]
pub enum AzioneModulo {
    /// Crea un prompt nuovo con titolo/corpo indicati.
    Crea,
    /// Usa un prompt già nel vault: gli import verso `titolo` vengono
    /// riscritti sul titolo del prompt esistente.
    Riusa { prompt_id: String },
    /// Non creare: i prompt non devono più importarlo.
    Scarta,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ModuloDaApplicare {
    pub titolo: String,
    pub tipo: TipoModulo,
    pub corpo: String,
    pub azione: AzioneModulo,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PromptDaApplicare {
    pub id: String,
    /// Titolo da salvare: di default quello originale; l'utente può
    /// cambiarlo in revisione. Il titolo proposto dal modello è solo un
    /// suggerimento (review PR-2: niente rinomine silenziose).
    pub titolo: String,
    pub corpo: String,
    /// Corpo dell'originale al momento dell'analisi: se nel frattempo è
    /// cambiato, l'applicazione si rifiuta (l'utente rilancia l'analisi).
    pub corpo_originale_atteso: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RichiestaApplica {
    pub moduli: Vec<ModuloDaApplicare>,
    pub prompt: Vec<PromptDaApplicare>,
}

// ───────────────────────────────── esito ─────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RiferimentoPrompt {
    pub id: String,
    pub titolo: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct EsitoApplica {
    pub moduli_creati: Vec<RiferimentoPrompt>,
    pub moduli_riusati: Vec<RiferimentoPrompt>,
    pub prompt_aggiornati: Vec<RiferimentoPrompt>,
    /// Path della cartella dei moduli (es. `/Sviluppo/Moduli`), se creati.
    pub cartella_moduli: Option<String>,
}

// ───────────────────────────── cartella comune ─────────────────────────────

/// Cartella condivisa da tutti gli originali (`None` = radice, o cartelle
/// diverse fra loro). È il parent della sottocartella `Moduli`.
pub(crate) fn cartella_comune(
    conn: &Connection,
    prompt_ids: &[String],
) -> Result<Option<String>, PapErrore> {
    let mut comune: Option<Option<String>> = None;
    for id in prompt_ids {
        let folder: Option<String> = conn.query_row(
            "SELECT FolderId FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
            [id],
            |r| r.get(0),
        )?;
        match &comune {
            None => comune = Some(folder),
            Some(c) if *c != folder => return Ok(None),
            Some(_) => {}
        }
    }
    Ok(comune.flatten())
}

/// Path leggibile della cartella dei moduli che verrebbe usata.
pub(crate) fn path_cartella_moduli(
    conn: &Connection,
    prompt_ids: &[String],
) -> Result<String, PapErrore> {
    let parent = cartella_comune(conn, prompt_ids)?;
    let base = match parent {
        Some(id) => conn.query_row("SELECT Path FROM Folders WHERE Id = ?1", [id], |r| {
            r.get::<_, String>(0)
        })?,
        None => String::new(),
    };
    Ok(format!("{base}/{NOME_CARTELLA_MODULI}"))
}

/// Id della sottocartella `Moduli` sotto `parent`, creandola se manca.
fn assicura_cartella_moduli(
    conn: &Connection,
    parent: Option<&str>,
) -> Result<String, PapErrore> {
    let esistente: Option<String> = conn
        .query_row(
            "SELECT Id FROM Folders
             WHERE WorkspaceId = 'ws-personale' AND DeletedAt IS NULL
               AND Name = ?1 AND COALESCE(ParentFolderId, '') = COALESCE(?2, '')",
            rusqlite::params![NOME_CARTELLA_MODULI, parent],
            |r| r.get(0),
        )
        .ok();
    if let Some(id) = esistente {
        return Ok(id);
    }
    crate::cartelle::crea_pure(
        conn,
        &crate::cartelle::NuovaCartella {
            nome: NOME_CARTELLA_MODULI.to_string(),
            parent_folder_id: parent.map(str::to_string),
        },
    )
}

// ─────────────────────────────── ri-verifica ───────────────────────────────

fn errori_linter(corpo: &str) -> Vec<String> {
    analizza(corpo)
        .into_iter()
        .filter(|i| i.severita == Severita::Error)
        .map(|i| format!("{}: {}", i.code, i.messaggio))
        .collect()
}

fn rifiuta(messaggio: impl Into<String>) -> PapErrore {
    PapErrore::Generico(messaggio.into())
}

/// Controlla gli originali: esistono, non cestinati, corpo identico a
/// quello analizzato. Ritorna i titoli correnti (per audit e descrizioni).
fn verifica_originali(
    conn: &Connection,
    prompt: &[PromptDaApplicare],
) -> Result<HashMap<String, String>, PapErrore> {
    if prompt.is_empty() {
        return Err(rifiuta("Nessun prompt da applicare."));
    }
    if prompt.len() > MAX_PROMPT_RICHIESTA {
        return Err(rifiuta(format!(
            "Al massimo {MAX_PROMPT_RICHIESTA} prompt per volta."
        )));
    }
    let mut titoli = HashMap::new();
    for p in prompt {
        if titoli.contains_key(&p.id) {
            return Err(rifiuta(format!(
                "Il prompt «{}» compare due volte nella richiesta.",
                p.titolo
            )));
        }
        let riga: Option<(String, String)> = conn
            .query_row(
                "SELECT Title, Body FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
                [&p.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        let Some((titolo, body)) = riga else {
            return Err(rifiuta(format!(
                "Il prompt «{}» non esiste più: rilancia la scomposizione.",
                p.titolo
            )));
        };
        if body != p.corpo_originale_atteso {
            return Err(rifiuta(format!(
                "«{titolo}» è stato modificato dopo l'analisi: rilancia la scomposizione."
            )));
        }
        if p.corpo.trim().is_empty() {
            return Err(rifiuta(format!("Il corpo ricomposto di «{titolo}» è vuoto.")));
        }
        if p.titolo.trim().is_empty() {
            return Err(rifiuta(format!("Il titolo di «{titolo}» non può essere vuoto.")));
        }
        titoli.insert(p.id.clone(), titolo);
    }
    Ok(titoli)
}

/// Controlla i moduli da creare e prepara le rinomine per quelli riusati.
/// Ritorna `(rinomine titolo-proposto→titolo-effettivo, riusati)`.
fn verifica_moduli(
    conn: &Connection,
    moduli: &[ModuloDaApplicare],
) -> Result<(HashMap<String, String>, Vec<RiferimentoPrompt>), PapErrore> {
    if moduli.len() > MAX_MODULI_RICHIESTA {
        return Err(rifiuta(format!(
            "Al massimo {MAX_MODULI_RICHIESTA} moduli per volta."
        )));
    }
    let mut rinomine = HashMap::new();
    let mut riusati = Vec::new();
    let mut titoli_crea: HashSet<String> = HashSet::new();
    for m in moduli {
        let titolo = m.titolo.trim();
        match &m.azione {
            AzioneModulo::Scarta => {}
            AzioneModulo::Riusa { prompt_id } => {
                let esistente: Option<String> = conn
                    .query_row(
                        "SELECT Title FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
                        [prompt_id],
                        |r| r.get(0),
                    )
                    .ok();
                let Some(titolo_esistente) = esistente else {
                    return Err(rifiuta(format!(
                        "Il prompt da riusare per «{titolo}» non esiste più."
                    )));
                };
                if titolo_esistente.trim().to_lowercase() != titolo.to_lowercase() {
                    rinomine.insert(titolo.to_lowercase(), titolo_esistente.clone());
                }
                riusati.push(RiferimentoPrompt {
                    id: prompt_id.clone(),
                    titolo: titolo_esistente,
                });
            }
            AzioneModulo::Crea => {
                if titolo.is_empty() {
                    return Err(rifiuta("Un modulo da creare non ha titolo."));
                }
                if titolo.contains('/') {
                    return Err(rifiuta(format!(
                        "Il titolo «{titolo}» contiene «/»: non sarebbe importabile."
                    )));
                }
                if m.corpo.trim().is_empty() {
                    return Err(rifiuta(format!("Il modulo «{titolo}» ha il corpo vuoto.")));
                }
                if !parse_imports(&m.corpo).is_empty() {
                    return Err(rifiuta(format!(
                        "Il modulo «{titolo}» contiene un import: i moduli sono foglie."
                    )));
                }
                let errori = errori_linter(&m.corpo);
                if let Some(e) = errori.first() {
                    return Err(rifiuta(format!("Il modulo «{titolo}» ha un errore: {e}")));
                }
                if !titoli_crea.insert(titolo.to_lowercase()) {
                    return Err(rifiuta(format!("Due moduli da creare hanno lo stesso titolo «{titolo}».")));
                }
                if resolve_path(conn, titolo)?.is_some() {
                    return Err(rifiuta(format!(
                        "Esiste già un prompt intitolato «{titolo}»: rinomina il modulo o riusa quello esistente."
                    )));
                }
            }
        }
    }
    Ok((rinomine, riusati))
}

// ─────────────────────────────── applicazione ───────────────────────────────

/// Applica la richiesta in una transazione. Testabile senza Tauri.
pub(crate) fn applica_pure(
    conn: &Connection,
    rt_state: &EmbeddingsState,
    richiesta: &RichiestaApplica,
) -> Result<EsitoApplica, PapErrore> {
    let titoli_originali = verifica_originali(conn, &richiesta.prompt)?;
    let (rinomine, moduli_riusati) = verifica_moduli(conn, &richiesta.moduli)?;

    let ids: Vec<String> = richiesta.prompt.iter().map(|p| p.id.clone()).collect();
    let parent = cartella_comune(conn, &ids)?;
    let da_creare: Vec<&ModuloDaApplicare> = richiesta
        .moduli
        .iter()
        .filter(|m| m.azione == AzioneModulo::Crea)
        .collect();

    let tx = conn.unchecked_transaction()?;

    // 1. cartella + moduli
    let mut cartella_moduli = None;
    let mut moduli_creati = Vec::with_capacity(da_creare.len());
    if !da_creare.is_empty() {
        let folder_id = assicura_cartella_moduli(&tx, parent.as_deref())?;
        cartella_moduli = Some(
            tx.query_row("SELECT Path FROM Folders WHERE Id = ?1", [&folder_id], |r| {
                r.get::<_, String>(0)
            })?,
        );
        let nomi_originali: Vec<&str> = titoli_originali.values().map(String::as_str).collect();
        for m in &da_creare {
            let tipo = serde_json::to_value(m.tipo)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_else(|| "altro".to_string());
            let descrizione = format!(
                "Modulo ({tipo}) estratto con Cartamodello da: {}.",
                nomi_originali.join(", ")
            );
            let id = crate::editor::prompt_crea_in_db(
                &tx,
                rt_state,
                &crate::editor::NuovoPrompt {
                    titolo: m.titolo.trim().to_string(),
                    descrizione,
                    body: m.corpo.clone(),
                    visibilita: "private".to_string(),
                    tag_nomi: vec![],
                    target_model: None,
                    folder_id: Some(folder_id.clone()),
                },
            )?;
            moduli_creati.push(RiferimentoPrompt {
                id,
                titolo: m.titolo.trim().to_string(),
            });
        }
    }

    // 2. prompt ricomposti: gli import devono risolvere ORA, con i moduli
    //    appena creati e le rinomine dei riusati.
    let mut prompt_aggiornati = Vec::with_capacity(richiesta.prompt.len());
    for p in &richiesta.prompt {
        let corpo = rinomina_import(&p.corpo, &rinomine);
        for imp in parse_imports(&corpo) {
            if resolve_path(&tx, &imp.path)?.is_none() {
                return Err(rifiuta(format!(
                    "In «{}» l'import «{}» non risolve a nessun prompt.",
                    p.titolo.trim(),
                    imp.path
                )));
            }
        }
        if let Some(e) = errori_linter(&corpo).first() {
            return Err(rifiuta(format!("«{}» ha un errore: {e}", p.titolo.trim())));
        }
        let body = corpo.trim();
        // La descrizione resta quella dell'utente: «solo scomporre» (D5).
        tx.execute(
            "UPDATE Prompts
             SET Title = ?1, Body = ?2,
                 Version = Version + 1, UpdatedAt = datetime('now'),
                 UpdatedByUserId = 'usr-locale'
             WHERE Id = ?3 AND DeletedAt IS NULL",
            rusqlite::params![p.titolo.trim(), body, p.id],
        )?;
        crate::versioning::snapshot_versione(&tx, &p.id, "usr-locale")?;
        crate::editor::aggiorna_embedding(&tx, rt_state, &p.id, body)?;
        crate::prompt_componibili::aggiorna_imports(&tx, &p.id, body)?;
        crate::audit::registra(
            &tx,
            "prompt.scomposto",
            "Prompt",
            &p.id,
            Some(&format!(
                "moduli_creati={} moduli_riusati={}",
                moduli_creati.len(),
                moduli_riusati.len()
            )),
        );
        prompt_aggiornati.push(RiferimentoPrompt {
            id: p.id.clone(),
            titolo: p.titolo.trim().to_string(),
        });
    }
    // Ogni prompt ricomposto deve compilare (niente cicli, profondità
    // entro il limite) PRIMA del commit: un ciclo introdotto da un riuso
    // farebbe rollback invece di lasciare un prompt che non compila.
    for p in &richiesta.prompt {
        crate::prompt_componibili::compila_prompt_pure(&tx, &p.id).map_err(|e| {
            rifiuta(format!(
                "«{}» non compila dopo la scomposizione: {e}",
                p.titolo.trim()
            ))
        })?;
    }
    crate::editor::ricostruisci_fts(&tx)?;
    tx.commit()?;

    Ok(EsitoApplica {
        moduli_creati,
        moduli_riusati,
        prompt_aggiornati,
        cartella_moduli,
    })
}

/// Cmd Tauri: applica la proposta confermata. Ri-verifica tutto e scrive in
/// una transazione; niente rete.
#[tauri::command]
pub fn cartamodello_applica(
    richiesta: RichiestaApplica,
    state: tauri::State<'_, crate::vault::VaultState>,
    rt_state: tauri::State<'_, EmbeddingsState>,
) -> Result<EsitoApplica, PapErrore> {
    state.with_conn(|conn| applica_pure(conn, &rt_state, &richiesta))
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

    fn rt() -> EmbeddingsState {
        EmbeddingsState::new()
    }

    fn inserisci_prompt(conn: &Connection, id: &str, titolo: &str, body: &str, folder: Option<&str>) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version, FolderId, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, 'private', 1, ?4, datetime('now'), datetime('now'))",
            rusqlite::params![id, titolo, body, folder],
        )
        .unwrap();
        crate::versioning::snapshot_versione(conn, id, "usr-locale").unwrap();
    }

    fn cartella(conn: &Connection, id: &str, nome: &str, path: &str) {
        conn.execute(
            "INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path) VALUES (?1, 'ws-personale', NULL, ?2, ?3)",
            rusqlite::params![id, nome, path],
        )
        .unwrap();
    }

    const ORIG: &str = "Sei un senior engineer. Fai la review di {{codice}}";
    const RUOLO: &str = "Sei un senior engineer.";

    fn modulo_crea(titolo: &str, corpo: &str) -> ModuloDaApplicare {
        ModuloDaApplicare {
            titolo: titolo.into(),
            tipo: TipoModulo::Ruolo,
            corpo: corpo.into(),
            azione: AzioneModulo::Crea,
        }
    }

    fn prompt(id: &str, titolo: &str, corpo: &str, atteso: &str) -> PromptDaApplicare {
        PromptDaApplicare {
            id: id.into(),
            titolo: titolo.into(),
            corpo: corpo.into(),
            corpo_originale_atteso: atteso.into(),
        }
    }

    fn richiesta_base() -> RichiestaApplica {
        RichiestaApplica {
            moduli: vec![modulo_crea("Ruolo senior engineer", RUOLO)],
            prompt: vec![prompt(
                "p1",
                "Code review",
                "{{import \"Ruolo senior engineer\"}} Fai la review di {{codice}}",
                ORIG,
            )],
        }
    }

    #[test]
    fn applica_crea_modulo_in_moduli_e_nuova_versione_dell_originale() {
        let conn = db_test();
        cartella(&conn, "fld-dev", "Sviluppo", "/Sviluppo");
        inserisci_prompt(&conn, "p1", "Review", ORIG, Some("fld-dev"));

        let esito = applica_pure(&conn, &rt(), &richiesta_base()).unwrap();

        assert_eq!(esito.moduli_creati.len(), 1);
        assert_eq!(esito.cartella_moduli.as_deref(), Some("/Sviluppo/Moduli"));
        assert_eq!(esito.prompt_aggiornati[0].titolo, "Code review");
        // Il modulo sta nella sottocartella Moduli di /Sviluppo.
        let (folder_path, body_modulo): (String, String) = conn
            .query_row(
                "SELECT f.Path, p.Body FROM Prompts p JOIN Folders f ON p.FolderId = f.Id WHERE p.Id = ?1",
                [&esito.moduli_creati[0].id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(folder_path, "/Sviluppo/Moduli");
        assert_eq!(body_modulo, RUOLO);
        // L'originale è alla versione 2 con il corpo ricomposto; la v1 è in cronologia.
        let (ver, body, title): (i64, String, String) = conn
            .query_row("SELECT Version, Body, Title FROM Prompts WHERE Id = 'p1'", [], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .unwrap();
        assert_eq!(ver, 2);
        assert!(body.starts_with("{{import \"Ruolo senior engineer\"}}"));
        assert_eq!(title, "Code review");
        let v1: String = conn
            .query_row("SELECT Body FROM PromptVersions WHERE PromptId = 'p1' AND Version = 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v1, ORIG);
        // Il grafo import conosce la relazione.
        let n_imp: i64 = conn
            .query_row("SELECT COUNT(*) FROM PromptImports WHERE ParentPromptId = 'p1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n_imp, 1);
        // L'anteprima compilata equivale all'originale.
        let compilato = crate::prompt_componibili::compila_prompt_pure(&conn, "p1").unwrap();
        assert_eq!(compilato, ORIG);
    }

    #[test]
    fn applica_riusa_la_cartella_moduli_se_esiste_e_mette_a_radice_se_cartelle_diverse() {
        let conn = db_test();
        inserisci_prompt(&conn, "p1", "Uno", ORIG, None);
        inserisci_prompt(&conn, "p2", "Due", "Sei un senior engineer. Scrivi test per {{codice}}", None);
        let mut r = richiesta_base();
        r.prompt.push(prompt(
            "p2",
            "Test",
            "{{import \"Ruolo senior engineer\"}}\nScrivi test per {{codice}}",
            "Sei un senior engineer. Scrivi test per {{codice}}",
        ));
        let esito = applica_pure(&conn, &rt(), &r).unwrap();
        assert_eq!(esito.cartella_moduli.as_deref(), Some("/Moduli"));
        assert_eq!(esito.prompt_aggiornati.len(), 2);

        // Seconda applicazione con un altro modulo: la cartella Moduli non si duplica.
        inserisci_prompt(&conn, "p3", "Tre", "Rispondi in JSON. {{x}}", None);
        let r2 = RichiestaApplica {
            moduli: vec![modulo_crea("Formato JSON", "Rispondi in JSON.")],
            prompt: vec![prompt("p3", "Tre", "{{import \"Formato JSON\"}} {{x}}", "Rispondi in JSON. {{x}}")],
        };
        applica_pure(&conn, &rt(), &r2).unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM Folders WHERE Name = 'Moduli'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn applica_riusa_un_modulo_esistente_riscrivendo_l_import() {
        let conn = db_test();
        inserisci_prompt(&conn, "vecchio", "Ruolo tech lead", RUOLO, None);
        inserisci_prompt(&conn, "p1", "Review", ORIG, None);
        let r = RichiestaApplica {
            moduli: vec![ModuloDaApplicare {
                azione: AzioneModulo::Riusa { prompt_id: "vecchio".into() },
                ..modulo_crea("Ruolo senior engineer", RUOLO)
            }],
            prompt: richiesta_base().prompt,
        };
        let esito = applica_pure(&conn, &rt(), &r).unwrap();
        assert!(esito.moduli_creati.is_empty());
        assert_eq!(esito.moduli_riusati[0].titolo, "Ruolo tech lead");
        assert!(esito.cartella_moduli.is_none(), "nessuna cartella creata senza moduli nuovi");
        let body: String = conn
            .query_row("SELECT Body FROM Prompts WHERE Id = 'p1'", [], |r| r.get(0))
            .unwrap();
        assert!(body.starts_with("{{import \"Ruolo tech lead\"}}"), "{body}");
    }

    #[test]
    fn applica_rifiuta_se_l_originale_e_cambiato_e_non_scrive_nulla() {
        let conn = db_test();
        inserisci_prompt(&conn, "p1", "Review", "corpo cambiato nel frattempo", None);
        let err = applica_pure(&conn, &rt(), &richiesta_base()).unwrap_err();
        assert!(err.to_string().contains("modificato dopo l'analisi"));
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM Prompts", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
        let n_f: i64 = conn.query_row("SELECT COUNT(*) FROM Folders", [], |r| r.get(0)).unwrap();
        assert_eq!(n_f, 0);
    }

    #[test]
    fn applica_rifiuta_titolo_gia_esistente_import_non_risolto_e_modulo_con_import() {
        let conn = db_test();
        inserisci_prompt(&conn, "p1", "Review", ORIG, None);
        inserisci_prompt(&conn, "occupato", "Ruolo senior engineer", "altro", None);
        let err = applica_pure(&conn, &rt(), &richiesta_base()).unwrap_err();
        assert!(err.to_string().contains("Esiste già un prompt intitolato"));

        // Import verso un modulo scartato → l'applicazione si ferma e fa rollback.
        conn.execute("UPDATE Prompts SET DeletedAt = datetime('now') WHERE Id = 'occupato'", []).unwrap();
        let mut r = richiesta_base();
        r.moduli[0].azione = AzioneModulo::Scarta;
        let err = applica_pure(&conn, &rt(), &r).unwrap_err();
        assert!(err.to_string().contains("non risolve"), "{err}");
        let ver: i64 = conn.query_row("SELECT Version FROM Prompts WHERE Id = 'p1'", [], |r| r.get(0)).unwrap();
        assert_eq!(ver, 1, "rollback atteso");

        let mut r = richiesta_base();
        r.moduli[0].corpo = "Sei X. {{import \"Y\"}}".into();
        let err = applica_pure(&conn, &rt(), &r).unwrap_err();
        assert!(err.to_string().contains("foglie"));
    }

    #[test]
    fn applica_rollback_se_un_modulo_fallisce_dopo_il_primo() {
        // Due moduli da creare con lo stesso titolo: la verifica lo coglie
        // prima della transazione, ma se anche passasse, nulla resta scritto.
        let conn = db_test();
        inserisci_prompt(&conn, "p1", "Review", ORIG, None);
        let mut r = richiesta_base();
        r.moduli.push(modulo_crea("ruolo senior engineer", "Altro corpo."));
        let err = applica_pure(&conn, &rt(), &r).unwrap_err();
        assert!(err.to_string().contains("stesso titolo"));
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM Prompts", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn applica_rifiuta_id_duplicati_e_richieste_oltre_i_tetti() {
        let conn = db_test();
        inserisci_prompt(&conn, "p1", "Review", ORIG, None);
        let mut r = richiesta_base();
        r.prompt.push(r.prompt[0].clone());
        let err = applica_pure(&conn, &rt(), &r).unwrap_err();
        assert!(err.to_string().contains("due volte"));

        let mut r = richiesta_base();
        for i in 0..MAX_MODULI_RICHIESTA {
            r.moduli.push(modulo_crea(&format!("M{i}"), "corpo"));
        }
        let err = applica_pure(&conn, &rt(), &r).unwrap_err();
        assert!(err.to_string().contains("moduli per volta"));
        let ver: i64 = conn.query_row("SELECT Version FROM Prompts WHERE Id = 'p1'", [], |r| r.get(0)).unwrap();
        assert_eq!(ver, 1);
    }

    #[test]
    fn applica_fa_rollback_se_il_ricomposto_non_compila() {
        // Un riuso che punta a un prompt il quale importa a sua volta
        // l'originale crea un ciclo: la compilazione fallisce dentro la
        // transazione e nulla resta scritto.
        let conn = db_test();
        inserisci_prompt(&conn, "p1", "Review", ORIG, None);
        inserisci_prompt(&conn, "ciclico", "Ruolo ciclico", "{{import \"Review\"}} e altro", None);
        let r = RichiestaApplica {
            moduli: vec![ModuloDaApplicare {
                azione: AzioneModulo::Riusa { prompt_id: "ciclico".into() },
                ..modulo_crea("Ruolo senior engineer", RUOLO)
            }],
            prompt: vec![prompt(
                "p1",
                "Review",
                "{{import \"Ruolo senior engineer\"}} Fai la review di {{codice}}",
                ORIG,
            )],
        };
        let err = applica_pure(&conn, &rt(), &r).unwrap_err();
        assert!(err.to_string().contains("non compila"), "{err}");
        let (ver, body): (i64, String) = conn
            .query_row("SELECT Version, Body FROM Prompts WHERE Id = 'p1'", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert_eq!(ver, 1);
        assert_eq!(body, ORIG);
    }

    #[test]
    fn path_cartella_moduli_segue_la_cartella_comune() {
        let conn = db_test();
        cartella(&conn, "fld-a", "A", "/A");
        inserisci_prompt(&conn, "p1", "Uno", "x", Some("fld-a"));
        inserisci_prompt(&conn, "p2", "Due", "y", Some("fld-a"));
        inserisci_prompt(&conn, "p3", "Tre", "z", None);
        assert_eq!(path_cartella_moduli(&conn, &["p1".into(), "p2".into()]).unwrap(), "/A/Moduli");
        assert_eq!(path_cartella_moduli(&conn, &["p1".into(), "p3".into()]).unwrap(), "/Moduli");
    }
}
