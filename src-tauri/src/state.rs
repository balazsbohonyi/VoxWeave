// AppState — Tauri managed state. Single source of truth for all mutable
// application data accessed across commands, tray, and window lifecycle.

use crate::audio::encode::EncodedAudio;
use crate::config::{persistence, AppConfig};
use crate::indicator::events::IndicatorVisualState;
use crate::platform::ForegroundWindowInfo;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

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

/// Runtime-owned audio session details for active recording.
#[derive(Debug)]
pub struct AudioSessionState {
    pub active_device: String,
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub started_at: SystemTime,
    pub level_emitter_stop: Option<Arc<std::sync::atomic::AtomicBool>>,
    pub level_emitter_thread: Option<std::thread::JoinHandle<()>>,
    /// Shared PCM accumulation buffer. The capture thread pushes 16kHz mono f32
    /// samples into this buffer; the stop path drains it for encoding.
    pub pcm_buffer: Arc<Mutex<Vec<f32>>>,
    /// Stop flag shared with the PCM capture path. Mirrors level_emitter_stop
    /// — the same AtomicBool signals both the level emitter loop and the
    /// PCM accumulation path to stop.
    pub pcm_emitter_stop: Option<Arc<std::sync::atomic::AtomicBool>>,
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

    /// Active recording session owned by the audio module while capturing.
    pub audio_session: Arc<Mutex<Option<AudioSessionState>>>,

    /// Whether indicator drag mode is currently active (temporary interactivity).
    pub indicator_drag_active: Arc<Mutex<bool>>,

    /// Last known indicator visual state for late frontend subscribers.
    pub indicator_visual_state: Arc<Mutex<IndicatorVisualState>>,

    /// Last user-set position of the settings window (x, y in physical pixels).
    /// Persists within the current app session only — cleared on restart.
    /// `None` means the window has not been moved yet; show_settings_window
    /// will center it via tauri.conf.json `center: true` initial placement.
    pub settings_position: Arc<Mutex<Option<(i32, i32)>>>,

    /// Set to `true` when the app is quitting via the tray Quit action.
    /// The close-to-hide handler checks this to allow window destruction
    /// instead of hiding, so WebView2 tears down cleanly before exit.
    pub quitting: Arc<Mutex<bool>>,

    /// Last successfully encoded audio blob. Stored before spawning transcription
    /// so the retry and fallback commands can re-send the same audio without
    /// requiring the user to record again.
    pub last_encoded_audio: Arc<Mutex<Option<EncodedAudio>>>,

    /// The foreground window captured at recording START (before the indicator
    /// shows). Injection uses this to restore focus to the correct window.
    pub foreground_window: Arc<Mutex<Option<ForegroundWindowInfo>>>,
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
                    audio_session: Arc::new(Mutex::new(None)),
                    indicator_drag_active: Arc::new(Mutex::new(false)),
                    indicator_visual_state: Arc::new(Mutex::new(IndicatorVisualState::Hidden)),
                    settings_position: Arc::new(Mutex::new(None)),
                    quitting: Arc::new(Mutex::new(false)),
                    last_encoded_audio: Arc::new(Mutex::new(None)),
                    foreground_window: Arc::new(Mutex::new(None)),
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
                    audio_session: Arc::new(Mutex::new(None)),
                    indicator_drag_active: Arc::new(Mutex::new(false)),
                    indicator_visual_state: Arc::new(Mutex::new(IndicatorVisualState::Hidden)),
                    settings_position: Arc::new(Mutex::new(None)),
                    quitting: Arc::new(Mutex::new(false)),
                    last_encoded_audio: Arc::new(Mutex::new(None)),
                    foreground_window: Arc::new(Mutex::new(None)),
                }
            }
        }
    }
}
