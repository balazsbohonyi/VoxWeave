// AppState — Tauri managed state. Single source of truth for all mutable
// application data accessed across commands, tray, and window lifecycle.

use crate::config::{persistence, AppConfig};
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

/// Startup hotkey availability state.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyAvailability {
    Unknown,
    Registered,
    Unavailable,
}

impl Default for HotkeyAvailability {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Hotkey warning payload stored for UI consumption.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HotkeyWarning {
    pub hotkey: String,
    pub message: String,
}

/// Top-level managed state stored in `tauri::Manager`.
pub struct AppState {
    /// Typed, in-memory application configuration.
    pub config: Arc<Mutex<AppConfig>>,

    /// Raw JSON value as last read from disk. Preserves unknown fields so a
    /// newer config written by a future app version survives round-trips
    /// through this older version.
    pub config_raw: Arc<Mutex<serde_json::Value>>,

    /// Current recording lifecycle state.
    pub recording_state: Arc<Mutex<RecordingState>>,

    /// Canonical hotkey binding currently active at runtime.
    pub hotkey_binding: Arc<Mutex<String>>,

    /// Startup hotkey registration status.
    pub hotkey_availability: Arc<Mutex<HotkeyAvailability>>,

    /// Optional warning payload for hotkey conflicts or parse failures.
    pub hotkey_warning: Arc<Mutex<Option<HotkeyWarning>>>,

    /// Set to `true` when the user requests cancellation of an in-flight
    /// transcription. Checked by the transcription worker thread.
    pub cancel_flag: Arc<Mutex<bool>>,

    /// Set to `true` when the app is quitting via the tray Quit action.
    /// The close-to-hide handler checks this to allow window destruction
    /// instead of hiding, so WebView2 tears down cleanly before exit.
    pub quitting: Arc<Mutex<bool>>,
}

impl AppState {
    /// Create AppState by loading config from disk.
    /// Falls back to defaults if the file does not exist or is malformed.
    pub fn load() -> Self {
        match persistence::load() {
            Ok(loaded) => {
                let config = loaded.config;
                let hotkey = config.hotkey.clone();
                Self {
                    config: Arc::new(Mutex::new(config)),
                    config_raw: Arc::new(Mutex::new(loaded.raw)),
                    recording_state: Arc::new(Mutex::new(RecordingState::default())),
                    hotkey_binding: Arc::new(Mutex::new(hotkey)),
                    hotkey_availability: Arc::new(Mutex::new(HotkeyAvailability::default())),
                    hotkey_warning: Arc::new(Mutex::new(None)),
                    cancel_flag: Arc::new(Mutex::new(false)),
                    quitting: Arc::new(Mutex::new(false)),
                }
            }
            Err(e) => {
                log::error!("Failed to load config from disk: {e}. Using defaults.");
                let config = AppConfig::default();
                let hotkey = config.hotkey.clone();
                let raw = serde_json::to_value(&config).unwrap_or(serde_json::Value::Object(
                    serde_json::Map::new(),
                ));
                Self {
                    config: Arc::new(Mutex::new(config)),
                    config_raw: Arc::new(Mutex::new(raw)),
                    recording_state: Arc::new(Mutex::new(RecordingState::default())),
                    hotkey_binding: Arc::new(Mutex::new(hotkey)),
                    hotkey_availability: Arc::new(Mutex::new(HotkeyAvailability::default())),
                    hotkey_warning: Arc::new(Mutex::new(None)),
                    cancel_flag: Arc::new(Mutex::new(false)),
                    quitting: Arc::new(Mutex::new(false)),
                }
            }
        }
    }
}
