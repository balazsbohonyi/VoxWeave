// commands/config.rs — Tauri commands for config read/write.
// Thin handlers only — business logic lives in config::persistence.

use crate::config::{persistence, AppConfig};
use crate::state::AppState;
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
pub fn save_config(state: State<AppState>, config: AppConfig) -> Result<(), String> {
    let mut raw = state.config_raw.lock().map_err(|e| e.to_string())?;
    persistence::save(&config, &mut raw)?;

    let mut current = state.config.lock().map_err(|e| e.to_string())?;
    *current = config;

    Ok(())
}
