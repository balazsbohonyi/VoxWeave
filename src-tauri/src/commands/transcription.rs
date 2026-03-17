// Transcription commands — thin command handlers for retry and fallback flows.
// All business logic lives in transcription::service; these handlers only read
// AppState, call the service, and return results.

use crate::config::TranscriptionProvider;
use crate::state::AppState;
use crate::transcription;
use tauri::{AppHandle, Manager, Runtime};

/// Re-run transcription with a fresh attempt counter.
/// Called by the frontend "Retry" button from a transcription-error event.
#[tauri::command]
pub async fn retry_transcription<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let encoded = {
        let state = app.state::<AppState>();
        let guard = state.last_encoded_audio.lock().unwrap();
        guard.clone()
    };
    match encoded {
        None => Err("No audio available to retry.".into()),
        Some(audio) => transcription::transcribe_with_retry(&app, &audio)
            .await
            .map(|_| ())
            .map_err(|_| "Transcription failed.".into()),
    }
}

/// Run transcription with a specific fallback provider (one attempt, no further fallback).
/// Called by the frontend "Try with X?" button from a transcription-error event.
/// `provider` is a serialized TranscriptionProvider string — "openai", "groq", or "openrouter".
#[tauri::command]
pub async fn retry_transcription_with_fallback<R: Runtime>(
    app: AppHandle<R>,
    provider: String,
) -> Result<(), String> {
    let target = provider
        .parse::<TranscriptionProvider>()
        .map_err(|_| format!("Unknown provider: {provider}"))?;
    let encoded = {
        let state = app.state::<AppState>();
        let guard = state.last_encoded_audio.lock().unwrap();
        guard.clone()
    };
    match encoded {
        None => Err("No audio available to retry.".into()),
        Some(audio) => transcription::transcribe_with_provider(&app, &audio, target)
            .await
            .map(|_| ())
            .map_err(|_| "Fallback transcription failed.".into()),
    }
}

/// Opens the settings window. Tab navigation to specific provider is a Phase 8 concern.
#[tauri::command]
pub async fn open_settings_on_transcription_tab<R: Runtime>(
    app: AppHandle<R>,
    _provider: Option<String>,
) -> Result<(), String> {
    crate::tray::show_settings_window(&app);
    Ok(())
}
