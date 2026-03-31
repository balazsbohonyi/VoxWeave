# VoxWeave

## What This Is

VoxWeave is a cross-platform voice-to-text dictation tool that works in any application — including terminals and command-line windows. The user presses a hotkey, speaks, and the transcribed text is automatically injected into the active window. It's a BYOK (bring your own key) alternative to Wispr Flow, targeting power users who want full control over their transcription providers without subscriptions, accounts, or telemetry.

## Core Value

Text lands in any window — terminals, editors, browsers — without friction. If everything else fails, the hotkey-to-text-injected pipeline must work seamlessly.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Global hotkey (default `Ctrl+Shift+Space`) triggers toggle-mode recording from any application
- [ ] Audio captured from selected microphone at 16kHz mono, encoded as Opus (cloud) or WAV (local)
- [ ] Floating always-on-top indicator with real-time waveform during recording
- [ ] Cloud transcription via OpenAI and Groq with per-provider API key and model config
- [ ] Local transcription via whisper.cpp with on-demand model download (tiny/base/small/medium)
- [ ] FlashPaste injection (default): clipboard save → paste → restore, with terminal-aware shortcuts
- [ ] Simulated keystroke injection via SendInput with KEYEVENTF_UNICODE and configurable speed
- [ ] Manual clipboard mode (copy without auto-paste)
- [ ] Terminal detection by window class for appropriate paste shortcut selection
- [ ] Elevation detection with relaunch prompt or clipboard fallback
- [ ] Injection cancellation via Escape or hotkey during active injection
- [ ] Automatic fallback chain: Keystrokes → FlashPaste → Clipboard
- [ ] System tray icon with context menu, recording state indicator, minimize-to-tray
- [ ] First-launch setup wizard (engine → provider/key → hotkey)
- [ ] Settings UI: General, Audio, Transcription (Cloud/Local), Injection sections
- [ ] Config persistence to JSON in app data directory
- [ ] Auto-start on Windows startup (opt-in)
- [ ] Toast notifications for status, errors, and actionable prompts

### Out of Scope

- AI post-processing (grammar, punctuation, filler removal) — v2+
- Streaming/real-time transcription display — batch mode only for v1
- Personal dictionary or custom vocabulary — v2+
- Per-app tone adaptation — v2+
- Multi-language auto-detection — user sets language (auto-detect as single option is fine)
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
| Tauri v2 over Electron | Smaller binary, Rust backend for performance-critical audio/injection code, native Windows API access | — Pending |
| FlashPaste as default injection | Most universally compatible method; clipboard-based avoids integrity level issues | — Pending |
| Hardcoded model lists (not dynamic) | Simpler, more reliable; avoids API calls just to populate dropdowns | — Pending |
| OpenRouter via chat completions (not Whisper endpoint) | OpenRouter has no Whisper-style STT endpoint; all audio goes through chat completions with input_audio | — Pending |
| Platform abstraction from day one | macOS is next target; invest in trait-based platform layer now to avoid rewrite later | — Pending |
| std::thread for whisper.cpp (not Tokio) | whisper.cpp is CPU-bound; blocking a Tokio thread starves the async runtime | — Pending |

---
*Last updated: 2026-03-14 after initialization*
