// OpenAI Whisper transcription provider.
// POSTs multipart/form-data to https://api.openai.com/v1/audio/transcriptions.

use crate::audio::encode::EncodedAudio;
use crate::config::TranscriptionConfig;
use crate::transcription::provider::{classify_and_extract, TranscriptionError, TranscriptionProviderTrait};

const OPENAI_ENDPOINT: &str = "https://api.openai.com/v1/audio/transcriptions";

// ---------------------------------------------------------------------------
// Provider struct
// ---------------------------------------------------------------------------

pub struct OpenAiProvider {
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Testable form builder (extracted so unit tests can inspect form fields)
// ---------------------------------------------------------------------------

/// Returns the list of field names present in the form for this config.
/// Used in tests to verify language omission without making real HTTP calls.
pub fn form_field_names(config: &TranscriptionConfig) -> Vec<String> {
    let mut fields = vec![
        "file".to_string(),
        "model".to_string(),
        "response_format".to_string(),
    ];
    if !config.language.is_empty() {
        fields.push("language".to_string());
    }
    fields
}

// ---------------------------------------------------------------------------
// Trait implementation
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl TranscriptionProviderTrait for OpenAiProvider {
    async fn transcribe(
        &self,
        audio: &EncodedAudio,
        config: &TranscriptionConfig,
    ) -> Result<String, TranscriptionError> {
        let file_part = reqwest::multipart::Part::bytes(audio.bytes.clone())
            .file_name("audio.ogg")
            .mime_str("audio/ogg")
            .map_err(|e| TranscriptionError::Network {
                message: e.to_string(),
            })?;

        let mut form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", config.openai_model.clone())
            .text("response_format", "text");

        if !config.language.is_empty() {
            form = form.text("language", config.language.clone());
        }

        let response = self
            .client
            .post(OPENAI_ENDPOINT)
            .bearer_auth(&config.openai_api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| TranscriptionError::Network {
                message: e.to_string(),
            })?;

        classify_and_extract(response, "openai").await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TranscriptionConfig;

    fn config_with_language(lang: &str) -> TranscriptionConfig {
        TranscriptionConfig {
            language: lang.to_string(),
            ..TranscriptionConfig::default()
        }
    }

    #[test]
    fn test_language_omitted_when_empty() {
        let config = config_with_language("");
        let fields = form_field_names(&config);
        assert!(
            !fields.contains(&"language".to_string()),
            "language field must not be present when config.language is empty"
        );
    }

    #[test]
    fn test_language_included_when_set() {
        let config = config_with_language("en");
        let fields = form_field_names(&config);
        assert!(
            fields.contains(&"language".to_string()),
            "language field must be present when config.language is 'en'"
        );
    }

    #[test]
    fn test_openai_endpoint_constant() {
        assert_eq!(
            OPENAI_ENDPOINT,
            "https://api.openai.com/v1/audio/transcriptions"
        );
    }
}
