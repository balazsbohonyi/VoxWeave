// TypeScript types that mirror Rust structs exactly.
// Keep in sync with src-tauri/src/config/mod.rs and src-tauri/src/state.rs.

// ---------------------------------------------------------------------------
// Config enums
// ---------------------------------------------------------------------------

export type InjectionMode = "flash_paste" | "keystroke" | "clipboard";

export type KeystrokeSpeed = "slow" | "normal" | "fast";

export type TranscriptionProvider = "openai" | "groq" | "local";

// ---------------------------------------------------------------------------
// Nested config sections
// ---------------------------------------------------------------------------

export interface AudioConfig {
  device: string | null;
  vad_threshold: number;
  vad_silence_ms: number;
}

export interface CloudProviderConfig {
  api_key: string;
  model: string;
}

export interface LocalProviderConfig {
  model_path: string | null;
}

export interface TranscriptionProviders {
  openai: CloudProviderConfig;
  groq: CloudProviderConfig;
  local: LocalProviderConfig;
}

export interface TranscriptionConfig {
  provider: TranscriptionProvider;
  providers: TranscriptionProviders;
  /** BCP-47 language hint (e.g. "en"). Empty string = auto-detect. */
  language: string;
  fallback_order: TranscriptionProvider[];
}

export interface InjectionConfig {
  mode: InjectionMode;
  keystroke_speed: KeystrokeSpeed;
  auto_fallback: boolean;
  paste_delay_ms: number;
}

export interface IndicatorConfig {
  show: boolean;
  position_x: number | null;
  position_y: number | null;
  show_on_startup: boolean;
}

// ---------------------------------------------------------------------------
// Root AppConfig
// ---------------------------------------------------------------------------

export interface AppConfig {
  hotkey: string;
  push_to_talk: boolean;
  audio: AudioConfig;
  transcription: TranscriptionConfig;
  injection: InjectionConfig;
  indicator: IndicatorConfig;
  launch_at_login: boolean;
  first_launch: boolean;
}

export interface RuntimeInfo {
  is_portable: boolean;
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

export type IndicatorVisualState =
  | "recording"
  | "processing"
  | "injecting"
  | "hidden";

export interface IndicatorStatePayload {
  state: IndicatorVisualState;
}

export interface InjectionDonePayload {
  success: boolean;
  error: string | null;
}

export interface StateChangedPayload {
  state: RecordingState;
}

// ---------------------------------------------------------------------------
// Injection result payloads (Rust -> Frontend)
// ---------------------------------------------------------------------------

export type InjectionErrorCode =
  | "all_methods_failed"
  | "elevation_required";

export interface InjectionErrorPayload {
  code: InjectionErrorCode;
  message: string;
}
