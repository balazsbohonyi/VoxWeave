mod commands;
mod config;
mod platform;
mod state;
mod tray;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Managed state — single source of truth across commands and tray
            let app_state = AppState::new();
            app.manage(app_state);

            // Build tray icon; tray drives all window visibility (tray-first bootstrap)
            tray::setup_tray(app)?;

            // Settings window is created hidden; tray "Open Settings" shows it on demand.
            // The window is defined in tauri.conf.json with `visible: false`.

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VoxFlow");
}
