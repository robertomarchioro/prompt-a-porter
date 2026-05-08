// Modulo linting — Fase 3 Step 5.
//
// Avvisi proattivi che aiutano a scrivere prompt migliori, senza essere
// paternalistici. Eseguito on-demand dal frontend via comando Tauri,
// con due livelli:
//   - body-only (no DB): regole pure sul testo
//   - completo (con vault): aggiunge le regole IMP* su grafo di import
//
// 11 regole implementate (di 14 nello spec):
//   - LEN001/002: lunghezza body
//   - PH001/003: segnaposti
//   - PII001/003/004: privacy
//   - STY001: ripetizione n-gram
//   - IMP001: import non risolto
//   - IMP002: ciclo di import
//   - IMP003: profondità di import oltre il limite
//   - IMP004: prompt importato da altri (info, cross-prompt linting v0.7 Step 5)
//
// Skippate per ora (motivo nello spec / PR successive):
//   - PH002 (segnaposto dichiarato non usato): semantica ambigua, il
//     nostro modello non distingue dichiarazione da uso
//   - PII002 (codice fiscale italiano): regex compleessa, low-priority
//   - STY002 (mancanza istruzioni chiare): richiede NLP IT/EN, troppo
//     fragile a regex

use regex::Regex;
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::OnceLock;
use tauri::State;

use crate::errore::PapErrore;
use crate::prompt_componibili::{parse_imports, resolve_path, MAX_DEPTH};
use crate::vault::VaultState;

// ─────────── Tipi pubblici ───────────

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severita {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub code: &'static str,
    pub severita: Severita,
    pub messaggio: String,
    /// Posizione 1-based nel body se applicabile.
    pub linea: Option<usize>,
    pub colonna: Option<usize>,
}

// ─────────── Regex statici (compile-once) ───────────

fn re_segnaposto_doppio() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\{\{\s*[A-Za-z_][A-Za-z0-9_]*\s*\}\}").unwrap())
}

/// Match per segnaposto MALFORMATO `{nome}` (single brace), escludendo
/// quelli ben formati `{{nome}}`. Il regex usa look-around emulato:
/// match `{nome}` solo se NON preceduto e seguito da `{`/`}`.
fn re_segnaposto_singolo() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // Pattern: NON-{ + { + nome + } + NON-}. Usiamo gruppi di cattura.
    R.get_or_init(|| {
        Regex::new(r"(^|[^\{])\{([A-Za-z_][A-Za-z0-9_]*)\}([^\}]|$)").unwrap()
    })
}

/// Segnaposto col nome che contiene caratteri non consentiti (spazi,
/// trattini, dot, etc.) — qualcosa fra `{{` e `}}` che non è il pattern
/// "pulito".
fn re_segnaposto_caratteri_speciali() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\{\{\s*([^}]*?)\s*\}\}").unwrap())
}

fn re_email() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}").unwrap()
    })
}

/// 13-19 cifre consecutive (eventuali separatori spazio/dash) — candidato
/// a numero di carta di credito. Filtro Luhn applicato in `regola_pii003`.
fn re_carta_credito() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        // 4 gruppi di 4 separati da spazio/dash, oppure 13-19 cifre senza
        // separatori, NON contigui ad altre cifre.
        Regex::new(r"(?:^|[^\d])(\d[\d \-]{11,22}\d)(?:[^\d]|$)").unwrap()
    })
}

fn re_api_keys() -> &'static [(&'static str, &'static Regex)] {
    static SLOTS: OnceLock<Vec<(&'static str, Regex)>> = OnceLock::new();
    static REFS: OnceLock<Vec<(&'static str, &'static Regex)>> = OnceLock::new();
    REFS.get_or_init(|| {
        let owned = SLOTS.get_or_init(|| {
            vec![
                ("OpenAI", Regex::new(r"sk-[A-Za-z0-9]{20,}").unwrap()),
                ("Anthropic", Regex::new(r"sk-ant-[A-Za-z0-9_\-]{20,}").unwrap()),
                ("AWS Access Key", Regex::new(r"AKIA[A-Z0-9]{16}").unwrap()),
                ("GitHub", Regex::new(r"ghp_[A-Za-z0-9]{36}").unwrap()),
                ("GitHub fine-grained", Regex::new(r"github_pat_[A-Za-z0-9_]{50,}").unwrap()),
                ("Google API", Regex::new(r"AIza[A-Za-z0-9_\-]{35}").unwrap()),
            ]
        });
        owned.iter().map(|(n, r)| (*n, r)).collect()
    })
}

// ─────────── Costanti ───────────

const LEN_MAX_BODY: usize = 4000;
const LEN_MIN_BODY: usize = 30;
/// N-gram size per analisi ripetizione (3-grammi parola).
const NGRAM_SIZE: usize = 3;
/// Soglia: se uno stesso n-gram appare ≥ N volte nel body, è ripetitivo.
const NGRAM_THRESHOLD: usize = 4;

// ─────────── Regole ───────────

fn regola_len001(body: &str, out: &mut Vec<Issue>) {
    let n = body.chars().count();
    if n > LEN_MAX_BODY {
        out.push(Issue {
            code: "LEN001",
            severita: Severita::Warning,
            messaggio: format!(
                "Body lungo {n} caratteri (> {LEN_MAX_BODY}). Possibile spreco di token al rendering."
            ),
            linea: None,
            colonna: None,
        });
    }
}

fn regola_len002(body: &str, out: &mut Vec<Issue>) {
    let n = body.trim().chars().count();
    if n < LEN_MIN_BODY && n > 0 {
        out.push(Issue {
            code: "LEN002",
            severita: Severita::Info,
            messaggio: format!(
                "Body corto ({n} caratteri, < {LEN_MIN_BODY}). Probabilmente incompleto."
            ),
            linea: None,
            colonna: None,
        });
    }
}

fn regola_ph001_segnaposti_malformati(body: &str, out: &mut Vec<Issue>) {
    for cap in re_segnaposto_singolo().captures_iter(body) {
        let nome = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let pos = cap.get(2).map(|m| m.start()).unwrap_or(0);
        let (linea, colonna) = pos_a_linea_col(body, pos);
        out.push(Issue {
            code: "PH001",
            severita: Severita::Error,
            messaggio: format!(
                "Segnaposto malformato `{{{}}}`. Usa `{{{{{}}}}}` (doppie graffe).",
                nome, nome
            ),
            linea: Some(linea),
            colonna: Some(colonna),
        });
    }
}

fn regola_ph003_caratteri_speciali(body: &str, out: &mut Vec<Issue>) {
    let valido = re_segnaposto_doppio();
    for cap in re_segnaposto_caratteri_speciali().captures_iter(body) {
        let intero = cap.get(0).map(|m| m.as_str()).unwrap_or("");
        // Skip i ben formati (matchano il regex valido).
        if valido.is_match(intero) {
            continue;
        }
        let nome = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        // Saltiamo `{{}}` vuoti — niente nome significa nessuna problematica
        // di char invalidi (e in pratica è gestito separatamente).
        if nome.trim().is_empty() {
            continue;
        }
        let pos = cap.get(0).map(|m| m.start()).unwrap_or(0);
        let (linea, colonna) = pos_a_linea_col(body, pos);
        out.push(Issue {
            code: "PH003",
            severita: Severita::Warning,
            messaggio: format!(
                "Nome segnaposto `{nome}` contiene caratteri non consentiti. Usa solo lettere, cifre, underscore."
            ),
            linea: Some(linea),
            colonna: Some(colonna),
        });
    }
}

fn regola_pii001_email(body: &str, out: &mut Vec<Issue>) {
    for m in re_email().find_iter(body) {
        let (linea, colonna) = pos_a_linea_col(body, m.start());
        out.push(Issue {
            code: "PII001",
            severita: Severita::Warning,
            messaggio: format!(
                "Possibile email rilevata (`{}`). Verifica che non sia un dato personale da rimuovere.",
                m.as_str()
            ),
            linea: Some(linea),
            colonna: Some(colonna),
        });
    }
}

fn luhn_valido(numero: &str) -> bool {
    let cifre: Vec<u32> = numero.chars().filter_map(|c| c.to_digit(10)).collect();
    if cifre.len() < 13 || cifre.len() > 19 {
        return false;
    }
    let somma: u32 = cifre
        .iter()
        .rev()
        .enumerate()
        .map(|(i, d)| {
            if i % 2 == 1 {
                let raddoppio = d * 2;
                if raddoppio > 9 { raddoppio - 9 } else { raddoppio }
            } else {
                *d
            }
        })
        .sum();
    somma % 10 == 0
}

fn regola_pii003_carta_credito(body: &str, out: &mut Vec<Issue>) {
    for cap in re_carta_credito().captures_iter(body) {
        let cattura = cap.get(1).unwrap();
        let candidato = cattura.as_str();
        if luhn_valido(candidato) {
            let (linea, colonna) = pos_a_linea_col(body, cattura.start());
            out.push(Issue {
                code: "PII003",
                severita: Severita::Error,
                messaggio: format!(
                    "Possibile numero di carta di credito (Luhn-valido). Rimuovi prima di salvare il prompt."
                ),
                linea: Some(linea),
                colonna: Some(colonna),
            });
        }
    }
}

fn regola_pii004_api_keys(body: &str, out: &mut Vec<Issue>) {
    for (provider, re) in re_api_keys() {
        for m in re.find_iter(body) {
            let (linea, colonna) = pos_a_linea_col(body, m.start());
            out.push(Issue {
                code: "PII004",
                severita: Severita::Error,
                messaggio: format!(
                    "Possibile chiave API {provider} rilevata. Rimuovi prima di salvare."
                ),
                linea: Some(linea),
                colonna: Some(colonna),
            });
        }
    }
}

fn regola_sty001_ripetizione(body: &str, out: &mut Vec<Issue>) {
    use std::collections::HashMap;
    // Tokenizzazione naive a parole (whitespace).
    let parole: Vec<&str> = body.split_whitespace().collect();
    if parole.len() < NGRAM_SIZE {
        return;
    }
    let mut conteggi: HashMap<String, usize> = HashMap::new();
    for finestra in parole.windows(NGRAM_SIZE) {
        let key = finestra.join(" ").to_lowercase();
        *conteggi.entry(key).or_insert(0) += 1;
    }
    let mut ripetuti: Vec<(String, usize)> = conteggi
        .into_iter()
        .filter(|(_, c)| *c >= NGRAM_THRESHOLD)
        .collect();
    ripetuti.sort_by(|a, b| b.1.cmp(&a.1));

    for (ngram, count) in ripetuti.into_iter().take(3) {
        out.push(Issue {
            code: "STY001",
            severita: Severita::Info,
            messaggio: format!(
                "N-gramma `{ngram}` ripetuto {count} volte. Considera di riformulare per ridurre token."
            ),
            linea: None,
            colonna: None,
        });
    }
}

// ─────────── Helper ───────────

/// Converte una posizione byte-offset in (linea 1-based, colonna 1-based).
fn pos_a_linea_col(testo: &str, byte_offset: usize) -> (usize, usize) {
    let mut linea = 1usize;
    let mut colonna = 1usize;
    for (i, c) in testo.char_indices() {
        if i >= byte_offset {
            break;
        }
        if c == '\n' {
            linea += 1;
            colonna = 1;
        } else {
            colonna += 1;
        }
    }
    (linea, colonna)
}

// ─────────── Regole IMP (richiedono accesso DB) ───────────

/// Esito di un walk DFS dal target di un import.
#[derive(Debug)]
enum WalkExit {
    /// Sottografo OK, profondità sotto soglia, nessun ciclo.
    Ok,
    /// Trovato ciclo (un id già visitato è stato re-incontrato).
    Cycle,
    /// La profondità massima è stata superata. `depth` è il livello che
    /// ha tripped il limite (sempre > MAX_DEPTH).
    TooDeep { depth: usize },
}

/// DFS dal nodo `current_id`. Il chiamante deve aver già aggiunto al
/// `visitati` set tutti gli antenati (incluso il root del prompt che si
/// sta linting). `depth` è la profondità di `current_id` rispetto al
/// root (root = 0, target di un import al primo livello = 1, etc.).
///
/// Ritorna alla prima anomalia trovata: per il linter è sufficiente
/// segnalare un ciclo / depth-exceeded sul singolo import — l'utente
/// risolve quello, poi rilinta.
fn walk_dfs(
    conn: &Connection,
    current_id: &str,
    visitati: &mut HashSet<String>,
    depth: usize,
) -> WalkExit {
    if depth > MAX_DEPTH {
        return WalkExit::TooDeep { depth };
    }
    if !visitati.insert(current_id.to_string()) {
        return WalkExit::Cycle;
    }

    let body: String = match conn.query_row(
        "SELECT Body FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
        [current_id],
        |r| r.get(0),
    ) {
        Ok(b) => b,
        Err(_) => {
            visitati.remove(current_id);
            return WalkExit::Ok;
        }
    };

    for imp in parse_imports(&body) {
        let child_id = match resolve_path(conn, &imp.path) {
            Ok(Some(id)) => id,
            _ => continue, // import non risolvibili lungo la catena: skippa, IMP001 li flagga al root
        };
        let exit = walk_dfs(conn, &child_id, visitati, depth + 1);
        if !matches!(exit, WalkExit::Ok) {
            visitati.remove(current_id);
            return exit;
        }
    }

    visitati.remove(current_id);
    WalkExit::Ok
}

/// Analizza gli import del body e genera issue IMP001/002/003/004.
///
/// `parent_id_opt` è l'id del prompt che si sta linting (None se non è
/// ancora stato salvato): quando presente, viene seminato nel set di
/// visitati per rilevare cicli che includono il root, e usato per
/// IMP004 (cross-prompt linting: chi importa questo prompt).
fn regole_imp(
    conn: &Connection,
    body: &str,
    parent_id_opt: Option<&str>,
    out: &mut Vec<Issue>,
) {
    // IMP004 (v0.7.0 Step 5): se questo prompt è target di altri prompt
    // via {{import "..."}}, mostra info "importato da N altri".
    // Skip se parent_id non noto (prompt non ancora salvato).
    if let Some(self_id) = parent_id_opt {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT ParentPromptId) FROM PromptImports
                 WHERE ImportedPromptId = ?1",
                [self_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if count > 0 {
            let plurale = if count == 1 { "prompt" } else { "prompt" };
            let messaggio = if count == 1 {
                format!(
                    "Questo prompt è importato da 1 altro {plurale}. \
                     Modifiche al body si propagano alla compilazione di chi lo importa."
                )
            } else {
                format!(
                    "Questo prompt è importato da {count} altri {plurale}. \
                     Modifiche al body si propagano alla compilazione di chi lo importa."
                )
            };
            out.push(Issue {
                code: "IMP004",
                severita: Severita::Info,
                messaggio,
                linea: None,
                colonna: None,
            });
        }
    }

    for imp in parse_imports(body) {
        let (linea, colonna) = pos_a_linea_col(body, imp.byte_start);
        let target_id = match resolve_path(conn, &imp.path) {
            Ok(Some(id)) => id,
            Ok(None) => {
                out.push(Issue {
                    code: "IMP001",
                    severita: Severita::Error,
                    messaggio: format!(
                        "Import non risolto: \"{}\". Verifica che il prompt esista nella libreria.",
                        imp.path
                    ),
                    linea: Some(linea),
                    colonna: Some(colonna),
                });
                continue;
            }
            Err(_) => continue, // errore DB: niente issue, fallback silenzioso
        };

        // Caso ciclo banale: il prompt importa se stesso (self-loop).
        if let Some(self_id) = parent_id_opt {
            if self_id == target_id {
                out.push(Issue {
                    code: "IMP002",
                    severita: Severita::Error,
                    messaggio: format!(
                        "Ciclo di import: \"{}\" punta al prompt stesso.",
                        imp.path
                    ),
                    linea: Some(linea),
                    colonna: Some(colonna),
                });
                continue;
            }
        }

        let mut visitati: HashSet<String> = HashSet::new();
        if let Some(self_id) = parent_id_opt {
            visitati.insert(self_id.to_string());
        }
        match walk_dfs(conn, &target_id, &mut visitati, 1) {
            WalkExit::Ok => {}
            WalkExit::Cycle => {
                out.push(Issue {
                    code: "IMP002",
                    severita: Severita::Error,
                    messaggio: format!(
                        "Ciclo di import rilevato attraverso \"{}\". I prompt non possono importarsi a vicenda.",
                        imp.path
                    ),
                    linea: Some(linea),
                    colonna: Some(colonna),
                });
            }
            WalkExit::TooDeep { depth } => {
                out.push(Issue {
                    code: "IMP003",
                    severita: Severita::Warning,
                    messaggio: format!(
                        "Profondità di import {depth} (max {MAX_DEPTH}) raggiunta tramite \"{}\". Considera di appiattire la catena.",
                        imp.path
                    ),
                    linea: Some(linea),
                    colonna: Some(colonna),
                });
            }
        }
    }
}

// ─────────── Entrypoint ───────────

pub fn analizza(body: &str) -> Vec<Issue> {
    let mut out = Vec::new();
    regola_len001(body, &mut out);
    regola_len002(body, &mut out);
    regola_ph001_segnaposti_malformati(body, &mut out);
    regola_ph003_caratteri_speciali(body, &mut out);
    regola_pii001_email(body, &mut out);
    regola_pii003_carta_credito(body, &mut out);
    regola_pii004_api_keys(body, &mut out);
    regola_sty001_ripetizione(body, &mut out);
    out
}

/// Variante completa: include `analizza` body-only + le regole IMP che
/// richiedono accesso al vault per risolvere e camminare il grafo.
pub fn analizza_completo(
    conn: &Connection,
    body: &str,
    parent_id_opt: Option<&str>,
) -> Vec<Issue> {
    let mut out = analizza(body);
    regole_imp(conn, body, parent_id_opt, &mut out);
    out
}

/// Comando Tauri unificato. Esegue sempre il lint body-only; se il vault
/// è aperto, aggiunge anche le regole IMP. Se il vault non è
/// disponibile (login non ancora effettuato) il fallback è silenzioso —
/// il lint rimane utile anche con db chiuso.
/// Filtra un Vec<Issue> escludendo quelli il cui prefisso lettere del code
/// è nelle categorie disabilitate (es. `["IMP", "PII"]` skippa IMP001,
/// PII001, ecc.). v0.6.0 Step 6.
pub(crate) fn filtra_categorie(
    issues: Vec<Issue>,
    disabilitate: &[String],
) -> Vec<Issue> {
    if disabilitate.is_empty() {
        return issues;
    }
    let set: std::collections::HashSet<&str> =
        disabilitate.iter().map(|s| s.as_str()).collect();
    issues
        .into_iter()
        .filter(|i| {
            let prefisso: String = i
                .code
                .chars()
                .take_while(|c| c.is_ascii_alphabetic())
                .collect();
            !set.contains(prefisso.as_str())
        })
        .collect()
}

#[tauri::command]
pub fn prompt_lint(
    body: String,
    prompt_id: Option<String>,
    categorie_disabilitate: Option<Vec<String>>,
    state: State<'_, VaultState>,
) -> Result<Vec<Issue>, PapErrore> {
    let mut out = analizza(&body);
    if let Ok(imp) = state.with_conn(|conn| {
        let mut buf = Vec::new();
        regole_imp(conn, &body, prompt_id.as_deref(), &mut buf);
        Ok(buf)
    }) {
        out.extend(imp);
    }
    if let Some(disable) = categorie_disabilitate {
        out = filtra_categorie(out, &disable);
    }
    Ok(out)
}

// ─────────── Test ───────────

#[cfg(test)]
mod test {
    use super::*;

    fn ha_codice(issues: &[Issue], code: &str) -> bool {
        issues.iter().any(|i| i.code == code)
    }

    fn conta_codice(issues: &[Issue], code: &str) -> usize {
        issues.iter().filter(|i| i.code == code).count()
    }

    #[test]
    fn body_ok_zero_issue() {
        let body = "Riscrivi questa email in tono più formale: {{testo}}";
        let issues = analizza(body);
        // 50+ char, segnaposto valido, no PII → zero issues
        assert_eq!(issues.len(), 0, "body pulito non deve avere issue: {issues:?}");
    }

    #[test]
    fn len001_body_lungo() {
        let body = "x".repeat(LEN_MAX_BODY + 1);
        let issues = analizza(&body);
        assert!(ha_codice(&issues, "LEN001"));
    }

    #[test]
    fn len002_body_corto() {
        let body = "ciao";
        let issues = analizza(body);
        assert!(ha_codice(&issues, "LEN002"));
    }

    #[test]
    fn len002_body_vuoto_no_issue() {
        // Body vuoto ha sense diverso da "troppo corto" — non emettiamo LEN002
        let issues = analizza("");
        assert!(!ha_codice(&issues, "LEN002"));
    }

    #[test]
    fn ph001_segnaposto_singolo_e_errore() {
        let body = "Ciao {nome}, come stai? Lungo abbastanza per evitare LEN002.";
        let issues = analizza(body);
        assert!(ha_codice(&issues, "PH001"));
        let issue = issues.iter().find(|i| i.code == "PH001").unwrap();
        assert!(matches!(issue.severita, Severita::Error));
    }

    #[test]
    fn ph001_segnaposto_doppio_no_falso_positivo() {
        let body = "Ciao {{nome}}, come stai? Sufficientemente lungo per evitare LEN002.";
        let issues = analizza(body);
        assert!(!ha_codice(&issues, "PH001"));
    }

    #[test]
    fn ph003_caratteri_speciali_nel_nome() {
        let body = "Ciao {{nome con spazi}}, sufficientemente lungo per evitare LEN002.";
        let issues = analizza(body);
        assert!(ha_codice(&issues, "PH003"));
    }

    #[test]
    fn pii001_email_rilevata() {
        let body =
            "Manda un messaggio a mario.rossi@example.com appena puoi grazie.";
        let issues = analizza(body);
        assert!(ha_codice(&issues, "PII001"));
    }

    #[test]
    fn pii003_carta_di_credito_luhn_valida() {
        // Visa test number: 4111 1111 1111 1111 (valido Luhn)
        let body = "La carta è 4111 1111 1111 1111 grazie. Sufficientemente lungo per LEN002.";
        let issues = analizza(body);
        assert!(ha_codice(&issues, "PII003"));
    }

    #[test]
    fn pii003_numero_random_no_falso_positivo() {
        // Sequenza di cifre che NON è Luhn-valida (es. 1234 5678 9012 3456)
        let body = "Codice ordine: 1234 5678 9012 3456 verifica grazie. Sufficientemente lungo.";
        let issues = analizza(body);
        assert!(!ha_codice(&issues, "PII003"), "non-Luhn deve essere ignorato");
    }

    #[test]
    fn pii004_api_key_openai() {
        let body = "Configura API key sk-abcdefghijklmnopqrstuvwxyz1234567890 nelle env.";
        let issues = analizza(body);
        assert!(ha_codice(&issues, "PII004"));
    }

    #[test]
    fn pii004_api_key_anthropic() {
        let body = "Test con sk-ant-api03-AbCdEf1234567890_-XyZ_AbCdEfGh nel codice.";
        let issues = analizza(body);
        assert!(ha_codice(&issues, "PII004"));
    }

    #[test]
    fn pii004_api_key_aws() {
        let body = "AWS_ACCESS_KEY=AKIAIOSFODNN7EXAMPLE da rimuovere assolutamente.";
        let issues = analizza(body);
        assert!(ha_codice(&issues, "PII004"));
    }

    #[test]
    fn pii004_no_falso_positivo_su_word_simili() {
        let body = "Le parole skidoo, askance, basket non devono triggerare. LEN002 evitato.";
        let issues = analizza(body);
        assert!(!ha_codice(&issues, "PII004"));
    }

    #[test]
    fn sty001_ripetizione_ngram() {
        // Stesso 3-gram ripetuto 4+ volte → flag
        let body = "alpha beta gamma alpha beta gamma alpha beta gamma alpha beta gamma fine.";
        let issues = analizza(body);
        assert!(ha_codice(&issues, "STY001"));
    }

    #[test]
    fn sty001_no_falso_positivo_su_testo_normale() {
        let body = "Riscrivi questa email in tono più formale mantenendo i punti chiave.";
        let issues = analizza(body);
        assert!(!ha_codice(&issues, "STY001"));
    }

    #[test]
    fn pos_a_linea_col_calcolo_corretto() {
        let body = "linea 1\nlinea 2\nlinea 3";
        let pos_l3 = body.find("linea 3").unwrap();
        let (linea, colonna) = pos_a_linea_col(body, pos_l3);
        assert_eq!(linea, 3);
        assert_eq!(colonna, 1);
    }

    #[test]
    fn luhn_valido_numero_test_visa() {
        assert!(luhn_valido("4111111111111111"));
        assert!(!luhn_valido("4111111111111112"));
        assert!(!luhn_valido("123")); // troppo corto
    }

    #[test]
    fn analizza_combinazione_multiple_issues() {
        let body = "Email a x@y.com con sk-abcdefghijklmnopqrstuvwxyz1234567890 chiave OpenAI.";
        let issues = analizza(body);
        assert!(ha_codice(&issues, "PII001"));
        assert!(ha_codice(&issues, "PII004"));
        // entrambe almeno una volta
        assert!(conta_codice(&issues, "PII001") >= 1);
        assert!(conta_codice(&issues, "PII004") >= 1);
    }

    // ─────────── Test regole IMP (richiedono DB) ───────────

    use rusqlite::params;

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    fn inserisci_prompt(conn: &Connection, id: &str, titolo: &str, body: &str) {
        conn.execute(
            "INSERT INTO Prompts (Id, WorkspaceId, AuthorUserId, Title, Body,
                Visibility, FolderId, Version, CreatedAt, UpdatedAt)
             VALUES (?1, 'ws-personale', 'usr-locale', ?2, ?3, 'private', NULL, 1,
                     datetime('now'), datetime('now'))",
            params![id, titolo, body],
        )
        .unwrap();
    }

    #[test]
    fn imp001_import_non_risolto() {
        let conn = db_test();
        // Niente prompt "fantasma" inserito. Il body lo importa.
        let body = r#"Inizio testo. {{import "fantasma"}} fine. Lungo a sufficienza per LEN002."#;
        let issues = analizza_completo(&conn, body, None);
        assert!(ha_codice(&issues, "IMP001"));
        let i = issues.iter().find(|i| i.code == "IMP001").unwrap();
        assert!(matches!(i.severita, Severita::Error));
        assert!(i.linea.is_some());
        assert!(i.colonna.is_some());
        assert!(i.messaggio.contains("fantasma"));
    }

    #[test]
    fn imp001_no_falso_positivo_quando_path_risolve() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "ruolo-esperto", "Sei un esperto di...");
        let body = r#"Setup {{import "ruolo-esperto"}} continua. Lungo per evitare LEN002."#;
        let issues = analizza_completo(&conn, body, None);
        assert!(!ha_codice(&issues, "IMP001"));
    }

    #[test]
    fn imp002_self_loop_diretto() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-1", "auto", "self: {{import \"auto\"}}");
        let body = "self: {{import \"auto\"}}";
        // parent_id_opt = Some("prm-1") simula il prompt che si sta editando.
        let issues = analizza_completo(&conn, body, Some("prm-1"));
        assert!(ha_codice(&issues, "IMP002"));
    }

    #[test]
    fn imp002_ciclo_indiretto_a_b_a() {
        let conn = db_test();
        // A → B → A
        inserisci_prompt(&conn, "prm-a", "A", r#"{{import "B"}}"#);
        inserisci_prompt(&conn, "prm-b", "B", r#"{{import "A"}}"#);
        // Stiamo linting A (che importa B che importa A).
        let body = r#"{{import "B"}}"#;
        let issues = analizza_completo(&conn, body, Some("prm-a"));
        assert!(ha_codice(&issues, "IMP002"));
    }

    #[test]
    fn imp002_no_falso_positivo_su_grafo_aciclico() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-a", "A", r#"{{import "B"}}"#);
        inserisci_prompt(&conn, "prm-b", "B", "foglia.");
        let body = r#"{{import "B"}}"#;
        let issues = analizza_completo(&conn, body, Some("prm-a"));
        assert!(!ha_codice(&issues, "IMP002"));
    }

    #[test]
    fn imp003_profondita_oltre_max() {
        let conn = db_test();
        // Catena di 7: root(prm-r) → A → B → C → D → E → F (depth=6 supera MAX_DEPTH=5)
        inserisci_prompt(&conn, "prm-a", "A", r#"{{import "B"}}"#);
        inserisci_prompt(&conn, "prm-b", "B", r#"{{import "C"}}"#);
        inserisci_prompt(&conn, "prm-c", "C", r#"{{import "D"}}"#);
        inserisci_prompt(&conn, "prm-d", "D", r#"{{import "E"}}"#);
        inserisci_prompt(&conn, "prm-e", "E", r#"{{import "F"}}"#);
        inserisci_prompt(&conn, "prm-f", "F", "foglia profonda.");
        let body = r#"{{import "A"}}"#;
        let issues = analizza_completo(&conn, body, Some("prm-r"));
        assert!(ha_codice(&issues, "IMP003"));
        let i = issues.iter().find(|i| i.code == "IMP003").unwrap();
        assert!(matches!(i.severita, Severita::Warning));
    }

    #[test]
    fn imp003_no_falso_positivo_a_5_livelli() {
        let conn = db_test();
        // Catena root → A → B → C → D → E (depth massimo = 5 = MAX_DEPTH, OK)
        inserisci_prompt(&conn, "prm-a", "A", r#"{{import "B"}}"#);
        inserisci_prompt(&conn, "prm-b", "B", r#"{{import "C"}}"#);
        inserisci_prompt(&conn, "prm-c", "C", r#"{{import "D"}}"#);
        inserisci_prompt(&conn, "prm-d", "D", r#"{{import "E"}}"#);
        inserisci_prompt(&conn, "prm-e", "E", "foglia.");
        let body = r#"{{import "A"}}"#;
        let issues = analizza_completo(&conn, body, Some("prm-r"));
        assert!(!ha_codice(&issues, "IMP003"));
    }

    #[test]
    fn analizza_completo_non_perde_regole_body_only() {
        let conn = db_test();
        // Body con email PII001 + import non risolto → entrambe segnalate.
        let body = r#"Email a x@y.com e {{import "missing"}} grazie. Lungo per LEN002."#;
        let issues = analizza_completo(&conn, body, None);
        assert!(ha_codice(&issues, "PII001"));
        assert!(ha_codice(&issues, "IMP001"));
    }

    #[test]
    fn regole_imp_senza_parent_id_funzionano_comunque() {
        // Use case: editor con prompt nuovo (ancora senza id) — IMP001
        // funziona, IMP002 non può detectare self-loop (manca self_id) ma
        // detecta cicli che NON includono il root.
        let conn = db_test();
        inserisci_prompt(&conn, "prm-a", "A", r#"{{import "B"}}"#);
        inserisci_prompt(&conn, "prm-b", "B", r#"{{import "A"}}"#);
        // Body lint = "{{import "A"}}", parent_id None → walk parte da A,
        // visita B, B importa A, A non era ancora visited (root non
        // iniettato), quindi ENTRA in A. Poi A importa B che è in
        // visited → IMP002.
        let body = r#"{{import "A"}}"#;
        let issues = analizza_completo(&conn, body, None);
        assert!(ha_codice(&issues, "IMP002"));
    }

    // ─────────── v0.6.0 Step 6: filtra_categorie ───────────

    fn issue_di(code: &'static str) -> Issue {
        Issue {
            code,
            severita: Severita::Warning,
            messaggio: "test".into(),
            linea: None,
            colonna: None,
        }
    }

    #[test]
    fn filtra_categorie_lista_vuota_no_op() {
        let issues = vec![issue_di("PH001"), issue_di("PII001"), issue_di("LEN001")];
        let r = super::filtra_categorie(issues.clone(), &[]);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn filtra_categorie_skippa_prefisso_pii() {
        let issues = vec![
            issue_di("PH001"),
            issue_di("PII001"),
            issue_di("PII003"),
            issue_di("LEN001"),
        ];
        let r = super::filtra_categorie(issues, &["PII".to_string()]);
        assert_eq!(r.len(), 2);
        assert!(r.iter().all(|i| !i.code.starts_with("PII")));
    }

    #[test]
    fn filtra_categorie_disabilita_multiple() {
        let issues = vec![
            issue_di("PH001"),
            issue_di("PH003"),
            issue_di("IMP001"),
            issue_di("IMP002"),
            issue_di("LEN001"),
        ];
        let r = super::filtra_categorie(
            issues,
            &["PH".to_string(), "IMP".to_string()],
        );
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].code, "LEN001");
    }

    #[test]
    fn filtra_categorie_categoria_inesistente_no_op() {
        let issues = vec![issue_di("PH001"), issue_di("LEN001")];
        let r = super::filtra_categorie(issues, &["FOO".to_string()]);
        assert_eq!(r.len(), 2);
    }

    // ─────────── v0.7.0 Step 5: IMP004 cross-prompt linting ───────────

    #[test]
    fn imp004_assente_se_parent_id_none() {
        // Senza parent_id (prompt non salvato), IMP004 non si attiva.
        let conn = db_test();
        let issues = analizza_completo(&conn, "ciao mondo abbastanza lungo", None);
        assert!(!ha_codice(&issues, "IMP004"));
    }

    #[test]
    fn imp004_assente_se_nessuno_importa_il_prompt() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-solo", "Solo", "body lungo abbastanza per LEN");
        let issues = analizza_completo(&conn, "body senza import abbastanza lungo", Some("prm-solo"));
        assert!(!ha_codice(&issues, "IMP004"));
    }

    #[test]
    fn imp004_attivato_se_un_prompt_importa_il_target() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-target", "Target", "body target abbastanza lungo");
        inserisci_prompt(&conn, "prm-a", "A", r#"{{import "Target"}}"#);
        // Aggiorna PromptImports per simulare la scrittura post-save.
        crate::prompt_componibili::aggiorna_imports(
            &conn,
            "prm-a",
            r#"{{import "Target"}}"#,
        )
        .unwrap();

        let issues = analizza_completo(
            &conn,
            "body target abbastanza lungo",
            Some("prm-target"),
        );
        assert!(ha_codice(&issues, "IMP004"));
        let imp004 = issues.iter().find(|i| i.code == "IMP004").unwrap();
        assert!(matches!(imp004.severita, Severita::Info));
        assert!(imp004.messaggio.contains("1 altro"));
    }

    #[test]
    fn imp004_conta_distinct_parent() {
        let conn = db_test();
        inserisci_prompt(&conn, "prm-target", "Target", "body target abbastanza lungo");
        inserisci_prompt(&conn, "prm-a", "A", r#"{{import "Target"}}"#);
        inserisci_prompt(&conn, "prm-b", "B", r#"{{import "Target"}}"#);
        crate::prompt_componibili::aggiorna_imports(&conn, "prm-a", r#"{{import "Target"}}"#)
            .unwrap();
        crate::prompt_componibili::aggiorna_imports(&conn, "prm-b", r#"{{import "Target"}}"#)
            .unwrap();

        let issues = analizza_completo(
            &conn,
            "body target abbastanza lungo",
            Some("prm-target"),
        );
        let imp004 = issues.iter().find(|i| i.code == "IMP004").unwrap();
        assert!(imp004.messaggio.contains("2 altri"));
    }
}
