// commands/config.rs — Tauri commands for config read/write.
// Thin handlers only — business logic lives in config::persistence.

use crate::config::AppConfig;
use crate::hotkey::service;
use crate::state::AppState;
use reqwest::Client;
use std::time::Duration;
use tauri::AppHandle;
use tauri::State;
use tauri_plugin_autostart::ManagerExt;

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

// ---------------------------------------------------------------------------
// Provider model lists (hardcoded — no dynamic API calls per CLAUDE.md)
// ---------------------------------------------------------------------------

/// Hardcoded model lists for each supported cloud transcription provider.
#[derive(Debug, serde::Serialize)]
pub struct ProviderModels {
    pub openai: Vec<String>,
    pub groq: Vec<String>,
}

/// Return hardcoded model lists for the settings UI dropdowns.
#[tauri::command]
pub fn get_provider_models() -> ProviderModels {
    ProviderModels {
        openai: vec![
            "whisper-1".to_string(),
            "gpt-4o-transcribe".to_string(),
            "gpt-4o-mini-transcribe".to_string(),
        ],
        groq: vec![
            "whisper-large-v3-turbo".to_string(),
            "whisper-large-v3".to_string(),
            "distil-whisper-large-v3-en".to_string(),
        ],
    }
}

// ---------------------------------------------------------------------------
// Connection testing
// ---------------------------------------------------------------------------

/// Probe the given provider's transcription endpoint with the stored API key.
/// Returns Ok(()) if the server acknowledges the request (any non-401 response).
/// Returns Err("Invalid API key.") on HTTP 401.
/// Returns Err("Connection failed: …") on network-level errors or timeout.
#[tauri::command]
pub async fn test_connection(
    state: State<'_, AppState>,
    provider: String,
) -> Result<(), String> {
    // Clone config fields before any await — never hold MutexGuard across await.
    let (api_key, endpoint) = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        match provider.as_str() {
            "openai" => (
                cfg.transcription.providers.openai.api_key.clone(),
                "https://api.openai.com/v1/audio/transcriptions",
            ),
            "groq" => (
                cfg.transcription.providers.groq.api_key.clone(),
                "https://api.groq.com/openai/v1/audio/transcriptions",
            ),
            _ => return Err(format!("Unknown provider: {provider}")),
        }
    };

    if api_key.is_empty() {
        return Err("API key is not configured.".to_string());
    }

    // Send a minimal multipart request with a 1-byte dummy file.
    // OpenAI/Groq return 400 (bad audio, but key accepted) or 401 (invalid key).
    // Any non-401, non-network-error response means the key is valid.
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let model = match provider.as_str() {
        "groq" => "whisper-large-v3-turbo",
        _ => "whisper-1",
    };

    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(vec![0u8])
                .file_name("test.wav")
                .mime_str("audio/wav")
                .map_err(|e| e.to_string())?,
        )
        .text("model", model);

    let response = client
        .post(endpoint)
        .bearer_auth(&api_key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {e}"))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err("Invalid API key.".to_string());
    }
    // Any other response (400, 422, 200, …) means the key was accepted.
    Ok(())
}

// ---------------------------------------------------------------------------
// Launch at login (autostart)
// ---------------------------------------------------------------------------

/// Enable or disable launching VoxFlow at Windows login via the autostart plugin.
/// Wraps HKCU\Software\Microsoft\Windows\CurrentVersion\Run registry key.
#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<(), String> {
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())?;
    } else {
        autostart.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_provider_models_openai_has_exactly_three_items() {
        let models = get_provider_models();
        assert_eq!(models.openai.len(), 3);
        assert!(models.openai.contains(&"whisper-1".to_string()));
        assert!(models.openai.contains(&"gpt-4o-transcribe".to_string()));
        assert!(models.openai.contains(&"gpt-4o-mini-transcribe".to_string()));
    }

    #[test]
    fn get_provider_models_groq_has_exactly_three_items() {
        let models = get_provider_models();
        assert_eq!(models.groq.len(), 3);
        assert!(models.groq.contains(&"whisper-large-v3-turbo".to_string()));
        assert!(models.groq.contains(&"whisper-large-v3".to_string()));
        assert!(models.groq.contains(&"distil-whisper-large-v3-en".to_string()));
    }
}
