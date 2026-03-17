// Transcription module — provider trait, error types, and cloud provider impls.
// Items here are infrastructure consumed by the service layer in plan 02.
#![allow(dead_code, unused_imports)]

pub mod groq;
pub mod openai;
pub mod openrouter;
pub mod provider;

pub use provider::TranscriptionError;
pub use provider::TranscriptionProviderTrait;
