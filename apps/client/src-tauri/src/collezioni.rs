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

/// Deserializza una collezione come `ExportV1` e la importa in modalità
/// `skip`. Separata dal download per essere testabile sui file committati.
pub(crate) fn importa_collezione_pura(
    conn: &rusqlite::Connection,
    slug: &str,
    json: &str,
) -> Result<ImportReport, PapErrore> {
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

/// Cmd Tauri: elenca le collezioni disponibili. Chiamato solo su azione
/// esplicita dell'utente; `async` perché il download non deve bloccare il
/// thread principale della webview.
#[tauri::command(async)]
pub fn collezioni_elenca() -> Result<Vec<CollezioneInfo>, PapErrore> {
    scarica_indice()
}

/// Cmd Tauri: scarica la collezione `slug`, ne verifica lo sha256 contro
/// l'indice (riscaricato: il comando è stateless e l'indice è minuscolo) e
/// la importa in modalità `skip`.
#[tauri::command(async)]
pub fn collezioni_importa(
    slug: String,
    state: State<'_, VaultState>,
) -> Result<ImportReport, PapErrore> {
    if !slug_valido(&slug) {
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

    state.with_conn(|conn| importa_collezione_pura(conn, &slug, &json))
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

    #[test]
    fn importa_collezione_pura_rifiuta_json_non_valido() {
        let conn = db_test();
        let err = importa_collezione_pura(&conn, "x", "{not json").unwrap_err();
        assert!(err.to_string().contains("formato leggibile"));
    }
}
