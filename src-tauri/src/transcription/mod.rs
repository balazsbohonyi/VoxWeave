// Transcription module — provider trait, error types, cloud provider impls, and service layer.
// The service module (plan 02) is the single entry point consumed by the hotkey pipeline.
#![allow(dead_code, unused_imports)]

pub mod groq;
pub mod openai;
pub mod openrouter;
pub mod provider;
pub mod service;

pub use provider::TranscriptionError;
pub use provider::TranscriptionProviderTrait;
pub use service::transcribe_with_provider;
pub use service::transcribe_with_retry;
