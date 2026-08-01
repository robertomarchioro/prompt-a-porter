//! Stima del costo (USD) di un'esecuzione golden.
//!
//! I provider restituiscono (quasi sempre) i soli token di **output**; i
//! token di **input** non arrivano, quindi li stimiamo dalla lunghezza del
//! prompt compilato (~4 caratteri per token). Il costo è perciò una **stima**
//! che serve a dare un ordine di grandezza, non una fattura.
//!
//! I prezzi vengono dal **registro dei modelli** condiviso con il frontend
//! (`src/lib/modelli-registro.json`), aggiornato dal workflow settimanale
//! `modelli-refresh.yml`: sono gli stessi dati che alimentano le tendine, e
//! si aggiornano insieme a quelle. Prima erano una tabella a parte, allineata
//! a mano su listini ~2026-01 e ormai fuori scala (Opus era a 15/75 quando il
//! listino dice 5/25).
//!
//! Ordine di risoluzione:
//!   1. `ollama` → locale, costo 0;
//!   2. corrispondenza **esatta** su (provider, id) nel registro;
//!   3. euristica per famiglia, come rete di sicurezza per gli id non in
//!      registro (configurazioni vecchie, varianti scritte a mano);
//!   4. `None` → la UI mostra "—", mai un numero inventato.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

/// Registro condiviso col frontend, incluso a compile time: il crate si
/// ricompila da solo quando il workflow settimanale lo aggiorna.
const REGISTRO_JSON: &str = include_str!("../../src/lib/modelli-registro.json");

/// Prezzo per 1M di token, distinto tra input e output.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Prezzo {
    input: f64,
    output: f64,
}

#[derive(Deserialize)]
struct PrezzoRegistro {
    input: f64,
    output: f64,
}

#[derive(Deserialize)]
struct VoceRegistro {
    id: String,
    provider: String,
    prezzo: Option<PrezzoRegistro>,
}

#[derive(Deserialize)]
struct Registro {
    modelli: Vec<VoceRegistro>,
}

/// Circa 4 caratteri per token (euristica grezza, ma sufficiente per una
/// stima dell'ordine di grandezza del costo di input).
const CHAR_PER_TOKEN: usize = 4;

fn stima_token_input(prompt: &str) -> u64 {
    (prompt.chars().count() / CHAR_PER_TOKEN) as u64
}

/// Indice (provider, id minuscolo) → prezzo, costruito una sola volta.
///
/// Un registro malformato qui è un errore di build, non un caso da gestire a
/// runtime: il test `il_registro_incluso_e_valido` lo intercetta in CI.
fn indice() -> &'static HashMap<(String, String), Prezzo> {
    static INDICE: OnceLock<HashMap<(String, String), Prezzo>> = OnceLock::new();
    INDICE.get_or_init(|| {
        let r: Registro = serde_json::from_str(REGISTRO_JSON)
            .expect("modelli-registro.json non valido: correggere il registro");
        r.modelli
            .into_iter()
            .filter_map(|m| {
                let p = m.prezzo?;
                Some((
                    (m.provider, m.id.to_lowercase()),
                    Prezzo {
                        input: p.input,
                        output: p.output,
                    },
                ))
            })
            .collect()
    })
}

/// Rete di sicurezza per gli id non presenti in registro: riconosce la
/// famiglia dal nome. Deliberatamente grossolana — serve solo a non far
/// sparire la stima a chi ha in configurazione un id vecchio o scritto a mano
/// (es. `claude-sonnet-4.6` col punto invece del trattino).
fn prezzo_per_famiglia(provider: &str, m: &str) -> Option<Prezzo> {
    match provider {
        "anthropic" => {
            if m.contains("opus") {
                Some(Prezzo {
                    input: 5.0,
                    output: 25.0,
                })
            } else if m.contains("haiku") {
                Some(Prezzo {
                    input: 1.0,
                    output: 5.0,
                })
            } else if m.contains("sonnet") {
                Some(Prezzo {
                    input: 3.0,
                    output: 15.0,
                })
            } else {
                None
            }
        }
        "openai" => {
            // "mini"/"nano" prima di "gpt-4o": `gpt-4o-mini` contiene entrambi.
            if m.contains("nano") {
                Some(Prezzo {
                    input: 0.1,
                    output: 0.4,
                })
            } else if m.contains("mini") {
                Some(Prezzo {
                    input: 0.4,
                    output: 1.6,
                })
            } else if m.contains("gpt-4o") || m.contains("gpt-4.1") {
                Some(Prezzo {
                    input: 2.5,
                    output: 10.0,
                })
            } else if m.contains("gpt-5") {
                Some(Prezzo {
                    input: 1.25,
                    output: 10.0,
                })
            } else {
                None
            }
        }
        "gemini" => {
            // "flash" prima di "pro": non esiste un id con entrambi, ma
            // l'ordine rende esplicita la precedenza voluta.
            if m.contains("flash") {
                Some(Prezzo {
                    input: 0.3,
                    output: 2.5,
                })
            } else if m.contains("pro") {
                Some(Prezzo {
                    input: 1.25,
                    output: 10.0,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Prezzo per (provider, modello). `None` se il modello non è risolvibile.
fn prezzo(provider: &str, model: &str) -> Option<Prezzo> {
    // Locale: nessun costo per-token.
    if provider == "ollama" {
        return Some(Prezzo {
            input: 0.0,
            output: 0.0,
        });
    }

    let m = model.to_lowercase();
    if let Some(p) = indice().get(&(provider.to_string(), m.clone())) {
        return Some(*p);
    }
    prezzo_per_famiglia(provider, &m)
}

/// Stima il costo in USD di un'esecuzione: token di input **stimati** dal
/// prompt + token di output **reali** dal provider. Ritorna `None` se il
/// modello non è risolvibile (così la UI mostra "—" invece di un numero
/// inventato).
pub fn stima_costo(
    provider: &str,
    model: &str,
    prompt: &str,
    tokens_output: Option<u32>,
) -> Option<f64> {
    let p = prezzo(provider, model)?;
    let in_tok = stima_token_input(prompt) as f64;
    let out_tok = tokens_output.unwrap_or(0) as f64;
    let costo = in_tok / 1_000_000.0 * p.input + out_tok / 1_000_000.0 * p.output;
    Some(costo)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn il_registro_incluso_e_valido() {
        let i = indice();
        assert!(!i.is_empty(), "registro vuoto");
        for p in ["anthropic", "openai", "gemini"] {
            let n = i.keys().filter(|(prov, _)| prov == p).count();
            assert!(n >= 3, "solo {n} modelli con prezzo per '{p}'");
        }
    }

    #[test]
    fn nessun_prezzo_negativo_o_assurdo() {
        for ((prov, id), p) in indice() {
            assert!(p.input >= 0.0 && p.output >= 0.0, "{prov}/{id} ha prezzo negativo");
            assert!(
                p.input < 1000.0 && p.output < 1000.0,
                "{prov}/{id} ha un prezzo fuori scala: {p:?}"
            );
        }
    }

    #[test]
    fn modello_sconosciuto_ritorna_none() {
        assert!(stima_costo("openai", "modello-ignoto", "ciao", Some(100)).is_none());
        assert!(stima_costo("provider-x", "qualsiasi", "ciao", Some(100)).is_none());
    }

    #[test]
    fn ollama_e_locale_costo_zero() {
        let c = stima_costo("ollama", "llama3.2", "un prompt qualsiasi", Some(500)).unwrap();
        assert_eq!(c, 0.0);
    }

    #[test]
    fn usa_il_prezzo_esatto_del_registro_non_la_famiglia() {
        // Opus 4.8 sta a 5/25 nel listino reale; la vecchia tabella diceva
        // 15/75. Se questo test tornasse al vecchio valore, la stima è di
        // nuovo scollegata dal registro.
        let prompt = "x".repeat(4000); // ~1000 token input
        let c = stima_costo("anthropic", "claude-opus-4-8", &prompt, Some(1000)).unwrap();
        // 1000/1e6*5.0 + 1000/1e6*25.0 = 0.005 + 0.025
        assert!((c - 0.030).abs() < 1e-9, "costo = {c}");
    }

    #[test]
    fn la_corrispondenza_e_insensibile_al_maiuscolo() {
        let prompt = "x".repeat(4000);
        let a = stima_costo("anthropic", "claude-opus-4-8", &prompt, Some(100));
        let b = stima_costo("anthropic", "CLAUDE-OPUS-4-8", &prompt, Some(100));
        assert_eq!(a, b);
    }

    #[test]
    fn anthropic_sonnet_somma_input_stimato_e_output_reale() {
        // prompt di 40 caratteri → ~10 token input; 1000 token output.
        let prompt = "x".repeat(40);
        let c = stima_costo("anthropic", "claude-sonnet-4-6", &prompt, Some(1000)).unwrap();
        // input: 10/1e6 * 3.0 = 0.00003 ; output: 1000/1e6 * 15.0 = 0.015
        assert!((c - 0.01503).abs() < 1e-9, "costo = {c}");
    }

    #[test]
    fn opus_piu_caro_di_haiku() {
        let prompt = "y".repeat(400);
        let opus = stima_costo("anthropic", "claude-opus-4-8", &prompt, Some(1000)).unwrap();
        let haiku = stima_costo("anthropic", "claude-haiku-4-5", &prompt, Some(1000)).unwrap();
        assert!(opus > haiku, "opus {opus} deve costare più di haiku {haiku}");
    }

    #[test]
    fn un_id_non_in_registro_ricade_sulla_famiglia() {
        // Forma col punto invece del trattino: presente in configurazioni
        // vecchie, assente dal registro. Non deve perdere la stima.
        let prompt = "x".repeat(40);
        let c = stima_costo("anthropic", "claude-sonnet-4.6", &prompt, Some(1000));
        assert!(c.is_some(), "il fallback per famiglia non ha risposto");
    }

    #[test]
    fn openai_mini_e_nano_hanno_precedenza_nel_fallback() {
        // Id fuori registro, così si esercita davvero il fallback.
        let prompt = "z".repeat(4000); // ~1000 token input
        let mini = stima_costo("openai", "gpt-4o-mini-vecchio", &prompt, Some(1000)).unwrap();
        // mini: 1000/1e6*0.4 + 1000/1e6*1.6 = 0.0004 + 0.0016 = 0.002
        assert!((mini - 0.002).abs() < 1e-9, "costo mini = {mini}");

        let nano = stima_costo("openai", "gpt-4o-nano-vecchio", &prompt, Some(1000)).unwrap();
        assert!(nano < mini, "nano {nano} deve costare meno di mini {mini}");
    }

    #[test]
    fn output_none_conta_solo_input() {
        let prompt = "w".repeat(4000); // ~1000 token input
        let c = stima_costo("openai", "gpt-4o", &prompt, None).unwrap();
        // gpt-4o è fuori registro (rilasciato 2024, sotto la soglia): risolve
        // per famiglia a 2.50/10 → solo input: 1000/1e6 * 2.50 = 0.0025
        assert!((c - 0.0025).abs() < 1e-9, "costo = {c}");
    }
}
