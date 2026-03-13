## Context

VoxFlow is a brand-new Windows desktop application built with Tauri v2 (Rust backend + Vue 3 frontend). There is no existing codebase. The app must work silently in the background and inject transcribed speech into any foreground application, including elevated (admin) processes and terminal emulators, without interrupting the user's workflow.

## Goals / Non-Goals

**Goals:**
- Define the core architecture for the Rust backend pipeline: hotkey → record → encode → transcribe → inject
- Define the frontend component structure for the floating indicator and settings UI
- Establish inter-process communication (IPC) patterns between Rust commands and Vue frontend
- Specify how audio data flows from `cpal` capture to Opus encoding and transcription API upload
- Document the Windows-specific injection strategy (FlashPaste, SendInput, clipboard) and the elevation/terminal detection logic
- Clarify the multi-provider abstraction for cloud transcription

**Non-Goals:**
- Line-by-line implementation details (covered by specs and code)
- Android/macOS support (future phases)
- AI post-processing, streaming transcription, personal vocabulary (out of scope v1)
- Telemetry, user accounts, cloud sync

## Decisions

### D1: Tauri v2 with separate floating indicator window
**Decision**: Use Tauri v2's multi-window support to render the floating indicator as a separate `WebviewWindow` with `always_on_top: true`, `decorations: false`, `transparent: true`, `skip_taskbar: true`.
**Why**: A single window can't be both the settings panel and an always-on-top overlay. Separate windows let each have independent focus behavior. Tauri v2's window API handles this cleanly.
**Alternative considered**: Single window with an overlay `<div>` — rejected because the app window would need focus, stealing it from the target input.

### D2: Audio pipeline in Rust with cpal + Opus
**Decision**: `cpal` captures raw PCM (16kHz mono i16) into a ring buffer on a dedicated audio thread. On recording stop, the buffer is flushed, then encoded to Opus via the `opus` crate for cloud upload, or kept as WAV for whisper.cpp.
**Why**: `cpal` is the idiomatic cross-platform Rust audio I/O crate. Encoding on stop (not during recording) keeps the audio thread simple and avoids encoding latency artifacts in the waveform. Opus at 16kHz mono gives ~50-100KB for 30s vs ~960KB WAV — significant upload savings.
**Alternative considered**: Encoding during recording — rejected due to added complexity and potential audio glitches.

### D3: RMS amplitude emitted to frontend via Tauri events
**Decision**: The audio capture thread computes RMS amplitude per ~33ms chunk and emits a `audio-level` Tauri event to the frontend at ~30fps.
**Why**: The frontend waveform visualization needs amplitude data in real time. Tauri's event system is the correct IPC mechanism for streaming data from Rust to Vue.

### D4: TranscriptionProvider trait for multi-provider abstraction
**Decision**: Define a `TranscriptionProvider` async trait in Rust with `async fn transcribe(audio: AudioData, config: ProviderConfig) -> Result<String, TranscriptionError>`. Implementations: `OpenAIProvider`, `GroqProvider`, `OpenRouterProvider`. Each encapsulates its endpoint URL, auth header, request body shape, and response parsing. `OpenRouterProvider` has an additional code path for non-Whisper models (chat-completion style with base64 audio).
**Why**: Keeps provider-specific logic isolated and testable. New providers can be added without touching the core pipeline.

### D5: FlashPaste as the default injection method
**Decision**: FlashPaste pipeline — (1) `GetForegroundWindow` to save target handle, (2) integrity level check, (3) read clipboard via `arboard`, (4) detect terminal by window class (`ConsoleWindowClass`, `CASCADIA_HOSTING_WINDOW_CLASS`, `mintty`, `VirtualConsoleClass`), (5) write transcription to clipboard, (6) `SendInput` Ctrl+V (or Ctrl+Shift+V / Shift+Insert for terminals), (7) `sleep(500ms)`, (8) restore original clipboard.
**Why**: Works universally without per-app focus management. Clipboard is the most compatible injection vector. Terminal detection by window class is well-established. The 500ms delay gives virtually all apps time to process the paste before clipboard is restored.
**Alternative considered**: Keystroke injection as default — rejected because `SendInput` requires matching integrity level and is blocked by some apps.

### D6: Elevation detection via process integrity level
**Decision**: Before injection, VoxFlow calls `GetWindowThreadProcessId` on the foreground window, opens the process with `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)`, calls `OpenProcessToken` + `GetTokenInformation(TokenIntegrityLevel)`, and compares the SID to `SECURITY_MANDATORY_HIGH_RID`. If the target is elevated and VoxFlow is not, emit a `elevation-required` Tauri event to show the frontend dialog.
**Why**: `SendInput` is silently rejected when injecting into an elevated process from a non-elevated one. Early detection and user prompt is better than a silent failure.

### D7: Config stored as JSON in %APPDATA%/VoxFlow/
**Decision**: Use `serde_json` + `serde` to serialize/deserialize a strongly-typed `AppConfig` struct to `%APPDATA%/VoxFlow/config.json`. Load on startup, save on any change.
**Why**: Simple, human-readable, no migration complexity for v1. Tauri provides the app data directory path via `app.path().app_data_dir()`.

### D8: whisper.cpp via whisper-rs on a std::thread
**Decision**: Local transcription runs on a `std::thread` (not Tokio task) because whisper.cpp is CPU-bound and blocking. Audio is passed as `Vec<f32>` PCM samples. Model files live in `%APPDATA%/VoxFlow/models/`.
**Why**: Tokio tasks are designed for I/O-bound work. A blocking CPU-bound task on a Tokio thread would starve the async runtime. `std::thread` + channel is the correct pattern.

### D9: Injection cancellation via shared atomic flag
**Decision**: A `Arc<AtomicBool>` cancel flag is set when Escape or the recording hotkey is detected during keystroke injection. The injection loop checks this flag between characters.
**Why**: Simple, lock-free cancellation without complex channel plumbing. The flag is reset at the start of each injection session.

### D10: Tauri plugins for hotkey, clipboard, autostart, tray
**Decision**: Use official Tauri v2 plugins: `tauri-plugin-global-shortcut` (hotkey), `tauri-plugin-clipboard-manager` (clipboard read/write), `tauri-plugin-autostart` (startup registry), `tauri-plugin-notification` (toasts). Use `arboard` directly for clipboard save/restore in FlashPaste (need synchronous access outside of Tauri's async clipboard API).
**Why**: Official plugins are maintained and compatible with Tauri v2's permission model. Direct `arboard` use for FlashPaste avoids async overhead in the tight paste timing window.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Tauri v2 transparent window flakiness on Windows | Test on Win10/Win11; fall back to semi-transparent non-transparent background if needed |
| Terminal detection by window class misses some emulators | Expose user-configurable terminal class override in settings (future); document known-working list |
| FlashPaste clipboard restore race condition (Electron apps) | 500ms delay; document limitation; user can switch to keystroke mode |
| whisper-rs build requires MSVC + C++ toolchain | Document build prerequisites; CI must have MSVC installed |
| `SendInput` blocked by anti-cheat / UAC | Fall back to FlashPaste → clipboard; show actionable toast |
| OpenRouter non-Whisper model API contract variability | Per-model adapter map in OpenRouterProvider; conservative error handling |
| Global hotkey conflicts (Discord, OBS) | Detect conflict on registration; show warning with suggested alternatives |
| Large local model files (up to 1.5GB) | Download on-demand only; show disk usage; provide delete button |

## Migration Plan

New project — no migration needed. First-time users run the setup wizard on first launch.

## Open Questions

- Should the Groq model list be fetched dynamically from the API or hardcoded? (PRD says "fetched dynamically if possible" — implement dynamic fetch with hardcoded fallback)
- Exact window class names for all supported terminal emulators — needs testing on each
- Whether `arboard` clipboard operations are reliable enough inside the 500ms FlashPaste window on all target apps — needs integration testing
