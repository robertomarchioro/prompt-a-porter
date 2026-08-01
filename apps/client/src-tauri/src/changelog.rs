//! #552 — Impostazioni → Aggiornamenti: le note di rilascio mostravano il
//! corpo della release GitHub, un template fisso generato da `release.yml`
//! che rimanda a `CHANGELOG.md` senza che il box lo mostri davvero. Il
//! CHANGELOG della versione *nuova* offerta dall'updater non è nel binario
//! installato: va recuperato da remoto, al tag della versione offerta, ed
//! estratta la sola sezione di quella versione.
//!
//! Scelta di progettazione: l'URL è costruito **qui**, sempre dallo stesso
//! org/repo hardcoded (`REPO_RAW_BASE`) più il tag `v<versione>` — mai da
//! un campo della release (`update.body`, un eventuale `download_url`,
//! ecc.). `versione` arriva comunque dall'updater (`update.version`), quindi
//! resta un dato esterno: viene validata come identificatore semver
//! plausibile PRIMA di essere interpolata nell'URL, così una stringa
//! inattesa (es. `../..`) non può alterare il path della richiesta.

use std::io::Read;
use std::time::Duration;

use crate::errore::PapErrore;

const REPO_RAW_BASE: &str = "https://raw.githubusercontent.com/robertomarchioro/prompt-a-porter";
const HTTP_TIMEOUT_SEC: u64 = 10;
/// Limite difensivo sulla risposta: un CHANGELOG.md reale è ordini di
/// grandezza più piccolo; evita di accumulare in memoria una risposta
/// anomala (mirror compromesso, redirect verso una risorsa enorme).
const MAX_BYTES_RISPOSTA: u64 = 2 * 1024 * 1024;

/// Valida che `versione` sia un identificatore semver plausibile
/// (`X.Y.Z` con eventuale suffisso pre-release/build alfanumerico) prima
/// di comporre l'URL: solo cifre, lettere, punti e trattini, e deve
/// iniziare con una cifra.
fn versione_valida(versione: &str) -> bool {
    !versione.is_empty()
        && versione.len() <= 32
        && versione.starts_with(|c: char| c.is_ascii_digit())
        && versione
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

/// Estrae dal testo di `CHANGELOG.md` la sezione che inizia con
/// `## v<versione>` (fino al prossimo termine di riga o spazio, per non
/// confondere `0.8.4` con `0.8.43`) e finisce alla prossima heading
/// `## v...` o a fine file. Ritorna `None` se la sezione non è presente.
pub fn estrai_sezione(changelog: &str, versione: &str) -> Option<String> {
    let prefisso = format!("## v{versione}");
    let righe: Vec<&str> = changelog.lines().collect();

    let inizio = righe.iter().position(|riga| {
        riga.strip_prefix(&prefisso)
            .is_some_and(|resto| resto.is_empty() || resto.starts_with(' '))
    })?;

    let fine = righe[inizio + 1..]
        .iter()
        .position(|riga| riga.starts_with("## v"))
        .map(|i| inizio + 1 + i)
        .unwrap_or(righe.len());

    let sezione = righe[inizio..fine].join("\n");
    let sezione = sezione.trim();
    if sezione.is_empty() {
        None
    } else {
        Some(sezione.to_string())
    }
}

fn scarica_changelog(versione: &str) -> Result<String, PapErrore> {
    let url = format!("{REPO_RAW_BASE}/v{versione}/CHANGELOG.md");
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(HTTP_TIMEOUT_SEC))
        .build();
    let resp = agent.get(&url).call().map_err(|e| {
        PapErrore::dominio(
            "Recupero del changelog remoto non riuscito. Verifica la connessione.",
            e,
        )
    })?;
    let mut testo = String::new();
    resp.into_reader()
        .take(MAX_BYTES_RISPOSTA)
        .read_to_string(&mut testo)
        .map_err(|e| PapErrore::dominio("Lettura del changelog remoto non riuscita.", e))?;
    Ok(testo)
}

/// Cmd Tauri: ritorna la sezione di `CHANGELOG.md` (markdown grezzo, non
/// sanificato: la sanificazione avviene lato frontend come per il corpo
/// della release, vedi `updater-notes.ts`) relativa a `versione`, la
/// versione offerta dall'updater. Se il recupero fallisce o la sezione
/// non esiste, il frontend ricade sul corpo della release GitHub già
/// ottenuto da `check()` — mai un box vuoto.
#[tauri::command]
pub fn changelog_sezione_remota(versione: String) -> Result<String, PapErrore> {
    if !versione_valida(&versione) {
        return Err(PapErrore::dominio(
            "Numero di versione non valido.",
            format!("versione rifiutata dalla validazione: {versione:?}"),
        ));
    }
    let testo = scarica_changelog(&versione)?;
    estrai_sezione(&testo, &versione).ok_or_else(|| {
        PapErrore::dominio(
            "Sezione del changelog non trovata per questa versione.",
            format!("nessuna heading '## v{versione}' nel changelog scaricato"),
        )
    })
}

#[cfg(test)]
mod test {
    use super::*;

    const CHANGELOG_FINTO: &str = "\
# Changelog — Prompt a Porter

## v0.8.43 — Audit di sicurezza chiuso (2026-08-01)

> Corpo v43, riga 1.
> Corpo v43, riga 2.

### Sicurezza

- voce v43

## v0.8.42 — Rifiniture updater e vault (2026-07-22)

> Corpo v42.

### Fix

- voce v42

## v0.8.41 — Hotfix (2026-07-18)

Corpo v41.
";

    #[test]
    fn estrae_sezione_intermedia_fino_alla_prossima_heading() {
        let sezione = estrai_sezione(CHANGELOG_FINTO, "0.8.43").unwrap();
        assert!(sezione.starts_with("## v0.8.43"));
        assert!(sezione.contains("Corpo v43, riga 1."));
        assert!(sezione.contains("voce v43"));
        assert!(!sezione.contains("Corpo v42"));
    }

    #[test]
    fn estrae_ultima_sezione_fino_a_fine_file() {
        let sezione = estrai_sezione(CHANGELOG_FINTO, "0.8.41").unwrap();
        assert!(sezione.contains("Corpo v41."));
    }

    #[test]
    fn non_confonde_prefissi_di_versione_simili() {
        // "0.8.4" è prefisso di "0.8.43" e "0.8.41": non deve fare match
        // con nessuna delle due (la versione richiesta non esiste).
        assert!(estrai_sezione(CHANGELOG_FINTO, "0.8.4").is_none());
    }

    #[test]
    fn versione_assente_ritorna_none() {
        assert!(estrai_sezione(CHANGELOG_FINTO, "9.9.9").is_none());
    }

    #[test]
    fn versione_valida_accetta_semver_plausibile() {
        assert!(versione_valida("0.8.43"));
        assert!(versione_valida("1.0.0-rc.1"));
        assert!(versione_valida("0.8.9-test4"));
    }

    #[test]
    fn versione_valida_rifiuta_input_pericoloso_o_vuoto() {
        assert!(!versione_valida(""));
        assert!(!versione_valida("../../etc/passwd"));
        assert!(!versione_valida("0.8.43/../evil"));
        assert!(!versione_valida("v0.8.43")); // il "v" lo antepone il codice, non l'input
        assert!(!versione_valida("0.8.43 "));
        assert!(!versione_valida(&"9".repeat(64)));
    }

    #[test]
    fn changelog_sezione_remota_rifiuta_versione_non_valida_senza_rete() {
        // Nessuna chiamata di rete deve partire per un input palesemente
        // malformato: l'errore torna subito dalla validazione.
        let risultato = changelog_sezione_remota("../evil".to_string());
        assert!(risultato.is_err());
    }
}
