## Why

VoxFlow needs to be built from scratch as a Windows MVP — a voice-to-text dictation tool that works in any application (including terminals) via a global hotkey. There is no existing implementation; this change delivers the full v1 feature set as defined in PRD.md.

## What Changes

- New Tauri v2 (Rust + Vue 3 + TypeScript + Tailwind CSS) desktop application
- Global hotkey registration (`Ctrl+Shift+Space` default) triggering toggle-mode audio recording
- Real-time floating waveform indicator (always-on-top, click-through, draggable)
- Cloud transcription via OpenAI, Groq, and OpenRouter APIs with per-provider configuration
- Local transcription via whisper.cpp (on-demand model download, background thread)
- Three text injection modes: FlashPaste (default), simulated keystrokes (`SendInput`), manual clipboard
- Terminal-aware paste shortcut detection (`Ctrl+Shift+V` / `Shift+Insert` for terminal windows)
- Admin elevation detection with relaunch prompt or clipboard fallback
- Injection cancellation (Escape key) during keystroke injection
- System tray icon with context menu and recording state indicator
- First-launch setup wizard (3-step: engine → provider/API key → hotkey)
- Settings UI (General, Audio, Transcription, Injection sections)
- Config persistence to JSON in `%APPDATA%/VoxFlow/`
- Auto-start on Windows startup (opt-in)

## Capabilities

### New Capabilities

- `global-hotkey`: Register and manage a configurable global hotkey; toggle-mode start/stop recording; conflict detection
- `audio-capture`: Capture microphone audio via `cpal`; encode to Opus (cloud) or WAV (local); emit RMS amplitude events for waveform at ~30fps; device selection
- `floating-indicator`: Separate always-on-top Tauri window with waveform visualization, recording/processing/pasting states, draggable position persistence
- `cloud-transcription`: HTTP transcription requests to OpenAI, Groq, and OpenRouter; per-provider API key, model selection, language hint; retry with backoff; fallback provider offer
- `local-transcription`: whisper.cpp via `whisper-rs`; on-demand model download with progress; background thread execution; model management (download/delete)
- `text-injection`: FlashPaste pipeline (clipboard save/restore + Ctrl+V), keystroke injection (`SendInput` + `KEYEVENTF_UNICODE`), clipboard mode; terminal detection; elevation check; cancellation; auto-fallback chain
- `toast-notifications`: Status toasts (success/error/fallback/cancellation) near floating indicator or bottom-right; auto-dismiss 4s
- `system-tray`: Tray icon with context menu (Settings, Start/Stop Recording, Quit); recording state icon overlay; minimize-to-tray behavior
- `settings-ui`: Single-page settings window with General, Audio, Transcription (Cloud/Local), and Injection sections; all PRD-specified controls
- `setup-wizard`: 3-step first-launch wizard (engine selection → provider setup with API key validation → hotkey confirmation); re-accessible from settings
- `config-persistence`: JSON config file in `%APPDATA%/VoxFlow/config.json`; all user preferences; auto-load on startup

### Modified Capabilities

## Impact

- New project — all code is new
- Rust dependencies: `tauri@2`, `cpal`, `opus`, `whisper-rs`, `reqwest`, `tokio`, `serde_json`, `arboard`, `windows-sys` (for `SendInput`, `GetForegroundWindow`, integrity level check)
- Tauri plugins: `tauri-plugin-global-shortcut`, `tauri-plugin-clipboard-manager`, `tauri-plugin-autostart`, `tauri-plugin-notification`
- Frontend: Vue 3, TypeScript, Vite, Tailwind CSS
- Build toolchain: requires MSVC + C/C++ toolchain for whisper.cpp compilation
- Platform: Windows 10/11 only for v1
