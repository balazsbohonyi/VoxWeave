// commands::download — Tauri command handlers for model download management.
// These commands are thin wrappers; all download logic lives in transcription::download.

use crate::config::persistence;
use crate::state::AppState;
use crate::transcription::download;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::State;

/// Start downloading a whisper model in the background.
///
/// Returns Ok(()) immediately — the download runs as a background task.
/// Progress is emitted via "model-download-progress" events.
/// Only one download is allowed at a time; concurrent calls return Err.
#[tauri::command]
pub async fn start_model_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    model_id: String,
) -> Result<(), String> {
    // Validate model_id before touching state
    if !download::VALID_MODEL_IDS.contains(&model_id.as_str()) {
        return Err(format!(
            "Invalid model ID '{}'. Valid IDs: {}",
            model_id,
            download::VALID_MODEL_IDS.join(", ")
        ));
    }

    // Guard: only one download at a time
    // Clone what we need before releasing the lock
    {
        let guard = state.downloading_model.lock().unwrap();
        if guard.is_some() {
            return Err("A download is already in progress".to_string());
        }
    }

    // Create cancel flag and store state
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = cancel.clone();

    {
        let mut dl_cancel = state.download_cancel.lock().unwrap();
        *dl_cancel = Some(cancel);
    }
    {
        let mut dl_model = state.downloading_model.lock().unwrap();
        *dl_model = Some(model_id.clone());
    }

    // Clone handles for the background task
    let app_clone = app.clone();
    let state_config = state.config.clone();
    let state_config_raw = state.config_raw.clone();
    let state_download_cancel = state.download_cancel.clone();
    let state_downloading_model = state.downloading_model.clone();
    let model_id_clone = model_id.clone();

    tauri::async_runtime::spawn(async move {
        let result =
            download::run_download(app_clone, model_id_clone.clone(), cancel_clone).await;

        match result {
            Ok(model_path) => {
                // Update config: set local model_path
                let save_result = {
                    let mut config = state_config.lock().unwrap();
                    config.transcription.providers.local.model_path = Some(model_path.clone());
                    let mut raw = state_config_raw.lock().unwrap();
                    persistence::save(&config, &mut raw)
                };
                if let Err(e) = save_result {
                    log::error!("Failed to save config after model download: {e}");
                }
                // Clear download state
                *state_download_cancel.lock().unwrap() = None;
                *state_downloading_model.lock().unwrap() = None;
            }
            Err(e) => {
                // Log non-cancellation errors (cancellation is normal flow)
                if !e.contains("cancelled") {
                    log::error!("Model download error: {e}");
                }
                // Clear download state regardless
                *state_download_cancel.lock().unwrap() = None;
                *state_downloading_model.lock().unwrap() = None;
            }
        }
    });

    Ok(())
}

/// Cancel an in-progress model download.
///
/// Sets the atomic cancel flag; the background task will detect this,
/// delete the partial file, and emit a "model-download-cancelled" event.
#[tauri::command]
pub fn cancel_model_download(state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.download_cancel.lock().unwrap();
    if let Some(cancel) = guard.as_ref() {
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    Ok(())
}

/// Returns a list of model IDs that have been fully downloaded to disk.
///
/// Note: models currently being downloaded are NOT included here; the
/// frontend tracks in-progress state via download events.
#[tauri::command]
pub fn get_downloaded_models(
    _state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    download::get_downloaded_model_ids()
}

/// Delete a downloaded model file and clear the config if it was the active model.
#[tauri::command]
pub fn delete_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<(), String> {
    let path = download::model_file_path(&model_id)?;

    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete model file: {e}"))?;
    }

    // If this was the configured model path, clear it
    let model_path_str = path
        .to_str()
        .ok_or_else(|| "Model path is not valid UTF-8".to_string())?
        .to_string();

    let mut config = state.config.lock().unwrap();
    let currently_configured = config
        .transcription
        .providers
        .local
        .model_path
        .as_deref()
        .map(|p| p == model_path_str)
        .unwrap_or(false);

    if currently_configured {
        config.transcription.providers.local.model_path = None;
        let mut raw = state.config_raw.lock().unwrap();
        persistence::save(&config, &mut raw)
            .map_err(|e| format!("Failed to save config after model deletion: {e}"))?;
    }

    Ok(())
}
