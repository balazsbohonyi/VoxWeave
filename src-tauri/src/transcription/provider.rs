// TranscriptionProvider trait + error types for cloud transcription.
// The classify_and_extract helper is called by all provider impls.

use crate::audio::encode::EncodedAudio;
use crate::config::TranscriptionConfig;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum TranscriptionError {
    /// API key missing or rejected (HTTP 401/403).
    InvalidKey { provider: String },
    /// Rate-limited (HTTP 429).
    RateLimit,
    /// Network-level failure or unexpected state.
    Network { message: String },
    /// Non-retryable server error.
    Server { status: u16, message: String },
    /// Transcription was cancelled by the user.
    Cancelled,
}

// ---------------------------------------------------------------------------
// Helper: map HTTP status → error variant
// ---------------------------------------------------------------------------

/// Pure function for unit testing status-to-error mapping without HTTP mocks.
pub fn status_to_error(status: u16, provider_name: &str, body: &str) -> TranscriptionError {
    match status {
        401 | 403 => TranscriptionError::InvalidKey {
            provider: provider_name.to_string(),
        },
        429 => TranscriptionError::RateLimit,
        500..=599 => TranscriptionError::Server {
            status,
            message: body.to_string(),
        },
        _ => TranscriptionError::Network {
            message: format!("Unexpected status {status}: {body}"),
        },
    }
}

/// Inspect a reqwest Response: 200 → extract text body, otherwise map to error.
pub async fn classify_and_extract(
    response: reqwest::Response,
    provider_name: &str,
) -> Result<String, TranscriptionError> {
    let status = response.status().as_u16();
    if status == 200 {
        response
            .text()
            .await
            .map_err(|e| TranscriptionError::Network {
                message: e.to_string(),
            })
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(status_to_error(status, provider_name, &body))
    }
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait TranscriptionProviderTrait: Send + Sync {
    async fn transcribe(
        &self,
        audio: &EncodedAudio,
        config: &TranscriptionConfig,
    ) -> Result<String, TranscriptionError>;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{TranscriptionConfig, TranscriptionProvider};

    #[test]
    fn test_invalid_key_classified_401() {
        let err = status_to_error(401, "openai", "Unauthorized");
        assert!(matches!(err, TranscriptionError::InvalidKey { provider } if provider == "openai"));
    }

    #[test]
    fn test_invalid_key_classified_403() {
        let err = status_to_error(403, "groq", "Forbidden");
        assert!(matches!(err, TranscriptionError::InvalidKey { provider } if provider == "groq"));
    }

    #[test]
    fn test_rate_limit_classified() {
        let err = status_to_error(429, "openai", "Too Many Requests");
        assert!(matches!(err, TranscriptionError::RateLimit));
    }

    #[test]
    fn test_server_error_classified() {
        let err = status_to_error(503, "openai", "Service Unavailable");
        assert!(
            matches!(err, TranscriptionError::Server { status, .. } if status == 503)
        );
    }

    #[test]
    fn test_fallback_order_default() {
        let config: TranscriptionConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(
            config.fallback_order,
            vec![
                TranscriptionProvider::Openai,
                TranscriptionProvider::Groq,
            ]
        );
    }
}
