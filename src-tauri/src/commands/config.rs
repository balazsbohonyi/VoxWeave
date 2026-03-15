// commands/config.rs — Tauri commands for config read/write.
// Thin handlers only — business logic lives in config::persistence.

use crate::config::AppConfig;
use crate::hotkey::service;
use crate::state::AppState;
use tauri::AppHandle;
use tauri::State;

/// Return the current in-memory config.
#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// Persist updated config fields to disk and update in-memory state.
/// Unknown fields from previous load are preserved in the output file.
#[tauri::command]
pub fn save_config(app: AppHandle, _state: State<AppState>, config: AppConfig) -> Result<AppConfig, String> {
    service::apply_config_update(&app, config)
}
