// Transcription service — orchestration layer wrapping provider HTTP calls
// with retry logic, exponential backoff, error classification, and event emission.
//
// This module is the single entry point consumed by the hotkey pipeline (plan 03).
// It never holds a MutexGuard across an .await point.

use crate::audio::encode::EncodedAudio;
use crate::config::{TranscriptionConfig, TranscriptionProvider};
use crate::indicator;
use crate::transcription::groq::GroqProvider;
use crate::transcription::openai::OpenAiProvider;
use crate::transcription::openrouter::OpenRouterProvider;
use crate::transcription::provider::{TranscriptionError, TranscriptionProviderTrait};
use tauri::Emitter;

// ---------------------------------------------------------------------------
// Event name constants
// ---------------------------------------------------------------------------

pub const TRANSCRIPTION_ERROR_EVENT: &str = "transcription-error";
pub const TRANSCRIPTION_DONE_EVENT: &str = "transcription-done";

// ---------------------------------------------------------------------------
// Payload types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionErrorCode {
    InvalidKey,
    RateLimit,
    Network,
    Server,
    Cancelled,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionErrorPayload {
    pub code: TranscriptionErrorCode,
    pub message: String,
    /// Name of the provider that failed (e.g. "openai", "groq").
    pub provider: Option<String>,
    /// Name of the next provider that can be tried, if any.
    pub fallback_provider: Option<String>,
    /// Whether the frontend may offer a manual retry.
    pub retryable: bool,
}

// ---------------------------------------------------------------------------
// Provider dispatch
// ---------------------------------------------------------------------------

/// Instantiate a concrete provider implementation for the given config variant.
pub fn make_provider(p: &TranscriptionProvider) -> Box<dyn TranscriptionProviderTrait> {
    match p {
        TranscriptionProvider::Openai => Box::new(OpenAiProvider::new()),
        TranscriptionProvider::Groq => Box::new(GroqProvider::new()),
        TranscriptionProvider::Openrouter => Box::new(OpenRouterProvider::new()),
        TranscriptionProvider::Local => {
            unimplemented!("local transcription is phase 10")
        }
    }
}

// ---------------------------------------------------------------------------
// Fallback provider selection
// ---------------------------------------------------------------------------

/// Walk config.fallback_order and return the name of the first provider
/// that is (a) not the current primary provider and (b) has a non-empty API key.
/// Returns None when no eligible fallback exists.
pub fn find_fallback_provider(
    current: &TranscriptionProvider,
    config: &TranscriptionConfig,
) -> Option<String> {
    for candidate in &config.fallback_order {
        // Skip the provider that just failed.
        if candidate == current {
            continue;
        }

        // Skip providers that have no configured API key.
        let has_key = match candidate {
            TranscriptionProvider::Openai => !config.openai_api_key.is_empty(),
            TranscriptionProvider::Groq => !config.groq_api_key.is_empty(),
            TranscriptionProvider::Openrouter => !config.openrouter_api_key.is_empty(),
            TranscriptionProvider::Local => false,
        };

        if has_key {
            return Some(provider_display_name(candidate));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Pure retry helper (testable without AppHandle)
// ---------------------------------------------------------------------------

/// Drive a single transcription attempt up to `max_retries` times.
///
/// `attempt_fn` receives the current zero-based attempt index and returns a
/// future that resolves to `Result<String, TranscriptionError>`.
///
/// Retryable errors (RateLimit, Network, Server) cause an exponential sleep
/// (1s / 2s / 4s for attempts 0 / 1 / 2) before the next iteration.
/// Non-retryable errors (InvalidKey, Cancelled) short-circuit immediately.
///
/// After `max_retries` exhausted attempts the last error is returned.
pub(crate) async fn run_with_retry_inner<F, Fut>(
    attempt_fn: F,
    max_retries: u32,
) -> Result<String, TranscriptionError>
where
    F: Fn(u32) -> Fut,
    Fut: std::future::Future<Output = Result<String, TranscriptionError>>,
{
    let mut attempt = 0u32;
    loop {
        match attempt_fn(attempt).await {
            Ok(text) => return Ok(text),
            Err(TranscriptionError::InvalidKey { provider }) => {
                return Err(TranscriptionError::InvalidKey { provider });
            }
            Err(TranscriptionError::Cancelled) => {
                return Err(TranscriptionError::Cancelled);
            }
            Err(_err) if attempt < max_retries => {
                let delay_ms = 1000u64 * (1u64 << attempt);
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    }
}

// ---------------------------------------------------------------------------
// Internal emit helper
// ---------------------------------------------------------------------------

/// Shows the toast window and delivers the payload via eval (not Tauri events).
/// WebView2 may not deliver events to hidden windows, so the payload is pushed
/// via `window.__voxflowShowToast` after the window is made visible.
fn emit_transcription_error<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    payload: TranscriptionErrorPayload,
) {
    let _ = indicator::show_toast_window(app, &payload);
}

// ---------------------------------------------------------------------------
// Public transcription entry point
// ---------------------------------------------------------------------------

/// Transcribe audio with retry/backoff and emit Tauri events for every outcome.
///
/// Callers (hotkey service, manual retry command) use this as the single entry
/// point. The function never blocks the Tokio runtime with CPU work.
///
/// # Return value
/// `Ok(text)` when transcription succeeds (TRANSCRIPTION_DONE_EVENT also emitted).
/// `Err(())` for any failure (TRANSCRIPTION_ERROR_EVENT emitted before returning).
pub async fn transcribe_with_retry<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    audio: &EncodedAudio,
) -> Result<String, ()> {
    use crate::state::AppState;
    use tauri::Manager;

    // Clone config before first .await — never hold a MutexGuard across await.
    // Explicit intermediate binding forces the MutexGuard to drop before the
    // block end, satisfying the borrow checker without holding across .await.
    let config = {
        let state = app.state::<AppState>();
        let guard = state.config.lock().unwrap();
        guard.transcription.clone()
    };

    let provider_impl = make_provider(&config.provider);
    let mut attempt = 0u32;

    loop {
        // Check cancel flag before each attempt and clear it on detection.
        {
            let state = app.state::<AppState>();
            let mut flag = state.cancel_flag.lock().unwrap();
            if *flag {
                *flag = false;
                let _ = app.emit(
                    TRANSCRIPTION_ERROR_EVENT,
                    TranscriptionErrorPayload {
                        code: TranscriptionErrorCode::Cancelled,
                        message: "Transcription cancelled.".into(),
                        provider: None,
                        fallback_provider: None,
                        retryable: false,
                    },
                );
                return Err(());
            }
        }

        match provider_impl.transcribe(audio, &config).await {
            Ok(text) => {
                let _ = app.emit(TRANSCRIPTION_DONE_EVENT, text.clone());
                return Ok(text);
            }

            Err(TranscriptionError::InvalidKey { provider }) => {
                emit_transcription_error(app, TranscriptionErrorPayload {
                    code: TranscriptionErrorCode::InvalidKey,
                    message: "Invalid API key. Open settings to fix.".into(),
                    provider: Some(provider),
                    fallback_provider: None,
                    retryable: false,
                });
                return Err(());
            }

            Err(TranscriptionError::Cancelled) => {
                let _ = app.emit(
                    TRANSCRIPTION_ERROR_EVENT,
                    TranscriptionErrorPayload {
                        code: TranscriptionErrorCode::Cancelled,
                        message: "Transcription cancelled.".into(),
                        provider: None,
                        fallback_provider: None,
                        retryable: false,
                    },
                );
                return Err(());
            }

            // Retryable error and still within retry budget.
            Err(_err) if attempt < 3 => {
                let delay_ms = 1000u64 * (1u64 << attempt);
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                attempt += 1;
            }

            // Retries exhausted — emit final error with optional fallback hint.
            Err(err) => {
                let fallback = find_fallback_provider(&config.provider, &config);
                let (code, message) = match &err {
                    TranscriptionError::RateLimit => (
                        TranscriptionErrorCode::RateLimit,
                        "Rate limit reached after 3 attempts.".into(),
                    ),
                    TranscriptionError::Network { .. } => (
                        TranscriptionErrorCode::Network,
                        "Network error. Check your connection and try again.".into(),
                    ),
                    TranscriptionError::Server { status, message } => (
                        TranscriptionErrorCode::Server,
                        format!("Server error {status}: {message}"),
                    ),
                    // InvalidKey and Cancelled are handled above — this branch is
                    // unreachable in practice but must be exhaustive.
                    _ => (
                        TranscriptionErrorCode::Network,
                        "Unknown transcription error.".into(),
                    ),
                };
                emit_transcription_error(app, TranscriptionErrorPayload {
                    code,
                    message,
                    provider: None,
                    fallback_provider: fallback,
                    retryable: true,
                });
                return Err(());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Single-provider override entry point (used by fallback command)
// ---------------------------------------------------------------------------

/// Transcribe audio using a specific provider, overriding the configured active provider
/// for this call only. Does not mutate AppState config. Used by the fallback flow.
pub async fn transcribe_with_provider<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    audio: &EncodedAudio,
    target_provider: TranscriptionProvider,
) -> Result<String, ()> {
    use crate::state::AppState;
    use tauri::Manager;

    // Clone config and override provider for this single invocation
    let mut call_config = {
        let state = app.state::<AppState>();
        let guard = state.config.lock().unwrap();
        guard.transcription.clone()
    };
    call_config.provider = target_provider;

    // Delegate to the provider implementation directly (no retry — fallback gets one attempt)
    let provider_impl = make_provider(&call_config.provider);
    match provider_impl.transcribe(audio, &call_config).await {
        Ok(text) => {
            let _ = app.emit(TRANSCRIPTION_DONE_EVENT, text.clone());
            Ok(text)
        }
        Err(err) => {
            // Emit error with no further fallback_provider (fallback already tried)
            let (code, message) = match &err {
                crate::transcription::provider::TranscriptionError::InvalidKey { provider } => (
                    TranscriptionErrorCode::InvalidKey,
                    format!("Invalid API key for {provider}."),
                ),
                crate::transcription::provider::TranscriptionError::RateLimit => {
                    (TranscriptionErrorCode::RateLimit, "Rate limit reached.".into())
                }
                crate::transcription::provider::TranscriptionError::Network { .. } => (
                    TranscriptionErrorCode::Network,
                    "Network error. Check your connection and try again.".into(),
                ),
                crate::transcription::provider::TranscriptionError::Server { status, message } => (
                    TranscriptionErrorCode::Server,
                    format!("Server error {status}: {message}"),
                ),
                crate::transcription::provider::TranscriptionError::Cancelled => {
                    (TranscriptionErrorCode::Cancelled, "Transcription cancelled.".into())
                }
            };
            emit_transcription_error(app, TranscriptionErrorPayload {
                code,
                message,
                provider: Some(provider_display_name(&call_config.provider)),
                fallback_provider: None,
                retryable: false,
            });
            Err(())
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helper
// ---------------------------------------------------------------------------

fn provider_display_name(p: &TranscriptionProvider) -> String {
    match p {
        TranscriptionProvider::Openai => "openai".to_string(),
        TranscriptionProvider::Groq => "groq".to_string(),
        TranscriptionProvider::Openrouter => "openrouter".to_string(),
        TranscriptionProvider::Local => "local".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{TranscriptionConfig, TranscriptionProvider};

    // -----------------------------------------------------------------------
    // find_fallback_provider tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_find_fallback_skips_current() {
        let config = TranscriptionConfig {
            fallback_order: vec![TranscriptionProvider::Openai, TranscriptionProvider::Groq],
            groq_api_key: "sk-groq-key".to_string(),
            ..TranscriptionConfig::default()
        };
        let result = find_fallback_provider(&TranscriptionProvider::Openai, &config);
        assert_eq!(result, Some("groq".to_string()));
    }

    #[test]
    fn test_find_fallback_skips_empty_key() {
        let config = TranscriptionConfig {
            fallback_order: vec![
                TranscriptionProvider::Openai,
                TranscriptionProvider::Groq,
                TranscriptionProvider::Openrouter,
            ],
            // Groq key is empty — should be skipped.
            groq_api_key: "".to_string(),
            openrouter_api_key: "or-key".to_string(),
            ..TranscriptionConfig::default()
        };
        let result = find_fallback_provider(&TranscriptionProvider::Openai, &config);
        assert_eq!(result, Some("openrouter".to_string()));
    }

    #[test]
    fn test_find_fallback_none_available() {
        let config = TranscriptionConfig {
            fallback_order: vec![TranscriptionProvider::Openai, TranscriptionProvider::Groq],
            // All keys empty.
            openai_api_key: "".to_string(),
            groq_api_key: "".to_string(),
            ..TranscriptionConfig::default()
        };
        let result = find_fallback_provider(&TranscriptionProvider::Openai, &config);
        assert_eq!(result, None);
    }

    // -----------------------------------------------------------------------
    // run_with_retry_inner tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_retry_three_times_on_rate_limit() {
        use std::sync::{Arc, Mutex};

        let call_count = Arc::new(Mutex::new(0u32));
        let call_count_clone = call_count.clone();

        let result = run_with_retry_inner(
            move |_attempt| {
                let cc = call_count_clone.clone();
                async move {
                    *cc.lock().unwrap() += 1;
                    Err::<String, _>(TranscriptionError::RateLimit)
                }
            },
            3,
        )
        .await;

        // Should have tried once + 3 retries = 4 total calls.
        assert_eq!(*call_count.lock().unwrap(), 4);
        assert!(matches!(result, Err(TranscriptionError::RateLimit)));
    }

    #[tokio::test]
    async fn test_success_on_second_attempt() {
        use std::sync::{Arc, Mutex};

        let call_count = Arc::new(Mutex::new(0u32));
        let call_count_clone = call_count.clone();

        let result = run_with_retry_inner(
            move |_attempt| {
                let cc = call_count_clone.clone();
                async move {
                    let n = {
                        let mut guard = cc.lock().unwrap();
                        *guard += 1;
                        *guard
                    };
                    if n == 1 {
                        Err(TranscriptionError::RateLimit)
                    } else {
                        Ok("hello world".to_string())
                    }
                }
            },
            3,
        )
        .await;

        assert_eq!(*call_count.lock().unwrap(), 2);
        assert_eq!(result.unwrap(), "hello world");
    }

    #[tokio::test]
    async fn test_invalid_key_no_retry() {
        use std::sync::{Arc, Mutex};

        let call_count = Arc::new(Mutex::new(0u32));
        let call_count_clone = call_count.clone();

        let result = run_with_retry_inner(
            move |_attempt| {
                let cc = call_count_clone.clone();
                async move {
                    *cc.lock().unwrap() += 1;
                    Err::<String, _>(TranscriptionError::InvalidKey {
                        provider: "openai".to_string(),
                    })
                }
            },
            3,
        )
        .await;

        // attempt_fn must have been called exactly once — no retries.
        assert_eq!(*call_count.lock().unwrap(), 1);
        assert!(matches!(
            result,
            Err(TranscriptionError::InvalidKey { provider }) if provider == "openai"
        ));
    }
}
