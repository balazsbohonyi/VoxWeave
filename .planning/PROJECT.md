# VoxWeave

## What This Is

VoxWeave is a cross-platform voice-to-text dictation tool that works in any application — including terminals and command-line windows. The user presses a hotkey, speaks, and the transcribed text is automatically injected into the active window. It's a BYOK (bring your own key) alternative to Wispr Flow, targeting power users who want full control over their transcription providers without subscriptions, accounts, or telemetry.

## Core Value

Text lands in any window — terminals, editors, browsers — without friction. If everything else fails, the hotkey-to-text-injected pipeline must work seamlessly.

## Requirements

### Validated

- [x] Global hotkey (default `Ctrl+Shift+Space`) triggers toggle-mode recording from any application
- [x] Audio captured from selected microphone at 16kHz mono, encoded as Opus (cloud) or WAV (local)
- [x] Floating always-on-top indicator with real-time waveform during recording
- [x] Cloud transcription via OpenAI and Groq with per-provider API key and model config
- [x] Local transcription via whisper.cpp with on-demand model download (tiny/base/small/medium)
- [x] FlashPaste injection (default): clipboard save → paste → restore, with terminal-aware shortcuts
- [x] Simulated keystroke injection via SendInput with KEYEVENTF_UNICODE and configurable speed
- [x] Manual clipboard mode (copy without auto-paste)
- [x] Terminal detection by window class for appropriate paste shortcut selection
- [x] Elevation detection with relaunch prompt or clipboard fallback
- [x] Injection cancellation via Escape or hotkey during active injection
- [x] Automatic fallback chain: Keystrokes → FlashPaste → Clipboard
- [x] System tray icon with context menu, recording state indicator, minimize-to-tray
- [x] First-launch setup wizard (engine → provider/key → hotkey)
- [x] Settings UI: General, Audio, Transcription (Cloud/Local), Injection sections
- [x] Config persistence to JSON in app data directory
- [x] Auto-start on Windows startup (opt-in)
- [x] Toast notifications for status, errors, and actionable prompts
- [x] Language selection with Auto-detect option in Settings (Transcription section)

### Active

(None — all v1 requirements implemented)

### Out of Scope

- AI post-processing (grammar, punctuation, filler removal) — v2+
- Streaming/real-time transcription display — batch mode only for v1
- Personal dictionary or custom vocabulary — v2+
- Per-app tone adaptation — v2+
- ~~Multi-language auto-detection — user sets language (auto-detect as single option is fine)~~
  - **Implemented:** language dropdown in Settings with Auto-detect option
- Usage tracking, analytics, or telemetry — never
- User accounts, subscription, or cloud sync — never (BYOK philosophy)
- Android support — Phase 2 (future milestone)
- macOS support — Phase 3 (future milestone, next after Windows MVP)

## Context

- **Competitive context**: Wispr Flow is the commercial incumbent. VoxWeave differentiates on BYOK model, zero cost (beyond API usage), privacy (audio never touches third-party servers beyond the user's chosen provider), and terminal support.
- **Distribution**: Pre-built Windows installer via GitHub Releases. Open source.
- **Cross-platform intent**: Windows is the MVP, but macOS is the next planned platform. The architecture must use proper abstractions for platform-specific code (audio capture, text injection, clipboard operations, hotkey registration, elevation checks) so adding macOS requires implementing platform traits — not rewriting core logic.
- **Provider model lists**: Hardcoded model lists for all providers. OpenAI: whisper-1, gpt-4o-transcribe, gpt-4o-mini-transcribe. Groq: whisper-large-v3-turbo (default), whisper-large-v3, distil-whisper-large-v3-en.
- **OpenRouter dropped**: OpenRouter has no Whisper-style STT endpoint. Its chat completions approach (base64 audio) produced inconsistent results and is not purpose-built for dictation. Confirmed by user testing — do not re-add without re-evaluation.

## Constraints

- **Tech stack**: Tauri v2 (Rust backend) + Vue 3 + TypeScript + Vite + Tailwind CSS — decided, non-negotiable
- **Platform**: Windows 10/11 for v1 — but architecture must abstract platform-specific code behind traits/interfaces
- **Build toolchain**: MSVC + C/C++ toolchain required for whisper-rs/whisper.cpp compilation
- **Performance**: Hotkey → recording < 200ms, cloud transcription end-to-end < 5s, local (base model) < 3s, idle memory < 50MB
- **Installer size**: Under 50MB excluding local model files
- **Zero telemetry**: No data collection, no phone-home, no analytics — ever

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri v2 over Electron | Smaller binary, Rust backend for performance-critical audio/injection code, native Windows API access | Shipped — Windows installer under 50MB |
| FlashPaste as default injection | Most universally compatible method; clipboard-based avoids integrity level issues | Shipped — default injection mode |
| Hardcoded model lists (not dynamic) | Simpler, more reliable; avoids API calls just to populate dropdowns | Shipped — via `get_provider_models` command |
| OpenRouter dropped (never re-add) | No Whisper-style STT endpoint; chat completions with base64 audio produced inconsistent results | Confirmed by user testing — removed in v1 |
| Platform abstraction from day one | macOS is next target; invest in trait-based platform layer now to avoid rewrite later | Shipped — all platform code behind traits in `platform/` |
| `spawn_blocking` for whisper.cpp (not Tokio thread) | whisper.cpp is CPU-bound; blocking a Tokio thread starves the async runtime | Shipped — CPU inference on blocking thread pool |

---
*Last updated: 2026-04-01 — all v1 requirements implemented across 10 phases*
