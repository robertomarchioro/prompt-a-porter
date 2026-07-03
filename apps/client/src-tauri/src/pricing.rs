//! Stima del costo (USD) di un'esecuzione golden.
//!
//! I provider restituiscono (quasi sempre) i soli token di **output**; i
//! token di **input** non arrivano, quindi li stimiamo dalla lunghezza del
//! prompt compilato (~4 caratteri per token). Il costo è perciò una **stima**
//! che serve a dare un ordine di grandezza, non una fattura.
//!
//! I prezzi sono **indicativi** (USD per 1M di token, listini pubblici
//! ~2026-01) e vanno aggiornati periodicamente. Modelli non in tabella →
//! `None` (nessuna stima inventata). Ollama è locale → costo 0.

/// Prezzo per 1M di token, distinto tra input e output.
struct Prezzo {
    input: f64,
    output: f64,
}

/// Circa 4 caratteri per token (euristica grezza, ma sufficiente per una
/// stima dell'ordine di grandezza del costo di input).
const CHAR_PER_TOKEN: usize = 4;

fn stima_token_input(prompt: &str) -> u64 {
    (prompt.chars().count() / CHAR_PER_TOKEN) as u64
}

/// Prezzo per (provider, modello). `None` se il modello non è in tabella.
fn prezzo(provider: &str, model: &str) -> Option<Prezzo> {
    let m = model.to_lowercase();
    match provider {
        // Locale: nessun costo per-token.
        "ollama" => Some(Prezzo {
            input: 0.0,
            output: 0.0,
        }),
        "anthropic" => {
            if m.contains("opus") {
                Some(Prezzo {
                    input: 15.0,
                    output: 75.0,
                })
            } else if m.contains("haiku") {
                Some(Prezzo {
                    input: 0.80,
                    output: 4.0,
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
            // Controlla "mini" prima di "gpt-4o" (gpt-4o-mini contiene entrambi).
            if m.contains("mini") {
                Some(Prezzo {
                    input: 0.15,
                    output: 0.60,
                })
            } else if m.contains("gpt-4o") || m.contains("gpt-4.1") {
                Some(Prezzo {
                    input: 2.50,
                    output: 10.0,
                })
            } else {
                None
            }
        }
        "gemini" => {
            if m.contains("flash") {
                Some(Prezzo {
                    input: 0.075,
                    output: 0.30,
                })
            } else if m.contains("pro") {
                Some(Prezzo {
                    input: 1.25,
                    output: 5.0,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Stima il costo in USD di un'esecuzione: token di input **stimati** dal
/// prompt + token di output **reali** dal provider. Ritorna `None` se il
/// modello non è nella tabella dei prezzi (così la UI mostra "—" invece di
/// un numero inventato).
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
    fn openai_mini_ha_precedenza_su_gpt4o() {
        // gpt-4o-mini contiene sia "mini" sia "gpt-4o": deve usare il prezzo mini.
        let prompt = "z".repeat(4000); // ~1000 token input
        let mini = stima_costo("openai", "gpt-4o-mini", &prompt, Some(1000)).unwrap();
        // mini: 1000/1e6*0.15 + 1000/1e6*0.60 = 0.00015 + 0.0006 = 0.00075
        assert!((mini - 0.00075).abs() < 1e-9, "costo mini = {mini}");
    }

    #[test]
    fn output_none_conta_solo_input() {
        let prompt = "w".repeat(4000); // ~1000 token input
        let c = stima_costo("openai", "gpt-4o", &prompt, None).unwrap();
        // solo input: 1000/1e6 * 2.50 = 0.0025
        assert!((c - 0.0025).abs() < 1e-9, "costo = {c}");
    }
}
