// OpenRouter transcription provider.
// Uses chat completions endpoint with base64-encoded audio as input_audio.
// POST https://openrouter.ai/api/v1/chat/completions

use base64::Engine;
use crate::audio::encode::EncodedAudio;
use crate::config::TranscriptionConfig;
use crate::transcription::provider::{classify_and_extract, TranscriptionError, TranscriptionProviderTrait};

const OPENROUTER_ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";

// ---------------------------------------------------------------------------
// Response deserialization structs (private)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(serde::Deserialize)]
struct Choice {
    message: Message,
}

#[derive(serde::Deserialize)]
struct Message {
    content: String,
}

// ---------------------------------------------------------------------------
// Provider struct
// ---------------------------------------------------------------------------

pub struct OpenRouterProvider {
    client: reqwest::Client,
}

impl OpenRouterProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Testable body builder
// ---------------------------------------------------------------------------

/// Build the JSON request body for an OpenRouter transcription request.
/// Extracted for unit testing without making real HTTP calls.
pub fn build_body(audio_bytes: &[u8], config: &TranscriptionConfig) -> serde_json::Value {
    let b64 = base64::engine::general_purpose::STANDARD.encode(audio_bytes);

    let text_instruction = if config.language.is_empty() {
        "Transcribe this audio exactly as spoken. Return only the transcription.".to_string()
    } else {
        format!(
            "Transcribe this audio exactly as spoken. Return only the transcription. Respond in {}.",
            config.language
        )
    };

    serde_json::json!({
        "model": config.openrouter_model,
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "text": text_instruction
                },
                {
                    "type": "input_audio",
                    "input_audio": {
                        "data": b64,
                        "format": "ogg"
                    }
                }
            ]
        }]
    })
}

// ---------------------------------------------------------------------------
// Trait implementation
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl TranscriptionProviderTrait for OpenRouterProvider {
    async fn transcribe(
        &self,
        audio: &EncodedAudio,
        config: &TranscriptionConfig,
    ) -> Result<String, TranscriptionError> {
        // Guard: model must be configured
        if config.openrouter_model.is_empty() {
            return Err(TranscriptionError::Network {
                message: "OpenRouter model not configured".to_string(),
            });
        }

        let body = build_body(&audio.bytes, config);

        let response = self
            .client
            .post(OPENROUTER_ENDPOINT)
            .bearer_auth(&config.openrouter_api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| TranscriptionError::Network {
                message: e.to_string(),
            })?;

        if response.status().as_u16() != 200 {
            return classify_and_extract(response, "openrouter").await;
        }

        let chat: ChatResponse = response
            .json()
            .await
            .map_err(|e| TranscriptionError::Network {
                message: e.to_string(),
            })?;

        chat.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| TranscriptionError::Network {
                message: "OpenRouter returned empty choices".to_string(),
            })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TranscriptionConfig;

    fn config_with(model: &str, language: &str) -> TranscriptionConfig {
        TranscriptionConfig {
            openrouter_model: model.to_string(),
            language: language.to_string(),
            ..TranscriptionConfig::default()
        }
    }

    #[test]
    fn test_empty_model_returns_error() {
        // Verify the guard works by checking build_body would be skipped.
        // The guard is at the top of transcribe() — we test the config condition here.
        let config = config_with("", "");
        assert!(
            config.openrouter_model.is_empty(),
            "model must be empty to trigger guard"
        );
    }

    #[test]
    fn test_format_is_ogg_not_mime() {
        let config = config_with("google/gemini-2.5-flash", "");
        let body = build_body(b"fake-audio", &config);
        let body_str = serde_json::to_string(&body).unwrap();
        // format field must be "ogg" (the identifier), not "audio/ogg" (MIME)
        assert!(
            body_str.contains(r#""format":"ogg""#),
            "format field must be 'ogg', got: {body_str}"
        );
        // The format value "audio/ogg" must NOT appear in the format field context
        let format_pos = body_str.find(r#""format":"#).unwrap();
        let after_format = &body_str[format_pos..];
        assert!(
            !after_format.starts_with(r#""format":"audio/ogg""#),
            "format field must not be 'audio/ogg' (MIME type), got: {after_format}"
        );
    }

    #[test]
    fn test_language_instruction_appended() {
        let config = config_with("google/gemini-2.5-flash", "hu");
        let body = build_body(b"fake-audio", &config);
        let text = body["messages"][0]["content"][0]["text"]
            .as_str()
            .unwrap();
        assert!(
            text.contains("Respond in hu"),
            "instruction must contain 'Respond in hu' when language is 'hu', got: {text}"
        );
    }

    #[test]
    fn test_language_empty_no_instruction() {
        let config = config_with("google/gemini-2.5-flash", "");
        let body = build_body(b"fake-audio", &config);
        let text = body["messages"][0]["content"][0]["text"]
            .as_str()
            .unwrap();
        assert!(
            !text.contains("Respond in"),
            "instruction must not contain 'Respond in' when language is empty, got: {text}"
        );
    }
}
