// AppConfig — persisted application settings.
// Stored at %APPDATA%/VoxFlow/config.json.
// Missing fields use serde defaults. Unknown fields are preserved via
// raw JSON round-trip: load merges typed deserialization back into the raw
// Value so unknown keys survive save/load cycles.

pub mod persistence;

// ---------------------------------------------------------------------------
// Enumerations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionProvider {
    Openai,
    Groq,
    Local,
}

impl Default for TranscriptionProvider {
    fn default() -> Self {
        Self::Openai
    }
}

impl std::str::FromStr for TranscriptionProvider {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "openai" => Ok(Self::Openai),
            "groq" => Ok(Self::Groq),
            "local" => Ok(Self::Local),
            _ => Err(()),
        }
    }
}

// ---------------------------------------------------------------------------
// Nested config sections
// ---------------------------------------------------------------------------

/// Audio capture settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioConfig {
    /// Specific audio input device name, or None for system default.
    #[serde(default)]
    pub device: Option<String>,

    /// Voice-activity-detection silence threshold in RMS (0.0–1.0).
    /// Recording stops automatically when RMS falls below this for
    /// `vad_silence_ms` milliseconds.
    #[serde(default = "default_vad_threshold")]
    pub vad_threshold: f32,

    /// How long (ms) silence must persist before auto-stop.
    #[serde(default = "default_vad_silence_ms")]
    pub vad_silence_ms: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            device: None,
            vad_threshold: default_vad_threshold(),
            vad_silence_ms: default_vad_silence_ms(),
        }
    }
}

/// Cloud + local transcription settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionConfig {
    #[serde(default)]
    pub provider: TranscriptionProvider,

    // --- API keys ---
    #[serde(default)]
    pub openai_api_key: String,
    #[serde(default)]
    pub groq_api_key: String,

    // --- Model selections (hardcoded lists in UI, stored as string) ---
    #[serde(default = "default_openai_model")]
    pub openai_model: String,
    #[serde(default = "default_groq_model")]
    pub groq_model: String,

    /// BCP-47 language hint sent to the transcription API (e.g. "en", "hu").
    /// Empty string means auto-detect.
    #[serde(default)]
    pub language: String,

    /// Path to local whisper.cpp model file. Used only when provider = Local.
    #[serde(default)]
    pub local_model_path: Option<String>,

    /// Ordered list of providers to try when the primary provider fails.
    /// Deserialized from old configs that lack this field using the default.
    #[serde(default = "default_fallback_order")]
    pub fallback_order: Vec<TranscriptionProvider>,
}

impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self {
            provider: TranscriptionProvider::default(),
            openai_api_key: String::new(),
            groq_api_key: String::new(),
            openai_model: default_openai_model(),
            groq_model: default_groq_model(),
            language: String::new(),
            local_model_path: None,
            fallback_order: default_fallback_order(),
        }
    }
}

/// Text injection settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InjectionConfig {
    #[serde(default)]
    pub mode: InjectionMode,
}

impl Default for InjectionConfig {
    fn default() -> Self {
        Self {
            mode: InjectionMode::default(),
        }
    }
}

/// Floating indicator window settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndicatorConfig {
    /// Whether to show the indicator at all.
    #[serde(default = "default_true")]
    pub show: bool,

    /// Horizontal position of the indicator window in logical pixels.
    /// None means centered at bottom of primary display.
    #[serde(default)]
    pub position_x: Option<i32>,

    /// Vertical position of the indicator window in logical pixels.
    #[serde(default)]
    pub position_y: Option<i32>,

    /// Show indicator window when app starts.
    #[serde(default = "default_true")]
    pub show_on_startup: bool,
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        Self {
            show: true,
            position_x: None,
            position_y: None,
            show_on_startup: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Root AppConfig
// ---------------------------------------------------------------------------

/// Full application configuration. All fields have defaults so a missing or
/// empty config file produces a valid working configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    /// Global push-to-talk hotkey in the format "Modifier+Key"
    /// (e.g. "Ctrl+Shift+Space").
    #[serde(default = "default_hotkey")]
    pub hotkey: String,

    /// Audio capture settings.
    #[serde(default)]
    pub audio: AudioConfig,

    /// Transcription engine and provider settings.
    #[serde(default)]
    pub transcription: TranscriptionConfig,

    /// Text injection settings.
    #[serde(default)]
    pub injection: InjectionConfig,

    /// Floating indicator window settings.
    #[serde(default)]
    pub indicator: IndicatorConfig,

    /// Launch VoxFlow automatically when Windows starts.
    #[serde(default)]
    pub launch_at_login: bool,

    /// True until the user completes first-launch wizard.
    #[serde(default = "default_true")]
    pub first_launch: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
            audio: AudioConfig::default(),
            transcription: TranscriptionConfig::default(),
            injection: InjectionConfig::default(),
            indicator: IndicatorConfig::default(),
            launch_at_login: false,
            first_launch: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Default helpers
// ---------------------------------------------------------------------------

fn default_hotkey() -> String {
    "Ctrl+Shift+Space".to_string()
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
fn default_vad_threshold() -> f32 {
    0.01
}
fn default_vad_silence_ms() -> u32 {
    1500
}
fn default_fallback_order() -> Vec<TranscriptionProvider> {
    vec![
        TranscriptionProvider::Openai,
        TranscriptionProvider::Groq,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keystroke_speed_delay_ms() {
        assert_eq!(KeystrokeSpeed::Slow.delay_ms(), 10);
        assert_eq!(KeystrokeSpeed::Normal.delay_ms(), 5);
        assert_eq!(KeystrokeSpeed::Fast.delay_ms(), 2);
    }

    #[test]
    fn injection_config_defaults() {
        let cfg = InjectionConfig::default();
        assert_eq!(cfg.keystroke_speed, KeystrokeSpeed::Normal);
        assert_eq!(cfg.auto_fallback, true);
        assert_eq!(cfg.paste_delay_ms, 500);
    }

    #[test]
    fn injection_config_serde_round_trip() {
        let cfg = InjectionConfig {
            mode: InjectionMode::Keystroke,
            keystroke_speed: KeystrokeSpeed::Fast,
            auto_fallback: false,
            paste_delay_ms: 250,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: InjectionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.keystroke_speed, KeystrokeSpeed::Fast);
        assert_eq!(back.auto_fallback, false);
        assert_eq!(back.paste_delay_ms, 250);
    }
}
