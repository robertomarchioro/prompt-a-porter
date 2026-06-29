use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errore::PapErrore;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preferenze {
    pub profilo: String,
    pub hotkey: String,
    pub tema: String,
    pub tono: String,
    pub lingua: String,
    pub onboarding_completato: bool,
    pub crea_prompt_esempio: bool,
    #[serde(default)]
    pub sync_server_url: String,
    #[serde(default)]
    pub sync_email: String,
    #[serde(default)]
    pub sync_token: String,
    #[serde(default = "default_sync_intervallo")]
    pub sync_intervallo_sec: u32,
    #[serde(default)]
    pub sync_abilitato: bool,
    /// Ricerca semantica abilitata (Fase 3): triggera download modello +
    /// init Session al boot del client se true.
    #[serde(default)]
    pub ricerca_semantica_abilitata: bool,
    /// Peso semantico nella ricerca ibrida (alpha ∈ [0,1]).
    /// 0 = solo FTS5 lessicale, 1 = solo vec0 semantico, 0.5 = bilanciato.
    #[serde(default = "default_ricerca_alpha")]
    pub ricerca_alpha: f64,
    /// Step 10: dopo quanti secondi di inattività la Session ort viene
    /// droppata per liberare RAM (~150 MB modello + runtime).
    /// `0` = disattivato (Session resta caricata fino a chiusura app).
    /// Valori ammessi: 0 oppure ≥ 60. Sotto 60 viene trattato come 0.
    /// Default 300s = 5 minuti.
    #[serde(default = "default_idle_unload_secondi")]
    pub idle_unload_secondi: u32,
    /// v0.8.7 Sezione Sviluppo → Debug log. Quando true, il logger
    /// (tauri-plugin-log) opera a livello DEBUG e scrive su file in
    /// LogDir; quando false, opera a livello WARN (solo errori).
    /// Il toggle ON/OFF è esposto in Impostazioni → Sviluppo. Default
    /// false per non sporcare disco utenti standard.
    #[serde(default)]
    pub debug_log_abilitato: bool,
    /// v1.0 M1.4b Sezione Sviluppo → Aggiornamenti. Quando true, l'app
    /// mostra il bottone "Verifica aggiornamenti" in Impostazioni →
    /// Sviluppo; quando false, il check è disabilitato e l'app non
    /// contatta GitHub Releases per gli update meta. Default true:
    /// l'utente decide quando controllare, ma il check è disponibile.
    /// Vedi docs/utente/auto-update.md §"Come disabilitare il check".
    #[serde(default = "default_updater_abilitato")]
    pub updater_abilitato: bool,
    /// M10 — Editor: ritardo autosave in millisecondi dopo l'ultima
    /// modifica del body. Range UI 500-5000ms (clampato lato Svelte).
    /// Default 2000ms = 2s (storico hardcoded fino a v0.8.11).
    #[serde(default = "default_autosave_delay_ms")]
    pub autosave_delay_ms: u32,
    /// M10 — Editor: line wrapping (soft wrap) CodeMirror. Default true
    /// (piu' adatto a markdown e prompt body). Quando false, lo scroll
    /// orizzontale e' attivo.
    #[serde(default = "default_line_wrapping")]
    pub line_wrapping: bool,
    /// M10 — Editor: numero di spazi per livello di indent CodeMirror.
    /// Valori ammessi: 2 o 4 (preferenza personale). Default 2.
    #[serde(default = "default_indent_size")]
    pub indent_size: u8,
    /// M10 — Editor: dimensione font in pixel. Range UI 12-20.
    /// Default 13 (coerente con il valore CSS storico).
    #[serde(default = "default_font_size")]
    pub font_size: u8,
    /// M10 — Editor: mostra i numeri di riga nel gutter sinistro.
    /// Default true.
    #[serde(default = "default_show_line_numbers")]
    pub show_line_numbers: bool,
    /// M10 — Editor: evidenzia la riga sotto il cursore. Default false
    /// (riduce rumore visivo su body brevi).
    #[serde(default)]
    pub highlight_active_line: bool,
    /// #404: nome del vault scelto dall'utente in onboarding, mostrato
    /// nello switcher in alto a sinistra (al posto del fisso "Personale").
    /// Default "Personale" per retro-compatibilità coi file pre-#404.
    #[serde(default = "default_nome_vault")]
    pub nome_vault: String,
}

fn default_nome_vault() -> String {
    "Personale".to_string()
}

fn default_updater_abilitato() -> bool {
    true
}

fn default_sync_intervallo() -> u32 {
    60
}

fn default_ricerca_alpha() -> f64 {
    0.5
}

fn default_idle_unload_secondi() -> u32 {
    300
}

fn default_autosave_delay_ms() -> u32 {
    2000
}

fn default_line_wrapping() -> bool {
    true
}

fn default_indent_size() -> u8 {
    2
}

fn default_font_size() -> u8 {
    13
}

fn default_show_line_numbers() -> bool {
    true
}

impl Default for Preferenze {
    fn default() -> Self {
        Self {
            profilo: "personale".to_string(),
            hotkey: "Ctrl+Shift+P".to_string(),
            tema: "dark".to_string(),
            tono: "zinc".to_string(),
            lingua: "it".to_string(),
            onboarding_completato: false,
            crea_prompt_esempio: true,
            sync_server_url: String::new(),
            sync_email: String::new(),
            sync_token: String::new(),
            sync_intervallo_sec: 60,
            sync_abilitato: false,
            ricerca_semantica_abilitata: false,
            ricerca_alpha: 0.5,
            idle_unload_secondi: 300,
            debug_log_abilitato: false,
            updater_abilitato: true,
            autosave_delay_ms: 2000,
            line_wrapping: true,
            indent_size: 2,
            font_size: 13,
            show_line_numbers: true,
            highlight_active_line: false,
            nome_vault: "Personale".to_string(),
        }
    }
}

pub struct PreferenzeState {
    data_dir: PathBuf,
}

impl PreferenzeState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    fn file_path(&self) -> PathBuf {
        self.data_dir.join("preferenze.json")
    }
}

pub fn carica_pure(path: &Path) -> Result<Preferenze, PapErrore> {
    if !path.exists() {
        return Ok(Preferenze::default());
    }
    let json = fs::read_to_string(path)?;
    let prefs: Preferenze = serde_json::from_str(&json)?;
    Ok(prefs)
}

pub fn salva_pure(data_dir: &Path, preferenze: &Preferenze) -> Result<(), PapErrore> {
    fs::create_dir_all(data_dir)?;
    let json = serde_json::to_string_pretty(preferenze)?;
    let path = data_dir.join("preferenze.json");
    fs::write(&path, json)?;
    // SECURITY: preferenze.json contiene `sync_token` in chiaro. Su Unix
    // restringiamo i permessi a 0600 (solo owner) per ridurre l'esposizione
    // su sistemi multi-utente. Su Windows l'AppData è già per-utente.
    // TODO: spostare i segreti nel keychain OS (crate `keyring`).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = fs::set_permissions(&path, fs::Permissions::from_mode(0o600)) {
            log::warn!("set_permissions 0600 su preferenze.json fallito: {e}");
        }
    }
    Ok(())
}

#[tauri::command]
pub fn preferenze_carica(state: State<'_, PreferenzeState>) -> Result<Preferenze, PapErrore> {
    carica_pure(&state.file_path())
}

#[tauri::command]
pub fn preferenze_salva(
    preferenze: Preferenze,
    state: State<'_, PreferenzeState>,
) -> Result<(), PapErrore> {
    salva_pure(&state.data_dir, &preferenze)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_preferenze() {
        let prefs = Preferenze::default();
        assert_eq!(prefs.profilo, "personale");
        assert_eq!(prefs.hotkey, "Ctrl+Shift+P");
        assert!(!prefs.onboarding_completato);
    }

    #[test]
    fn salva_e_carica() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferenze.json");

        let prefs = Preferenze {
            profilo: "team".to_string(),
            hotkey: "Ctrl+Alt+P".to_string(),
            tema: "light".to_string(),
            tono: "slate".to_string(),
            lingua: "en".to_string(),
            onboarding_completato: true,
            crea_prompt_esempio: false,
            sync_server_url: "https://sync.example.com".to_string(),
            sync_email: "test@example.com".to_string(),
            sync_token: String::new(),
            sync_intervallo_sec: 120,
            sync_abilitato: true,
            ricerca_semantica_abilitata: true,
            ricerca_alpha: 0.7,
            idle_unload_secondi: 600,
            debug_log_abilitato: false,
            updater_abilitato: true,
            autosave_delay_ms: 2000,
            line_wrapping: true,
            indent_size: 2,
            font_size: 13,
            show_line_numbers: true,
            highlight_active_line: false,
            nome_vault: "Personale".to_string(),
        };

        let json = serde_json::to_string_pretty(&prefs).unwrap();
        fs::write(&path, &json).unwrap();

        let letto: Preferenze = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(letto.profilo, "team");
        assert!(letto.onboarding_completato);
    }

    #[test]
    fn carica_pure_file_inesistente_ritorna_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferenze.json");
        let prefs = carica_pure(&path).unwrap();
        assert_eq!(prefs.profilo, "personale");
        assert!(!prefs.onboarding_completato);
        assert_eq!(prefs.idle_unload_secondi, 300);
        assert!(prefs.updater_abilitato);
        assert_eq!(prefs.autosave_delay_ms, 2000);
        assert!(prefs.line_wrapping);
        assert_eq!(prefs.indent_size, 2);
        assert_eq!(prefs.font_size, 13);
        assert!(prefs.show_line_numbers);
        assert!(!prefs.highlight_active_line);
    }

    #[test]
    fn carica_pure_json_malformato_errore() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferenze.json");
        fs::write(&path, "{ non json").unwrap();
        assert!(carica_pure(&path).is_err());
    }

    #[test]
    fn salva_pure_crea_dir_se_assente() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub").join("dir");
        assert!(!sub.exists());
        let prefs = Preferenze::default();
        salva_pure(&sub, &prefs).unwrap();
        assert!(sub.join("preferenze.json").exists());
    }

    #[test]
    fn round_trip_salva_carica_pure_preserva_valori() {
        let dir = tempfile::tempdir().unwrap();
        let prefs = Preferenze {
            profilo: "team".to_string(),
            hotkey: "Ctrl+Alt+P".to_string(),
            tema: "light".to_string(),
            tono: "slate".to_string(),
            lingua: "en".to_string(),
            onboarding_completato: true,
            crea_prompt_esempio: false,
            sync_server_url: "https://sync.example.com".to_string(),
            sync_email: "test@example.com".to_string(),
            sync_token: String::new(),
            sync_intervallo_sec: 120,
            sync_abilitato: true,
            ricerca_semantica_abilitata: true,
            ricerca_alpha: 0.7,
            idle_unload_secondi: 600,
            debug_log_abilitato: true,
            updater_abilitato: false,
            autosave_delay_ms: 1500,
            line_wrapping: false,
            indent_size: 4,
            font_size: 16,
            show_line_numbers: false,
            highlight_active_line: true,
            nome_vault: "Marketing".to_string(),
        };
        salva_pure(dir.path(), &prefs).unwrap();
        let letto = carica_pure(&dir.path().join("preferenze.json")).unwrap();
        assert_eq!(letto.profilo, "team");
        assert_eq!(letto.hotkey, "Ctrl+Alt+P");
        assert!(letto.onboarding_completato);
        assert_eq!(letto.idle_unload_secondi, 600);
        assert!(letto.debug_log_abilitato);
        assert!(!letto.updater_abilitato);
        assert_eq!(letto.autosave_delay_ms, 1500);
        assert_eq!(letto.nome_vault, "Marketing");
        assert!(!letto.line_wrapping);
        assert_eq!(letto.indent_size, 4);
        assert_eq!(letto.font_size, 16);
        assert!(!letto.show_line_numbers);
        assert!(letto.highlight_active_line);
    }

    #[test]
    fn carica_pure_applica_default_su_campi_mancanti() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferenze.json");
        // JSON con solo i campi base (no sync_*, no updater_*).
        fs::write(
            &path,
            r#"{
                "profilo": "personale",
                "hotkey": "Ctrl+Shift+P",
                "tema": "dark",
                "tono": "zinc",
                "lingua": "it",
                "onboarding_completato": true,
                "crea_prompt_esempio": false
            }"#,
        )
        .unwrap();
        let prefs = carica_pure(&path).unwrap();
        assert_eq!(prefs.sync_intervallo_sec, 60);
        assert_eq!(prefs.ricerca_alpha, 0.5);
        assert_eq!(prefs.idle_unload_secondi, 300);
        assert!(prefs.updater_abilitato);
    }
}
