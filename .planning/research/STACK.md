# Stack Research: VoxWeave

## Core Framework

| Component | Choice | Version | Rationale | Confidence |
|-----------|--------|---------|-----------|------------|
| App framework | Tauri v2 | ^2.0 | Decided. Small binary, Rust backend, native Windows APIs, multi-window support | High |
| Frontend | Vue 3 | ^3.4 | Decided. Composition API, good TypeScript support, reactive state for waveform | High |
| Build tool | Vite | ^6.0 | Standard for Vue 3 + Tauri, fast HMR | High |
| Styling | Tailwind CSS | ^4.0 | Decided. Utility-first, good for small custom UI like floating indicator | High |
| Language (backend) | Rust | stable | Decided. Required by Tauri, ideal for audio processing and Windows API calls | High |
| Language (frontend) | TypeScript | ^5.5 | Decided. Type safety for IPC contracts between Rust and Vue | High |

## Rust Crates

### Audio

| Crate | Version | Purpose | Confidence |
|-------|---------|---------|------------|
| `cpal` | ^0.15 | Cross-platform audio I/O — mic capture at 16kHz mono. Well-maintained, idiomatic Rust | High |
| `opus` | ^0.3 | Opus encoding for cloud upload. Wraps libopus. 16kHz mono → small file sizes | Medium |
| `hound` | ^3.5 | WAV file writing for local whisper.cpp input. Simple, reliable | High |

**Alternative considered:** `rodio` — higher-level, but less control over raw PCM capture. `cpal` is the right choice for real-time audio with RMS computation.

**Note on `opus`:** The `opus` crate requires `libopus` C library. Alternative: `audiopus` (pure Rust bindings to libopus) or `ogg-opus` for container format. Evaluate build complexity. If Opus proves too complex to build, consider sending WAV to cloud providers too (larger but universally supported).

### Transcription

| Crate | Version | Purpose | Confidence |
|-------|---------|---------|------------|
| `reqwest` | ^0.12 | HTTP client for cloud provider APIs. Async, supports multipart form uploads | High |
| `whisper-rs` | ^0.13 | Rust bindings for whisper.cpp. Requires MSVC C++ toolchain | Medium |
| `tokio` | ^1.0 | Async runtime for cloud HTTP requests and event handling | High |

**Note on `whisper-rs`:** Build complexity is significant. Requires cmake + MSVC. Consider feature-gating local transcription behind a cargo feature flag so the app can build without whisper.cpp initially.

### Windows APIs

| Crate | Version | Purpose | Confidence |
|-------|---------|---------|------------|
| `windows` | ^0.58 | Microsoft's official Rust bindings for Windows APIs. Used for SendInput, GetForegroundWindow, process integrity checks, ShellExecuteW | High |

**Why `windows` over `windows-sys`:** The `windows` crate provides safe wrappers and better ergonomics. `windows-sys` is raw FFI bindings — more error-prone. The `windows` crate is Microsoft-maintained and actively developed.

**Why not `winapi`:** Legacy crate, superseded by Microsoft's official `windows` crate. Not recommended for new projects.

### Clipboard & Config

| Crate | Version | Purpose | Confidence |
|-------|---------|---------|------------|
| `arboard` | ^3.4 | Clipboard read/write. Used in FlashPaste pipeline for synchronous clipboard save/restore | High |
| `serde` | ^1.0 | Serialization framework for config and IPC | High |
| `serde_json` | ^1.0 | JSON config file serialization | High |

## Tauri Plugins

| Plugin | Version | Purpose | Confidence |
|--------|---------|---------|------------|
| `tauri-plugin-global-shortcut` | ^2.0 | Global hotkey registration (Ctrl+Shift+Space) | High |
| `tauri-plugin-autostart` | ^2.0 | Windows startup registry management | High |
| `tauri-plugin-notification` | ^2.0 | System toast notifications | Medium |
| `tauri-plugin-shell` | ^2.0 | Needed for admin relaunch via ShellExecuteW | Medium |

**Note:** Do NOT use `tauri-plugin-clipboard-manager` for the FlashPaste pipeline — it's async and may introduce timing issues in the tight save→paste→restore flow. Use `arboard` directly for FlashPaste. The Tauri clipboard plugin is fine for the manual clipboard mode.

## Frontend Libraries

| Package | Version | Purpose | Confidence |
|---------|---------|---------|------------|
| `@tauri-apps/api` | ^2.0 | Tauri IPC (invoke commands, listen to events) | High |
| `@tauri-apps/plugin-global-shortcut` | ^2.0 | Frontend bindings for global shortcut plugin | High |
| `@tauri-apps/plugin-autostart` | ^2.0 | Frontend bindings for autostart plugin | High |
| `@tauri-apps/plugin-notification` | ^2.0 | Frontend bindings for notification plugin | Medium |
| `vue-router` | ^4.0 | Optional — may not need if settings is single-page | Low |

**What NOT to use:**
- `electron` — Tauri is decided, Electron's memory footprint is too large
- `pinia` — Overkill for this app's state; Tauri managed state + Vue reactivity is sufficient
- Heavy UI component libraries (Vuetify, PrimeVue) — custom minimal UI is better for the floating indicator and settings; Tailwind is sufficient

## Build Requirements

- **Rust stable toolchain** (rustup)
- **Node.js 20+** (for Vite/Vue frontend)
- **MSVC Build Tools** (Visual Studio Build Tools 2022) — required for `windows` crate and `whisper-rs`
- **CMake** — required for whisper.cpp compilation via `whisper-rs`
- **libopus** (if using `opus` crate) — may need vcpkg or bundled build

## Platform Abstraction Strategy

Define Rust traits for platform-specific operations:
- `trait AudioCapture` — mic enumeration, recording
- `trait TextInjector` — FlashPaste, keystroke injection, clipboard
- `trait ClipboardManager` — read/write/restore
- `trait HotkeyManager` — register/unregister global shortcuts
- `trait ElevationChecker` — process integrity level detection
- `trait SystemIntegration` — tray, autostart, admin relaunch

Windows implementations use the `windows` crate. Future macOS implementations would use CoreAudio, CGEvent, NSPasteboard, etc.
