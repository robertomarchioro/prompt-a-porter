use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errore::PapErrore;
use crate::vault::VaultState;

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
    /// Fix #455: NON deve mai essere persistito in chiaro in
    /// preferenze.json. Il valore "vero" vive nel vault cifrato
    /// (`SyncMeta.LastSyncToken`, vedi `sync::sync_token_carica_pure` /
    /// `sync::sync_token_salva_pure`). Il campo resta nella struct per
    /// compatibilità con l'API esistente verso il frontend (`preferenze_carica`
    /// / `preferenze_salva`): i comandi Tauri (non le funzioni `_pure` qui
    /// sotto, usate anche nei test) lo instradano da/verso il vault e lo
    /// azzerano SEMPRE prima di scrivere su disco.
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
    // SECURITY: manteniamo comunque i permessi 0600 su Unix come difesa in
    // profondità, anche se dal fix #455 `sync_token` non è più il segreto
    // in chiaro che c'era qui (spostato nel vault cifrato, vedi comando
    // `preferenze_salva`). Su Windows l'AppData è già per-utente.
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
pub fn preferenze_carica(
    state: State<'_, PreferenzeState>,
    vault: State<'_, VaultState>,
) -> Result<Preferenze, PapErrore> {
    preferenze_carica_impl(&state, &vault)
}

/// Fix #455: logica testabile di `preferenze_carica`. Oltre a leggere
/// `preferenze.json`, gestisce il `sync_token`:
/// - se il file contiene ancora un token legacy in chiaro (build
///   precedenti a questo fix) E il vault è aperto, lo migra subito nel
///   vault (`SyncMeta.LastSyncToken`) e riscrive il file senza il segreto
///   (migrazione one-shot, best-effort: se il vault è chiuso il file resta
///   intatto e si riprova al prossimo load a vault aperto);
/// - altrimenti, se il vault è aperto, popola `sync_token` con il valore
///   corrente letto dal vault (il file su disco non lo contiene più).
///   A vault chiuso il campo torna vuoto: nessun segreto disponibile senza
///   sbloccare, comportamento pulito invece di un errore che romperebbe
///   il caricamento di TUTTE le altre preferenze.
pub(crate) fn preferenze_carica_impl(
    state: &PreferenzeState,
    vault: &VaultState,
) -> Result<Preferenze, PapErrore> {
    let path = state.file_path();
    let mut prefs = carica_pure(&path)?;

    if !prefs.sync_token.is_empty() {
        let token_legacy = std::mem::take(&mut prefs.sync_token);
        if vault
            .with_conn(|conn| crate::sync::sync_token_salva_pure(conn, &token_legacy))
            .is_ok()
        {
            // `prefs.sync_token` è già vuoto qui: il file scritto adesso
            // non contiene più il segreto in chiaro.
            salva_pure(&state.data_dir, &prefs)?;
        }
        // Il chiamante riceve comunque il valore corrente (vive nel vault
        // se la migrazione è riuscita, altrimenti è rimasto nel file: il
        // vault era chiuso, si riprova al prossimo load a vault aperto).
        prefs.sync_token = token_legacy;
    } else if let Ok(Some(token)) = vault.with_conn(crate::sync::sync_token_carica_pure) {
        prefs.sync_token = token;
    }

    Ok(prefs)
}

#[tauri::command]
pub fn preferenze_salva(
    preferenze: Preferenze,
    state: State<'_, PreferenzeState>,
    vault: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    preferenze_salva_impl(preferenze, &state, &vault)
}

/// Fix #455: logica testabile di `preferenze_salva`. Il `sync_token`
/// ricevuto dal chiamante NON viene mai scritto su `preferenze.json`:
/// - se non vuoto (nuovo login/token aggiornato), va salvato nel vault
///   cifrato — richiede il vault aperto, altrimenti errore pulito
///   (`PapErrore::VaultChiuso`) PRIMA di toccare il disco, così un
///   salvataggio fallito non perde né il token né le altre preferenze;
/// - se vuoto (es. logout esplicito, o semplicemente nessun token nella
///   chiamata corrente), il tentativo di ripulire il vault è best-effort:
///   non blocchiamo il salvataggio di preferenze non correlate (tema,
///   editor, ...) solo perché il vault è chiuso in quel momento.
pub(crate) fn preferenze_salva_impl(
    mut preferenze: Preferenze,
    state: &PreferenzeState,
    vault: &VaultState,
) -> Result<(), PapErrore> {
    let token = std::mem::take(&mut preferenze.sync_token);

    if !token.is_empty() {
        vault.with_conn(|conn| crate::sync::sync_token_salva_pure(conn, &token))?;
    } else {
        let _ = vault.with_conn(|conn| crate::sync::sync_token_salva_pure(conn, &token));
    }

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

    // ─────────── Fix #455: sync_token instradato verso il vault ───────────

    /// Vault temporaneo NON cifrato ma già aperto (niente password richiesta
    /// nei test): basta ad esercitare `with_conn`/`SyncMeta`, che non
    /// dipendono dalla cifratura.
    fn vault_temp_aperto() -> (tempfile::TempDir, VaultState) {
        crate::embeddings_store::registra_auto_extension();
        let dir = tempfile::tempdir().unwrap();
        let state = VaultState::new(dir.path().to_path_buf());
        crate::vault::vault_crea_aperto_impl(&state).unwrap();
        (dir, state)
    }

    #[test]
    fn preferenze_salva_impl_con_token_su_vault_aperto_va_nel_vault_non_su_disco() {
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let (_vdir, vault) = vault_temp_aperto();

        let mut prefs = Preferenze::default();
        prefs.sync_token = "tok-segreto".to_string();
        preferenze_salva_impl(prefs, &pref_state, &vault).unwrap();

        // Il file su disco non deve MAI contenere il token in chiaro.
        let json = fs::read_to_string(pref_state.file_path()).unwrap();
        assert!(!json.contains("tok-segreto"));

        // Il valore vero vive nel vault.
        let dal_vault = vault.with_conn(crate::sync::sync_token_carica_pure).unwrap();
        assert_eq!(dal_vault, Some("tok-segreto".to_string()));
    }

    #[test]
    fn preferenze_salva_impl_con_token_su_vault_chiuso_e_errore_pulito() {
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let vault = VaultState::new(dir.path().join("vault-non-aperto"));

        let mut prefs = Preferenze::default();
        prefs.sync_token = "tok-segreto".to_string();
        let r = preferenze_salva_impl(prefs, &pref_state, &vault);
        assert!(matches!(r, Err(PapErrore::VaultChiuso)));

        // Nessuna scrittura su disco: né il token né il resto delle
        // preferenze devono essere persistiti se il salvataggio fallisce.
        assert!(!pref_state.file_path().exists());
    }

    #[test]
    fn preferenze_salva_impl_senza_token_su_vault_chiuso_non_fallisce() {
        // Salvare tema/tono/editor con vault chiuso deve continuare a
        // funzionare: nessun nuovo segreto da proteggere, non è
        // responsabilità di questo comando forzare l'unlock del vault.
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let vault = VaultState::new(dir.path().join("vault-non-aperto"));

        let mut prefs = Preferenze::default();
        prefs.tema = "light".to_string();
        preferenze_salva_impl(prefs, &pref_state, &vault).unwrap();

        let letto = carica_pure(&pref_state.file_path()).unwrap();
        assert_eq!(letto.tema, "light");
        assert_eq!(letto.sync_token, "");
    }

    #[test]
    fn preferenze_carica_impl_ripopola_sync_token_dal_vault_se_aperto() {
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let (_vdir, vault) = vault_temp_aperto();

        vault
            .with_conn(|conn| crate::sync::sync_token_salva_pure(conn, "tok-dal-vault"))
            .unwrap();
        salva_pure(&dir.path().to_path_buf(), &Preferenze::default()).unwrap();

        let prefs = preferenze_carica_impl(&pref_state, &vault).unwrap();
        assert_eq!(prefs.sync_token, "tok-dal-vault");
    }

    #[test]
    fn preferenze_carica_impl_vault_chiuso_torna_token_vuoto() {
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let vault = VaultState::new(dir.path().join("vault-non-aperto"));
        salva_pure(&dir.path().to_path_buf(), &Preferenze::default()).unwrap();

        let prefs = preferenze_carica_impl(&pref_state, &vault).unwrap();
        assert_eq!(prefs.sync_token, "");
    }

    #[test]
    fn preferenze_carica_impl_migra_token_legacy_da_file_a_vault() {
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let (_vdir, vault) = vault_temp_aperto();

        // Simula un preferenze.json scritto da una build precedente al fix
        // #455, con il token ancora in chiaro sul file.
        let mut legacy = Preferenze::default();
        legacy.sync_token = "tok-legacy-in-chiaro".to_string();
        salva_pure(&dir.path().to_path_buf(), &legacy).unwrap();

        let prefs = preferenze_carica_impl(&pref_state, &vault).unwrap();
        assert_eq!(prefs.sync_token, "tok-legacy-in-chiaro");

        // Il file deve essere stato ripulito: il token non c'è più in chiaro.
        let json = fs::read_to_string(pref_state.file_path()).unwrap();
        assert!(!json.contains("tok-legacy-in-chiaro"));

        // ...e vive ora nel vault.
        let dal_vault = vault.with_conn(crate::sync::sync_token_carica_pure).unwrap();
        assert_eq!(dal_vault, Some("tok-legacy-in-chiaro".to_string()));
    }

    #[test]
    fn preferenze_carica_impl_migrazione_rimandata_se_vault_chiuso() {
        let dir = tempfile::tempdir().unwrap();
        let pref_state = PreferenzeState::new(dir.path().to_path_buf());
        let vault = VaultState::new(dir.path().join("vault-non-aperto"));

        let mut legacy = Preferenze::default();
        legacy.sync_token = "tok-legacy-in-chiaro".to_string();
        salva_pure(&dir.path().to_path_buf(), &legacy).unwrap();

        // Vault chiuso: la migrazione non può avvenire ora. Il caricamento
        // NON deve fallire (le altre preferenze restano leggibili) e il
        // token resta nel file, in attesa del prossimo load a vault aperto.
        let prefs = preferenze_carica_impl(&pref_state, &vault).unwrap();
        assert_eq!(prefs.sync_token, "tok-legacy-in-chiaro");
        let json = fs::read_to_string(pref_state.file_path()).unwrap();
        assert!(json.contains("tok-legacy-in-chiaro"));
    }
}
