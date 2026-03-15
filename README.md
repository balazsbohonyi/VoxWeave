# VoxFlow

A cross-platform voice-to-text dictation tool that works in any application — including terminals and command-line windows. Press a hotkey, speak, and the transcribed text is automatically injected into the active window.

VoxFlow is a **BYOK (bring your own key)** alternative to Wispr Flow, built for power users who want full control over their transcription providers without subscriptions, accounts, or telemetry.

---

## Features

- **Global hotkey** — toggle-mode recording from any application (`Ctrl+Shift+Space` by default)
- **Universal text injection** — works in terminals, editors, browsers, and any other window
- **Multiple injection methods** — FlashPaste (default), simulated keystrokes, or clipboard-only
- **Cloud transcription** — OpenAI, Groq, and OpenRouter with per-provider API keys and model selection
- **Local transcription** — offline whisper.cpp with on-demand model download (tiny/base/small/medium)
- **Floating indicator** — always-on-top pill window with real-time waveform during recording
- **System tray** — minimal footprint; runs in the background with a tray icon
- **First-launch setup wizard** — guided configuration on first run
- **Zero telemetry** — no data collection, no analytics, no phone-home, ever

---

## How It Works

1. Press the global hotkey from any window
2. A floating indicator appears and recording begins
3. Press the hotkey again to stop recording
4. Audio is transcribed via your configured provider (cloud or local)
5. Transcribed text is injected directly into the active window

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust (Tauri v2) |
| Frontend | Vue 3 + TypeScript + Vite |
| Styling | Tailwind CSS |
| Build | MSVC + C/C++ toolchain |

---

## Platform Support

| Platform | Status |
|----------|--------|
| Windows 10/11 | v1 (current) |
| macOS | v2 (planned) |
| Android | v3 (planned) |

---

## Transcription Providers

### Cloud

| Provider | Models |
|----------|--------|
| OpenAI | whisper-1, gpt-4o-transcribe, gpt-4o-mini-transcribe |
| Groq | whisper-large-v3-turbo *(default)*, whisper-large-v3, distil-whisper-large-v3-en |
| OpenRouter | google/gemini-2.5-flash *(default)*, google/gemini-2.5-pro, google/gemini-2.5-flash-lite, openai/gpt-4o-audio-preview, openai/gpt-audio, openai/gpt-audio-mini |

### Local (whisper.cpp)

| Model | Size | Notes |
|-------|------|-------|
| tiny | ~75 MB | Fastest |
| base | ~150 MB | Recommended balance |
| small | ~500 MB | Higher accuracy |
| medium | ~1.5 GB | Best accuracy |

Models are downloaded on-demand and stored in `%APPDATA%/VoxFlow/models/`.

---

## Text Injection

VoxFlow supports three injection methods, automatically falling back if the primary method fails:

1. **FlashPaste** *(default)* — saves clipboard → pastes text → restores clipboard; uses terminal-aware shortcuts (`Ctrl+Shift+V` / `Shift+Insert` for detected terminal windows)
2. **Simulated Keystrokes** — character-by-character via `SendInput` with `KEYEVENTF_UNICODE`; configurable speed (slow/normal/fast)
3. **Clipboard** — copies to clipboard without auto-pasting

Fallback chain: **Keystrokes → FlashPaste → Clipboard** (configurable).

---

## Configuration

All settings persist in `%APPDATA%/VoxFlow/config.json`. Changes take effect immediately — no save button required.

**Configurable options:**
- Global hotkey binding
- Audio input device
- Transcription engine (cloud or local) and provider settings
- Injection method and speed
- Autostart on Windows startup (default: off)
- Floating indicator position

---

## Roadmap

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Foundation — project scaffold, config, system tray | Not started |
| 2 | Hotkey — global hotkey registration and toggle-mode state machine | Not started |
| 3 | Audio Capture — microphone capture, encoding, device management | Not started |
| 4 | Floating Indicator — always-on-top pill window with waveform | Not started |
| 5 | Cloud Transcription — OpenAI, Groq, OpenRouter with error handling | Not started |
| 6 | Text Injection — FlashPaste, keystrokes, clipboard with fallback chain | Not started |
| 7 | Pipeline Integration — end-to-end hotkey-to-text with notifications | Not started |
| 8 | Settings UI — full settings window for all configurable parameters | Not started |
| 9 | Setup Wizard — first-launch onboarding flow | Not started |
| 10 | Local Transcription — whisper.cpp with on-demand model download | Not started |

---

## Performance Targets

| Metric | Target |
|--------|--------|
| Hotkey → recording start | < 200ms |
| Cloud transcription end-to-end | < 5s |
| Local transcription (base model) | < 3s |
| Idle memory usage | < 50MB |
| Installer size (excl. models) | < 50MB |

---

## Distribution

Pre-built Windows installer distributed via GitHub Releases. Open source.

---

## License

[MIT](LICENSE)
