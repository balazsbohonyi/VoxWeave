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
