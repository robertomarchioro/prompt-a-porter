use std::fs;
use std::path::PathBuf;

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
}

fn default_sync_intervallo() -> u32 {
    60
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

#[tauri::command]
pub fn preferenze_carica(state: State<'_, PreferenzeState>) -> Result<Preferenze, PapErrore> {
    let path = state.file_path();
    if !path.exists() {
        return Ok(Preferenze::default());
    }
    let json = fs::read_to_string(&path)?;
    let prefs: Preferenze = serde_json::from_str(&json)?;
    Ok(prefs)
}

#[tauri::command]
pub fn preferenze_salva(
    preferenze: Preferenze,
    state: State<'_, PreferenzeState>,
) -> Result<(), PapErrore> {
    fs::create_dir_all(&state.data_dir)?;
    let json = serde_json::to_string_pretty(&preferenze)?;
    fs::write(state.file_path(), json)?;
    Ok(())
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
        };

        let json = serde_json::to_string_pretty(&prefs).unwrap();
        fs::write(&path, &json).unwrap();

        let letto: Preferenze = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(letto.profilo, "team");
        assert!(letto.onboarding_completato);
    }
}
