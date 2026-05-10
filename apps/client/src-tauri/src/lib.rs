// `pub mod` per essere accessibili da `examples/` e `benches/`
// (Step 10 quality gate). I tipi privati rimangono privati per
// visibilità Rust ordinaria.
pub mod audit;
pub mod cartelle;
pub mod editor;
pub mod embeddings;
pub mod embeddings_backfill;
pub mod embeddings_store;
pub mod errore;
pub mod fork;
pub mod import_export;
pub mod libreria;
pub mod linting;
pub mod migrazione;
pub mod preferenze;
pub mod prompt;
pub mod prompt_componibili;
pub mod provider_ai;
pub mod rating;
pub mod regression;
pub mod ricerca_ibrida;
pub mod similarity;
pub mod statistiche;
pub mod sync;
pub mod tags_suggest;
pub mod varianti;
pub mod vault;
pub mod segnaposti_globali;
pub mod versioning;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

fn toggle_palette(app: &tauri::AppHandle) {
    if let Some(palette) = app.get_webview_window("palette") {
        if palette.is_visible().unwrap_or(false) {
            let _ = palette.hide();
        } else {
            let _ = palette.show();
            let _ = palette.center();
            let _ = palette.set_focus();
        }
    }
}

fn converti_tasto(tasto: &str) -> Option<Code> {
    match tasto {
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        "0" => Some(Code::Digit0),
        "1" => Some(Code::Digit1),
        "2" => Some(Code::Digit2),
        "3" => Some(Code::Digit3),
        "4" => Some(Code::Digit4),
        "5" => Some(Code::Digit5),
        "6" => Some(Code::Digit6),
        "7" => Some(Code::Digit7),
        "8" => Some(Code::Digit8),
        "9" => Some(Code::Digit9),
        "SPACE" => Some(Code::Space),
        "ENTER" => Some(Code::Enter),
        "TAB" => Some(Code::Tab),
        "BACKSPACE" => Some(Code::Backspace),
        "DELETE" => Some(Code::Delete),
        "ESCAPE" => Some(Code::Escape),
        "ARROWUP" => Some(Code::ArrowUp),
        "ARROWDOWN" => Some(Code::ArrowDown),
        "ARROWLEFT" => Some(Code::ArrowLeft),
        "ARROWRIGHT" => Some(Code::ArrowRight),
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        _ => None,
    }
}

fn parse_hotkey(combo: &str) -> Result<Shortcut, String> {
    let parti: Vec<&str> = combo.split('+').collect();
    if parti.is_empty() {
        return Err("Hotkey vuota".into());
    }

    let mut mods = Modifiers::empty();
    for p in &parti[..parti.len() - 1] {
        match p.to_lowercase().as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "shift" => mods |= Modifiers::SHIFT,
            "alt" => mods |= Modifiers::ALT,
            "super" | "meta" | "cmd" | "command" => mods |= Modifiers::META,
            _ => return Err(format!("Modificatore sconosciuto: {p}")),
        }
    }

    let tasto = parti.last().unwrap();
    let tasto_upper = tasto.to_uppercase();
    let code = converti_tasto(&tasto_upper)
        .ok_or_else(|| format!("Tasto non supportato: {tasto}"))?;

    let mods_opt = if mods.is_empty() { None } else { Some(mods) };
    Ok(Shortcut::new(mods_opt, code))
}

fn carica_hotkey_preferenze(data_dir: &std::path::Path) -> String {
    let prefs_path = data_dir.join("preferenze.json");
    if let Ok(json) = std::fs::read_to_string(prefs_path) {
        if let Ok(prefs) = serde_json::from_str::<preferenze::Preferenze>(&json) {
            return prefs.hotkey;
        }
    }
    "Ctrl+Shift+P".to_string()
}

/// Legge `idle_unload_secondi` dal file preferenze. `None` se file
/// inesistente o malformato (caller usa il default).
fn carica_idle_unload_pref(data_dir: &std::path::Path) -> Option<u32> {
    let prefs_path = data_dir.join("preferenze.json");
    let json = std::fs::read_to_string(prefs_path).ok()?;
    let prefs: preferenze::Preferenze = serde_json::from_str(&json).ok()?;
    Some(prefs.idle_unload_secondi)
}

fn registra_shortcut(app: &tauri::AppHandle, combo: &str) -> Result<(), String> {
    let _ = app.global_shortcut().unregister_all();
    let shortcut = parse_hotkey(combo)?;
    app.global_shortcut()
        .on_shortcut(shortcut, |app_handle, _, event| {
            if event.state == ShortcutState::Pressed {
                toggle_palette(app_handle);
            }
        })
        .map_err(|e| e.to_string())?;
    log::info!("Hotkey registrata: {combo}");
    Ok(())
}

#[tauri::command]
fn registra_hotkey(combo: String, app: tauri::AppHandle) -> Result<(), String> {
    registra_shortcut(&app, &combo)
}

pub fn run() {
    // Registra sqlite-vec come auto-extension PRIMA che venga aperta qualunque
    // connessione SQLite (vault SQLCipher incluso). Idempotente via std::sync::Once.
    // Vedi docs/architettura/decisioni/sqlite-vec-sqlcipher.md.
    embeddings_store::registra_auto_extension();

    tauri::Builder::default()
        // Issue #144: single-instance DEVE essere il primo plugin
        // registrato (vedi docs Tauri 2). Quando una seconda istanza
        // viene lanciata, il callback focusa la finestra già attiva
        // e la seconda istanza si chiude → no più doppia tray icon.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(finestra) = app.get_webview_window("libreria") {
                let _ = finestra.show();
                let _ = finestra.unminimize();
                let _ = finestra.set_focus();
            }
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Impossibile ottenere la directory dati dell'app");

            app.manage(vault::VaultState::new(data_dir.clone()));
            app.manage(preferenze::PreferenzeState::new(data_dir.clone()));
            app.manage(embeddings::EmbeddingsState::new());

            // ── Menu contestuale del tray ──

            let apri = MenuItem::with_id(app, "apri_palette", "Apri palette", true, None::<&str>)?;
            let nuovo =
                MenuItem::with_id(app, "nuovo_prompt", "Nuovo prompt", true, None::<&str>)?;
            let libreria =
                MenuItem::with_id(app, "mostra_libreria", "Mostra libreria", true, None::<&str>)?;
            let impostazioni =
                MenuItem::with_id(app, "impostazioni", "Impostazioni", true, None::<&str>)?;
            let esci = MenuItem::with_id(app, "esci", "Esci", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&apri, &nuovo, &libreria, &impostazioni, &esci])?;

            // Icona del tray: usa l'icona di default della finestra (compilata
            // da tauri.conf.json bundle.icon). Senza questa, su Windows
            // l'icona del tray appare come spazio vuoto e Windows talvolta
            // mostra una seconda icona "running app" non hookata al menu.
            let icona_tray = app
                .default_window_icon()
                .cloned()
                .ok_or("Icona di default non disponibile")?;

            let _tray = TrayIconBuilder::new()
                .icon(icona_tray)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("Prompt a Porter")
                .on_tray_icon_event(|tray, event| {
                    // Click sinistro su tray → mostra/focus libreria.
                    // Click destro → menù contestuale (default Tauri 2 con
                    // show_menu_on_left_click(false)). Vedi issue #3.
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(finestra) =
                            tray.app_handle().get_webview_window("libreria")
                        {
                            let _ = finestra.show();
                            let _ = finestra.set_focus();
                        }
                    }
                })
                .on_menu_event(|app, event| {
                    // Issue #146: gli item "nuovo_prompt" e "impostazioni"
                    // facevano solo show+focus della finestra senza
                    // notificare l'azione al webview → utente vedeva la
                    // finestra ma niente accadeva. Ora dopo show+focus
                    // emette un event Tauri che il client ascolta in
                    // Shell.svelte e traduce in apriModale / dispatch
                    // pap:nuovo-prompt.
                    let mostra_libreria = || {
                        if let Some(finestra) = app.get_webview_window("libreria") {
                            let _ = finestra.show();
                            let _ = finestra.unminimize();
                            let _ = finestra.set_focus();
                        }
                    };
                    match event.id.as_ref() {
                        "apri_palette" => {
                            toggle_palette(app);
                        }
                        "nuovo_prompt" => {
                            mostra_libreria();
                            let _ = app.emit("tray:nuovo-prompt", ());
                        }
                        "mostra_libreria" => {
                            mostra_libreria();
                        }
                        "impostazioni" => {
                            mostra_libreria();
                            let _ = app.emit("tray:apri-impostazioni", ());
                        }
                        "esci" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // ── Registra hotkey globale ──

            let hotkey_combo = carica_hotkey_preferenze(&data_dir);
            if let Err(e) = registra_shortcut(app.handle(), &hotkey_combo) {
                log::warn!("Impossibile registrare hotkey '{hotkey_combo}': {e}");
            }

            // ── Task background: idle-unload Session embeddings (Step 10) ──
            //
            // Ogni IDLE_CHECK_INTERVAL_SEC controlla la preferenza
            // utente `idle_unload_secondi`. Se > 0 e la Session ort è
            // stata inattiva da almeno quella soglia, la droppa per
            // liberare RAM. La preferenza viene riletta a ogni tick così
            // l'utente può cambiarla dall'UI senza riavvio.
            //
            // Thread OS dedicato (no async) — il workload è molto leggero
            // (~1 lock check ogni 30s) e non richiede tokio.
            const IDLE_CHECK_INTERVAL_SEC: u64 = 30;
            let handle_idle = app.handle().clone();
            let data_dir_idle = data_dir.clone();
            std::thread::spawn(move || {
                let intervallo =
                    std::time::Duration::from_secs(IDLE_CHECK_INTERVAL_SEC);
                loop {
                    std::thread::sleep(intervallo);
                    let soglia_sec =
                        carica_idle_unload_pref(&data_dir_idle).unwrap_or(300);
                    if soglia_sec == 0 {
                        continue; // disattivata
                    }
                    let rt_state = handle_idle.state::<embeddings::EmbeddingsState>();
                    let _ = embeddings::unload_se_idle(
                        &rt_state,
                        std::time::Duration::from_secs(soglia_sec as u64),
                    );
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            vault::vault_esiste,
            vault::vault_aperto,
            vault::vault_crea,
            vault::vault_crea_aperto,
            vault::vault_cifrato,
            vault::vault_unlock,
            vault::vault_lock,
            vault::vault_cambia_password,
            vault::vault_percorso,
            vault::vault_elimina,
            preferenze::preferenze_carica,
            preferenze::preferenze_salva,
            prompt::prompt_cerca,
            ricerca_ibrida::prompt_cerca_ibrida,
            libreria::libreria_conteggi,
            libreria::libreria_lista,
            libreria::libreria_dettaglio,
            libreria::libreria_toggle_preferito,
            libreria::libreria_tag_lista,
            editor::prompt_crea,
            editor::prompt_aggiorna,
            editor::prompt_registra_uso,
            editor::prompt_elimina,
            versioning::prompt_get_history,
            versioning::prompt_rollback,
            sync::sync_applica_delta,
            audit::audit_lista,
            audit::audit_query,
            audit::audit_export_csv,
            audit::audit_cleanup_oltre_giorni,
            import_export::vault_export_json,
            import_export::vault_export_folder_json,
            import_export::vault_import_json,
            import_export::prompt_export_markdown,
            statistiche::statistiche_query,
            embeddings::embeddings_status,
            embeddings::embeddings_download,
            embeddings::embeddings_init,
            embeddings::embeddings_compute,
            embeddings_backfill::embeddings_backfill,
            tags_suggest::tags_suggest,
            linting::prompt_lint,
            prompt_componibili::prompt_compila,
            prompt_componibili::prompt_resolve_import_preview,
            cartelle::folder_lista,
            cartelle::folder_crea,
            cartelle::folder_rinomina,
            cartelle::folder_sposta,
            cartelle::folder_riordina,
            cartelle::folder_elimina,
            cartelle::prompt_sposta,
            cartelle::prompt_riordina,
            regression::golden_crea,
            regression::golden_aggiorna,
            regression::golden_elimina,
            regression::golden_lista,
            regression::golden_esegui,
            regression::regression_report,
            regression::regression_report_csv,
            provider_ai::provider_ollama_genera,
            provider_ai::provider_config_lista,
            provider_ai::provider_config_salva,
            provider_ai::provider_config_elimina,
            varianti::prompt_crea_variante,
            varianti::varianti_lista,
            fork::prompt_fork,
            fork::fork_info,
            rating::rating_aggiungi,
            rating::rating_aggregato,
            segnaposti_globali::globale_placeholder_lista,
            segnaposti_globali::globale_placeholder_aggiorna,
            segnaposti_globali::globale_placeholder_elimina,
            registra_hotkey,
        ])
        .run(tauri::generate_context!())
        .expect("Errore durante l'avvio di Prompt a Porter");
}
