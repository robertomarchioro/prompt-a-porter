// Modulo linting — Fase 3 Step 5.
//
// Avvisi proattivi che aiutano a scrivere prompt migliori, senza essere
// paternalistici. Eseguito on-demand dal frontend via comando Tauri,
// nessun salvataggio dei risultati (riprodotti al volo).
//
// 8 regole implementate (di 14 nello spec):
//   - LEN001/002: lunghezza body
//   - PH001/003: segnaposti
//   - PII001/003/004: privacy
//   - STY001: ripetizione n-gram
//
// Skippate per ora (motivo nello spec / PR successive):
//   - PH002 (segnaposto dichiarato non usato): semantica ambigua, il
//     nostro modello non distingue dichiarazione da uso
//   - PII002 (codice fiscale italiano): regex compleessa, low-priority
//   - STY002 (mancanza istruzioni chiare): richiede NLP IT/EN, troppo
//     fragile a regex
//   - IMP001/002/003 (import): richiede Step 8 (prompt componibili);
//     verranno integrate in PR successiva quando il parser di import
//     atterra

use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;

use crate::errore::PapErrore;

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

#[tauri::command]
pub fn prompt_lint(body: String) -> Result<Vec<Issue>, PapErrore> {
    Ok(analizza(&body))
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
}
