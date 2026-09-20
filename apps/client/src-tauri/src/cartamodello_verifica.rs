//! Cartamodello — verifica locale della proposta del modello.
//!
//! La risposta dell'LLM è un dato non fidato: qui la si controlla e
//! arricchisce PRIMA di mostrarla, con funzioni deterministiche e testabili
//! su un DB in memoria (nessuna rete):
//!
//! - **dedup**: moduli con lo stesso corpo (normalizzato) vengono fusi e gli
//!   import dei prompt riscritti sul superstite;
//! - **moduli**: corpo non vuoto, nessun `{{import}}` (i moduli sono foglie),
//!   nessun errore del linter, titolo non vuoto e non già usato nel vault;
//! - **riuso**: per ogni modulo si cerca nel vault un prompt equivalente —
//!   semantico (embedding, cosine ≥ soglia) se il modello è caricato,
//!   altrimenti lessicale (Jaccard sulle parole) — e lo si propone al posto
//!   della creazione;
//! - **prompt**: ogni `{{import}}` deve risolvere a un modulo proposto o a un
//!   prompt del vault; si scartano i prompt con id sconosciuto e si segnalano
//!   gli originali che il modello non ha restituito;
//! - **anteprima espansa**: import sostituiti (un livello, `with` applicato)
//!   per il diff di fedeltà con l'originale, lato UI.
//!
//! Blueprint: `docs/roadmap/cartamodello.md` §3.4.

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use serde::Serialize;

use crate::cartamodello::{ModuloProposto, PromptOriginale, RispostaModello, TipoModulo};
use crate::embeddings::{assicura_session_caricata, compute_embedding_opt, EmbeddingsState};
use crate::errore::PapErrore;
use crate::linting::{analizza, Severita};
use crate::prompt_componibili::{applica_variabili_scoped, parse_imports, resolve_path};
use crate::ricerca_ibrida::valuta_disponibilita_modello;
use crate::vault::VaultState;

/// Cosine minima perché un prompt del vault sia proposto come riuso
/// (embedding L2-normalizzati: cosine = 1 − d²/2).
const SOGLIA_COSINE_RIUSO: f64 = 0.85;
/// Jaccard minimo sulle parole per il riuso lessicale (fallback senza modello).
const SOGLIA_JACCARD_RIUSO: f64 = 0.6;
/// Quanti vicini semantici esaminare prima di rinunciare.
const K_VICINI: usize = 5;

// ───────────────────────────── tipi in uscita ─────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FonteRiuso {
    Semantica,
    Lessicale,
}

/// Un prompt già nel vault equivalente a un modulo proposto.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Riuso {
    pub prompt_id: String,
    pub titolo: String,
    /// Cosine (semantica) o Jaccard (lessicale), in [0, 1].
    pub similarita: f64,
    pub fonte: FonteRiuso,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModuloVerificato {
    pub chiave: String,
    pub titolo: String,
    pub tipo: TipoModulo,
    pub corpo: String,
    pub usato_da: Vec<String>,
    /// Problemi che bloccano la creazione (linter, import nel modulo, …).
    pub problemi: Vec<String>,
    /// Prompt esistente proposto al posto della creazione.
    pub riuso: Option<Riuso>,
    /// Id di un prompt del vault che ha già questo titolo: creare il modulo
    /// renderebbe ambiguo `{{import "Titolo"}}`. La UI chiede di rinominare.
    pub titolo_in_conflitto: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PromptVerificato {
    pub id: String,
    pub titolo_originale: String,
    pub titolo: String,
    pub descrizione: Option<String>,
    pub corpo: String,
    pub corpo_originale: String,
    /// Corpo con gli import sostituiti (un livello). `None` se un import
    /// non risolve.
    pub anteprima_espansa: Option<String>,
    pub import_non_risolti: Vec<String>,
    pub problemi: Vec<String>,
    /// Segnaposti estratti dal corpo ricomposto (non dalla lista del modello).
    pub segnaposti: Vec<String>,
}

/// La proposta verificata, così come arriva alla UI.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Proposta {
    pub moduli: Vec<ModuloVerificato>,
    pub prompt: Vec<PromptVerificato>,
    /// Note del modello.
    pub note: Vec<String>,
    /// Avvisi della verifica locale (prompt mancanti, id sconosciuti, dedup).
    pub avvisi: Vec<String>,
    /// Il riuso semantico era disponibile (modello embedding caricato)?
    pub riuso_semantico: bool,
    pub tokens_used: Option<u32>,
    pub costo_stimato: Option<f64>,
    pub provider: String,
    pub model: String,
    pub troncato: bool,
}

// ─────────────────────────────── helper puri ───────────────────────────────

/// Normalizza un corpo per il confronto: spazi collassati, minuscolo.
fn normalizza(testo: &str) -> String {
    testo.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

fn parole(testo: &str) -> HashSet<String> {
    testo
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 3)
        .map(|w| w.to_lowercase())
        .collect()
}

fn jaccard_insiemi(pa: &HashSet<String>, pb: &HashSet<String>) -> f64 {
    if pa.is_empty() && pb.is_empty() {
        return 1.0;
    }
    let inter = pa.intersection(pb).count() as f64;
    let union = pa.union(pb).count() as f64;
    inter / union
}

/// Jaccard sulle parole (≥ 3 caratteri) di due testi, in [0, 1].
#[cfg(test)]
pub(crate) fn jaccard_parole(a: &str, b: &str) -> f64 {
    jaccard_insiemi(&parole(a), &parole(b))
}

/// Cosine da distanza L2 fra vettori unitari.
pub(crate) fn cosine_da_l2(d: f64) -> f64 {
    (1.0 - d * d / 2.0).clamp(-1.0, 1.0)
}

fn errori_linter(corpo: &str) -> Vec<String> {
    analizza(corpo)
        .into_iter()
        .filter(|i| i.severita == Severita::Error)
        .map(|i| format!("{}: {}", i.code, i.messaggio))
        .collect()
}

/// Segnaposti `{{nome}}` del corpo, in ordine di apparizione, senza
/// duplicati; esclude globali, import e commenti.
pub(crate) fn estrai_segnaposti(corpo: &str) -> Vec<String> {
    use regex::Regex;
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\{\{\s*([A-Za-z_][A-Za-z0-9_]*)\s*\}\}").unwrap());
    let senza_commenti = rimuovi_commenti(corpo);
    let mut visti = HashSet::new();
    let mut out = Vec::new();
    for cap in re.captures_iter(&senza_commenti) {
        let nome = cap[1].to_string();
        if nome == "global" || nome == "import" {
            continue;
        }
        if visti.insert(nome.clone()) {
            out.push(nome);
        }
    }
    out
}

fn rimuovi_commenti(corpo: &str) -> String {
    let mut out = String::with_capacity(corpo.len());
    let mut resto = corpo;
    while let Some(i) = resto.find("{{!--") {
        out.push_str(&resto[..i]);
        match resto[i + 5..].find("--}}") {
            Some(j) => resto = &resto[i + 5 + j + 4..],
            None => return out,
        }
    }
    out.push_str(resto);
    out
}

/// Riscrive i path degli `{{import}}` che puntano a un modulo scartato dal
/// dedup verso il superstite. Lavora sui token veri (range di
/// `parse_imports`, dall'ultimo al primo) e confronta i titoli senza
/// distinguere maiuscole/spazi, come tutte le altre risoluzioni di titolo
/// in questo file (review PR-1: una `replace` testuale case-sensitive
/// lasciava import pendenti e toccava anche la prosa).
fn rinomina_import(corpo: &str, rinomine: &HashMap<String, String>) -> String {
    if rinomine.is_empty() {
        return corpo.to_string();
    }
    let mut out = corpo.to_string();
    for imp in parse_imports(corpo).into_iter().rev() {
        let chiave = imp.path.trim().trim_start_matches('/').to_lowercase();
        let Some(nuovo) = rinomine.get(&chiave) else {
            continue;
        };
        let token = &corpo[imp.byte_start..imp.byte_end];
        let vecchio_quotato = format!("\"{}\"", imp.path);
        if let Some(pos) = token.find(&vecchio_quotato) {
            let inizio = imp.byte_start + pos;
            let fine = inizio + vecchio_quotato.len();
            out.replace_range(inizio..fine, &format!("\"{nuovo}\""));
        }
    }
    out
}

// ─────────────────────────────── dedup ───────────────────────────────

/// Fonde i moduli con lo stesso corpo normalizzato; ritorna i superstiti e
/// la mappa titolo-scartato → titolo-superstite per riscrivere gli import.
fn deduplica(moduli: Vec<ModuloProposto>) -> (Vec<ModuloProposto>, HashMap<String, String>) {
    let mut per_corpo: HashMap<String, usize> = HashMap::new();
    let mut superstiti: Vec<ModuloProposto> = Vec::new();
    let mut rinomine = HashMap::new();
    for m in moduli {
        let chiave = normalizza(&m.corpo);
        match per_corpo.get(&chiave) {
            Some(&i) if !chiave.is_empty() => {
                let s = &mut superstiti[i];
                for u in m.usato_da {
                    if !s.usato_da.contains(&u) {
                        s.usato_da.push(u);
                    }
                }
                let chiave_titolo = m.titolo.trim().to_lowercase();
                if chiave_titolo != s.titolo.trim().to_lowercase() {
                    rinomine.insert(chiave_titolo, s.titolo.clone());
                }
            }
            _ => {
                per_corpo.insert(chiave, superstiti.len());
                superstiti.push(m);
            }
        }
    }
    (superstiti, rinomine)
}

// ─────────────────────────────── riuso ───────────────────────────────

fn carica_titolo_e_body(conn: &Connection, id: &str) -> Option<(String, String)> {
    conn.query_row(
        "SELECT Title, Body FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .ok()
}

fn riuso_semantico(
    conn: &Connection,
    rt: &EmbeddingsState,
    corpo: &str,
    escludi: &HashSet<String>,
) -> Result<Option<Riuso>, PapErrore> {
    let Some(emb) = compute_embedding_opt(rt, corpo)? else {
        return Ok(None);
    };
    for (id, dist) in crate::embeddings_store::search_nearest(conn, &emb, K_VICINI)? {
        if escludi.contains(&id) {
            continue;
        }
        let sim = cosine_da_l2(dist);
        if sim < SOGLIA_COSINE_RIUSO {
            break; // ordinati per distanza: i successivi sono peggiori
        }
        if let Some((titolo, _)) = carica_titolo_e_body(conn, &id) {
            return Ok(Some(Riuso {
                prompt_id: id,
                titolo,
                similarita: sim,
                fonte: FonteRiuso::Semantica,
            }));
        }
    }
    Ok(None)
}

/// Vault tokenizzato una volta per `verifica` (non per modulo): id, titolo e
/// insieme di parole di ogni prompt attivo fuori dal lotto.
struct IndiceLessicale {
    righe: Vec<(String, String, HashSet<String>)>,
}

impl IndiceLessicale {
    fn carica(conn: &Connection, escludi: &HashSet<String>) -> Result<Self, PapErrore> {
        let mut stmt =
            conn.prepare("SELECT Id, Title, Body FROM Prompts WHERE DeletedAt IS NULL")?;
        let righe = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .filter(|(id, _, _)| !escludi.contains(id))
            .map(|(id, titolo, body)| (id, titolo, parole(&body)))
            .collect();
        Ok(Self { righe })
    }

    /// Fallback senza modello: Jaccard sulle parole contro tutto il vault.
    /// Il vault personale è di centinaia di prompt: O(N) per modulo va bene.
    fn riuso(&self, corpo: &str) -> Option<Riuso> {
        let pc = parole(corpo);
        let mut migliore: Option<Riuso> = None;
        for (id, titolo, pb) in &self.righe {
            let sim = jaccard_insiemi(&pc, pb);
            let meglio = migliore.as_ref().map_or(true, |m| sim > m.similarita);
            if sim >= SOGLIA_JACCARD_RIUSO && meglio {
                migliore = Some(Riuso {
                    prompt_id: id.clone(),
                    titolo: titolo.clone(),
                    similarita: sim,
                    fonte: FonteRiuso::Lessicale,
                });
            }
        }
        migliore
    }
}

// ─────────────────────────────── verifica ───────────────────────────────

/// Verifica e arricchisce la proposta. `riuso` = `(vault, embeddings)` per
/// il riuso semantico; `None` = solo lessicale (test, o chiamanti senza
/// runtime embedding).
pub(crate) fn verifica(
    conn: &Connection,
    originali: &[PromptOriginale],
    risposta: RispostaModello,
    riuso: Option<(&VaultState, &EmbeddingsState)>,
) -> Result<Proposta, PapErrore> {
    let mut avvisi = Vec::new();
    let ids_originali: HashSet<String> = originali.iter().map(|p| p.id.clone()).collect();

    // 1. dedup moduli
    let n_prima = risposta.moduli.len();
    let (moduli, rinomine) = deduplica(risposta.moduli);
    if n_prima != moduli.len() {
        avvisi.push(format!(
            "{} moduli con lo stesso contenuto sono stati fusi.",
            n_prima - moduli.len()
        ));
    }

    // 2. riuso semantico disponibile?
    let rt_disponibile = riuso.and_then(|(vault, rt)| {
        valuta_disponibilita_modello(assicura_session_caricata(rt, vault)).then_some(rt)
    });

    // 3. moduli. L'indice lessicale si costruisce una volta sola, e solo se
    // serve (nessun modello embedding e almeno un modulo con corpo).
    let indice_lessicale = match rt_disponibile {
        None if moduli.iter().any(|m| !m.corpo.trim().is_empty()) => {
            Some(IndiceLessicale::carica(conn, &ids_originali)?)
        }
        _ => None,
    };
    let mut titoli_moduli: HashMap<String, usize> = HashMap::new();
    let mut moduli_verificati = Vec::with_capacity(moduli.len());
    for (i, m) in moduli.into_iter().enumerate() {
        let mut problemi = Vec::new();
        if m.titolo.trim().is_empty() {
            problemi.push("Titolo mancante.".to_string());
        }
        if m.corpo.trim().is_empty() {
            problemi.push("Corpo vuoto.".to_string());
        }
        if !parse_imports(&m.corpo).is_empty() {
            problemi.push("Un modulo non può contenere {{import}}: i moduli sono foglie.".to_string());
        }
        if m.titolo.contains('/') {
            // `{{import "a/b"}}` legge `a` come cartella: un titolo con la barra
            // non sarebbe importabile per titolo.
            problemi.push("Il titolo non può contenere «/».".to_string());
        }
        problemi.extend(errori_linter(&m.corpo));
        let chiave_titolo = m.titolo.trim().to_lowercase();
        if titoli_moduli.insert(chiave_titolo, i).is_some() {
            problemi.push("Titolo uguale a un altro modulo proposto.".to_string());
        }
        let titolo_in_conflitto = resolve_path(conn, m.titolo.trim())?;
        let riuso_trovato = if m.corpo.trim().is_empty() {
            None
        } else {
            match (rt_disponibile, &indice_lessicale) {
                (Some(rt), _) => riuso_semantico(conn, rt, &m.corpo, &ids_originali)?,
                (None, Some(indice)) => indice.riuso(&m.corpo),
                (None, None) => None,
            }
        };
        moduli_verificati.push(ModuloVerificato {
            chiave: m.chiave,
            titolo: m.titolo,
            tipo: m.tipo,
            corpo: m.corpo,
            usato_da: m.usato_da,
            problemi,
            riuso: riuso_trovato,
            titolo_in_conflitto,
        });
    }

    // 4. prompt
    let mut coperti: HashSet<String> = HashSet::new();
    let mut prompt_verificati = Vec::new();
    for p in risposta.prompt {
        let Some(orig) = originali.iter().find(|o| o.id == p.id) else {
            avvisi.push(format!(
                "Il modello ha restituito un prompt con id sconosciuto ({:?}): ignorato.",
                p.id.chars().take(40).collect::<String>()
            ));
            continue;
        };
        if !coperti.insert(p.id.clone()) {
            avvisi.push(format!("Il modello ha restituito due volte «{}»: tenuto il primo.", orig.titolo));
            continue;
        }
        let corpo = rinomina_import(&p.corpo, &rinomine);
        let mut problemi = errori_linter(&corpo);
        if corpo.trim().is_empty() {
            problemi.push("Corpo vuoto.".to_string());
        }
        let mut import_non_risolti = Vec::new();
        let mut espansa = corpo.clone();
        // Sostituzione dall'ultimo al primo: i range byte restano validi.
        for imp in parse_imports(&corpo).into_iter().rev() {
            let chiave = imp.path.trim().trim_start_matches('/').to_lowercase();
            let body_modulo = titoli_moduli
                .get(&chiave)
                .map(|&i| moduli_verificati[i].corpo.clone())
                .or_else(|| {
                    resolve_path(conn, &imp.path)
                        .ok()
                        .flatten()
                        .and_then(|id| carica_titolo_e_body(conn, &id))
                        .map(|(_, body)| body)
                });
            match body_modulo {
                Some(body) => {
                    let sostituito = applica_variabili_scoped(&body, &imp.variables);
                    espansa.replace_range(imp.byte_start..imp.byte_end, &sostituito);
                }
                None => import_non_risolti.push(imp.path.clone()),
            }
        }
        let anteprima_espansa = import_non_risolti.is_empty().then_some(espansa);
        let segnaposti = estrai_segnaposti(&corpo);
        prompt_verificati.push(PromptVerificato {
            id: p.id,
            titolo_originale: orig.titolo.clone(),
            titolo: if p.titolo.trim().is_empty() {
                orig.titolo.clone()
            } else {
                p.titolo
            },
            descrizione: p.descrizione,
            corpo,
            corpo_originale: orig.body.clone(),
            anteprima_espansa,
            import_non_risolti,
            problemi,
            segnaposti,
        });
    }
    for o in originali {
        if !coperti.contains(&o.id) {
            avvisi.push(format!(
                "Il modello non ha restituito una versione ricomposta di «{}».",
                o.titolo
            ));
        }
    }

    Ok(Proposta {
        moduli: moduli_verificati,
        prompt: prompt_verificati,
        note: risposta.note,
        avvisi,
        riuso_semantico: rt_disponibile.is_some(),
        ..Default::default()
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cartamodello::PromptProposto;

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    fn inserisci_prompt(conn: &Connection, id: &str, titolo: &str, body: &str) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body, Visibility, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, 'private', 1, datetime('now'), datetime('now'))",
            rusqlite::params![id, titolo, body],
        )
        .unwrap();
    }

    fn orig(id: &str, titolo: &str, body: &str) -> PromptOriginale {
        PromptOriginale { id: id.into(), titolo: titolo.into(), body: body.into() }
    }

    fn modulo(chiave: &str, titolo: &str, corpo: &str) -> ModuloProposto {
        ModuloProposto {
            chiave: chiave.into(),
            titolo: titolo.into(),
            tipo: TipoModulo::Ruolo,
            corpo: corpo.into(),
            usato_da: vec![],
        }
    }

    fn prompt(id: &str, titolo: &str, corpo: &str) -> PromptProposto {
        PromptProposto {
            id: id.into(),
            titolo: titolo.into(),
            descrizione: None,
            corpo: corpo.into(),
            segnaposti: vec![],
        }
    }

    const RUOLO: &str = "Sei un senior engineer specializzato in {{linguaggio}} con dieci anni di esperienza su sistemi in produzione.";

    // ───────── helper puri ─────────

    #[test]
    fn jaccard_e_cosine_hanno_i_valori_attesi() {
        assert_eq!(jaccard_parole("", ""), 1.0);
        assert_eq!(jaccard_parole("uno due tre", "quattro cinque sei"), 0.0);
        assert!((jaccard_parole("uno due tre", "uno due tre") - 1.0).abs() < 1e-9);
        assert!((jaccard_parole("alpha beta gamma delta", "alpha beta") - 0.5).abs() < 1e-9);
        assert!((cosine_da_l2(0.0) - 1.0).abs() < 1e-9);
        assert!((cosine_da_l2(2.0_f64.sqrt()) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn estrai_segnaposti_ignora_globali_import_e_commenti() {
        let corpo = "{{import \"Ruolo\" with x=1}}\n{{testo}} {{global autore}} {{!-- {{nascosto}} --}} {{testo}} {{lingua_a}}";
        assert_eq!(estrai_segnaposti(corpo), vec!["testo", "lingua_a"]);
    }

    #[test]
    fn deduplica_fonde_per_corpo_e_registra_le_rinomine() {
        let (sup, rin) = deduplica(vec![
            ModuloProposto { usato_da: vec!["p1".into()], ..modulo("a", "Ruolo A", "Sei  un esperto.") },
            ModuloProposto { usato_da: vec!["p2".into()], ..modulo("b", "Ruolo B", "sei un esperto.") },
            modulo("c", "Altro", "Formato: JSON."),
        ]);
        assert_eq!(sup.len(), 2);
        assert_eq!(sup[0].usato_da, vec!["p1", "p2"]);
        assert_eq!(rin.get("ruolo b").map(String::as_str), Some("Ruolo A"));
    }

    // ───────── verifica ─────────

    #[test]
    fn verifica_caso_felice_espande_e_estrae_segnaposti() {
        let conn = db_test();
        let originali = [orig("p1", "Review", "Sei un senior engineer… fai la review di {{codice}}")];
        let risposta = RispostaModello {
            moduli: vec![modulo("r", "Ruolo senior engineer", RUOLO)],
            prompt: vec![prompt(
                "p1",
                "Code review",
                "{{import \"Ruolo senior engineer\" with linguaggio=Rust}}\n\nFai la review di:\n{{codice}}",
            )],
            note: vec!["ok".into()],
        };

        let p = verifica(&conn, &originali, risposta, None).unwrap();

        assert!(p.avvisi.is_empty(), "{:?}", p.avvisi);
        assert_eq!(p.moduli.len(), 1);
        assert!(p.moduli[0].problemi.is_empty(), "{:?}", p.moduli[0].problemi);
        assert!(p.moduli[0].riuso.is_none());
        assert!(p.moduli[0].titolo_in_conflitto.is_none());
        let pr = &p.prompt[0];
        assert!(pr.import_non_risolti.is_empty());
        assert!(pr.problemi.is_empty(), "{:?}", pr.problemi);
        let esp = pr.anteprima_espansa.as_deref().unwrap();
        assert!(esp.starts_with("Sei un senior engineer specializzato in Rust"));
        assert!(esp.contains("{{codice}}"));
        assert_eq!(pr.segnaposti, vec!["codice"]);
        assert_eq!(pr.corpo_originale, originali[0].body);
        assert!(!p.riuso_semantico);
    }

    #[test]
    fn verifica_segnala_import_non_risolti_e_prompt_mancanti() {
        let conn = db_test();
        let originali = [orig("p1", "Uno", "a"), orig("p2", "Due", "b")];
        let risposta = RispostaModello {
            moduli: vec![],
            prompt: vec![
                prompt("p1", "Uno", "{{import \"Non esiste\"}}\n{{testo}}"),
                prompt("p9", "Fantasma", "x"),
            ],
            note: vec![],
        };
        let p = verifica(&conn, &originali, risposta, None).unwrap();
        assert_eq!(p.prompt.len(), 1);
        assert_eq!(p.prompt[0].import_non_risolti, vec!["Non esiste"]);
        assert!(p.prompt[0].anteprima_espansa.is_none());
        assert!(p.avvisi.iter().any(|a| a.contains("id sconosciuto")));
        assert!(p.avvisi.iter().any(|a| a.contains("«Due»")));
    }

    #[test]
    fn verifica_risolve_import_verso_prompt_gia_nel_vault() {
        let conn = db_test();
        inserisci_prompt(&conn, "esistente", "Ruolo editor", "Sei un editor.");
        let originali = [orig("p1", "Uno", "Sei un editor. Correggi {{testo}}")];
        let risposta = RispostaModello {
            moduli: vec![],
            prompt: vec![prompt("p1", "Uno", "{{import \"Ruolo editor\"}}\nCorreggi {{testo}}")],
            note: vec![],
        };
        let p = verifica(&conn, &originali, risposta, None).unwrap();
        assert!(p.prompt[0].import_non_risolti.is_empty());
        assert_eq!(p.prompt[0].anteprima_espansa.as_deref(), Some("Sei un editor.\nCorreggi {{testo}}"));
    }

    #[test]
    fn verifica_blocca_moduli_con_import_linter_o_titolo_duplicato() {
        let conn = db_test();
        let originali = [orig("p1", "Uno", "x")];
        let risposta = RispostaModello {
            moduli: vec![
                modulo("a", "Ruolo", "Sei X. {{import \"Altro\"}}"),
                modulo("b", "ruolo", "Segnaposto a graffa singola {nome}"),
                modulo("c", "", "   "),
            ],
            prompt: vec![],
            note: vec![],
        };
        let p = verifica(&conn, &originali, risposta, None).unwrap();
        assert!(p.moduli[0].problemi.iter().any(|e| e.contains("foglie")));
        assert!(p.moduli[1].problemi.iter().any(|e| e.contains("altro modulo proposto")));
        assert!(p.moduli[1].problemi.iter().any(|e| e.starts_with("PH001")), "{:?}", p.moduli[1].problemi);
        assert!(p.moduli[2].problemi.iter().any(|e| e.contains("Titolo mancante")));
        assert!(p.moduli[2].problemi.iter().any(|e| e.contains("Corpo vuoto")));
    }

    #[test]
    fn verifica_segnala_titolo_gia_usato_nel_vault_e_propone_riuso_lessicale() {
        let conn = db_test();
        inserisci_prompt(&conn, "vecchio", "Ruolo senior engineer", RUOLO);
        inserisci_prompt(&conn, "p1", "Review", "Sei un senior engineer… {{codice}}");
        let originali = [orig("p1", "Review", "Sei un senior engineer… {{codice}}")];
        let quasi_uguale = RUOLO.replace("dieci", "quindici");
        let risposta = RispostaModello {
            moduli: vec![modulo("r", "Ruolo senior engineer", &quasi_uguale)],
            prompt: vec![prompt("p1", "Review", "{{import \"Ruolo senior engineer\"}}\n{{codice}}")],
            note: vec![],
        };
        let p = verifica(&conn, &originali, risposta, None).unwrap();
        let m = &p.moduli[0];
        assert_eq!(m.titolo_in_conflitto.as_deref(), Some("vecchio"));
        let r = m.riuso.as_ref().expect("riuso lessicale atteso");
        assert_eq!(r.prompt_id, "vecchio");
        assert_eq!(r.fonte, FonteRiuso::Lessicale);
        assert!(r.similarita >= SOGLIA_JACCARD_RIUSO);
        // Il prompt del lotto (p1) non viene mai proposto come riuso di sé stesso.
        assert_ne!(r.prompt_id, "p1");
    }

    #[test]
    fn verifica_non_propone_riuso_sotto_soglia() {
        let conn = db_test();
        inserisci_prompt(&conn, "altro", "Tutt'altro", "Traduci il testo seguente in inglese mantenendo il tono.");
        let originali = [orig("p1", "Uno", "x")];
        let risposta = RispostaModello {
            moduli: vec![modulo("r", "Ruolo senior engineer", RUOLO)],
            prompt: vec![],
            note: vec![],
        };
        let p = verifica(&conn, &originali, risposta, None).unwrap();
        assert!(p.moduli[0].riuso.is_none());
    }

    #[test]
    fn verifica_riscrive_gli_import_dei_moduli_fusi() {
        let conn = db_test();
        let originali = [orig("p1", "Uno", "a"), orig("p2", "Due", "b")];
        let risposta = RispostaModello {
            moduli: vec![
                ModuloProposto { usato_da: vec!["p1".into()], ..modulo("a", "Ruolo A", "Sei un esperto.") },
                ModuloProposto { usato_da: vec!["p2".into()], ..modulo("b", "Ruolo B", "Sei un esperto.") },
            ],
            prompt: vec![
                prompt("p1", "Uno", "{{import \"Ruolo A\"}}\nA"),
                prompt("p2", "Due", "{{import \"ruolo b\"}}\nB"),
            ],
            note: vec![],
        };
        let p = verifica(&conn, &originali, risposta, None).unwrap();
        assert_eq!(p.moduli.len(), 1);
        assert_eq!(p.moduli[0].usato_da, vec!["p1", "p2"]);
        assert!(p.prompt[1].corpo.contains("import \"Ruolo A\""));
        assert!(p.prompt[1].import_non_risolti.is_empty());
        assert!(p.avvisi.iter().any(|a| a.contains("fusi")));
    }

    #[test]
    fn rinomina_import_e_case_insensitive_e_non_tocca_la_prosa() {
        let mut rin = HashMap::new();
        rin.insert("ruolo b".to_string(), "Ruolo A".to_string());
        let corpo = "Nella prosa scrivo import \"Ruolo B\" e non cambia.\n{{import \"ruolo B\" with x=1}}\n{{import \"Altro\"}}";
        let out = rinomina_import(corpo, &rin);
        assert!(out.starts_with("Nella prosa scrivo import \"Ruolo B\" e non cambia."));
        assert!(out.contains("{{import \"Ruolo A\" with x=1}}"));
        assert!(out.contains("{{import \"Altro\"}}"));
    }

    #[test]
    fn verifica_blocca_titolo_di_modulo_con_barra() {
        let conn = db_test();
        let risposta = RispostaModello {
            moduli: vec![modulo("a", "Ruoli/Tech lead", "Sei X.")],
            prompt: vec![],
            note: vec![],
        };
        let p = verifica(&conn, &[orig("p1", "Uno", "x")], risposta, None).unwrap();
        assert!(p.moduli[0].problemi.iter().any(|e| e.contains("«/»")));
    }

    #[test]
    fn verifica_tiene_il_primo_se_il_modello_ripete_un_prompt() {
        let conn = db_test();
        let originali = [orig("p1", "Uno", "a")];
        let risposta = RispostaModello {
            moduli: vec![],
            prompt: vec![prompt("p1", "Primo", "x"), prompt("p1", "Secondo", "y")],
            note: vec![],
        };
        let p = verifica(&conn, &originali, risposta, None).unwrap();
        assert_eq!(p.prompt.len(), 1);
        assert_eq!(p.prompt[0].titolo, "Primo");
        assert!(p.avvisi.iter().any(|a| a.contains("due volte")));
    }
}
