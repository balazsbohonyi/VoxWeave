## 1. Project Scaffolding

- [ ] 1.1 Initialize Tauri v2 project with `create-tauri-app` (Rust backend, Vue 3 + TypeScript + Vite + Tailwind CSS frontend)
- [ ] 1.2 Add Rust dependencies: `cpal`, `opus`, `whisper-rs`, `reqwest`, `tokio`, `serde`, `serde_json`, `arboard`, `windows-sys`
- [ ] 1.3 Add Tauri plugins: `tauri-plugin-global-shortcut`, `tauri-plugin-clipboard-manager`, `tauri-plugin-autostart`, `tauri-plugin-notification`
- [ ] 1.4 Configure Tauri window settings for main settings window (resizable, titled)
- [ ] 1.5 Configure Tauri permissions for all required plugins in `capabilities/`
- [ ] 1.6 Set up Tailwind CSS in frontend with base styles
- [ ] 1.7 Verify `npm run tauri dev` builds and runs without errors

## 2. Config Persistence

- [ ] 2.1 Define `AppConfig` struct in Rust with all fields from the config-persistence spec (serde Serialize/Deserialize)
- [ ] 2.2 Implement `load_config()` — reads `%APPDATA%/VoxFlow/config.json`, returns defaults for missing fields, ignores unknown fields
- [ ] 2.3 Implement `save_config(config: &AppConfig)` — writes to `%APPDATA%/VoxFlow/config.json`, creates directory if needed
- [ ] 2.4 Initialize config on app startup; store in Tauri managed state
- [ ] 2.5 Expose `get_config` and `update_config` Tauri commands to frontend
- [ ] 2.6 Write tests for config round-trip, missing fields, and unknown fields

## 3. System Tray

- [ ] 3.1 Create tray icon assets (default and recording-active variants)
- [ ] 3.2 Register tray icon in Tauri app setup with default icon
- [ ] 3.3 Build tray context menu: "Settings", "Start/Stop Recording", separator, "Quit"
- [ ] 3.4 Wire "Settings" menu item to open/focus the settings window
- [ ] 3.5 Wire "Quit" menu item to terminate the app
- [ ] 3.6 Implement close-to-tray: intercept main window close event and hide window instead
- [ ] 3.7 Implement tray double-click handler to show/focus settings window
- [ ] 3.8 Implement tray icon swap on recording state change (emit from audio capture module)

## 4. Global Hotkey

- [ ] 4.1 Implement `register_hotkey(shortcut: &str)` using `tauri-plugin-global-shortcut`
- [ ] 4.2 Implement `unregister_hotkey()` to remove current registration
- [ ] 4.3 Handle hotkey conflict detection: catch registration failure and emit `hotkey-conflict` event
- [ ] 4.4 Implement toggle-mode logic: idle → recording → transcribing (hotkey press during injection cancels)
- [ ] 4.5 Expose `set_hotkey(shortcut: String)` Tauri command; re-register and persist on change
- [ ] 4.6 Register default hotkey `Ctrl+Shift+Space` from config on app startup

## 5. Audio Capture

- [ ] 5.1 Implement `AudioCapture` struct using `cpal`: enumerate input devices, open stream at 16kHz mono i16
- [ ] 5.2 Implement `start_recording(device_id: Option<String>)` — opens cpal stream, buffers PCM to `Vec<i16>`
- [ ] 5.3 Implement `stop_recording()` — closes stream, returns buffered PCM data
- [ ] 5.4 Compute RMS amplitude per ~33ms chunk in the audio callback; emit `audio-level` Tauri event at ~30fps
- [ ] 5.5 Implement Opus encoding: `encode_to_opus(pcm: Vec<i16>) -> Vec<u8>` (16kHz mono, ~50-100ms for 30s audio)
- [ ] 5.6 Implement WAV export: `encode_to_wav(pcm: Vec<i16>) -> Vec<u8>` for local whisper.cpp
- [ ] 5.7 Expose `list_audio_devices()`, `start_recording()`, `stop_recording()` Tauri commands
- [ ] 5.8 Handle no-microphone error: emit `recording-error` event with message
- [ ] 5.9 Handle device disconnection during recording: stop recording, emit error event

## 6. Floating Indicator Window

- [ ] 6.1 Create second Tauri `WebviewWindow` config: `always_on_top: true`, `decorations: false`, `transparent: true`, `skip_taskbar: true`, `focus: false`
- [ ] 6.2 Build Vue component `FloatingIndicator.vue`: pill-shaped container (~200×48px), dark semi-transparent background
- [ ] 6.3 Implement waveform visualization: 8 animated bars responding to `audio-level` events (≥24fps)
- [ ] 6.4 Add red pulsing recording dot in recording state
- [ ] 6.5 Implement state transitions: recording → processing (spinner) → injecting (pasting/typing text cue)
- [ ] 6.6 Listen for Tauri state events (`recording-started`, `processing-started`, `injection-started`, `injection-done`) to drive state
- [ ] 6.7 Implement drag-to-reposition: track mousedown/mousemove on the indicator, call `setPosition` on window
- [ ] 6.8 Persist last position in config; restore position on show
- [ ] 6.9 Show/hide indicator window by emitting events from Rust pipeline stages

## 7. Cloud Transcription

- [ ] 7.1 Define `TranscriptionProvider` async trait: `transcribe(audio: AudioData, config: ProviderConfig) -> Result<String, TranscriptionError>`
- [ ] 7.2 Implement `OpenAIProvider`: POST to `https://api.openai.com/v1/audio/transcriptions`, multipart form with audio file + model + language
- [ ] 7.3 Implement `GroqProvider`: POST to Groq transcription endpoint; implement dynamic model list fetch with hardcoded fallback
- [ ] 7.4 Implement `OpenRouterProvider`: Whisper-compatible path + non-Whisper chat-completion path with base64 audio; model-type discrimination
- [ ] 7.5 Implement retry logic: exponential backoff on HTTP 429, max 3 retries
- [ ] 7.6 Implement error mapping: 401/403 → `InvalidApiKey`, 429 → `RateLimit`, network error → `NetworkError`
- [ ] 7.7 Implement language hint: include `language` parameter when configured, omit for auto-detect
- [ ] 7.8 Implement fallback provider logic: on failure after retries, if another provider is configured emit `fallback-available` event
- [ ] 7.9 Expose `transcribe_cloud(audio: Vec<u8>)` Tauri command; route to active provider
- [ ] 7.10 Expose `test_provider_connection(provider: String, api_key: String, model: String)` Tauri command for settings UI

## 8. Local Transcription

- [ ] 8.1 Add `whisper-rs` dependency; configure build to find MSVC toolchain
- [ ] 8.2 Implement model download: streaming HTTP download to `%APPDATA%/VoxFlow/models/<name>.bin` with progress events
- [ ] 8.3 Implement download cancellation: abort download and clean up partial file
- [ ] 8.4 Emit `model-download-progress` Tauri events during download (percentage, bytes downloaded)
- [ ] 8.5 Implement `list_local_models()`: return download status and disk size for each variant
- [ ] 8.6 Implement `delete_local_model(name: String)`: remove model file from disk
- [ ] 8.7 Implement `transcribe_local(pcm: Vec<f32>)` on a `std::thread`: load model, run whisper.cpp, return transcription
- [ ] 8.8 Handle missing model file: return `ModelMissing` error → emit event to frontend
- [ ] 8.9 Handle model load failure: return `ModelCorrupt` error → emit event to frontend
- [ ] 8.10 Expose all model management and transcription Tauri commands

## 9. Text Injection

- [ ] 9.1 Implement `get_foreground_window()` using `windows-sys`: returns HWND and window class name
- [ ] 9.2 Implement `get_process_integrity_level(hwnd: HWND) -> IntegrityLevel`: uses `OpenProcess` + `GetTokenInformation(TokenIntegrityLevel)`
- [ ] 9.3 Implement terminal detection: match window class against `ConsoleWindowClass`, `CASCADIA_HOSTING_WINDOW_CLASS`, `mintty`, `VirtualConsoleClass`
- [ ] 9.4 Implement `flashpaste(text: &str, target: HWND)`: clipboard save → write text → detect terminal → SendInput paste shortcut → sleep 500ms → restore clipboard
- [ ] 9.5 Implement `keystroke_inject(text: &str, speed_ms: u64, cancel_flag: Arc<AtomicBool>)`: iterate chars → SendInput KEYEVENTF_UNICODE; newlines as VK_RETURN; check cancel flag between chars
- [ ] 9.6 Implement `clipboard_copy(text: &str)` via `arboard`: write text to clipboard, no auto-paste
- [ ] 9.7 Implement fallback chain: Keystrokes → FlashPaste → Clipboard (when auto-fallback enabled)
- [ ] 9.8 Implement elevation check: if target integrity > VoxFlow integrity, emit `elevation-required` event
- [ ] 9.9 Implement admin relaunch: `ShellExecuteW` with `runas` verb
- [ ] 9.10 Implement focus restore: `SetForegroundWindow` to target HWND before injection
- [ ] 9.11 Implement injection cancellation: `Arc<AtomicBool>` cancel flag; global hotkey and Escape key listener set flag during active injection
- [ ] 9.12 Track characters injected vs. total for cancellation toast
- [ ] 9.13 Expose `inject_text(text: String)` Tauri command: reads injection method from config, runs appropriate pipeline
- [ ] 9.14 Emit injection state events for floating indicator state transitions

## 10. Toast Notifications

- [ ] 10.1 Create `Toast.vue` component: icon + message + preview text + optional action button + dismiss button
- [ ] 10.2 Implement auto-dismiss timer (4 seconds) with Vue `onMounted` + `setTimeout`
- [ ] 10.3 Create `ToastManager.vue` (or composable): maintains list of active toasts, positioned near floating indicator or bottom-right
- [ ] 10.4 Listen for Tauri events: `injection-success`, `injection-cancelled`, `transcription-error`, `provider-fallback`, `clipboard-copied`, `elevation-required`, `recording-error`
- [ ] 10.5 Map each event to appropriate toast message with action buttons (Open Settings, Retry, etc.)
- [ ] 10.6 Implement "Open Settings" action: bring settings window to focus
- [ ] 10.7 Implement "Retry" action: re-emit transcription request

## 11. Settings UI

- [ ] 11.1 Create `Settings.vue` root component with section navigation (General, Audio, Transcription, Injection)
- [ ] 11.2 Build `GeneralSettings.vue`: hotkey capture input (listens for keydown, formats combination), startup toggle, minimize-to-tray toggle
- [ ] 11.3 Build `AudioSettings.vue`: microphone device dropdown populated from `list_audio_devices()`
- [ ] 11.4 Build `TranscriptionSettings.vue`: Cloud/Local engine toggle; Cloud sub-section with 3 provider tabs
- [ ] 11.5 Build `ProviderTab.vue`: masked API key input with reveal toggle, model dropdown, "Test connection" button, "Set as active" button; active state visual highlight
- [ ] 11.6 Build `LocalModelSettings.vue`: model variant list with sizes, download/delete buttons, progress bar during download
- [ ] 11.7 Build `InjectionSettings.vue`: method selector (FlashPaste/Keystrokes/Clipboard), speed selector (shown only for Keystrokes), auto-fallback checkbox
- [ ] 11.8 Wire all settings controls to `update_config()` Tauri command for immediate persistence
- [ ] 11.9 Load current config values into all settings controls on mount
- [ ] 11.10 Add "Re-run setup wizard" button in General settings

## 12. Setup Wizard

- [ ] 12.1 Create `SetupWizard.vue` multi-step component with step indicator (1/3, 2/3, 3/3)
- [ ] 12.2 Build Step 1 `EngineSelect.vue`: Cloud vs. Local choice cards with descriptions
- [ ] 12.3 Build Step 2 cloud variant `ProviderSetup.vue`: tabbed provider interface with API key field and inline validation result
- [ ] 12.4 Build Step 2 local variant `ModelDownload.vue`: model variant list with download buttons and inline progress
- [ ] 12.5 Implement API key validation in wizard: call `test_provider_connection()`, show green checkmark or inline error
- [ ] 12.6 Disable "Next" on cloud step if no API key validated; show validation hint
- [ ] 12.7 Build Step 3 `HotkeyConfirm.vue`: hotkey capture input pre-filled with default `Ctrl+Shift+Space`
- [ ] 12.8 Implement "Finish" handler: save all wizard choices to config, close wizard, show "VoxFlow is ready" toast
- [ ] 12.9 Implement first-launch detection: check `first_launch_completed` flag in config; show wizard if false
- [ ] 12.10 Wire "Re-run setup wizard" button in settings to open wizard with current config values

## 13. Integration & Polish

- [ ] 13.1 Wire full pipeline end-to-end: hotkey → record → stop → encode → transcribe → inject → toast
- [ ] 13.2 Test FlashPaste in: VS Code, Chrome, Notepad, Word, Slack
- [ ] 13.3 Test FlashPaste in: Windows Terminal, PowerShell (verify Ctrl+Shift+V used)
- [ ] 13.4 Test keystroke injection in: VS Code, Chrome, Notepad, cmd.exe
- [ ] 13.5 Test clipboard mode and verify original clipboard is NOT restored
- [ ] 13.6 Test fallback chain: disable SendInput, verify FlashPaste fallback fires; disable FlashPaste, verify clipboard fallback
- [ ] 13.7 Test elevation detection: run Notepad as admin, verify VoxFlow shows elevation dialog
- [ ] 13.8 Test injection cancellation: long text via keystrokes, press Escape mid-injection
- [ ] 13.9 Test cloud provider error handling: invalid API key, rate limit simulation, network disconnect
- [ ] 13.10 Test local transcription: download base model, record 10s audio, verify transcription completes within 3 seconds
- [ ] 13.11 Verify floating indicator waveform renders at ≥24fps during recording
- [ ] 13.12 Verify app memory usage: idle < 50MB, recording < 150MB
- [ ] 13.13 Test setup wizard first-launch flow end-to-end
- [ ] 13.14 Test config persistence: change all settings, restart app, verify all settings preserved
- [ ] 13.15 Run `npm run typecheck` and resolve all TypeScript errors
