# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

### Plan Mode

- Make the plan extremely concise. Sacrifice grammar for the sake of concision.
- At the end of each plan, give me a list of unresolved questions to answer, if any.

## Project Overview

VoxFlow is a Windows voice-to-text dictation app built with Tauri v2 (Rust backend) + Vue 3 + TypeScript + Tailwind CSS. The user presses a global hotkey, speaks, and transcribed text is injected into the active window — including terminals. It is BYOK (bring your own key) with no telemetry, no accounts, and no subscriptions.

## Commands

### Development

```bash
cargo tauri dev          # Start Tauri dev server (Rust + Vite hot reload)
npm run dev              # Frontend only (Vite) — for UI-only work
```

### Building

```bash
cargo tauri build        # Production build + Windows installer
```

### Testing & Linting
For Rust tests, first do this: `cd src-tauri`

```bash
cargo test               # Run all Rust tests
cargo test <test_name>   # Run a single test by name
cargo clippy             # Rust linting (treat warnings as errors in CI)
npx vue-tsc --noEmit     # TypeScript typecheck (all Tauri commands require this to pass per PRD)
npm run lint             # ESLint for Vue/TS
```

### Build Requirements

- Rust stable toolchain (rustup)
- Node.js 20+
- MSVC Build Tools 2022 (required for `windows` crate and `whisper-rs`)
- CMake (required only for local transcription feature — `whisper-rs`/whisper.cpp)
- Feature-gate `whisper-rs` behind a cargo feature (`local-transcription`) so the app builds without CMake/whisper.cpp during early phases

## Architecture

### Two-Window App

Tauri manages two windows:
- **Settings window** (`src/windows/settings/`) — full configuration UI, opens on demand
- **Floating indicator** (`src/windows/indicator/`) — always-on-top, transparent, click-through, ~200×48px pill shown during recording

The indicator is a separate Tauri window with `always_on_top: true`, `decorations: false`, `skip_taskbar: true`, `transparent: true`.

### Rust Backend Structure

```
src-tauri/src/
├── main.rs                  # Tauri setup, plugin registration, window creation
├── commands/                # Thin Tauri command handlers only — no business logic here
├── audio/                   # cpal capture, RMS computation, Opus/WAV encoding
├── transcription/           # TranscriptionProvider trait + OpenAI/Groq/OpenRouter/Local impls
├── injection/               # FlashPaste, keystroke, clipboard modes + fallback pipeline
├── platform/                # Platform abstraction traits + Windows implementations
│   └── windows/             # GetForegroundWindow, SendInput, integrity checks, ShellExecuteW
├── config/                  # AppConfig serde struct, load/save to %APPDATA%/VoxFlow/config.json
├── tray.rs                  # System tray setup and menu event handling
└── state.rs                 # Tauri AppState (config, recording state, cancel flag)
```

### Frontend Structure

```
src/
├── windows/settings/        # Settings window components (General, Audio, Transcription, Injection)
├── windows/indicator/       # Floating indicator + Waveform.vue
├── windows/wizard/          # First-launch setup wizard (3 steps)
├── composables/             # useConfig.ts, useRecording.ts, useToast.ts
└── types/index.ts           # TypeScript types that mirror Rust structs exactly
```

### IPC Pattern

- **Frontend → Rust:** `invoke("command_name", { args })` — user-initiated actions (save config, start/stop recording, test connection, inject text)
- **Rust → Frontend:** `emit("event_name", payload)` — async updates pushed from Rust (audio levels at ~30fps, state changes, errors, download progress)
- **Tauri managed state:** `AppState` holds current config, recording state machine, and injection cancel flag

Rust commands return `Result<T, String>` — the frontend maps error strings to user-facing messages.

### Core Pipeline: Hotkey → Text Injected

```
Hotkey press → Rust shortcut handler → Recording state (save foreground window, open cpal stream)
→ Audio callback emits "audio-level" events at ~30fps to floating indicator
→ Second hotkey press → stop cpal → encode (Opus for cloud, WAV for local)
→ Transcribe (Tokio task for cloud, std::thread for local whisper.cpp — never block Tokio with CPU work)
→ Check target window integrity level → inject via selected method
→ Emit "injection-done" → hide indicator → show toast
```

### Platform Abstraction

All Windows-specific code lives behind traits in `platform/mod.rs`:

- `WindowInfo` — foreground window detection, window class name, terminal detection, focus restore
- `ElevationChecker` — process integrity level, is-elevated, relaunch-elevated
- `InputSimulator` — SendInput wrapper for paste shortcuts, unicode characters, VK_RETURN
- `ClipboardAccess` — read/write via `arboard` (not the Tauri clipboard plugin)

macOS implementations go in `platform/macos/` as stub modules. The trait layer is **non-negotiable** — adding macOS must only require implementing these traits, not touching core logic.

### Terminal Detection

Terminal window classes for FlashPaste paste shortcut switching: `ConsoleWindowClass`, `CASCADIA_HOSTING_WINDOW_CLASS`, `mintty`, `VirtualConsoleClass`. Terminals get `Ctrl+Shift+V` or `Shift+Insert` instead of `Ctrl+V`.

## Key Decisions (do not reverse without discussion)

| Decision | Reason |
|----------|--------|
| `arboard` directly for FlashPaste (not `tauri-plugin-clipboard-manager`) | Plugin is async and breaks the tight save→paste→restore timing |
| `std::thread` for whisper.cpp inference (not Tokio) | CPU-bound work starves the async runtime on a Tokio thread |
| `windows` crate (not `winapi` or `windows-sys`) | Microsoft-maintained, safe wrappers, actively developed |
| No `pinia` | Tauri managed state + Vue reactivity is sufficient; Pinia is overkill |
| No heavy UI libraries (Vuetify, PrimeVue) | Custom Tailwind UI keeps the floating indicator lightweight |
| Hardcoded model lists (not dynamic API calls) | Avoids API calls just to populate dropdowns |
| OpenRouter uses chat completions endpoint (not Whisper endpoint) | OpenRouter has no Whisper-style STT API |
| Feature-gate `whisper-rs` behind cargo feature | Avoids MSVC + CMake build requirement during cloud-only phases |

## Config

Stored at `%APPDATA%/VoxFlow/config.json`. Missing fields use defaults; unknown fields are preserved (forward/backward compatible). All settings persist immediately — no save button.

## Planning

All planning documents live in `.planning/`:
- `PROJECT.md` — product definition, constraints, key decisions
- `REQUIREMENTS.md` — 68 v1 requirements with requirement IDs (e.g. `HOTK-01`, `INJC-07`)
- `ROADMAP.md` — 10 phases with success criteria
- `STATE.md` — current phase, progress, blockers
- `PRD.md` — original product requirements document
- `research/` — stack, architecture, features, and pitfalls research

When implementing a phase, check `ROADMAP.md` for that phase's success criteria and `REQUIREMENTS.md` for the specific requirement IDs being satisfied.
