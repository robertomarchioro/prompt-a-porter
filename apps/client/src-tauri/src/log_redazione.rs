//! Redazione dei segreti HTTP nei log applicativi (fix sicurezza MEDIUM,
//! CWE-532 — inserimento di informazioni sensibili in un file di log).
//!
//! ## Il difetto
//!
//! `ureq` 2.12.1 registra a livello `debug!` l'intero preludio HTTP della
//! richiesta (metodo, header, riga vuota finale) in **un solo record**
//! con target che inizia per `ureq` — vedi `ureq-2.12.1/src/unit.rs:480`
//! (`debug!("writing prelude: {}", prelude)`). Il crate maschera da sé
//! **solo** gli header `Authorization` e `Cookie`
//! (`is_header_sensitive`, `unit.rs:487-489`). Questa app invia le chiavi
//! API dei provider in `x-api-key` (Anthropic, `provider_ai.rs`) e
//! `x-goog-api-key` (Gemini, `provider_ai.rs`), che restano quindi in
//! chiaro nel record — e da lì finiscono nel file `pap.log`, nell'evento
//! inoltrato alla webview e nell'archivio ZIP che l'app invita ad
//! allegare a una issue pubblica (`debug_log::debug_log_esporta_zip`).
//!
//! ## Perché i record non vengono scartati
//!
//! `ureq` non ha alcun `info!`/`warn!`: solo `debug!`/`trace!` (le
//! chiamate ai provider AI) e `error!` (2 occorrenze, certificati radice
//! in `rtls.rs`, nessun dato di rete). Scartare i record `DEBUG`/`TRACE`
//! renderebbe quindi invisibile ogni chiamata fallita a un provider —
//! `provider_ai.rs` non ha log propri, si affida interamente a quelli di
//! `ureq` per la diagnostica di rete. Questo modulo conserva ogni record
//! e ogni livello, e redige solo il **valore** degli header sensibili.
//!
//! ## Perché la redazione è ancorata al target
//!
//! Applicare la regola "cancella dal `:` a fine riga" a QUALUNQUE
//! messaggio (non solo al preludio HTTP) è la trappola in cui è caduto
//! un tentativo precedente: un `id` proveniente da un delta di sync o da
//! un backup importato può contenere per coincidenza il testo
//! `cookie:` — logs applicativi come `sync: prompt {id} scartato` o
//! `import_export` che stampano dati esterni verrebbero troncati,
//! perdendo sia il motivo dello scarto sia il valore. La redazione qui
//! si applica **solo** ai record il cui target inizia per `ureq` (unico
//! crate, tra le dipendenze di questa app, che logga preludi HTTP con
//! valori di header in chiaro — verificato: `reqwest` è solo dipendenza
//! transitiva, non usata direttamente per le chiamate ai provider AI).
//! I log applicativi (target `pap_lib::…`) non vengono mai toccati da
//! questa regex.
//!
//! ## Limiti noti (issue #591)
//!
//! `redigi_valori_header` filtra per **nome di header**, ancorato a inizio
//! riga (`^[ \t]*(?:nome)[ \t]*:`). Restano scoperti, per costruzione:
//!
//! 1. un nome di header che compare **a metà riga** invece che a inizio
//!    riga (es. `"header inatteso x-api-key: sk-..."` dentro un messaggio
//!    di log/panic libero);
//! 2. un header formattato con `Debug` — fra virgolette o graffe, come
//!    `{"x-api-key": "sk-..."}` (`{:?}` di una mappa di header);
//! 3. una **chiave nuda**, senza alcun prefisso di nome-header
//!    riconosciuto;
//! 4. un **token in query string** di un URL (`?key=sk-...`).
//!
//! Da qui in poi `redigi_valori_header` applica anche una seconda linea di
//! difesa **per forma di segreto** (non per nome di header):
//! `redigi_forme_di_segreto` cerca, ovunque nel testo, i prefissi di
//! valore non ambigui `sk-`, `AIza`, `Bearer ` seguiti da un token
//! sufficientemente lungo, e ne redige SOLO il token. Questo copre tutti
//! e quattro i casi sopra **quando** il segreto usa uno di questi
//! prefissi — è esattamente lo scenario delle chiavi API dei provider
//! usati da questa app (Anthropic `sk-ant-…`, Google `AIza…`, e il più
//! generico schema HTTP `Bearer`).
//!
//! **Cosa NON copre**, deliberatamente: non esiste alcuna regola generica
//! su stringhe ad alta entropia. L'issue #591 la scarta esplicitamente —
//! in un crash log un falso positivo che redige un percorso, un hash o un
//! ID di sincronizzazione rende la diagnostica inutilizzabile, che è lo
//! scopo stesso per cui quel file esiste. Un segreto con una forma NON
//! elencata sopra (un prefisso non riconosciuto, nessun prefisso
//! distintivo) resta scoperto anche da questa seconda linea di difesa. V.
//! anche la nota "Redazione dei segreti" in `panic_diagnostics`, che
//! applica questa stessa funzione al payload dei panic.
//!
//! **Coda residua di falsi positivi, accettata deliberatamente**: la
//! stessa euristica per prefisso può scattare su un identificatore
//! kebab-case che inizi per coincidenza con `sk-` seguito da almeno 10
//! caratteri dell'alfabeto del token — es. un tag BCP 47 concatenato come
//! `sk-SK-Latn-1234567890`. Accettato rispetto all'alternativa (una regola
//! a entropia, già scartata sopra per un motivo più grave): il costo di
//! questo falso positivo è la perdita locale di un identificatore in un
//! log di diagnostica, non l'inutilizzabilità sistematica del file.
use std::sync::OnceLock;

use regex::Regex;

/// Nomi di header il cui valore va redatto se compaiono in un record con
/// target `ureq`/`ureq::…`. Il match è case-insensitive (i nomi di
/// header HTTP lo sono, RFC 7230 §3.2). `authorization` e `cookie` sono
/// già mascherati da `ureq` stesso (`is_header_sensitive`); restano qui
/// come difesa in profondità, a costo zero se il crate cambia
/// comportamento in una versione futura.
const HEADER_SENSIBILI: &[&str] = &[
    "x-api-key",
    "x-goog-api-key",
    "authorization",
    "proxy-authorization",
    "cookie",
];

/// Vero se il target di un record può contenere il preludio HTTP loggato
/// da `ureq` (crate `ureq`, modulo `unit` — target di default
/// `module_path!()` = `"ureq::unit"`, mai sovrascritto con `target:` nel
/// sorgente del crate: verificato con `grep -rn "target:"` su
/// `ureq-2.12.1/src`, nessuna occorrenza).
pub fn target_potenzialmente_sensibile(target: &str) -> bool {
    target == "ureq" || target.starts_with("ureq::")
}

fn pattern_redazione() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    CELL.get_or_init(|| {
        let alternanza = HEADER_SENSIBILI
            .iter()
            .map(|h| regex::escape(h))
            .collect::<Vec<_>>()
            .join("|");
        // (?i) case-insensitive sul nome header (evasione: maiuscole miste).
        // (?m) ^ ancora a inizio di OGNI riga logica dentro il testo dato
        // (il preludio di ureq è un solo messaggio con \r\n letterali tra
        // gli header). [ \t]* prima e dopo il nome tollera spazi/tab
        // attorno ai due punti (evasione: "x-api-key : segreto"). Il
        // valore è [^\r\n]*: si ferma SEMPRE al primo \r o \n (o a fine
        // stringa) — mai "cancella fino a fine messaggio".
        let pattern = format!(r"(?im)^(?P<pre>[ \t]*(?:{alternanza})[ \t]*:[ \t]*)[^\r\n]*");
        Regex::new(&pattern).expect("pattern di redazione segreti header non valido")
    })
}

/// Sostituisce, in `testo`, il *valore* di ogni header sensibile con
/// `***`, preservando nome dell'header, spaziatura originale e
/// terminatori di riga. Safe su testo multibyte: opera su `&str`
/// (UTF-8), il crate `regex` non taglia mai a metà di un carattere.
///
/// Concatena SEMPRE, subito dopo, la seconda linea di difesa per forma di
/// segreto (`redigi_forme_di_segreto`, issue #591 — v. "Limiti noti" nella
/// doc di modulo): ogni chiamante di questa funzione (`panic_diagnostics`,
/// `redigi_testo_log_storico`, `debug_log::redigi_testo_per_export`)
/// riceve entrambe le linee di difesa senza doverle richiamare a parte.
pub fn redigi_valori_header(testo: &str) -> String {
    let per_nome_header = pattern_redazione().replace_all(testo, "${pre}***");
    redigi_forme_di_segreto(&per_nome_header)
}

/// Soglia di dimensione, in byte, condivisa fra la rotazione di `pap.log`
/// (`tauri_plugin_log`, v. `lib.rs::run`) e il tetto applicato a
/// `pap-crash.log` (`panic_diagnostics`, che non ruota mai da solo — ogni
/// panic APPENDE una sezione, v. doc di quel modulo) — issue #591, punto
/// minore "pap-crash.log non ruota". Un'unica costante evita che le due
/// soglie divergano in silenzio.
pub(crate) const MAX_LOG_FILE_SIZE_BYTES: u64 = 5 * 1024 * 1024;

/// Prefisso di valore riconosciuto come inequivocabilmente un segreto, a
/// bassissimo rischio di falso positivo (issue #591, punto 2). Il gruppo
/// `pre` cattura il carattere che precede il prefisso — o l'inizio della
/// stringa — cosicché il match scatti SOLO a un confine di token: senza
/// questo vincolo `sk-` matcherebbe anche dentro un percorso come
/// `/tmp/task-sk-build/x` (preceduto da `-`, che fa parte dello stesso
/// alfabeto del token) o dentro un hash esadecimale che lo contenesse per
/// coincidenza. Il crate `regex` non supporta i lookbehind: il gruppo
/// `pre`, catturato e poi riemesso in `redigi_forme_di_segreto`, è
/// l'equivalente "lookbehind-safe" — consuma il carattere di confine ma lo
/// restituisce intatto nella sostituzione.
fn pattern_forma_sk() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    CELL.get_or_init(|| {
        Regex::new(r"(?P<pre>^|[^A-Za-z0-9_-])sk-[A-Za-z0-9_-]{10,}")
            .expect("pattern di redazione per forma sk- non valido")
    })
}

/// Prefisso delle chiavi API Google (`AIza…`, es. Gemini) — stesso vincolo
/// di confine di `pattern_forma_sk`.
fn pattern_forma_aiza() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    CELL.get_or_init(|| {
        Regex::new(r"(?P<pre>^|[^A-Za-z0-9_-])AIza[A-Za-z0-9_-]{10,}")
            .expect("pattern di redazione per forma AIza non valido")
    })
}

/// Schema HTTP `Bearer` (RFC 6750): il valore ammette anche `.`, `~`, `+`,
/// `/`, `=` (alfabeto dei token base64url e dei JWT), quindi il vincolo di
/// confine su `pre` usa comunque il gruppo `[^A-Za-z0-9_-]` — l'alfabeto
/// del PREFISSO che precede `Bearer`, non quello del token — così
/// `"unBearer xxxxxxxxxx"` (preceduto da `n`, alfanumerico) non matcha.
///
/// `(?i)` — SOLO su questo pattern, non su `sk-`/`AIza` (quelli hanno un
/// casing fisso e non ambiguo) — perché RFC 7235 §2.1 dichiara lo schema
/// di autenticazione case-insensitive: `bearer <token>` è legale e alcune
/// librerie/dump lo emettono in minuscolo. `[ \t]+` al posto di un singolo
/// spazio letterale tollera più spazi/tab fra schema e token (evasione:
/// `"Bearer  segreto"`, doppio spazio — con un solo spazio letterale il
/// carattere consumato dopo "Bearer " resterebbe uno spazio, non l'inizio
/// del token, e il match fallirebbe) — stessa tolleranza di spaziatura già
/// usata da `pattern_redazione` per i due punti dopo il nome header.
fn pattern_forma_bearer() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    CELL.get_or_init(|| {
        Regex::new(r"(?i)(?P<pre>^|[^A-Za-z0-9_-])Bearer[ \t]+[A-Za-z0-9._~+/=-]{10,}")
            .expect("pattern di redazione per forma Bearer non valido")
    })
}

/// Seconda linea di difesa: redige per **forma di segreto** invece che
/// per nome di header (issue #591). A differenza di `pattern_redazione`
/// (ancorato a inizio riga e al NOME dell'header), queste regole cercano
/// ovunque nel testo un prefisso di VALORE non ambiguo e ne rimuovono
/// interamente il token, senza conservarne alcun carattere oltre il
/// prefisso diagnostico (`sk-…[REDATTO]`, `AIza…[REDATTO]`,
/// `Bearer …[REDATTO]`). Nessuna regola generica su entropia — v. "Limiti
/// noti" nella doc di modulo.
///
/// **Idempotenza voluta, non incidentale**: il `…` (U+2026, ellissi
/// tipografica) SUBITO dopo il prefisso è ciò che rende un marker già
/// prodotto non ri-matchabile da una seconda passata — `pap-crash.log`
/// viene ri-redatto per intero a ogni export
/// (`debug_log::redigi_testo_per_export`), quindi `f(f(x)) == f(x)` deve
/// valere sempre. `…` non fa parte dell'alfabeto di nessuno dei tre
/// pattern (`[A-Za-z0-9_-]` per `sk-`/`AIza`, `[A-Za-z0-9._~+/=-]` per
/// `Bearer`), quindi subito dopo il prefisso il token richiesto dal
/// quantificatore `{10,}` non può nemmeno iniziare a formarsi. Non
/// semplificare mai il marker eliminando il `…` (es. in `sk-[REDATTO]`
/// senza ellissi): resta la garanzia esplicita di non ri-match, va
/// preservata in ogni forma futura del marker anche se un cambio
/// dell'alfabeto di uno dei tre pattern rendesse la protezione implicita
/// meno ovvia da verificare a colpo d'occhio.
fn redigi_forme_di_segreto(testo: &str) -> String {
    let t = pattern_forma_sk().replace_all(testo, "${pre}sk-…[REDATTO]");
    let t = pattern_forma_aiza().replace_all(&t, "${pre}AIza…[REDATTO]");
    let t = pattern_forma_bearer().replace_all(&t, "${pre}Bearer …[REDATTO]");
    t.into_owned()
}

/// Timestamp UTC corrente, formattato `[YYYY-MM-DD][HH:MM:SS]` — la
/// stessa identica forma prodotta dal formatter di default di
/// `tauri-plugin-log` (`tauri-plugin-log-2.9.0/src/lib.rs:422-437`:
/// `format_description!` con parentesi quadre letterali attorno a data e
/// ora, `TimezoneStrategy::UseUtc` di default, mai sovrascritto in
/// `lib.rs::run` di questo crate). Riprodotto qui a mano (algoritmo
/// civil-from-days di Howard Hinnant, 2013, accurato 1970-9999) invece
/// di aggiungere una dipendenza diretta dal crate `time` solo per un
/// formato che questo stesso crate già replica altrove per uno scopo
/// diverso (`debug_log::format_iso_utc`, export ZIP/metadata, formato
/// ISO 8601 — non compatibile con `parse_riga`, per questo la
/// duplicazione qui è intenzionale e non un semplice riuso).
pub fn timestamp_bracket_utc_adesso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    timestamp_bracket_utc(secs)
}

fn timestamp_bracket_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let time_of_day = secs.rem_euclid(86_400);
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };

    format!("[{year:04}-{month:02}-{d:02}][{h:02}:{m:02}:{s:02}]")
}

/// Costruisce la riga di log completa nel formato
/// `[data][ora][target][LIVELLO] messaggio`, applicando la redazione dei
/// segreti SOLO se il target lo giustifica (vedi doc di modulo). Isolata
/// dal formatter registrato in `lib.rs::run` per poterla testare senza
/// costruire un `log::Record` reale.
pub fn formatta_riga(target: &str, level: log::Level, messaggio: &str) -> String {
    let ts = timestamp_bracket_utc_adesso();
    if target_potenzialmente_sensibile(target) {
        let redatto = redigi_valori_header(messaggio);
        format!("{ts}[{target}][{level}] {redatto}")
    } else {
        format!("{ts}[{target}][{level}] {messaggio}")
    }
}

fn pattern_prefisso_riga() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    CELL.get_or_init(|| {
        // Stessa forma a 4 parentesi quadre riconosciuta da
        // `debug_log::parse_riga` (data, ora, target, livello). Gruppo 1 =
        // prefisso completo (invariato), gruppo 2 = target, gruppo 3 =
        // resto della riga (il messaggio, con lo spazio iniziale).
        Regex::new(r"^(\[[^\]\r\n]*\]\[[^\]\r\n]*\]\[([^\]\r\n]*)\]\[[^\]\r\n]*\])(.*)$")
            .expect("pattern di riconoscimento riga di log non valido")
    })
}

/// Redazione "a freddo" per i log **già scritti su disco** prima di
/// questa fix — difesa in profondità: usata da
/// `debug_log::debug_log_esporta_zip` (bytes dello ZIP) e da
/// `debug_log::debug_log_leggi` (viewer in-app). Senza questa funzione
/// chi usa l'app da prima della fix esporterebbe/vedrebbe comunque le
/// chiavi già scritte in chiaro.
///
/// Il preludio di `ureq` è un solo record ma, contenendo `\r\n`
/// letterali, sul file appare come **più righe fisiche**: solo la prima
/// porta il prefisso `[data][ora][target][LIVELLO]` (le righe di
/// continuazione — gli header, inclusa la chiave — non hanno un target
/// proprio). Per questo la redazione è stateful riga per riga: una riga
/// col prefisso a 4 parentesi aggiorna il "target corrente"; una riga
/// senza prefisso eredita il target corrente (stessa euristica
/// "best-effort" già usata da `debug_log::parse_riga` per il viewer).
pub fn redigi_testo_log_storico(contenuto: &str) -> String {
    let prefisso = pattern_prefisso_riga();
    let mut target_corrente = String::new();

    let righe: Vec<String> = contenuto
        .split('\n')
        .map(|riga| {
            if let Some(cap) = prefisso.captures(riga) {
                target_corrente = cap[2].to_string();
                if target_potenzialmente_sensibile(&target_corrente) {
                    format!("{}{}", &cap[1], redigi_valori_header(&cap[3]))
                } else {
                    riga.to_string()
                }
            } else if target_potenzialmente_sensibile(&target_corrente) {
                redigi_valori_header(riga)
            } else {
                riga.to_string()
            }
        })
        .collect();

    righe.join("\n")
}

#[cfg(test)]
mod test {
    use super::*;

    // ─── target_potenzialmente_sensibile ───

    #[test]
    fn target_ureq_radice_e_sottomoduli_sono_sensibili() {
        assert!(target_potenzialmente_sensibile("ureq"));
        assert!(target_potenzialmente_sensibile("ureq::unit"));
        assert!(target_potenzialmente_sensibile("ureq::stream"));
    }

    #[test]
    fn target_applicativi_e_simili_a_ureq_non_sono_sensibili() {
        assert!(!target_potenzialmente_sensibile("pap_lib::editor"));
        assert!(!target_potenzialmente_sensibile("pap_lib::sync"));
        // Non deve bastare che il target CONTENGA "ureq" da qualche parte,
        // né un prefisso solo simile (niente falsi positivi larghi).
        assert!(!target_potenzialmente_sensibile("ureqsomethingelse"));
        assert!(!target_potenzialmente_sensibile("mio_crate::ureq_wrapper"));
    }

    // ─── redigi_valori_header — casi base ───

    #[test]
    fn redige_x_api_key_e_x_goog_api_key() {
        let testo = "x-api-key: sk-ant-segreto123\r\nx-goog-api-key: AIzaSegreto\r\n";
        let out = redigi_valori_header(testo);
        assert!(!out.contains("sk-ant-segreto123"));
        assert!(!out.contains("AIzaSegreto"));
        assert!(out.contains("x-api-key: ***"));
        assert!(out.contains("x-goog-api-key: ***"));
    }

    #[test]
    fn non_tocca_header_non_sensibili() {
        let testo = "content-type: application/json\r\nanthropic-version: 2023-06-01\r\n";
        assert_eq!(redigi_valori_header(testo), testo);
    }

    #[test]
    fn preludio_ureq_realistico_redige_solo_le_chiavi() {
        // Forma esatta prodotta da `PreludeBuilder` (ureq-2.12.1/src/unit.rs):
        // riga di richiesta, header (uno già "mascherato" da ureq con ***,
        // gli altri in chiaro), riga vuota finale.
        let preludio = "POST /v1/messages HTTP/1.1\r\n\
Authorization: ***\r\n\
x-api-key: sk-ant-segreto123\r\n\
anthropic-version: 2023-06-01\r\n\
content-type: application/json\r\n\
\r\n";
        let out = redigi_valori_header(preludio);
        assert!(out.contains("POST /v1/messages HTTP/1.1"));
        assert!(out.contains("anthropic-version: 2023-06-01"));
        assert!(out.contains("content-type: application/json"));
        assert!(!out.contains("sk-ant-segreto123"));
        assert!(out.contains("x-api-key: ***"));
    }

    #[test]
    fn preludio_openrouter_redige_authorization_ma_non_gli_header_di_attribuzione() {
        // OpenRouter (#668) riusa `Authorization: Bearer <key>` come OpenAI —
        // già copertO dal nome header generico `authorization`, nessuna voce
        // dedicata necessaria in HEADER_SENSIBILI. Gli header di attribuzione
        // `HTTP-Referer`/`X-Title` NON sono segreti e non vanno redatti.
        let preludio = "POST /api/v1/chat/completions HTTP/1.1\r\n\
Authorization: Bearer sk-or-v1-segreto\r\n\
HTTP-Referer: https://www.promptaporter.it\r\n\
X-Title: Prompt à Porter\r\n\
content-type: application/json\r\n\
\r\n";
        let out = redigi_valori_header(preludio);
        assert!(!out.contains("sk-or-v1-segreto"));
        assert!(out.contains("Authorization: ***"));
        assert!(out.contains("HTTP-Referer: https://www.promptaporter.it"));
        assert!(out.contains("X-Title: Prompt à Porter"));
    }

    // ─── evasioni richieste esplicitamente ───

    #[test]
    fn evasione_maiuscole_miste_nel_nome_header() {
        let out = redigi_valori_header("X-Api-Key: segreto\r\n");
        assert!(!out.contains("segreto"));
        let out2 = redigi_valori_header("X-GOOG-API-KEY: altrosegreto\r\n");
        assert!(!out2.contains("altrosegreto"));
    }

    #[test]
    fn evasione_spazi_prima_dei_due_punti() {
        let out = redigi_valori_header("x-api-key   : segreto\r\n");
        assert!(!out.contains("segreto"));
    }

    #[test]
    fn evasione_due_header_annidati_sulla_stessa_riga_restano_nascosti() {
        // Un valore che tenta di "impersonare" un secondo header sulla
        // stessa riga fisica finisce comunque dentro il valore redatto
        // del primo (la value stop-condition è \r\n/fine stringa, non un
        // secondo nome di header) — nessuna delle due stringhe segrete
        // sopravvive.
        let testo = "x-api-key: SEGRETO_UNO cookie: SEGRETO_DUE\r\n";
        let out = redigi_valori_header(testo);
        assert!(!out.contains("SEGRETO_UNO"));
        assert!(!out.contains("SEGRETO_DUE"));
    }

    #[test]
    fn due_header_su_righe_reali_consecutive_entrambi_mascherati() {
        let testo = "x-api-key: SEGRETO_UNO\r\ncookie: SEGRETO_DUE\r\ncontent-type: text/plain\r\n";
        let out = redigi_valori_header(testo);
        assert!(!out.contains("SEGRETO_UNO"));
        assert!(!out.contains("SEGRETO_DUE"));
        assert!(out.contains("content-type: text/plain"));
    }

    #[test]
    fn testo_multibyte_non_panica_e_viene_redatto() {
        let testo =
            "x-api-key: chiave-città-é-🔑-segreta\r\ncontent-type: text/plain; charset=utf-è\r\n";
        let out = redigi_valori_header(testo);
        assert!(!out.contains("chiave-città-é-🔑-segreta"));
        assert!(out.contains("x-api-key: ***"));
        // Un header non sensibile con testo multibyte nel VALORE non
        // deve essere toccato né far panicare il taglio delle stringhe.
        assert!(out.contains("charset=utf-è"));
    }

    #[test]
    fn stringa_vuota_non_panica() {
        assert_eq!(redigi_valori_header(""), "");
    }

    // ─── redigi_valori_header — seconda linea di difesa per forma (#591) ───

    #[test]
    fn redige_prefisso_sk_a_meta_riga_senza_nome_header_a_inizio_riga() {
        // Caso 1 dell'issue: il nome dell'header è a METÀ riga, non a
        // inizio riga — la prima linea di difesa (ancorata a `^`) non
        // può riconoscerlo; questa deve.
        let testo = "header inatteso x-api-key: sk-ant-1234567890abcdef\n";
        let out = redigi_valori_header(testo);
        assert!(!out.contains("sk-ant-1234567890abcdef"), "{out}");
        assert!(out.contains("sk-…[REDATTO]"), "{out}");
        assert!(out.contains("header inatteso x-api-key:"), "{out}");
    }

    #[test]
    fn redige_prefisso_aiza_dentro_mappa_debug_formattata() {
        // Caso 2 dell'issue: `{:?}` di una mappa di header — tra virgolette
        // e graffe, non a inizio riga.
        let testo = r#"headers ricevuti: {"x-goog-api-key": "AIzaSyABCDEFGHIJKLMNOP"}"#;
        let out = redigi_valori_header(testo);
        assert!(!out.contains("AIzaSyABCDEFGHIJKLMNOP"), "{out}");
        assert!(out.contains("AIza…[REDATTO]"), "{out}");
        assert!(out.contains(r#""x-goog-api-key":"#), "{out}");
    }

    #[test]
    fn redige_prefisso_bearer_come_chiave_nuda_senza_prefisso_di_header() {
        // Caso 3 dell'issue: nessun nome di header riconosciuto davanti.
        let testo = "credenziale catturata: Bearer abcDEF1234567890.ghIJKL";
        let out = redigi_valori_header(testo);
        assert!(!out.contains("abcDEF1234567890.ghIJKL"), "{out}");
        assert!(out.contains("Bearer …[REDATTO]"), "{out}");
    }

    #[test]
    fn redige_prefisso_bearer_minuscolo_case_insensitive() {
        // RFC 7235 §2.1: lo schema di autenticazione è case-insensitive —
        // "bearer" minuscolo è legale e alcune librerie/dump lo emettono
        // così. Prima della correzione (finding MEDIUM) il pattern
        // richiedeva "Bearer" letterale e questo passava intatto.
        let testo = "auth: bearer abcDEF1234567890.ghIJKL";
        let out = redigi_valori_header(testo);
        assert!(!out.contains("abcDEF1234567890.ghIJKL"), "{out}");
        assert!(out.contains("Bearer …[REDATTO]"), "{out}");
    }

    #[test]
    fn redige_prefisso_bearer_con_doppio_spazio() {
        // Un singolo spazio letterale nel pattern non tollera "Bearer  x"
        // (doppio spazio): il carattere consumato dopo "Bearer " resta uno
        // spazio, non l'inizio del token, e il match fallirebbe.
        let testo = "auth: Bearer  abcDEF1234567890";
        let out = redigi_valori_header(testo);
        assert!(!out.contains("abcDEF1234567890"), "{out}");
        assert!(out.contains("Bearer …[REDATTO]"), "{out}");
    }

    #[test]
    fn redige_token_sk_in_query_string_di_url() {
        // Caso 4 dell'issue: token in query string.
        let testo = "richiesta a https://x.test/webhook?key=sk-liveABCDEFGHIJ&altro=1";
        let out = redigi_valori_header(testo);
        assert!(!out.contains("sk-liveABCDEFGHIJ"), "{out}");
        assert!(out.contains("sk-…[REDATTO]"), "{out}");
        assert!(out.contains("&altro=1"), "{out}");
    }

    #[test]
    fn redige_token_aiza_in_query_string_di_url() {
        let testo = "https://example.com/callback?token=AIzaXXXXXXXXXXXXXXXX&stato=ok";
        let out = redigi_valori_header(testo);
        assert!(!out.contains("AIzaXXXXXXXXXXXXXXXX"), "{out}");
        assert!(out.contains("AIza…[REDATTO]"), "{out}");
        assert!(out.contains("&stato=ok"), "{out}");
    }

    // ─── redigi_valori_header — roundtrip legittimo, niente falsi positivi ───

    #[test]
    fn non_redige_sk_come_sottostringa_di_un_percorso_piu_lungo() {
        // "sk-" compare dentro un percorso, ma preceduto da `-` (stesso
        // alfabeto del token): non è un confine di token, non deve
        // scattare. Roundtrip byte per byte.
        let testo = "diagnostica: /tmp/task-sk-build-1234567890/output.log salvato";
        assert_eq!(redigi_valori_header(testo), testo);
    }

    #[test]
    fn non_redige_hash_esadecimale_che_non_contiene_alcun_prefisso() {
        let testo = "commit a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2 applicato";
        assert_eq!(redigi_valori_header(testo), testo);
    }

    #[test]
    fn non_redige_bearer_parola_semplice_seguita_da_testo_breve() {
        // "Bearer" come parola qualunque, seguita da un token troppo corto
        // per soddisfare la soglia minima (10 caratteri): niente match.
        let testo = "il campo si chiama Bearer test\n";
        assert_eq!(redigi_valori_header(testo), testo);
    }

    #[test]
    fn non_redige_sk_sotto_la_soglia_minima_di_lunghezza() {
        let testo = "chiave: sk-abc123 non valida\n";
        assert_eq!(redigi_valori_header(testo), testo);
    }

    #[test]
    fn non_redige_bearer_come_sottostringa_di_una_parola_italiana() {
        // "bearer" compare come sottostringa di "portatore/bearer" — ma il
        // token che segue è troppo corto (< 10 caratteri) per soddisfare
        // la soglia minima, quindi niente match anche con `(?i)` attivo.
        // Roundtrip byte per byte.
        let testo = "il portatore/bearer di turno consegna il pacco\n";
        assert_eq!(redigi_valori_header(testo), testo);
    }

    // ─── redigi_valori_header — idempotenza sui marker (review PR #637) ───

    #[test]
    fn redazione_e_idempotente_sui_marker() {
        // pap-crash.log viene ri-redatto per intero a ogni export
        // (`debug_log::redigi_testo_per_export`): un marker già prodotto
        // NON deve mai ri-matchare una seconda passata. Un testo con tutti
        // e tre i prefissi (uno per forma) più un header a inizio riga
        // (prima linea di difesa) deve dare lo stesso risultato alla
        // seconda applicazione della prima.
        let testo = "x-api-key: sk-ant-prima-passata-1234567890\n\
                      chiave sk-liveABCDEFGHIJ nel messaggio\n\
                      https://x.test/cb?token=AIzaXXXXXXXXXXXXXXXX\n\
                      auth: Bearer abcDEF1234567890.ghIJKL\n";
        let una_volta = redigi_valori_header(testo);
        let due_volte = redigi_valori_header(&una_volta);
        assert_eq!(
            una_volta, due_volte,
            "una seconda passata non deve alterare l'output già redatto"
        );
    }

    // ─── MAX_LOG_FILE_SIZE_BYTES ───

    #[test]
    fn costante_soglia_crash_log_corrisponde_a_5_mebibyte() {
        assert_eq!(MAX_LOG_FILE_SIZE_BYTES, 5 * 1024 * 1024);
        assert_ne!(MAX_LOG_FILE_SIZE_BYTES, 0);
    }

    // ─── timestamp_bracket_utc ───

    #[test]
    fn timestamp_bracket_epoch_zero() {
        assert_eq!(timestamp_bracket_utc(0), "[1970-01-01][00:00:00]");
    }

    #[test]
    fn timestamp_bracket_data_nota() {
        // 1778540400 secondi epoch = 2026-05-11T23:00:00Z (stesso istante
        // usato nei test di `debug_log::format_iso_utc`, verificato anche
        // lì solo con `starts_with` sulla data).
        assert_eq!(
            timestamp_bracket_utc(1_778_540_400),
            "[2026-05-11][23:00:00]"
        );
    }

    // ─── formatta_riga ───

    #[test]
    fn formatta_riga_target_applicativo_non_redige() {
        let riga = formatta_riga("pap_lib::editor", log::Level::Info, "prompt salvato id=abc");
        assert!(riga.contains("[pap_lib::editor][INFO] prompt salvato id=abc"));
    }

    #[test]
    fn formatta_riga_target_ureq_redige_la_chiave() {
        let riga = formatta_riga(
            "ureq::unit",
            log::Level::Debug,
            "writing prelude: POST / HTTP/1.1\r\nx-api-key: sk-segreto\r\n\r\n",
        );
        assert!(!riga.contains("sk-segreto"));
        assert!(riga.contains("[ureq::unit][DEBUG]"));
        assert!(riga.contains("x-api-key: ***"));
    }

    // ─── redigi_testo_log_storico — righe già scritte su disco ───

    #[test]
    fn storico_redige_riga_singola_con_prefisso() {
        // Il messaggio inizia DIRETTAMENTE con l'header (subito dopo le 4
        // parentesi): caso limite in cui la redazione avviene già sulla
        // prima riga fisica, senza bisogno di righe di continuazione.
        let contenuto = "[2026-08-01][10:00:00][ureq::unit][DEBUG] x-api-key: sk-vecchia\r\n";
        let out = redigi_testo_log_storico(contenuto);
        assert!(!out.contains("sk-vecchia"));
        assert!(out.starts_with("[2026-08-01][10:00:00][ureq::unit][DEBUG]"));
    }

    #[test]
    fn storico_redige_righe_di_continuazione_senza_prefisso() {
        // Come scrive realmente ureq: UN SOLO record (`[data][ora][ureq::unit][DEBUG]`)
        // seguito da righe fisiche di continuazione senza alcun prefisso —
        // il target va ereditato da quello dell'ultima riga con prefisso.
        let contenuto = "[2026-08-01][10:00:00][ureq::unit][DEBUG] POST /v1/messages HTTP/1.1\r\n\
x-api-key: sk-vecchia-continuazione\r\n\
content-type: application/json\r\n\
\r\n\
[2026-08-01][10:00:01][pap_lib::editor][INFO] prompt salvato id=abc\r\n";
        let out = redigi_testo_log_storico(contenuto);
        assert!(!out.contains("sk-vecchia-continuazione"));
        assert!(out.contains("x-api-key: ***"));
        assert!(out.contains("content-type: application/json"));
        // Il record applicativo successivo, con target diverso, resta
        // intatto (il "target corrente" ereditato deve essere aggiornato
        // dal nuovo prefisso, non restare bloccato su `ureq::unit`).
        assert!(out.contains("prompt salvato id=abc"));
    }

    #[test]
    fn storico_non_tocca_log_applicativi_anche_se_contengono_nomi_di_header() {
        // Riproduce l'esempio della trappola #2: un `id` che contiene per
        // coincidenza il testo "cookie:" in un log applicativo (target
        // pap_lib) NON deve perdere la sua coda.
        let contenuto =
            "[2026-08-01][10:00:00][pap_lib::sync][WARN] prompt cookie: scartato per conflitto\r\n";
        let out = redigi_testo_log_storico(contenuto);
        assert_eq!(out, contenuto);
    }

    #[test]
    fn storico_testo_vuoto_non_panica() {
        assert_eq!(redigi_testo_log_storico(""), "");
    }
}
