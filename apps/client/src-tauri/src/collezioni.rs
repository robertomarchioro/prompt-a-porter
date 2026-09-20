//! Collezioni curate — raccolte di prompt mantenute nel repo del progetto
//! (`docs/collezioni/`), scaricate a richiesta e importate nel vault.
//!
//! Decisione di rotta in `docs/roadmap/collezioni-curate.md`. Le regole
//! che contano qui:
//!
//! - **Rete solo su azione esplicita** dell'utente (click su «Sfoglia»),
//!   mai all'apertura delle impostazioni.
//! - **URL costruiti da costanti**: org/repo/branch/cartella sono hardcoded
//!   (`COLLEZIONI_BASE`, stesso schema di `changelog.rs`); l'unico dato
//!   esterno che entra nell'URL è lo `slug`, validato PRIMA
//!   dell'interpolazione (`[a-z0-9-]{1,40}`).
//! - **Integrità**: l'indice porta lo sha256 di ogni file; il file scaricato
//!   viene rifiutato se non corrisponde. Difende da download troncati e
//!   dallo scarto di cache fra indice e file sulla CDN di GitHub, non da un
//!   repo compromesso (stesso livello di fiducia dell'updater, che punta
//!   allo stesso repo).
//! - **Import sempre in modalità `skip`**: una collezione non sovrascrive
//!   mai un prompt, un tag o una cartella dell'utente; re-importarla è
//!   idempotente (tutto conteggiato come conflitto).

use std::io::Read;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;

use crate::errore::PapErrore;
use crate::import_export::{import_pure, ExportV1, ImportReport, SCHEMA_VERSION};
use crate::vault::VaultState;

const COLLEZIONI_BASE: &str =
    "https://raw.githubusercontent.com/robertomarchioro/prompt-a-porter/main/docs/collezioni";
const HTTP_TIMEOUT_CONNECT_SEC: u64 = 5;
const HTTP_TIMEOUT_READ_SEC: u64 = 10;
const HTTP_TIMEOUT_TOTALE_SEC: u64 = 20;
/// L'indice reale è di poche centinaia di byte per collezione.
const MAX_BYTES_INDICE: u64 = 64 * 1024;
/// Una collezione da 13 prompt pesa ~20 KB; 2 MB lascia margine a
/// collezioni molto più ricche senza accumulare risposte anomale.
const MAX_BYTES_COLLEZIONE: u64 = 2 * 1024 * 1024;
const MAX_LEN_SLUG: usize = 40;
const LEN_SHA256_HEX: usize = 64;
const SCHEMA_INDICE: u32 = 1;
/// Cap sulla lunghezza del dettaglio di un errore di parsing nel log.
const MAX_CHARS_LOG_ERRORE: usize = 200;

/// Una voce dell'indice, così come arriva al frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollezioneInfo {
    pub slug: String,
    pub titolo: String,
    pub descrizione: String,
    /// Numero di prompt dichiarato dall'indice (informativo, per la card).
    pub prompt: u32,
    pub sha256: String,
}

/// Voce dell'indice arricchita con lo stato locale: cosa è stato importato
/// in questo vault e con quale impronta. È ciò che riceve la modale.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CollezioneVoce {
    #[serde(flatten)]
    pub info: CollezioneInfo,
    /// Sha256 importato in questo vault, `None` se mai importata.
    pub importata_sha256: Option<String>,
    /// Data dell'ultima importazione o aggiornamento (ISO, UTC).
    pub importata_a: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Indice {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    collezioni: Vec<CollezioneInfo>,
}

/// Lo slug è l'unico dato esterno interpolato in un URL: minuscole, cifre e
/// trattini, lunghezza limitata. Niente `.`, `/` o `%` — nessun modo di
/// uscire dalla cartella `docs/collezioni`.
pub(crate) fn slug_valido(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= MAX_LEN_SLUG
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn sha256_valido(hex: &str) -> bool {
    hex.len() == LEN_SHA256_HEX && hex.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Parsa e valida l'indice: schema supportato, slug e sha256 ben formati,
/// nessuno slug duplicato. Un indice malformato è rifiutato per intero —
/// meglio nessuna collezione che una lista con voci non importabili.
pub(crate) fn parse_indice(testo: &str) -> Result<Vec<CollezioneInfo>, PapErrore> {
    let indice: Indice = serde_json::from_str(testo)
        .map_err(|e| PapErrore::dominio("L'indice delle collezioni non è leggibile.", e))?;
    if indice.schema_version != SCHEMA_INDICE {
        return Err(PapErrore::dominio(
            "L'indice delle collezioni richiede una versione più recente dell'app.",
            format!(
                "schemaVersion indice {} (supportata: {SCHEMA_INDICE})",
                indice.schema_version
            ),
        ));
    }
    let mut visti = std::collections::HashSet::new();
    for c in &indice.collezioni {
        if !slug_valido(&c.slug) {
            return Err(PapErrore::dominio(
                "L'indice delle collezioni contiene una voce non valida.",
                format!("slug rifiutato: {:?}", c.slug.chars().take(MAX_LEN_SLUG).collect::<String>()),
            ));
        }
        if !sha256_valido(&c.sha256) {
            return Err(PapErrore::dominio(
                "L'indice delle collezioni contiene una voce non valida.",
                format!("sha256 malformato per la collezione «{}»", c.slug),
            ));
        }
        if !visti.insert(c.slug.as_str()) {
            return Err(PapErrore::dominio(
                "L'indice delle collezioni contiene una voce non valida.",
                format!("slug duplicato: {}", c.slug),
            ));
        }
    }
    Ok(indice.collezioni)
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Parsa una collezione scaricata come `ExportV1`, rifiutando schemi futuri.
fn deserializza_collezione(slug: &str, json: &str) -> Result<ExportV1, PapErrore> {
    let export: ExportV1 = serde_json::from_str(json).map_err(|e| {
        // `{:?}` + cap: il testo dell'errore serde può citare frammenti del
        // JSON scaricato (dato esterno) — stessa difesa di `changelog.rs`
        // contro righe di log abnormi o con caratteri di controllo.
        let dettaglio: String = e.to_string().chars().take(MAX_CHARS_LOG_ERRORE).collect();
        PapErrore::dominio(
            "La collezione scaricata non è in un formato leggibile.",
            format!("collezione «{slug}»: {dettaglio:?}"),
        )
    })?;
    if export.schema_version > SCHEMA_VERSION {
        return Err(PapErrore::dominio(
            "Questa collezione richiede una versione più recente dell'app.",
            format!(
                "collezione «{slug}»: schemaVersion {} (max {SCHEMA_VERSION})",
                export.schema_version
            ),
        ));
    }
    Ok(export)
}

/// Deserializza una collezione come `ExportV1` e la importa in modalità
/// `skip`. Separata dal download per essere testabile sui file committati.
pub(crate) fn importa_collezione_pura(
    conn: &rusqlite::Connection,
    slug: &str,
    json: &str,
) -> Result<ImportReport, PapErrore> {
    let export = deserializza_collezione(slug, json)?;
    let report = import_pure(conn, &export, "skip")?;
    crate::audit::registra(
        conn,
        "collezione.importata",
        "Collezione",
        slug,
        Some(&format!(
            "nuovi={} conflitti={} errori={}",
            report.nuovi,
            report.conflitti,
            report.errori.len()
        )),
    );
    Ok(report)
}

/// Registra (o rinfresca) l'impronta della collezione importata in questo
/// vault. `ImportataA` resta la prima importazione; `AggiornataA` avanza.
pub(crate) fn registra_importazione(
    conn: &rusqlite::Connection,
    slug: &str,
    sha256: &str,
) -> Result<(), PapErrore> {
    conn.execute(
        "INSERT INTO CollezioniImportate (Slug, Sha256)
         VALUES (?1, ?2)
         ON CONFLICT(Slug) DO UPDATE SET
             Sha256 = excluded.Sha256,
             AggiornataA = datetime('now')",
        rusqlite::params![slug, sha256],
    )?;
    Ok(())
}

/// Unisce l'indice remoto con la tabella locale `CollezioniImportate`.
pub(crate) fn arricchisci(
    conn: &rusqlite::Connection,
    voci: Vec<CollezioneInfo>,
) -> Result<Vec<CollezioneVoce>, PapErrore> {
    let mut stmt =
        conn.prepare("SELECT Sha256, AggiornataA FROM CollezioniImportate WHERE Slug = ?1")?;
    voci.into_iter()
        .map(|info| {
            let locale: Option<(String, String)> = stmt
                .query_row([&info.slug], |r| Ok((r.get(0)?, r.get(1)?)))
                .ok();
            let (importata_sha256, importata_a) = match locale {
                Some((sha, a)) => (Some(sha), Some(a)),
                None => (None, None),
            };
            Ok(CollezioneVoce {
                info,
                importata_sha256,
                importata_a,
            })
        })
        .collect()
}

/// Come `importa_collezione_pura` ma in modalità `aggiorna`: i prompt mai
/// modificati dall'utente vengono riallineati alla collezione, quelli
/// modificati restano intatti (contati in `conflitti`), i nuovi aggiunti.
pub(crate) fn aggiorna_collezione_pura(
    conn: &rusqlite::Connection,
    slug: &str,
    json: &str,
) -> Result<ImportReport, PapErrore> {
    let export = deserializza_collezione(slug, json)?;
    let report = import_pure(conn, &export, "aggiorna")?;
    crate::audit::registra(
        conn,
        "collezione.aggiornata",
        "Collezione",
        slug,
        Some(&format!(
            "nuovi={} aggiornati={} conservati={} errori={}",
            report.nuovi,
            report.aggiornati,
            report.conflitti,
            report.errori.len()
        )),
    );
    Ok(report)
}

fn scarica(url: &str, max_bytes: u64, cosa: &str) -> Result<Vec<u8>, PapErrore> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(HTTP_TIMEOUT_CONNECT_SEC))
        .timeout_read(Duration::from_secs(HTTP_TIMEOUT_READ_SEC))
        // Tetto complessivo: i timeout per-operazione da soli non fermano
        // un peer che gocciola byte (stessa nota di `changelog.rs`).
        .timeout(Duration::from_secs(HTTP_TIMEOUT_TOTALE_SEC))
        .build();
    let resp = agent.get(url).call().map_err(|e| {
        PapErrore::dominio(
            format!("Download {cosa} non riuscito. Verifica la connessione."),
            e,
        )
    })?;
    let mut bytes = Vec::new();
    resp.into_reader()
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| PapErrore::dominio(format!("Lettura {cosa} non riuscita."), e))?;
    if bytes.len() as u64 > max_bytes {
        return Err(PapErrore::dominio(
            format!("Download {cosa} rifiutato: risposta troppo grande."),
            format!("{url}: oltre {max_bytes} byte"),
        ));
    }
    Ok(bytes)
}

fn scarica_indice() -> Result<Vec<CollezioneInfo>, PapErrore> {
    let bytes = scarica(
        &format!("{COLLEZIONI_BASE}/indice.json"),
        MAX_BYTES_INDICE,
        "dell'indice delle collezioni",
    )?;
    let testo = String::from_utf8(bytes)
        .map_err(|e| PapErrore::dominio("L'indice delle collezioni non è leggibile.", e))?;
    parse_indice(&testo)
}

/// Cmd Tauri: elenca le collezioni disponibili, con lo stato locale
/// (importata? con quale impronta?). Chiamato solo su azione esplicita
/// dell'utente; `async` perché il download non deve bloccare il thread
/// principale della webview. La rete viene prima del lock sul vault.
#[tauri::command(async)]
pub fn collezioni_elenca(
    state: State<'_, VaultState>,
) -> Result<Vec<CollezioneVoce>, PapErrore> {
    let voci = scarica_indice()?;
    state.with_conn(|conn| arricchisci(conn, voci))
}

/// Scarica `<slug>.json` e ne verifica lo sha256 contro l'indice
/// (riscaricato: stateless e minuscolo). Ritorna il JSON e l'impronta.
fn scarica_collezione_verificata(slug: &str) -> Result<(String, String), PapErrore> {
    if !slug_valido(slug) {
        return Err(PapErrore::dominio(
            "Identificativo della collezione non valido.",
            format!(
                "slug rifiutato: {:?}",
                slug.chars().take(MAX_LEN_SLUG).collect::<String>()
            ),
        ));
    }
    let indice = scarica_indice()?;
    let voce = indice.iter().find(|c| c.slug == slug).ok_or_else(|| {
        PapErrore::dominio(
            "Collezione non più disponibile. Aggiorna l'elenco.",
            format!("slug «{slug}» assente dall'indice"),
        )
    })?;

    let bytes = scarica(
        &format!("{COLLEZIONI_BASE}/{slug}.json"),
        MAX_BYTES_COLLEZIONE,
        "della collezione",
    )?;
    let calcolato = sha256_hex(&bytes);
    if calcolato != voce.sha256 {
        return Err(PapErrore::dominio(
            "Il file della collezione non corrisponde all'indice. Riprova fra qualche minuto.",
            format!("sha256 «{slug}»: atteso {} ottenuto {calcolato}", voce.sha256),
        ));
    }
    let json = String::from_utf8(bytes)
        .map_err(|e| PapErrore::dominio("La collezione scaricata non è leggibile.", e))?;
    Ok((json, calcolato))
}

/// Applica la collezione (import o aggiornamento) e registra l'impronta
/// SOLO se nessun elemento è fallito: con errori parziali la collezione
/// resta «da importare»/«aggiornabile», così l'utente può riprovare
/// (review PR passo 2: prima l'impronta veniva registrata comunque e il
/// bottone si disabilitava senza via di ritorno).
pub(crate) fn applica_e_registra(
    conn: &rusqlite::Connection,
    slug: &str,
    json: &str,
    sha256: &str,
    aggiorna: bool,
) -> Result<ImportReport, PapErrore> {
    let report = if aggiorna {
        aggiorna_collezione_pura(conn, slug, json)?
    } else {
        importa_collezione_pura(conn, slug, json)?
    };
    if report.errori.is_empty() {
        registra_importazione(conn, slug, sha256)?;
    } else {
        log::warn!(
            "collezione «{slug}»: {} errori, impronta non registrata",
            report.errori.len()
        );
    }
    Ok(report)
}

/// Cmd Tauri: scarica la collezione `slug` (verificata) e la importa in
/// modalità `skip`, poi registra l'impronta importata.
#[tauri::command(async)]
pub fn collezioni_importa(
    slug: String,
    state: State<'_, VaultState>,
) -> Result<ImportReport, PapErrore> {
    let (json, sha) = scarica_collezione_verificata(&slug)?;
    state.with_conn(|conn| applica_e_registra(conn, &slug, &json, &sha, false))
}

/// Cmd Tauri: scarica la collezione `slug` (verificata) e la applica in
/// modalità `aggiorna` — riallinea i prompt mai modificati, conserva quelli
/// modificati, aggiunge i nuovi — poi registra la nuova impronta.
#[tauri::command(async)]
pub fn collezioni_aggiorna(
    slug: String,
    state: State<'_, VaultState>,
) -> Result<ImportReport, PapErrore> {
    let (json, sha) = scarica_collezione_verificata(&slug)?;
    state.with_conn(|conn| applica_e_registra(conn, &slug, &json, &sha, true))
}

#[cfg(test)]
mod test {
    use super::*;
    use rusqlite::Connection;

    fn db_test() -> Connection {
        crate::embeddings_store::registra_auto_extension();
        let conn = Connection::open_in_memory().unwrap();
        crate::migrazione::esegui_migrazioni(&conn).unwrap();
        crate::libreria::assicura_dati_base(&conn).unwrap();
        conn
    }

    const INDICE_COMMITTATO: &str = include_str!("../../../../docs/collezioni/indice.json");
    const COLLEZIONI_COMMITTATE: &[(&str, &str)] = &[
        (
            "sviluppatore",
            include_str!("../../../../docs/collezioni/sviluppatore.json"),
        ),
        (
            "scrittura",
            include_str!("../../../../docs/collezioni/scrittura.json"),
        ),
    ];

    // ─────────── validazione slug e indice ───────────

    #[test]
    fn slug_valido_accetta_solo_minuscole_cifre_trattini() {
        assert!(slug_valido("sviluppatore"));
        assert!(slug_valido("analisi-dati-2"));
        assert!(!slug_valido(""));
        assert!(!slug_valido("Sviluppatore"));
        assert!(!slug_valido("../indice"));
        assert!(!slug_valido("a/b"));
        assert!(!slug_valido("a.json"));
        assert!(!slug_valido("a%2e%2e"));
        assert!(!slug_valido(&"a".repeat(MAX_LEN_SLUG + 1)));
        assert!(slug_valido(&"a".repeat(MAX_LEN_SLUG)));
    }

    #[test]
    fn parse_indice_legge_indice_committato() {
        let voci = parse_indice(INDICE_COMMITTATO).unwrap();
        assert_eq!(voci.len(), COLLEZIONI_COMMITTATE.len());
        for v in &voci {
            assert!(!v.titolo.is_empty(), "titolo vuoto per «{}»", v.slug);
            assert!(!v.descrizione.is_empty(), "descrizione vuota per «{}»", v.slug);
            assert!(v.prompt > 0);
        }
    }

    #[test]
    fn parse_indice_rifiuta_schema_futuro() {
        let err = parse_indice(r#"{"schemaVersion": 2, "collezioni": []}"#).unwrap_err();
        assert!(err.to_string().contains("versione più recente"));
    }

    #[test]
    fn parse_indice_rifiuta_slug_non_valido() {
        let json = format!(
            r#"{{"schemaVersion":1,"collezioni":[{{"slug":"../x","titolo":"t","descrizione":"d","prompt":1,"sha256":"{}"}}]}}"#,
            "0".repeat(64)
        );
        assert!(parse_indice(&json).is_err());
    }

    #[test]
    fn parse_indice_rifiuta_sha256_malformato_e_duplicati() {
        let corto = r#"{"schemaVersion":1,"collezioni":[{"slug":"a","titolo":"t","descrizione":"d","prompt":1,"sha256":"abc"}]}"#;
        assert!(parse_indice(corto).is_err());
        let dup = format!(
            r#"{{"schemaVersion":1,"collezioni":[{{"slug":"a","titolo":"t","descrizione":"d","prompt":1,"sha256":"{0}"}},{{"slug":"a","titolo":"t","descrizione":"d","prompt":1,"sha256":"{0}"}}]}}"#,
            "0".repeat(64)
        );
        assert!(parse_indice(&dup).is_err());
    }

    #[test]
    fn parse_indice_accetta_voce_ben_formata() {
        let json = format!(
            r#"{{"schemaVersion":1,"collezioni":[{{"slug":"a-1","titolo":"t","descrizione":"d","prompt":3,"sha256":"{}"}}]}}"#,
            "ab".repeat(32)
        );
        let voci = parse_indice(&json).unwrap();
        assert_eq!(voci[0].slug, "a-1");
        assert_eq!(voci[0].prompt, 3);
    }

    // ─────────── integrità dei file committati ───────────

    /// Guardia contro l'indice stale: ogni sha256 e conteggio dichiarato
    /// nell'indice deve corrispondere al file committato. Se rompe, esegui
    /// `node scripts/collezioni-indice.mjs`.
    #[test]
    fn indice_committato_corrisponde_ai_file() {
        let voci = parse_indice(INDICE_COMMITTATO).unwrap();
        for (slug, json) in COLLEZIONI_COMMITTATE {
            let voce = voci
                .iter()
                .find(|v| v.slug == *slug)
                .unwrap_or_else(|| panic!("collezione «{slug}» assente dall'indice"));
            assert_eq!(
                voce.sha256,
                sha256_hex(json.as_bytes()),
                "sha256 stale per «{slug}»: esegui `node scripts/collezioni-indice.mjs`"
            );
            let export: ExportV1 = serde_json::from_str(json).unwrap();
            assert_eq!(voce.prompt as usize, export.prompts.len(), "conteggio stale per «{slug}»");
        }
    }

    #[test]
    fn sha256_hex_valore_noto() {
        // sha256("abc")
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    // ─────────── import delle collezioni committate ───────────

    /// Ogni collezione committata deve importarsi pulita in un vault vuoto,
    /// e la sua re-importazione deve essere idempotente (solo conflitti).
    #[test]
    fn collezioni_committate_importano_pulite_e_idempotenti() {
        for (slug, json) in COLLEZIONI_COMMITTATE {
            let conn = db_test();
            let export: ExportV1 = serde_json::from_str(json).unwrap();
            let report = importa_collezione_pura(&conn, slug, json).unwrap();
            assert!(report.errori.is_empty(), "«{slug}»: {:?}", report.errori);
            assert_eq!(report.conflitti, 0, "«{slug}»: conflitti su vault vuoto");
            let n_prompt: i64 = conn
                .query_row("SELECT COUNT(*) FROM Prompts", [], |r| r.get(0))
                .unwrap();
            assert_eq!(n_prompt as usize, export.prompts.len(), "«{slug}»");

            let secondo = importa_collezione_pura(&conn, slug, json).unwrap();
            assert!(secondo.errori.is_empty(), "«{slug}» re-import: {:?}", secondo.errori);
            assert_eq!(secondo.nuovi, 0, "«{slug}» re-import ha creato prompt");
            let n_prompt_dopo: i64 = conn
                .query_row("SELECT COUNT(*) FROM Prompts", [], |r| r.get(0))
                .unwrap();
            assert_eq!(n_prompt_dopo, n_prompt, "«{slug}» re-import ha duplicato");
        }
    }

    /// Tutte le collezioni insieme, nell'ordine dell'indice, sopra il demo
    /// vault: la cartella radice `Collezioni` e i tag omonimi si condividono
    /// senza errori, e nessun prompt finisce a root per cartella mancante.
    #[test]
    fn collezioni_convivono_con_demo_vault_e_fra_loro() {
        let conn = db_test();
        let demo = include_str!("../../../../docs/demo/demo-vault.json");
        let demo_export: ExportV1 = serde_json::from_str(demo).unwrap();
        import_pure(&conn, &demo_export, "skip").unwrap();

        let mut attesi = demo_export.prompts.len();
        for (slug, json) in COLLEZIONI_COMMITTATE {
            let report = importa_collezione_pura(&conn, slug, json).unwrap();
            assert!(report.errori.is_empty(), "«{slug}» sopra demo: {:?}", report.errori);
            attesi += serde_json::from_str::<ExportV1>(json).unwrap().prompts.len();
        }
        let n_prompt: i64 = conn
            .query_row("SELECT COUNT(*) FROM Prompts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n_prompt as usize, attesi);
        let orfani: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Prompts WHERE Id LIKE 'prm-col-%' AND FolderId IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(orfani, 0, "prompt di collezione senza cartella");
    }

    /// Ogni `{{import "..."}}` delle collezioni deve risolvere a un prompt
    /// della stessa collezione: un import rotto sarebbe un IMP001 servito
    /// all'utente come prima impressione.
    #[test]
    fn import_interni_delle_collezioni_risolvono() {
        for (slug, json) in COLLEZIONI_COMMITTATE {
            let conn = db_test();
            importa_collezione_pura(&conn, slug, json).unwrap();
            let export: ExportV1 = serde_json::from_str(json).unwrap();
            for p in &export.prompts {
                for rif in crate::prompt_componibili::parse_imports(&p.body) {
                    let id = crate::prompt_componibili::resolve_path(&conn, &rif.path).unwrap();
                    assert!(
                        id.is_some(),
                        "«{slug}» / «{}»: import «{}» non risolve",
                        p.title,
                        rif.path
                    );
                }
            }
        }
    }

    // ─────────── modalità aggiorna + tracciamento ───────────

    /// Prende la collezione Sviluppatore e ne produce una "versione 2":
    /// corpo di un prompt cambiato, un prompt nuovo in coda.
    fn sviluppatore_v2() -> (String, &'static str, &'static str) {
        let mut export: serde_json::Value =
            serde_json::from_str(COLLEZIONI_COMMITTATE[0].1).unwrap();
        let prompts = export["prompts"].as_array_mut().unwrap();
        let id_modificato = "prm-col-dev-commit";
        let p = prompts
            .iter_mut()
            .find(|p| p["id"] == id_modificato)
            .unwrap();
        p["body"] = serde_json::Value::String("CORPO NUOVO {{diff}}".into());
        let mut nuovo = prompts[0].clone();
        nuovo["id"] = serde_json::Value::String("prm-col-dev-nuovo-v2".into());
        nuovo["title"] = serde_json::Value::String("Prompt nuovo della v2".into());
        prompts.push(nuovo);
        (export.to_string(), id_modificato, "prm-col-dev-nuovo-v2")
    }

    fn body_di(conn: &Connection, id: &str) -> String {
        conn.query_row("SELECT Body FROM Prompts WHERE Id = ?1", [id], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn aggiorna_riallinea_i_mai_modificati_e_aggiunge_i_nuovi() {
        // Arrange: v1 importata, l'utente sposta il prompt in un'altra
        // cartella e lo mette fra i preferiti (organizzazione sua).
        let conn = db_test();
        importa_collezione_pura(&conn, "sviluppatore", COLLEZIONI_COMMITTATE[0].1).unwrap();
        let (v2, id_mod, id_nuovo) = sviluppatore_v2();
        conn.execute(
            "UPDATE Prompts SET FolderId = NULL, IsFavorite = 1, UseCount = 7 WHERE Id = ?1",
            [id_mod],
        )
        .unwrap();

        // Act
        let report = aggiorna_collezione_pura(&conn, "sviluppatore", &v2).unwrap();

        // Assert: 1 aggiornato, 1 nuovo, nessun conservato, nessun errore.
        assert!(report.errori.is_empty(), "{:?}", report.errori);
        assert_eq!(report.aggiornati, 1);
        assert_eq!(report.nuovi, 1);
        assert_eq!(report.conflitti, 0, "cartelle/tag esistenti non sono conflitti in aggiorna");
        assert_eq!(body_di(&conn, id_mod), "CORPO NUOVO {{diff}}");
        assert!(!body_di(&conn, id_nuovo).is_empty());
        // L'organizzazione dell'utente resta sua; Version resta 1.
        let (folder, fav, usi, ver): (Option<String>, bool, i64, i64) = conn
            .query_row(
                "SELECT FolderId, IsFavorite, UseCount, Version FROM Prompts WHERE Id = ?1",
                [id_mod],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(folder, None);
        assert!(fav);
        assert_eq!(usi, 7);
        assert_eq!(ver, 1);
    }

    #[test]
    fn aggiorna_conserva_i_prompt_modificati_dall_utente() {
        // Arrange: l'utente ha salvato il prompt (Version 2, corpo suo).
        let conn = db_test();
        importa_collezione_pura(&conn, "sviluppatore", COLLEZIONI_COMMITTATE[0].1).unwrap();
        let (v2, id_mod, _) = sviluppatore_v2();
        conn.execute(
            "UPDATE Prompts SET Body = 'corpo dell utente', Version = 2 WHERE Id = ?1",
            [id_mod],
        )
        .unwrap();

        // Act
        let report = aggiorna_collezione_pura(&conn, "sviluppatore", &v2).unwrap();

        // Assert
        assert_eq!(report.aggiornati, 0);
        assert_eq!(report.conflitti, 1, "il prompt modificato va contato come conservato");
        assert_eq!(body_di(&conn, id_mod), "corpo dell utente");
    }

    #[test]
    fn aggiorna_e_idempotente_e_non_resuscita_dal_cestino() {
        let conn = db_test();
        importa_collezione_pura(&conn, "sviluppatore", COLLEZIONI_COMMITTATE[0].1).unwrap();
        let (v2, id_mod, _) = sviluppatore_v2();
        conn.execute(
            "UPDATE Prompts SET DeletedAt = datetime('now') WHERE Id = 'prm-col-dev-sql'",
            [],
        )
        .unwrap();

        let primo = aggiorna_collezione_pura(&conn, "sviluppatore", &v2).unwrap();
        let secondo = aggiorna_collezione_pura(&conn, "sviluppatore", &v2).unwrap();

        assert_eq!(primo.aggiornati, 1);
        assert_eq!((secondo.nuovi, secondo.aggiornati, secondo.conflitti), (0, 0, 0));
        assert!(secondo.errori.is_empty());
        assert_eq!(body_di(&conn, id_mod), "CORPO NUOVO {{diff}}");
        let cestinato: bool = conn
            .query_row(
                "SELECT DeletedAt IS NOT NULL FROM Prompts WHERE Id = 'prm-col-dev-sql'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(cestinato, "aggiorna non deve resuscitare un prompt cestinato");
    }

    /// La struttura (variante/parent) si fissa alla prima importazione:
    /// una v2 che demota la variante non deve lasciare IsVariant senza
    /// parent né toccare la relazione.
    #[test]
    fn aggiorna_non_tocca_la_struttura_delle_varianti() {
        let conn = db_test();
        importa_collezione_pura(&conn, "sviluppatore", COLLEZIONI_COMMITTATE[0].1).unwrap();
        let mut export: serde_json::Value =
            serde_json::from_str(COLLEZIONI_COMMITTATE[0].1).unwrap();
        let p = export["prompts"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|p| p["id"] == "prm-col-dev-refactoring-b")
            .unwrap();
        p["is_variant"] = serde_json::Value::Bool(false);
        p["parent_prompt_id"] = serde_json::Value::Null;
        p["body"] = serde_json::Value::String("corpo v2".into());

        let report = aggiorna_collezione_pura(&conn, "sviluppatore", &export.to_string()).unwrap();

        assert_eq!(report.aggiornati, 1);
        let (is_variant, parent): (bool, Option<String>) = conn
            .query_row(
                "SELECT IsVariant, ParentPromptId FROM Prompts WHERE Id = 'prm-col-dev-refactoring-b'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(is_variant, "IsVariant non deve cambiare in aggiorna");
        assert_eq!(parent.as_deref(), Some("prm-col-dev-refactoring"));
        assert_eq!(body_di(&conn, "prm-col-dev-refactoring-b"), "corpo v2");
    }

    /// Un tag omonimo dell'utente riusato non è un «conservato» in aggiorna.
    #[test]
    fn aggiorna_non_conta_i_tag_omonimi_come_conservati() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, Color, CreatedAt, UpdatedAt)
             VALUES ('tag-mio', 'ws-personale', 'codice', '#000', '2026-01-01', '2026-01-01')",
            [],
        )
        .unwrap();
        let report =
            aggiorna_collezione_pura(&conn, "sviluppatore", COLLEZIONI_COMMITTATE[0].1).unwrap();
        assert!(report.errori.is_empty(), "{:?}", report.errori);
        assert_eq!(report.conflitti, 0);
        assert_eq!(report.nuovi as usize, 13 + 6 + 6, "13 prompt, 6 cartelle, 6 tag nuovi");
    }

    /// Una cartella omonima dell'utente riusata non è un «conservato» in
    /// aggiorna (#666, stesso schema del test sui tag qui sopra): la radice
    /// «Collezioni» già presente si riusa, e la cartella figlia della
    /// collezione finisce sotto quella riusata, non sotto una seconda radice.
    #[test]
    fn aggiorna_non_conta_le_cartelle_omonime_come_conservate() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path, CreatedAt, UpdatedAt)
             VALUES ('fld-mia', 'ws-personale', NULL, 'Collezioni', '/Collezioni', '2026-01-01', '2026-01-01')",
            [],
        )
        .unwrap();

        let report =
            aggiorna_collezione_pura(&conn, "sviluppatore", COLLEZIONI_COMMITTATE[0].1).unwrap();

        assert!(report.errori.is_empty(), "{:?}", report.errori);
        assert_eq!(report.conflitti, 0, "una cartella omonima riusata non è un «conservato»");
        assert_eq!(
            report.nuovi as usize,
            13 + 5 + 7,
            "13 prompt, 5 cartelle nuove (la radice «Collezioni» si riusa), 7 tag nuovi"
        );

        let radici_collezioni: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Folders WHERE ParentFolderId IS NULL AND Name = 'Collezioni'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(radici_collezioni, 1, "non deve nascere una seconda radice «Collezioni»");
        let figlie_sotto_mia: i64 = conn
            .query_row("SELECT COUNT(*) FROM Folders WHERE ParentFolderId = 'fld-mia'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(figlie_sotto_mia, 1, "«Sviluppatore» deve finire sotto la cartella riusata");
    }

    /// Con errori parziali l'impronta NON va registrata: la collezione deve
    /// restare importabile/aggiornabile per riprovare.
    ///
    /// L'errore qui non arriva più da due cartelle omonime con id diversi
    /// (dedup #666, ora gestito senza errori — vedi i test sopra e in
    /// `import_export.rs`), ma da un tag omonimo di uno GIÀ CESTINATO: il
    /// vincolo `UNIQUE (WorkspaceId, Name)` di `Tags`, a differenza di
    /// quello delle cartelle, non è filtrato su `DeletedAt IS NULL`, quindi
    /// resta un modo genuino (e indipendente da questa issue) per produrre
    /// un errore parziale.
    #[test]
    fn applica_e_registra_non_registra_con_errori_parziali() {
        let conn = db_test();
        conn.execute(
            "INSERT INTO Tags (Id, WorkspaceId, Name, Color, CreatedAt, UpdatedAt, DeletedAt)
             VALUES ('tag-cestinato', 'ws-personale', 'Doppia', '#000', '2026-01-01', '2026-01-01',
                     datetime('now'))",
            [],
        )
        .unwrap();
        let json = r#"{"schemaVersion":1,"exportedAt":"2026-01-01T00:00:00Z",
            "workspace":{"id":"ws-personale","name":"Personale","type":"personal"},
            "tags":[{"id":"tag-nuovo","name":"Doppia","color":null,"created_at":"2026-01-01T00:00:00Z"}],
            "prompts":[],"versions":[],"global_placeholders":[],"folders":[]}"#;
        let sha = "1".repeat(64);

        let report = applica_e_registra(&conn, "rotta", json, &sha, false).unwrap();
        assert_eq!(report.errori.len(), 1, "{:?}", report.errori);
        let righe: i64 = conn
            .query_row("SELECT COUNT(*) FROM CollezioniImportate", [], |r| r.get(0))
            .unwrap();
        assert_eq!(righe, 0, "impronta registrata nonostante gli errori");

        // Senza errori, invece, registra (import e aggiorna).
        let ok = applica_e_registra(
            &conn,
            "sviluppatore",
            COLLEZIONI_COMMITTATE[0].1,
            &sha,
            false,
        )
        .unwrap();
        assert!(ok.errori.is_empty());
        let ok2 = applica_e_registra(
            &conn,
            "sviluppatore",
            COLLEZIONI_COMMITTATE[0].1,
            &"2".repeat(64),
            true,
        )
        .unwrap();
        assert!(ok2.errori.is_empty());
        let sha_locale: String = conn
            .query_row(
                "SELECT Sha256 FROM CollezioniImportate WHERE Slug = 'sviluppatore'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(sha_locale, "2".repeat(64));
    }

    #[test]
    fn registra_importazione_fa_upsert_e_arricchisci_riporta_lo_stato() {
        let conn = db_test();
        let voci = parse_indice(INDICE_COMMITTATO).unwrap();

        // Mai importata: nessuno stato locale.
        let prima = arricchisci(&conn, voci.clone()).unwrap();
        assert!(prima.iter().all(|v| v.importata_sha256.is_none()));

        // Importata con un'impronta vecchia → arricchisci la riporta.
        registra_importazione(&conn, "sviluppatore", &"0".repeat(64)).unwrap();
        let dopo = arricchisci(&conn, voci.clone()).unwrap();
        let dev = dopo.iter().find(|v| v.info.slug == "sviluppatore").unwrap();
        assert_eq!(dev.importata_sha256.as_deref(), Some("0".repeat(64).as_str()));
        assert!(dev.importata_a.is_some());
        let scr = dopo.iter().find(|v| v.info.slug == "scrittura").unwrap();
        assert!(scr.importata_sha256.is_none());

        // Upsert: la seconda registrazione sostituisce l'impronta, una riga sola.
        registra_importazione(&conn, "sviluppatore", &dev.info.sha256).unwrap();
        let righe: i64 = conn
            .query_row("SELECT COUNT(*) FROM CollezioniImportate", [], |r| r.get(0))
            .unwrap();
        assert_eq!(righe, 1);
        let dopo2 = arricchisci(&conn, voci).unwrap();
        let dev2 = dopo2.iter().find(|v| v.info.slug == "sviluppatore").unwrap();
        assert_eq!(dev2.importata_sha256.as_deref(), Some(dev2.info.sha256.as_str()));
    }

    #[test]
    fn importa_collezione_pura_rifiuta_json_non_valido() {
        let conn = db_test();
        let err = importa_collezione_pura(&conn, "x", "{not json").unwrap_err();
        assert!(err.to_string().contains("formato leggibile"));
    }
}
