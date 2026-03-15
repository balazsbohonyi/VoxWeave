mod commands;
mod config;
mod platform;
mod state;
mod tray;

use state::AppState;
use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Managed state — single source of truth across commands and tray
            let app_state = AppState::load();
            app.manage(app_state);

            // Build tray icon; tray drives all window visibility (tray-first bootstrap).
            tray::setup_tray(app)?;

            // Settings window is defined in tauri.conf.json with `visible: false`.
            // Register the close-to-hide handler so titlebar close hides rather than destroys.
            if let Some(settings_win) = app.get_webview_window("settings") {
                let win_clone = settings_win.clone();
                let quitting = app.state::<AppState>().quitting.clone();
                settings_win.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        if *quitting.lock().unwrap() {
                            // App is quitting — allow WebView2 to destroy cleanly.
                            return;
                        }
                        // Normal close: hide instead of destroy so the window is reusable.
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VoxFlow");
}
