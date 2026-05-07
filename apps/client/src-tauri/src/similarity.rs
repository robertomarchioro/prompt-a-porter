// Modulo similarity — Fase 4 Step 8c.
//
// Funzioni di similarità tra `expected` e `actual` per i golden examples.
// Lo Step 8c implementa 3 funzioni "veloci" che girano interamente
// client-side, senza chiamate AI aggiuntive:
//   - cosine: embedding di expected/actual + cosine sui vettori
//     L2-normalized (riusa il modello MiniLM di Fase 3)
//   - exact-match: confronto stringa esatto post-trim
//   - regex: l'expected è una regex, ritorna pass/fail
//
// La 4ª funzione `llm-judge` richiede una chiamata extra al provider
// AI ed è programmata per Step 8f insieme ai provider remote.
//
// Convenzione: tutte le funzioni ritornano f64 ∈ [0,1] dove 1.0 = match
// perfetto, 0.0 = mismatch totale. La soglia di "passato" è applicata
// dal caller (regression::esegui in 8d) tramite `is_passed`.

use regex::Regex;

use crate::embeddings::{compute_embedding_opt, EmbeddingsState};
use crate::errore::PapErrore;

/// Cosine similarity per vettori L2-normalized. Per embedding non
/// normalizzati il chiamante dovrebbe pre-normalizzare. La nostra
/// pipeline embedding (vedi `embeddings::compute_embedding_opt`) li
/// emette già L2-normalized, quindi il cosine si riduce a un semplice
/// dot product.
///
/// Output clampato a [0,1] (in teoria potrebbe essere [-1,1]; valori
/// negativi indicano "opposto" e per i golden non cambiano il verdict
/// — sono comunque sotto soglia tipica 0.85).
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    (dot.clamp(0.0_f32, 1.0_f32)) as f64
}

/// Cosine similarity tra due testi, calcolando prima i loro embedding.
/// Richiede la Session ort caricata: ritorna errore se non disponibile
/// (così il caller può registrare l'observation con `Errore` valorizzato).
pub fn similarita_cosine(
    rt: &EmbeddingsState,
    expected: &str,
    actual: &str,
) -> Result<f64, PapErrore> {
    let emb_e = compute_embedding_opt(rt, expected)?
        .ok_or_else(|| PapErrore::Generico("Session ort non caricata".into()))?;
    let emb_a = compute_embedding_opt(rt, actual)?
        .ok_or_else(|| PapErrore::Generico("Session ort non caricata".into()))?;
    Ok(cosine_similarity(&emb_e, &emb_a))
}

/// Confronto stringa esatto, post-trim su entrambi i lati.
pub fn similarita_exact_match(expected: &str, actual: &str) -> f64 {
    if expected.trim() == actual.trim() {
        1.0
    } else {
        0.0
    }
}

/// L'`expected` è una regex; se matcha sull'`actual` è 1.0, altrimenti 0.0.
/// Errore se la regex non compila — il caller può registrare l'observation
/// con messaggio di errore.
pub fn similarita_regex(expected: &str, actual: &str) -> Result<f64, PapErrore> {
    let re = Regex::new(expected)
        .map_err(|e| PapErrore::Generico(format!("Regex invalida: {e}")))?;
    Ok(if re.is_match(actual) { 1.0 } else { 0.0 })
}

/// Dispatch per la `SimilarityFn` salvata nel golden. Per `cosine`
/// serve la Session ort (chiamante passa `Some(rt)`); per `exact-match`
/// e `regex` `rt` può essere `None`.
pub fn calcola(
    funzione: &str,
    expected: &str,
    actual: &str,
    rt: Option<&EmbeddingsState>,
) -> Result<f64, PapErrore> {
    match funzione {
        "cosine" => {
            let rt = rt.ok_or_else(|| {
                PapErrore::Generico("EmbeddingsState richiesto per cosine".into())
            })?;
            similarita_cosine(rt, expected, actual)
        }
        "exact-match" => Ok(similarita_exact_match(expected, actual)),
        "regex" => similarita_regex(expected, actual),
        "llm-judge" => Err(PapErrore::Generico(
            "Funzione 'llm-judge' non disponibile in v0.4 alpha (Step 8f)".into(),
        )),
        altro => Err(PapErrore::Generico(format!(
            "Funzione similarità sconosciuta: '{altro}'"
        ))),
    }
}

/// Decide se una similarità "passa" il golden secondo la sua funzione.
/// `cosine` e `llm-judge` usano la `soglia` (default 0.85); `exact-match`
/// e `regex` richiedono 1.0 esatto (la soglia per loro è ignorata).
pub fn is_passed(funzione: &str, similarita: f64, soglia: f64) -> bool {
    match funzione {
        "exact-match" | "regex" => similarita >= 1.0,
        _ => similarita >= soglia,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn norm(v: Vec<f32>) -> Vec<f32> {
        let n: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if n < 1e-12 {
            return v;
        }
        v.into_iter().map(|x| x / n).collect()
    }

    // ─────────── cosine_similarity (puro) ───────────

    #[test]
    fn cosine_vettori_identici_e_uno() {
        let a = norm(vec![1.0, 0.5, 0.25]);
        let b = a.clone();
        let s = cosine_similarity(&a, &b);
        assert!((s - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_vettori_ortogonali_e_zero() {
        let a = norm(vec![1.0, 0.0, 0.0]);
        let b = norm(vec![0.0, 1.0, 0.0]);
        let s = cosine_similarity(&a, &b);
        assert!(s.abs() < 1e-6);
    }

    #[test]
    fn cosine_vettori_opposti_clampato_a_zero() {
        let a = norm(vec![1.0, 0.0]);
        let b = norm(vec![-1.0, 0.0]);
        let s = cosine_similarity(&a, &b);
        // dot = -1, clampato a 0.
        assert_eq!(s, 0.0);
    }

    #[test]
    fn cosine_lunghezze_diverse_e_zero() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn cosine_vettori_vuoti_e_zero() {
        let a: Vec<f32> = vec![];
        let b: Vec<f32> = vec![];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn cosine_simil_intermedia() {
        // Angolo 60° → cos = 0.5.
        let a = norm(vec![1.0, 0.0]);
        let b = norm(vec![0.5, 0.5_f32.sqrt() * (3.0_f32.sqrt() / 2.0) * 2.0]);
        // Il modo più semplice: hand-build vettori che danno dot = 0.5 dopo
        // normalizzazione. Uso una scorciatoia sicura.
        let a = norm(vec![1.0, 0.0]);
        let b = norm(vec![1.0, 1.0_f32.sqrt() * 1.7320508]); // sqrt(3)
        let s = cosine_similarity(&a, &b);
        // sqrt(3)/2 normalizzato a (1/2, sqrt(3)/2) → dot = 1/2.
        assert!((s - 0.5).abs() < 1e-3, "atteso ~0.5, ottenuto {s}");
    }

    // ─────────── exact-match ───────────

    #[test]
    fn exact_match_identico() {
        assert_eq!(similarita_exact_match("ciao", "ciao"), 1.0);
    }

    #[test]
    fn exact_match_trim_su_entrambi_lati() {
        assert_eq!(similarita_exact_match("  ciao  ", "ciao"), 1.0);
        assert_eq!(similarita_exact_match("\nciao\t", "  ciao "), 1.0);
    }

    #[test]
    fn exact_match_diverso() {
        assert_eq!(similarita_exact_match("ciao", "ciao!"), 0.0);
        assert_eq!(similarita_exact_match("ciao", "Ciao"), 0.0);
    }

    #[test]
    fn exact_match_unicode_preserved() {
        assert_eq!(similarita_exact_match("你好 🦀", "你好 🦀"), 1.0);
        assert_eq!(similarita_exact_match("你好", "你好🦀"), 0.0);
    }

    // ─────────── regex ───────────

    #[test]
    fn regex_match_semplice() {
        let s = similarita_regex(r"^\d{3}-\d{4}$", "123-4567").unwrap();
        assert_eq!(s, 1.0);
    }

    #[test]
    fn regex_no_match() {
        let s = similarita_regex(r"^\d{3}$", "abc").unwrap();
        assert_eq!(s, 0.0);
    }

    #[test]
    fn regex_invalida_e_errore() {
        let r = similarita_regex(r"[invalida(", "x");
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("Regex invalida"));
    }

    #[test]
    fn regex_match_unicode() {
        let s = similarita_regex(r"\p{Han}+", "ciao 你好 mondo").unwrap();
        assert_eq!(s, 1.0);
    }

    // ─────────── calcola (dispatch) ───────────

    #[test]
    fn calcola_exact_match_dispatch() {
        let s = calcola("exact-match", "x", "x", None).unwrap();
        assert_eq!(s, 1.0);
    }

    #[test]
    fn calcola_regex_dispatch() {
        let s = calcola("regex", r"^x$", "x", None).unwrap();
        assert_eq!(s, 1.0);
    }

    #[test]
    fn calcola_cosine_senza_state_e_errore() {
        let r = calcola("cosine", "a", "b", None);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("EmbeddingsState"));
    }

    #[test]
    fn calcola_cosine_con_state_senza_session_e_errore() {
        let rt = EmbeddingsState::new();
        let r = calcola("cosine", "a", "b", Some(&rt));
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("non caricata"));
    }

    #[test]
    fn calcola_llm_judge_e_not_yet() {
        let r = calcola("llm-judge", "a", "b", None);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("8f"));
    }

    #[test]
    fn calcola_funzione_sconosciuta_e_errore() {
        let r = calcola("fuzzy", "a", "b", None);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("sconosciuta"));
    }

    // ─────────── is_passed ───────────

    #[test]
    fn is_passed_cosine_sopra_soglia() {
        assert!(is_passed("cosine", 0.90, 0.85));
        assert!(is_passed("cosine", 0.85, 0.85));
    }

    #[test]
    fn is_passed_cosine_sotto_soglia() {
        assert!(!is_passed("cosine", 0.84, 0.85));
        assert!(!is_passed("cosine", 0.0, 0.5));
    }

    #[test]
    fn is_passed_exact_match_richiede_uno() {
        assert!(is_passed("exact-match", 1.0, 0.5));
        assert!(!is_passed("exact-match", 0.99, 0.5));
        // La soglia è ignorata: anche con soglia 0.0 serve 1.0 esatto.
        assert!(!is_passed("exact-match", 0.5, 0.0));
    }

    #[test]
    fn is_passed_regex_richiede_uno() {
        assert!(is_passed("regex", 1.0, 0.5));
        assert!(!is_passed("regex", 0.5, 0.5));
    }

    #[test]
    fn is_passed_llm_judge_usa_soglia() {
        assert!(is_passed("llm-judge", 0.9, 0.85));
        assert!(!is_passed("llm-judge", 0.5, 0.85));
    }
}
