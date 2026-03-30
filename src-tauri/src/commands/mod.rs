// Thin Tauri command handlers — no business logic here.
// Commands delegate to config/state; they never own data.

pub mod audio;
pub mod config;
pub mod download;
pub mod indicator;
pub mod transcription;
pub mod wizard;
