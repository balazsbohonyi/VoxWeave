// AppConfig — persisted application settings.
// Stored at %APPDATA%/VoxWeave/config.json.
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
pub enum KeystrokeSpeed {
    Slow,    // 10ms/char
    Normal,  // 5ms/char
    Fast,    // 2ms/char
}

impl Default for KeystrokeSpeed {
    fn default() -> Self {
        Self::Normal
    }
}

impl KeystrokeSpeed {
    pub fn delay_ms(&self) -> u64 {
        match self {
            Self::Slow   => 10,
            Self::Normal => 5,
            Self::Fast   => 2,
        }
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

/// Per-provider cloud transcription config (API key + model selection).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CloudProviderConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub model: String,
}

impl Default for CloudProviderConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: String::new(),
        }
    }
}

fn default_openai_provider() -> CloudProviderConfig {
    CloudProviderConfig {
        api_key: String::new(),
        model: "whisper-1".to_string(),
    }
}

fn default_groq_provider() -> CloudProviderConfig {
    CloudProviderConfig {
        api_key: String::new(),
        model: "whisper-large-v3".to_string(),
    }
}

/// Local Whisper provider config.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct LocalProviderConfig {
    /// Path to a local whisper.cpp model file (.bin). None = not yet downloaded.
    #[serde(default)]
    pub model_path: Option<String>,
}

/// Nested map of all provider configs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionProviders {
    #[serde(default = "default_openai_provider")]
    pub openai: CloudProviderConfig,
    #[serde(default = "default_groq_provider")]
    pub groq: CloudProviderConfig,
    #[serde(default)]
    pub local: LocalProviderConfig,
}

impl Default for TranscriptionProviders {
    fn default() -> Self {
        Self {
            openai: default_openai_provider(),
            groq: default_groq_provider(),
            local: LocalProviderConfig::default(),
        }
    }
}

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

    /// Per-provider API key and model selections.
    #[serde(default)]
    pub providers: TranscriptionProviders,

    /// BCP-47 language hint sent to the transcription API (e.g. "en", "hu").
    /// Empty string means auto-detect.
    #[serde(default)]
    pub language: String,

    /// Ordered list of providers to try when the primary provider fails.
    /// Deserialized from old configs that lack this field using the default.
    #[serde(default = "default_fallback_order")]
    pub fallback_order: Vec<TranscriptionProvider>,
}

impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self {
            provider: TranscriptionProvider::default(),
            providers: TranscriptionProviders::default(),
            language: String::new(),
            fallback_order: default_fallback_order(),
        }
    }
}

/// Text injection settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InjectionConfig {
    #[serde(default)]
    pub mode: InjectionMode,

    #[serde(default)]
    pub keystroke_speed: KeystrokeSpeed,

    #[serde(default = "default_true")]
    pub auto_fallback: bool,

    #[serde(default = "default_paste_delay_ms")]
    pub paste_delay_ms: u64,
}

impl Default for InjectionConfig {
    fn default() -> Self {
        Self {
            mode: InjectionMode::default(),
            keystroke_speed: KeystrokeSpeed::default(),
            auto_fallback: true,
            paste_delay_ms: default_paste_delay_ms(),
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
    /// Global recording hotkey in the format "Modifier+Key"
    /// (e.g. "Ctrl+Shift+Space").
    #[serde(default = "default_hotkey")]
    pub hotkey: String,

    /// Hold the hotkey to record and stop when it is released.
    #[serde(default)]
    pub push_to_talk: bool,

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

    /// Launch VoxWeave automatically when Windows starts.
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
            push_to_talk: false,
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
fn default_true() -> bool {
    true
}
fn default_vad_threshold() -> f32 {
    0.01
}
fn default_vad_silence_ms() -> u32 {
    15000
}
fn default_paste_delay_ms() -> u64 {
    500
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

    // --- TranscriptionConfig nested providers tests ---

    #[test]
    fn test_transcription_config_default_provider_models() {
        let config = TranscriptionConfig::default();
        assert_eq!(config.providers.openai.model, "whisper-1");
        assert_eq!(config.providers.groq.model, "whisper-large-v3");
        assert!(config.providers.openai.api_key.is_empty());
        assert!(config.providers.groq.api_key.is_empty());
    }

    #[test]
    fn test_transcription_config_nested_json_round_trips() {
        let json = r#"{"provider":"openai","providers":{"openai":{"api_key":"sk-abc","model":"gpt-4o-transcribe"},"groq":{"api_key":"","model":"whisper-large-v3"},"local":{"model_path":null}},"language":"","fallback_order":["openai","groq"]}"#;
        let config: TranscriptionConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.providers.openai.api_key, "sk-abc");
        assert_eq!(config.providers.openai.model, "gpt-4o-transcribe");
    }

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
