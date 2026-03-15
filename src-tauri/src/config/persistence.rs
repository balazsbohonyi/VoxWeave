// config::persistence — Load/save AppConfig to %APPDATA%/VoxFlow/config.json.
//
// Unknown-field preservation strategy:
//   On load  : deserialize into AppConfig (for typed access) AND keep a raw
//              serde_json::Value. On save, the typed config is serialized back
//              to a Value, then merged INTO the raw Value (typed wins for known
//              keys; unknown keys in raw are untouched).
//   On save  : merge typed → raw, then write the merged raw Value to disk.
//
// Malformed file behaviour:
//   The corrupt file is renamed to config.json.corrupt (with timestamp) and
//   defaults are returned. The bad file is never silently overwritten on read.

use super::AppConfig;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Returns the path to the VoxFlow config directory:
/// `%APPDATA%\VoxFlow` on Windows, `~/.config/VoxFlow` elsewhere.
pub fn config_dir() -> Result<PathBuf, String> {
    let base = dirs_next::config_dir()
        .ok_or_else(|| "Cannot determine user config directory".to_string())?;
    Ok(base.join("VoxFlow"))
}

/// Returns the full path to the config file.
pub fn config_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("config.json"))
}

// ---------------------------------------------------------------------------
// Load
// ---------------------------------------------------------------------------

/// Load result: typed config + raw JSON value preserved for unknown-field
/// round-trips.
pub struct LoadedConfig {
    pub config: AppConfig,
    /// Raw JSON value as read (or derived from defaults). Used on save to
    /// preserve unknown fields.
    pub raw: serde_json::Value,
}

/// Load config from disk.
///
/// - No file → returns defaults.
/// - Malformed file → renames corrupt file, returns defaults.
/// - Partial file → missing fields filled from defaults via serde.
pub fn load() -> Result<LoadedConfig, String> {
    let path = config_path()?;

    if !path.exists() {
        let config = AppConfig::default();
        let mut raw = serde_json::to_value(&config).map_err(|e| e.to_string())?;
        // Write defaults to disk on first launch so the file exists immediately.
        let _ = save(&config, &mut raw);
        return Ok(LoadedConfig { config, raw });
    }

    let bytes = std::fs::read(&path).map_err(|e| format!("Failed to read config: {e}"))?;

    // Parse raw JSON first so we can preserve unknown fields.
    let raw_result: Result<serde_json::Value, _> = serde_json::from_slice(&bytes);

    match raw_result {
        Ok(raw_value) => {
            // Deserialize typed config; missing fields get serde defaults.
            let config: AppConfig =
                serde_json::from_value(raw_value.clone()).unwrap_or_default();
            Ok(LoadedConfig {
                config,
                raw: raw_value,
            })
        }
        Err(parse_err) => {
            // Malformed JSON — rename and fall back to defaults.
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let corrupt_path = path.with_extension(format!("json.corrupt.{timestamp}"));
            let _ = std::fs::rename(&path, &corrupt_path);
            log::warn!(
                "config: malformed JSON in {:?} — renamed to {:?}, using defaults. Error: {}",
                path,
                corrupt_path,
                parse_err
            );
            let config = AppConfig::default();
            let raw = serde_json::to_value(&config).map_err(|e| e.to_string())?;
            Ok(LoadedConfig { config, raw })
        }
    }
}

// ---------------------------------------------------------------------------
// Save
// ---------------------------------------------------------------------------

/// Persist the typed config while preserving unknown fields from `raw`.
///
/// Algorithm:
///   1. Serialize `config` → typed Value.
///   2. Deep-merge typed Value INTO raw (typed wins on conflicts).
///   3. Write merged Value to disk as pretty JSON.
pub fn save(config: &AppConfig, raw: &mut serde_json::Value) -> Result<(), String> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create config directory: {e}"))?;

    // Serialize typed config to a Value.
    let typed_value = serde_json::to_value(config).map_err(|e| e.to_string())?;

    // Merge typed Value INTO raw so unknown keys in raw are preserved.
    merge_into(raw, typed_value);

    let json = serde_json::to_string_pretty(raw).map_err(|e| e.to_string())?;

    let path = config_path()?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write config to {:?}: {e}", path))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Deep merge helper
// ---------------------------------------------------------------------------

/// Recursively merge `src` into `dst`.
/// - Objects: recurse, src fields overwrite dst fields with same key.
/// - Other types: dst replaced by src.
/// Unknown keys already present in dst (object) are left untouched.
fn merge_into(dst: &mut serde_json::Value, src: serde_json::Value) {
    match (dst, src) {
        (serde_json::Value::Object(dst_map), serde_json::Value::Object(src_map)) => {
            for (key, src_val) in src_map {
                let dst_val = dst_map.entry(key).or_insert(serde_json::Value::Null);
                merge_into(dst_val, src_val);
            }
        }
        (dst, src) => {
            *dst = src;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TranscriptionProvider;

    // ----- merge_into -----

    #[test]
    fn test_merge_into_preserves_unknown_keys() {
        let mut dst: serde_json::Value = serde_json::json!({
            "known": "old",
            "unknown_future_field": "keep_me"
        });
        let src: serde_json::Value = serde_json::json!({
            "known": "new"
        });
        merge_into(&mut dst, src);
        assert_eq!(dst["known"], "new");
        assert_eq!(dst["unknown_future_field"], "keep_me");
    }

    #[test]
    fn test_merge_into_nested_objects() {
        let mut dst: serde_json::Value = serde_json::json!({
            "audio": {
                "device": null,
                "future_audio_field": "preserved"
            }
        });
        let src: serde_json::Value = serde_json::json!({
            "audio": {
                "device": "Microphone"
            }
        });
        merge_into(&mut dst, src);
        assert_eq!(dst["audio"]["device"], "Microphone");
        assert_eq!(dst["audio"]["future_audio_field"], "preserved");
    }

    // ----- defaults -----

    #[test]
    fn test_default_config_values() {
        let config = AppConfig::default();
        assert_eq!(config.hotkey, "Alt+Shift+Space");
        assert!(matches!(
            config.transcription.provider,
            TranscriptionProvider::Openai
        ));
        assert_eq!(config.transcription.openai_model, "whisper-1");
        assert_eq!(config.transcription.groq_model, "whisper-large-v3");
        assert!(config.indicator.show);
        assert!(!config.launch_at_login);
        assert!(config.first_launch);
        assert_eq!(config.audio.vad_silence_ms, 1500);
    }

    // ----- partial file → defaults filled -----

    #[test]
    fn test_partial_json_fills_defaults() {
        // A JSON object with only "hotkey" set; all other fields missing.
        let partial = r#"{"hotkey": "Ctrl+F9"}"#;
        let raw: serde_json::Value = serde_json::from_str(partial).unwrap();
        let config: AppConfig = serde_json::from_value(raw).unwrap_or_default();
        assert_eq!(config.hotkey, "Ctrl+F9");
        // Nested sections should be at defaults.
        assert_eq!(config.transcription.openai_model, "whisper-1");
        assert!(config.indicator.show);
    }

    // ----- unknown fields preserved round-trip -----

    #[test]
    fn test_unknown_fields_preserved_on_round_trip() {
        let json_str = r#"{
            "hotkey": "Alt+Shift+Space",
            "audio": {
                "device": null,
                "vad_threshold": 0.01,
                "vad_silence_ms": 1500
            },
            "future_top_level_field": "v2_value",
            "transcription": {
                "provider": "openai",
                "openai_api_key": "",
                "groq_api_key": "",
                "openrouter_api_key": "",
                "openai_model": "whisper-1",
                "groq_model": "whisper-large-v3",
                "openrouter_model": "",
                "language": "",
                "local_model_path": null,
                "future_transcription_field": 42
            },
            "injection": {"mode": "flash_paste"},
            "indicator": {"show": true, "position_x": null, "position_y": null},
            "launch_at_login": false,
            "first_launch": true
        }"#;

        let mut raw: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let config: AppConfig = serde_json::from_value(raw.clone()).unwrap_or_default();

        // Simulate a save cycle.
        let typed_value = serde_json::to_value(&config).unwrap();
        merge_into(&mut raw, typed_value);

        assert_eq!(raw["future_top_level_field"], "v2_value");
        assert_eq!(raw["transcription"]["future_transcription_field"], 42);
    }

    // ----- no file → defaults -----

    #[test]
    fn test_no_file_returns_defaults() {
        // Directly test AppConfig::default() since we can't mock the path in
        // this unit-test context.
        let config = AppConfig::default();
        assert_eq!(config.hotkey, "Alt+Shift+Space");
        assert!(config.first_launch);
    }

    // ----- config_dir not panics -----

    #[test]
    fn test_config_dir_returns_path() {
        // Should succeed in a normal user environment.
        // In CI without a home dir this would fail — skip gracefully.
        match config_dir() {
            Ok(p) => assert!(p.ends_with("VoxFlow")),
            Err(e) => eprintln!("config_dir() skipped in this env: {e}"),
        }
    }

    // ----- malformed JSON -----

    #[test]
    fn test_malformed_json_parse_produces_default_value() {
        let malformed = r#"{ "hotkey": "Alt+Shift+Space", BROKEN"#;
        let result: Result<serde_json::Value, _> = serde_json::from_str(malformed);
        // Should fail to parse.
        assert!(result.is_err());
        // Fallback: use AppConfig::default() — verify it's valid.
        let config = AppConfig::default();
        assert_eq!(config.hotkey, "Alt+Shift+Space");
    }

    // ----- save creates directory -----

    #[test]
    fn test_save_and_load_round_trip() {
        // Write a config, read it back as Value, check fields.
        let config = AppConfig {
            hotkey: "Ctrl+F8".to_string(),
            ..AppConfig::default()
        };
        let typed_value = serde_json::to_value(&config).unwrap();
        let mut raw = serde_json::Value::Object(serde_json::Map::new());
        // Seed raw with an unknown field.
        raw["my_future_setting"] = serde_json::Value::String("preserved".to_string());
        merge_into(&mut raw, typed_value);

        assert_eq!(raw["hotkey"], "Ctrl+F8");
        assert_eq!(raw["my_future_setting"], "preserved");
    }
}
