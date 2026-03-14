// AppConfig — persisted application settings.
// Stored at %APPDATA%/VoxFlow/config.json.
// Missing fields use serde defaults; unknown fields are preserved via
// `serde_json::Value` round-trip in the save/load helpers (Phase 8).

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionMode {
    FlashPaste,
    Keystroke,
    Clipboard,
}

impl Default for InjectionMode {
    fn default() -> Self {
        Self::FlashPaste
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionProvider {
    Openai,
    Groq,
    Openrouter,
    Local,
}

impl Default for TranscriptionProvider {
    fn default() -> Self {
        Self::Openai
    }
}

/// Full application configuration. Derives Default so AppState::new() works
/// without reading disk (disk load will happen in Phase 8).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,

    #[serde(default)]
    pub audio_device: Option<String>,

    #[serde(default)]
    pub transcription_provider: TranscriptionProvider,

    #[serde(default)]
    pub openai_api_key: String,
    #[serde(default)]
    pub groq_api_key: String,
    #[serde(default)]
    pub openrouter_api_key: String,

    #[serde(default = "default_openai_model")]
    pub openai_model: String,
    #[serde(default = "default_groq_model")]
    pub groq_model: String,
    #[serde(default)]
    pub openrouter_model: String,

    #[serde(default)]
    pub injection_mode: InjectionMode,

    #[serde(default = "default_true")]
    pub show_indicator: bool,
    #[serde(default)]
    pub launch_at_login: bool,
}

fn default_hotkey() -> String {
    "Alt+Shift+Space".to_string()
}
fn default_openai_model() -> String {
    "whisper-1".to_string()
}
fn default_groq_model() -> String {
    "whisper-large-v3".to_string()
}
fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
            audio_device: None,
            transcription_provider: TranscriptionProvider::default(),
            openai_api_key: String::new(),
            groq_api_key: String::new(),
            openrouter_api_key: String::new(),
            openai_model: default_openai_model(),
            groq_model: default_groq_model(),
            openrouter_model: String::new(),
            injection_mode: InjectionMode::default(),
            show_indicator: true,
            launch_at_login: false,
        }
    }
}
