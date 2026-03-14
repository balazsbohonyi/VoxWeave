// Thin Tauri command handlers — no business logic here.
// Commands delegate to config/state; they never own data.

use crate::config::AppConfig;
use crate::state::AppState;
use tauri::State;

/// Return the current persisted config.
#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// Persist updated config fields.
#[tauri::command]
pub fn save_config(state: State<AppState>, config: AppConfig) -> Result<(), String> {
    let mut current = state.config.lock().map_err(|e| e.to_string())?;
    *current = config;
    // TODO (Phase 8): write to disk via config::save()
    Ok(())
}
