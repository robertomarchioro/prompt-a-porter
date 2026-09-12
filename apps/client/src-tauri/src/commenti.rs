// Commenti inline nel body del prompt (issue #643): `{{!-- testo --}}`.
//
// Il commento è una nota dell'autore che non deve mai raggiungere il
// modello né gli appunti: viene tolto PRIMA di ogni altra elaborazione del
// body (import, segnaposti, golden). Resta invece nel vault, nelle versioni,
// nell'export, negli embeddings e nel Ritocco (dove il modello revisore lo
// vede e lo preserva nel diff), perché è parte del sorgente.
//
// Semantica condivisa con client TS, MCP server e Go CLI, verificata dalla
// fixture `packages/shared-schema/fixtures/commenti-conformita.json`:
// - unica forma `{{!-- … --}}`, chiude al primo `--}}`, multiriga;
// - `{{! … }}` NON è un commento; un commento non chiuso resta inalterato;
// - passo 1: si toglie il token; passo 2: ogni riga che conteneva un
//   commento e che dopo il passo 1 è vuota (solo spazi/tab/CR) sparisce con
//   il suo a-capo. Le righe già vuote in origine restano.

use regex::Regex;
use std::borrow::Cow;
use std::collections::HashSet;
use std::ops::Range;
use std::sync::OnceLock;

/// Apertura del token: usata dal linter per trovare i commenti non chiusi.
pub const APERTURA: &str = "{{!--";

fn re_commento() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // `(?s)` perché il commento può andare a capo.
    R.get_or_init(|| Regex::new(r"(?s)\{\{!--.*?--\}\}").unwrap())
}

/// Range byte `[start, end)` di ogni commento nel testo, in ordine.
pub fn intervalli_commenti(testo: &str) -> Vec<Range<usize>> {
    re_commento().find_iter(testo).map(|m| m.range()).collect()
}

/// True se l'offset byte `pos` cade dentro un commento.
pub fn dentro_commento(intervalli: &[Range<usize>], pos: usize) -> bool {
    intervalli.iter().any(|r| r.contains(&pos))
}

fn riga_vuota(riga: &str) -> bool {
    riga.bytes().all(|b| matches!(b, b' ' | b'\t' | b'\r'))
}

/// Restituisce il testo senza i commenti `{{!-- … --}}`. Senza commenti
/// non alloca.
pub fn rimuovi_commenti(testo: &str) -> Cow<'_, str> {
    let intervalli = intervalli_commenti(testo);
    if intervalli.is_empty() {
        return Cow::Borrowed(testo);
    }

    // Passo 1: togli i token, ricordando su quale riga dell'output cadevano.
    let mut senza_token = String::with_capacity(testo.len());
    let mut righe_toccate: HashSet<usize> = HashSet::new();
    let mut riga_corrente = 0usize;
    let mut cursore = 0usize;
    for r in &intervalli {
        let pezzo = &testo[cursore..r.start];
        riga_corrente += pezzo.bytes().filter(|b| *b == b'\n').count();
        righe_toccate.insert(riga_corrente);
        senza_token.push_str(pezzo);
        cursore = r.end;
    }
    senza_token.push_str(&testo[cursore..]);

    // Passo 2: elimina le righe toccate rimaste vuote.
    let righe: Vec<&str> = senza_token
        .split('\n')
        .enumerate()
        .filter(|(i, riga)| !(righe_toccate.contains(i) && riga_vuota(riga)))
        .map(|(_, riga)| riga)
        .collect();
    Cow::Owned(righe.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Caso {
        nome: String,
        body: String,
        atteso: String,
    }

    #[derive(Deserialize)]
    struct Fixture {
        casi: Vec<Caso>,
    }

    /// #643: la fixture è la fonte di verità condivisa con TS e Go.
    fn fixture() -> Vec<Caso> {
        let json =
            include_str!("../../../../packages/shared-schema/fixtures/commenti-conformita.json");
        let f: Fixture = serde_json::from_str(json).expect("fixture JSON valida");
        assert!(f.casi.len() > 10, "fixture sospettosamente corta");
        f.casi
    }

    #[test]
    fn conformita_cross_linguaggio() {
        for caso in fixture() {
            assert_eq!(
                rimuovi_commenti(&caso.body),
                caso.atteso,
                "caso `{}`",
                caso.nome
            );
        }
    }

    #[test]
    fn idempotente() {
        for caso in fixture() {
            assert_eq!(
                rimuovi_commenti(&caso.atteso),
                caso.atteso,
                "caso `{}`",
                caso.nome
            );
        }
    }

    #[test]
    fn senza_commenti_non_alloca() {
        assert!(matches!(
            rimuovi_commenti("Ciao {{nome}}"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn intervalli_coprono_il_token_intero() {
        let body = "a {{!-- x --}} b";
        let r = intervalli_commenti(body);
        assert_eq!(r, vec![2..14]);
        assert_eq!(&body[2..14], "{{!-- x --}}");
    }

    #[test]
    fn intervalli_non_chiuso_vuoto() {
        assert!(intervalli_commenti("{{!-- aperto").is_empty());
    }

    #[test]
    fn dentro_commento_confini() {
        let r = intervalli_commenti("a {{!-- x --}} b");
        assert!(!dentro_commento(&r, 1));
        assert!(dentro_commento(&r, 2));
        assert!(dentro_commento(&r, 13));
        assert!(!dentro_commento(&r, 14));
    }

    #[test]
    fn multibyte_prima_del_commento() {
        // Offset byte, non char: nessun panic su UTF-8 multibyte.
        assert_eq!(rimuovi_commenti("è {{!-- ✓ --}} ü"), "è  ü");
    }
}
