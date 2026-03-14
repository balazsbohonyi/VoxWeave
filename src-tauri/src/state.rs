// AppState — Tauri managed state. Single source of truth for all mutable
// application data accessed across commands, tray, and window lifecycle.

use crate::config::AppConfig;
use std::sync::{Arc, Mutex};

/// Recording lifecycle state machine.
/// Placeholder variants will be extended in Phase 3 (Recording).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingState {
    Idle,
    Recording,
    Transcribing,
}

impl Default for RecordingState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Top-level managed state stored in `tauri::Manager`.
pub struct AppState {
    /// Persisted application configuration.
    pub config: Arc<Mutex<AppConfig>>,

    /// Current recording lifecycle state.
    pub recording_state: Arc<Mutex<RecordingState>>,

    /// Set to `true` when the user requests cancellation of an in-flight
    /// transcription. Checked by the transcription worker thread.
    pub cancel_flag: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(AppConfig::default())),
            recording_state: Arc::new(Mutex::new(RecordingState::default())),
            cancel_flag: Arc::new(Mutex::new(false)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
