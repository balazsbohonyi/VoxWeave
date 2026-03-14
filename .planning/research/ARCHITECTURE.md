# Architecture Research: VoxFlow

## System Overview

VoxFlow is a Tauri v2 desktop app with two windows (settings + floating indicator), a Rust backend handling audio/transcription/injection, and a Vue 3 frontend for UI. The Rust backend does the heavy lifting; the frontend is thin.

```
┌─────────────────────────────────────────────────┐
│                  Tauri App Shell                 │
│                                                  │
│  ┌──────────────┐    ┌────────────────────────┐  │
│  │   Settings    │    │  Floating Indicator    │  │
│  │   Window      │    │  Window                │  │
│  │  (Vue 3)      │    │  (Vue 3)               │  │
│  │               │    │  always-on-top          │  │
│  │  - General    │    │  click-through          │  │
│  │  - Audio      │    │  transparent            │  │
│  │  - Transcript │    │  ~200x48px              │  │
│  │  - Injection  │    │                         │  │
│  │  - Wizard     │    │  - Waveform bars        │  │
│  └──────┬───────┘    │  - State indicator      │  │
│         │            └──────────┬──────────────┘  │
│         │    Tauri Commands     │  Tauri Events    │
│         │    (invoke)           │  (listen)        │
│  ┌──────┴───────────────────────┴──────────────┐  │
│  │              Rust Backend                    │  │
│  │                                              │  │
│  │  ┌─────────┐  ┌──────────┐  ┌────────────┐  │  │
│  │  │ Config  │  │  Audio   │  │Transcription│  │  │
│  │  │ Manager │  │ Pipeline │  │  Engine     │  │  │
│  │  └────┬────┘  └────┬─────┘  └─────┬──────┘  │  │
│  │       │            │               │          │  │
│  │  ┌────┴────┐  ┌────┴─────┐  ┌─────┴──────┐  │  │
│  │  │Platform │  │ Hotkey   │  │  Injection  │  │  │
│  │  │ Layer   │  │ Manager  │  │  Pipeline   │  │  │
│  │  └─────────┘  └──────────┘  └────────────┘  │  │
│  └──────────────────────────────────────────────┘  │
│                                                    │
│  ┌──────────────────────────────────────────────┐  │
│  │              System Tray                      │  │
│  └──────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────┘
```

## Rust Backend Modules

### Module Structure

```
src-tauri/src/
├── main.rs                  # Tauri app setup, plugin registration, window creation
├── commands/                # Tauri command handlers (thin layer)
│   ├── mod.rs
│   ├── audio.rs             # start_recording, stop_recording, list_devices
│   ├── transcription.rs     # transcribe, test_connection
│   ├── injection.rs         # inject_text
│   ├── config.rs            # get_config, update_config
│   └── models.rs            # list_models, download_model, delete_model
├── audio/                   # Audio capture and encoding
│   ├── mod.rs
│   ├── capture.rs           # cpal-based recording, RMS computation
│   ├── opus_encoder.rs      # PCM → Opus encoding
│   └── wav_encoder.rs       # PCM → WAV encoding
├── transcription/           # Transcription providers
│   ├── mod.rs
│   ├── provider.rs          # TranscriptionProvider trait
│   ├── openai.rs            # OpenAI implementation
│   ├── groq.rs              # Groq implementation
│   ├── openrouter.rs        # OpenRouter implementation (chat completions)
│   └── local.rs             # whisper.cpp implementation
├── injection/               # Text injection methods
│   ├── mod.rs
│   ├── flashpaste.rs        # Clipboard save → paste → restore
│   ├── keystroke.rs         # SendInput character-by-character
│   ├── clipboard.rs         # Manual clipboard mode
│   └── pipeline.rs          # Orchestrates method selection + fallback
├── platform/                # Platform abstraction layer
│   ├── mod.rs               # Trait definitions
│   ├── windows/             # Windows implementations
│   │   ├── mod.rs
│   │   ├── window_info.rs   # GetForegroundWindow, window class detection
│   │   ├── elevation.rs     # Process integrity level checks
│   │   ├── input.rs         # SendInput wrapper
│   │   └── shell.rs         # ShellExecuteW for admin relaunch
│   └── macos/               # Future — stub/placeholder
│       └── mod.rs
├── config/                  # Configuration management
│   ├── mod.rs
│   ├── types.rs             # AppConfig struct with serde
│   └── storage.rs           # Load/save JSON to app data dir
├── tray.rs                  # System tray setup and event handling
└── state.rs                 # Tauri managed state (AppState)
```

### Platform Abstraction Traits

```rust
// platform/mod.rs

pub trait WindowInfo {
    fn get_foreground_window(&self) -> Result<WindowHandle>;
    fn get_window_class(&self, handle: &WindowHandle) -> Result<String>;
    fn is_terminal(&self, class_name: &str) -> bool;
    fn set_foreground(&self, handle: &WindowHandle) -> Result<()>;
}

pub trait ElevationChecker {
    fn get_integrity_level(&self, handle: &WindowHandle) -> Result<IntegrityLevel>;
    fn is_elevated(&self) -> bool;  // Is VoxFlow itself elevated?
    fn relaunch_elevated(&self) -> Result<()>;
}

pub trait InputSimulator {
    fn send_paste_shortcut(&self, is_terminal: bool) -> Result<()>;
    fn send_character(&self, ch: char) -> Result<()>;
    fn send_key_return(&self) -> Result<()>;
}

pub trait ClipboardAccess {
    fn read_text(&self) -> Result<Option<String>>;
    fn write_text(&self, text: &str) -> Result<()>;
}
```

## Frontend Component Structure

```
src/
├── App.vue                        # Root — routes between settings and wizard
├── main.ts                        # Vue app entry
├── windows/
│   ├── settings/                  # Settings window
│   │   ├── SettingsApp.vue        # Root for settings window
│   │   ├── GeneralSettings.vue    # Hotkey, startup, minimize-to-tray
│   │   ├── AudioSettings.vue      # Mic device dropdown
│   │   ├── TranscriptionSettings.vue  # Cloud/Local toggle, provider tabs
│   │   ├── ProviderTab.vue        # API key, model, test connection
│   │   ├── LocalModelSettings.vue # Model download/delete/progress
│   │   └── InjectionSettings.vue  # Method selector, speed, fallback
│   ├── indicator/                 # Floating indicator window
│   │   ├── IndicatorApp.vue       # Root for indicator window
│   │   └── Waveform.vue           # Audio level bars visualization
│   └── wizard/                    # Setup wizard (can be in settings window)
│       ├── SetupWizard.vue        # Multi-step container
│       ├── EngineSelect.vue       # Step 1: Cloud vs Local
│       ├── ProviderSetup.vue      # Step 2a: API key entry
│       ├── ModelDownload.vue      # Step 2b: Model download
│       └── HotkeyConfirm.vue     # Step 3: Hotkey confirmation
├── components/
│   └── Toast.vue                  # Reusable toast notification
├── composables/
│   ├── useConfig.ts               # Config state + Tauri command wrappers
│   ├── useRecording.ts            # Recording state machine
│   └── useToast.ts                # Toast notification manager
└── types/
    └── index.ts                   # TypeScript types matching Rust structs
```

## Data Flow

### Main Pipeline: Hotkey → Text Injected

```
1. User presses hotkey (Ctrl+Shift+Space)
   ↓
2. Tauri global-shortcut handler fires in Rust
   ↓
3. State machine: Idle → Recording
   - Save foreground window handle
   - Open cpal audio stream (16kHz mono)
   - Show floating indicator window
   - Emit "recording-started" event to frontend
   ↓
4. Audio callback runs at ~30fps
   - Buffer PCM samples to Vec<i16>
   - Compute RMS amplitude per chunk
   - Emit "audio-level" event to floating indicator
   ↓
5. User presses hotkey again
   ↓
6. State machine: Recording → Transcribing
   - Stop cpal stream
   - Emit "processing-started" event
   - Encode audio (Opus for cloud, WAV for local)
   ↓
7. Transcribe
   - Cloud: Tokio task → reqwest POST to provider API
   - Local: std::thread → whisper-rs inference
   - On error: retry/fallback logic
   ↓
8. State machine: Transcribing → Injecting
   - Emit "injection-started" event
   - Check target window integrity level
   - If elevated → emit "elevation-required" → wait for user decision
   ↓
9. Inject text
   - FlashPaste: clipboard save → write → paste shortcut → 500ms → restore
   - Keystrokes: SendInput loop with cancel flag check
   - Clipboard: write to clipboard, done
   - On failure: fallback chain
   ↓
10. State machine: Injecting → Idle
    - Emit "injection-done" event
    - Hide floating indicator
    - Show success/error toast
```

### IPC Pattern

- **Commands (frontend → Rust):** `invoke("command_name", { args })` — for user-initiated actions (save config, start recording, test connection)
- **Events (Rust → frontend):** `emit("event_name", payload)` — for async updates (audio levels, state changes, errors, progress)
- **Managed State:** `AppState` struct in Tauri state — holds config, recording state, cancel flag

## Suggested Build Order

Based on dependencies:

1. **Project scaffolding** — Tauri v2 + Vue 3 + plugins + Rust crate deps
2. **Config system** — AppConfig struct, load/save, Tauri commands (everything reads config)
3. **System tray** — App shell, minimize-to-tray, quit (foundational UX shell)
4. **Global hotkey** — Registration, toggle state machine (triggers everything)
5. **Audio capture** — cpal recording, RMS events (must work before transcription)
6. **Floating indicator** — Second window, waveform visualization (needs audio events)
7. **Audio encoding** — Opus + WAV encoders (bridges capture → transcription)
8. **Cloud transcription** — Provider trait + OpenAI impl first, then Groq + OpenRouter
9. **Text injection** — FlashPaste first (most universal), then keystrokes, then clipboard mode
10. **End-to-end pipeline** — Wire hotkey → record → encode → transcribe → inject
11. **Settings UI** — All settings sections (can be built in parallel with pipeline)
12. **Setup wizard** — First-launch flow (depends on settings components)
13. **Local transcription** — whisper.cpp + model management (independent, complex build)
14. **Toast notifications** — Status/error feedback (polish layer)
15. **Polish** — Elevation detection, fallback chain, cancellation, edge cases

**Parallelizable pairs:**
- Audio capture + Text injection (test injection with hardcoded strings)
- Cloud transcription + Settings UI
- Local transcription + Toast notifications
- Setup wizard + Polish
