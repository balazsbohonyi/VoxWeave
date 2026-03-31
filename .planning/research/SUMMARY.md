# Research Summary: VoxWeave

## Stack Recommendation

**Framework:** Tauri v2 + Rust backend + Vue 3 + TypeScript + Vite + Tailwind CSS (decided)

**Key Rust crates:**
- `cpal` ^0.15 (audio I/O), `hound` ^3.5 (WAV encoding), `opus`/`audiopus` (Opus encoding — consider WAV-only fallback if build issues)
- `reqwest` ^0.12 (HTTP), `tokio` ^1.0 (async), `whisper-rs` ^0.13 (local STT — feature-gated)
- `windows` ^0.58 (Windows APIs — SendInput, GetForegroundWindow, integrity checks)
- `arboard` ^3.4 (clipboard), `serde`/`serde_json` ^1.0 (config)

**Tauri plugins:** global-shortcut, autostart, notification, shell

**Key decisions:** Use `windows` crate (not `winapi` or `windows-sys`). Use `arboard` directly for FlashPaste (not Tauri clipboard plugin). Feature-gate `whisper-rs` to simplify initial builds.

## Table Stakes Features

1. Global hotkey trigger (any app)
2. Toggle mode recording (press start / press stop)
3. Microphone capture
4. Visual recording indicator
5. Cloud transcription (at least one provider)
6. Text injection into active window
7. System tray presence
8. Settings persistence
9. Error feedback

## Key Differentiators

1. **Terminal support** — Windows Voice Typing and Wispr Flow don't work in terminals
2. **BYOK multi-provider** — OpenAI, Groq, OpenRouter with user's own keys
3. **Local transcription** — offline, zero cost, privacy
4. **FlashPaste** — clipboard-preserving injection
5. **Injection fallback chain** — resilient text delivery

## Critical Pitfalls (Top 5)

1. **Transparent window issues** — Test Tauri transparent/click-through windows very early, not late
2. **cpal audio callback blocking** — Use lock-free buffers, never allocate in callback
3. **SendInput vs elevated processes** — Check integrity BEFORE injection, not after silent failure
4. **FlashPaste clipboard race** — 500ms restore delay may not be enough for slow apps (Electron)
5. **whisper-rs build complexity** — Feature-gate it, build cloud-only first, add local later

## Architecture Highlights

- **Two Tauri windows:** Settings (main) + Floating indicator (always-on-top, click-through)
- **Platform trait layer:** `WindowInfo`, `ElevationChecker`, `InputSimulator`, `ClipboardAccess` — Windows impls now, macOS stubs
- **Pipeline:** Hotkey → Record → Encode → Transcribe → Inject (linear state machine)
- **IPC:** Commands for actions (frontend → Rust), Events for streaming updates (Rust → frontend)
- **State:** Config loaded at startup, stored in Tauri managed state, written on every change

## Suggested Build Order

1. Scaffolding + Config + System Tray (foundation)
2. Global Hotkey + State Machine (control flow)
3. Audio Capture + Encoding (data pipeline)
4. Floating Indicator (visual feedback)
5. Cloud Transcription (core value delivery)
6. Text Injection — FlashPaste + Keystrokes + Clipboard (core value delivery)
7. End-to-End Pipeline (integration)
8. Settings UI (user configuration)
9. Setup Wizard (onboarding)
10. Local Transcription (whisper.cpp — complex, independent)
11. Toast Notifications + Polish (error handling, edge cases)

## Risks to Monitor

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Tauri transparent window bugs | Medium | High | Test early, have solid-background fallback |
| Opus build failures | Medium | Medium | WAV-only fallback (larger uploads but works) |
| whisper-rs MSVC dependency chain | High | Medium | Feature-gate, cloud-only first |
| Terminal detection misses | Medium | Low | Log class names, plan for user config override |
| Clipboard race in Electron apps | Medium | Low | Document limitation, suggest keystroke mode |
