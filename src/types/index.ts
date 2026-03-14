// TypeScript types that mirror Rust structs exactly.
// Keep in sync with src-tauri/src/config/mod.rs and src-tauri/src/state.rs.

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

export type InjectionMode = "flash_paste" | "keystroke" | "clipboard";

export type TranscriptionProvider = "openai" | "groq" | "openrouter" | "local";

export interface AppConfig {
  // Hotkey
  hotkey: string;

  // Audio
  audio_device: string | null;

  // Transcription
  transcription_provider: TranscriptionProvider;
  openai_api_key: string;
  groq_api_key: string;
  openrouter_api_key: string;
  openai_model: string;
  groq_model: string;
  openrouter_model: string;

  // Injection
  injection_mode: InjectionMode;

  // UI
  show_indicator: boolean;
  launch_at_login: boolean;
}

// ---------------------------------------------------------------------------
// Recording state
// ---------------------------------------------------------------------------

export type RecordingState = "idle" | "recording" | "transcribing";

// ---------------------------------------------------------------------------
// IPC event payloads (Rust → Frontend)
// ---------------------------------------------------------------------------

export interface AudioLevelPayload {
  rms: number; // 0.0 – 1.0
}

export interface InjectionDonePayload {
  success: boolean;
  error: string | null;
}

export interface StateChangedPayload {
  state: RecordingState;
}
