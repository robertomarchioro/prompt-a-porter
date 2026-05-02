use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

#[tauri::command]
fn saluta(nome: &str) -> String {
    format!("Ciao {nome}, benvenuto in Prompt a Porter!")
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            // Menu contestuale del tray
            let apri = MenuItem::with_id(app, "apri_palette", "Apri palette", true, None::<&str>)?;
            let nuovo = MenuItem::with_id(app, "nuovo_prompt", "Nuovo prompt", true, None::<&str>)?;
            let libreria = MenuItem::with_id(app, "mostra_libreria", "Mostra libreria", true, None::<&str>)?;
            let impostazioni = MenuItem::with_id(app, "impostazioni", "Impostazioni", true, None::<&str>)?;
            let esci = MenuItem::with_id(app, "esci", "Esci", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[&apri, &nuovo, &libreria, &impostazioni, &esci],
            )?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Prompt a Porter")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "apri_palette" => {
                        if let Some(finestra) = app.get_webview_window("palette") {
                            let _ = finestra.show();
                            let _ = finestra.set_focus();
                        }
                    }
                    "mostra_libreria" => {
                        if let Some(finestra) = app.get_webview_window("libreria") {
                            let _ = finestra.show();
                            let _ = finestra.set_focus();
                        }
                    }
                    "impostazioni" => {
                        if let Some(finestra) = app.get_webview_window("libreria") {
                            let _ = finestra.show();
                            let _ = finestra.set_focus();
                            // TODO: navigare alla vista impostazioni via evento
                        }
                    }
                    "esci" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![saluta])
        .run(tauri::generate_context!())
        .expect("Errore durante l'avvio di Prompt a Porter");
}
