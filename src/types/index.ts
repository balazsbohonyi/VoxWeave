// TypeScript types that mirror Rust structs exactly.
// Keep in sync with src-tauri/src/config/mod.rs and src-tauri/src/state.rs.

// ---------------------------------------------------------------------------
// Config enums
// ---------------------------------------------------------------------------

export type InjectionMode = "flash_paste" | "keystroke" | "clipboard";

export type TranscriptionProvider = "openai" | "groq" | "openrouter" | "local";

// ---------------------------------------------------------------------------
// Nested config sections
// ---------------------------------------------------------------------------

export interface AudioConfig {
  device: string | null;
  vad_threshold: number;
  vad_silence_ms: number;
}

export interface TranscriptionConfig {
  provider: TranscriptionProvider;
  openai_api_key: string;
  groq_api_key: string;
  openrouter_api_key: string;
  openai_model: string;
  groq_model: string;
  openrouter_model: string;
  /** BCP-47 language hint (e.g. "en"). Empty string = auto-detect. */
  language: string;
  local_model_path: string | null;
}

export interface InjectionConfig {
  mode: InjectionMode;
}

export interface IndicatorConfig {
  show: boolean;
  position_x: number | null;
  position_y: number | null;
}

// ---------------------------------------------------------------------------
// Root AppConfig
// ---------------------------------------------------------------------------

export interface AppConfig {
  hotkey: string;
  audio: AudioConfig;
  transcription: TranscriptionConfig;
  injection: InjectionConfig;
  indicator: IndicatorConfig;
  launch_at_login: boolean;
  first_launch: boolean;
}

// ---------------------------------------------------------------------------
// Recording state
// ---------------------------------------------------------------------------

export type RecordingState = "idle" | "recording" | "transcribing";

// ---------------------------------------------------------------------------
// IPC event payloads (Rust -> Frontend)
// ---------------------------------------------------------------------------

export type HotkeyWarningSource = "startup" | "save";

export interface HotkeyWarningPayload {
  hotkey: string;
  message: string;
  source: HotkeyWarningSource;
}

export type AudioWarningCode = "selected_device_unavailable";

export interface AudioWarningPayload {
  code: AudioWarningCode;
  message: string;
  requested_device: string | null;
  active_device: string | null;
}

export interface AudioLevelPayload {
  rms: number; // 0.0 - 1.0
}

export interface InjectionDonePayload {
  success: boolean;
  error: string | null;
}

export interface StateChangedPayload {
  state: RecordingState;
}
