// Groq Whisper transcription provider.
// POSTs multipart/form-data to https://api.groq.com/openai/v1/audio/transcriptions.
// Shape is identical to OpenAI; only the endpoint, model field, and API key differ.

use crate::audio::encode::EncodedAudio;
use crate::config::TranscriptionConfig;
use crate::transcription::provider::{classify_and_extract, TranscriptionError, TranscriptionProviderTrait};

const GROQ_ENDPOINT: &str = "https://api.groq.com/openai/v1/audio/transcriptions";

// ---------------------------------------------------------------------------
// Provider struct
// ---------------------------------------------------------------------------

pub struct GroqProvider {
    client: reqwest::Client,
}

impl GroqProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Testable form builder
// ---------------------------------------------------------------------------

/// Returns the list of field names present in the form for this config.
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
impl TranscriptionProviderTrait for GroqProvider {
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
            .text("model", config.providers.groq.model.clone())
            .text("response_format", "text");

        if !config.language.is_empty() {
            form = form.text("language", config.language.clone());
        }

        let response = self
            .client
            .post(GROQ_ENDPOINT)
            .bearer_auth(&config.providers.groq.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| TranscriptionError::Network {
                message: e.to_string(),
            })?;

        classify_and_extract(response, "groq").await
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
        let config = config_with_language("hu");
        let fields = form_field_names(&config);
        assert!(
            fields.contains(&"language".to_string()),
            "language field must be present when config.language is 'hu'"
        );
    }

    #[test]
    fn test_groq_endpoint_constant() {
        assert_eq!(
            GROQ_ENDPOINT,
            "https://api.groq.com/openai/v1/audio/transcriptions"
        );
    }
}
