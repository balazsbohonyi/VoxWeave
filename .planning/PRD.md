# PRD: VoxWeave — Voice-to-Text Dictation App

## Introduction

VoxWeave is a cross-platform voice-to-text dictation app that works in any application — including terminals and command-line windows. The user presses a hotkey, speaks, and the transcribed text is automatically pasted into the active window via **FlashPaste** (the default mode: saves the current clipboard, writes the transcription to the clipboard, simulates `Ctrl+V`, and restores the original clipboard after 500ms). Two alternative injection modes are also available: simulated keystrokes (character-by-character via `SendInput`) and manual clipboard (copy to clipboard, user pastes manually).

It supports multiple cloud transcription providers — **OpenAI** (default), **Groq**, and **OpenRouter** — as well as local transcription via **whisper.cpp**. OpenRouter additionally supports non-Whisper models (e.g. Qwen Audio). No account required — just a personal API key for cloud mode.

This PRD covers the **Windows MVP (v1)** in full detail, with high-level phase placeholders for Android (v2) and macOS (v3).

Built with **Tauri v2 (Rust backend) + Vue 3 + TypeScript + Tailwind CSS**.

## Goals

- Provide fast, reliable voice-to-text that works in any Windows application, including terminals and shell windows
- Inject transcribed text via FlashPaste (default), simulated keystrokes, or manual clipboard — user's choice
- Support multiple cloud providers (OpenAI, Groq, OpenRouter) and local transcription (whisper.cpp)
- Show a minimal floating indicator with real-time audio waveform while recording
- Deliver transcribed text into the active window in under 5 seconds (cloud) / under 3 seconds (local) after stopping
- Keep the app lightweight — under 50MB installed (excluding local model files)
- Zero accounts, zero telemetry — personal tool with BYOK (bring your own key) for cloud mode

## User Stories

### US-001: First Launch and API Key Setup

**Description:** As a new user, I want a guided setup experience on first launch so I can configure at least one transcription provider and start using the app immediately.

**Acceptance Criteria:**
- [ ] On first launch (no config file exists), the app opens a setup wizard instead of minimizing to tray
- [ ] Wizard step 1: "Choose your transcription engine" — Cloud or Local
- [ ] If Cloud: wizard step 2 shows the provider tabs (OpenAI, Groq, OpenRouter) with a prompt to enter an API key for at least one provider
- [ ] If Local: wizard step 2 prompts the user to download a whisper.cpp model (with size/quality guidance)
- [ ] API key is validated on entry: a test transcription request is sent (short silent audio clip or health-check endpoint). On success: green checkmark. On failure: inline error with actionable message (e.g. "Invalid key", "Insufficient quota")
- [ ] Wizard step 3: confirm default hotkey (`Ctrl+Shift+Space`) with option to change
- [ ] "Finish" saves config and minimizes to tray with a toast: "VoxWeave is ready — press [hotkey] to start dictating"
- [ ] Setup wizard can be re-opened from Settings at any time
- [ ] Typecheck passes

### US-002: Configure Cloud Transcription Providers

**Description:** As a user, I want to configure one or more cloud transcription providers (OpenAI, Groq, OpenRouter) so I can choose the best service for my needs.

**Acceptance Criteria:**
- [ ] Settings → Transcription section has a tabbed interface with 3 tabs: "OpenAI", "Groq", "OpenRouter"
- [ ] Each tab displays the provider's logo/icon alongside the tab label
- [ ] Each tab contains: API key input (masked by default, toggle to reveal), model selector dropdown, "Test connection" button
- [ ] **OpenAI tab:** models include `whisper-1`, `gpt-4o-transcribe`, `gpt-4o-mini-transcribe`
- [ ] **Groq tab:** models include Groq's available Whisper variants (fetched dynamically if possible, or hardcoded list with manual update)
- [ ] **OpenRouter tab:** models include Whisper variants plus non-Whisper speech-to-text models (e.g. `qwen/qwen-audio-turbo`, other audio models available on OpenRouter). Model list should indicate which models support audio input
- [ ] An "Active provider" indicator shows which tab/provider is currently selected for transcription
- [ ] Switching the active provider is done by selecting the tab and clicking "Set as active" (or the tab of the active provider is visually highlighted)
- [ ] Only the active provider's API key is required — other tabs can be left empty
- [ ] Configurable language hint per provider (or auto-detect)
- [ ] All settings persist across app restarts
- [ ] Typecheck passes

### US-003: Configure Global Hotkey

**Description:** As a user, I want to set a global hotkey to start/stop recording so I can trigger dictation from any application.

**Acceptance Criteria:**
- [ ] Settings panel has a hotkey input field that captures key combinations (e.g. `Ctrl+Shift+Space`)
- [ ] Default hotkey is pre-configured: `Ctrl+Shift+Space`
- [ ] Hotkey works globally — triggers even when the app window is not focused
- [ ] Hotkey operates in toggle mode: first press starts recording, second press stops recording and triggers transcription
- [ ] Conflicting hotkey detection with a warning message
- [ ] Settings persist across app restarts
- [ ] Typecheck passes

### US-004: Record Audio from Microphone

**Description:** As a user, I want the app to capture audio from my microphone when I press the hotkey so my speech can be transcribed.

**Acceptance Criteria:**
- [ ] App requests microphone permission on first use
- [ ] Recording starts immediately when hotkey is pressed (< 200ms latency)
- [ ] Audio is captured from the system default microphone (or user-selected device)
- [ ] Audio is encoded as Opus (16kHz mono) for cloud transcription, or WAV for local whisper.cpp (which requires WAV/PCM input)
- [ ] Recording stops cleanly when hotkey is pressed again (no audio artifacts/clicks)
- [ ] If no microphone is available, show an error notification
- [ ] Typecheck passes

### US-005: Display Floating Recording Indicator with Waveform

**Description:** As a user, I want to see a small floating indicator with a live audio waveform while recording, so I know the app is listening and capturing my voice.

**Acceptance Criteria:**
- [ ] A small floating window (pill-shaped, ~200×48px) appears when recording starts
- [ ] Window is always-on-top and click-through (does not steal focus)
- [ ] Displays a real-time audio waveform visualization reflecting microphone input levels
- [ ] Shows a red recording dot or pulsing indicator alongside the waveform
- [ ] Window disappears when recording stops
- [ ] Position is draggable; last position is remembered across sessions
- [ ] Typecheck passes

### US-006: Transcribe Audio via Cloud Provider

**Description:** As a user, I want my recorded audio sent to my active cloud provider for transcription so I get accurate text back.

**Acceptance Criteria:**
- [ ] After recording stops, audio is encoded as Opus and sent to the active provider's API endpoint
- [ ] **OpenAI:** POST to `https://api.openai.com/v1/audio/transcriptions` with the selected model
- [ ] **Groq:** POST to Groq's transcription endpoint with the selected model
- [ ] **OpenRouter:** POST to OpenRouter's API with the selected model. For non-Whisper models (e.g. Qwen Audio), adapt the request format as required by the model's API contract
- [ ] Configurable language hint (or auto-detect) is sent with the request
- [ ] Transcription result is returned as plain text
- [ ] Handles errors gracefully per provider: invalid API key (show settings prompt with the relevant tab highlighted), rate limit (retry with exponential backoff, max 3 retries), network error (show notification with retry button)
- [ ] If the active provider fails after retries and another provider is configured, offer to retry with the fallback provider via toast action
- [ ] Typecheck passes

### US-007: Transcribe Audio Locally via whisper.cpp

**Description:** As a user, I want to transcribe audio locally using whisper.cpp so I can use the app without an API key or internet connection.

**Acceptance Criteria:**
- [ ] Models are NOT bundled with the installer — they are downloaded on-demand from within Settings
- [ ] Settings panel allows selecting model size: tiny (~75MB), base (~150MB), small (~500MB), medium (~1.5GB) — with quality/speed tradeoffs displayed alongside each option
- [ ] Model download is triggered by a "Download" button next to each model variant, with a progress bar and cancel option
- [ ] Downloaded models are stored in the app's data directory (e.g. `%APPDATA%/VoxWeave/models/`)
- [ ] A "Delete model" option is available to free disk space
- [ ] Transcription runs on a background thread — does not freeze the UI
- [ ] Audio is passed as WAV/PCM to whisper.cpp (Opus is decoded first if needed)
- [ ] Fallback error if model file is missing or corrupt, with a prompt to re-download
- [ ] Typecheck passes

### US-008: Inject Transcription via FlashPaste (Default)

**Description:** As a user, I want the transcribed text automatically pasted into the active window without overwriting my clipboard, so I get a seamless hands-free experience.

**Acceptance Criteria:**
- [ ] FlashPaste is the default injection method
- [ ] Flow: save current clipboard content → write transcription to clipboard → simulate `Ctrl+V` keystroke into the active window → wait 500ms → restore the original clipboard content
- [ ] Works in standard text inputs: browser text areas, VS Code editor, Notepad, Word, Slack message box
- [ ] Works in terminal/shell windows: cmd.exe, PowerShell, Windows Terminal, Git Bash — for terminals, simulate `Ctrl+Shift+V` or `Shift+Insert` instead of `Ctrl+V` (since `Ctrl+V` has different semantics in some shells). Auto-detect if the foreground window is a terminal emulator and use the appropriate paste shortcut
- [ ] Unicode text (accented characters, symbols) is handled correctly via clipboard
- [ ] Multi-line text preserves newlines through the clipboard paste
- [ ] Before pasting, the app checks if the target window is running elevated. If elevated and VoxWeave is not, show a prompt: "Target app is running as Administrator. Relaunch VoxWeave as Admin?" with "Relaunch" and "Copy to clipboard instead" buttons
- [ ] Pressing Escape or the recording hotkey during the paste operation cancels it (relevant mainly for the 500ms restore delay — the paste itself is near-instant)
- [ ] If clipboard save/restore fails, show a warning toast but still complete the injection
- [ ] A brief toast notification confirms "Text pasted" with a preview
- [ ] Typecheck passes

### US-009: Inject Transcription via Simulated Keystrokes

**Description:** As a user, I want the option to inject text character-by-character via simulated keystrokes, for scenarios where clipboard-based injection is undesirable.

**Acceptance Criteria:**
- [ ] After transcription completes, the app identifies the currently focused window/input field
- [ ] Text is injected character-by-character using the Windows `SendInput` API
- [ ] Injection speed is configurable: slow (10ms/char), normal (5ms/char), fast (2ms/char) — default: normal
- [ ] Focus is restored to the previously active window before injection starts (if the app's own window gained focus)
- [ ] Works in standard text inputs: browser address bar, text areas, VS Code editor, Notepad, Word, Slack message box
- [ ] Works in terminal/shell windows: cmd.exe, PowerShell, Windows Terminal, Git Bash
- [ ] Unicode characters (accented letters, symbols) are injected correctly via `KEYEVENTF_UNICODE`
- [ ] Newline characters in the transcription are injected as `VK_RETURN` keystrokes, preserving multi-line formatting
- [ ] Before injection, the app checks if the target window is running elevated (admin) using process integrity level check. If elevated and VoxWeave is not, show a prompt to relaunch or fall back
- [ ] Pressing Escape or the recording hotkey during injection cancels it immediately — already-injected characters remain, remaining characters are discarded, and a toast shows "Injection cancelled — X of Y characters typed"
- [ ] If the target window rejects simulated input, fall back to FlashPaste and notify the user
- [ ] A brief toast notification confirms injection complete with a preview of the text
- [ ] Typecheck passes

### US-010: Copy Transcription to Clipboard (Manual Mode)

**Description:** As a user, I want the option to copy transcribed text to my clipboard so I can paste it manually wherever I want.

**Acceptance Criteria:**
- [ ] When injection mode is set to "Clipboard", the app saves the current clipboard content, writes the transcription to clipboard, and does NOT auto-paste
- [ ] The original clipboard content is NOT restored in this mode (the user explicitly wants the transcription on their clipboard)
- [ ] A brief toast notification confirms "Copied to clipboard" with a preview of the text (first ~50 chars)
- [ ] If clipboard write fails, show an error notification
- [ ] Clipboard mode is also used as a last-resort fallback when both FlashPaste and keystroke injection fail
- [ ] Typecheck passes

### US-011: Configure Text Injection Method

**Description:** As a user, I want to choose between FlashPaste, simulated keystrokes, and manual clipboard in settings.

**Acceptance Criteria:**
- [ ] Settings panel has an "Injection Method" selector with 3 options: "FlashPaste (recommended)", "Keystrokes", "Clipboard"
- [ ] Default is "FlashPaste"
- [ ] When "Keystrokes" is selected, show injection speed sub-option (slow/normal/fast)
- [ ] Option to enable/disable "auto-fallback" chain when the selected method fails (default: enabled). Fallback order: Keystrokes → FlashPaste → Clipboard; FlashPaste → Clipboard
- [ ] Settings persist across app restarts
- [ ] Typecheck passes

### US-012: Show Transcription Status and Errors

**Description:** As a user, I want to see clear feedback about the transcription and injection status (processing, injecting, done, error) so I know what's happening.

**Acceptance Criteria:**
- [ ] Floating indicator shows a "processing" state (e.g. spinner or pulsing animation) after recording stops and while transcription is in progress
- [ ] Floating indicator shows a "typing" state during keystroke injection (e.g. brief text cursor animation), or a brief "pasting" flash for FlashPaste
- [ ] On cancellation (Escape or hotkey during injection): toast shows "Injection cancelled — X of Y characters typed" (keystroke mode) or "Paste cancelled" (FlashPaste)
- [ ] On elevated target detected: modal dialog with "Relaunch as Admin" and "Copy to clipboard instead" buttons
- [ ] On success: toast notification with text preview + method confirmation ("Text pasted" / "Text typed" / "Copied to clipboard")
- [ ] On error: toast notification with actionable message (e.g. "Invalid API key — open settings?", "Injection failed — copied to clipboard instead")
- [ ] On provider fallback: toast with "Groq failed — retried with OpenAI" (or similar)
- [ ] Toast notifications auto-dismiss after 4 seconds, or can be manually dismissed
- [ ] Typecheck passes

### US-013: System Tray Integration

**Description:** As a user, I want the app to live in the system tray so it's always accessible but not cluttering my taskbar.

**Acceptance Criteria:**
- [ ] App shows a tray icon on launch
- [ ] Right-click tray icon shows context menu: "Settings", "Start/Stop Recording", separator, "Quit"
- [ ] Double-click tray icon opens the settings window
- [ ] Closing the main window minimizes to tray (does not quit)
- [ ] Tray icon changes appearance when recording (e.g. red dot overlay)
- [ ] Typecheck passes

### US-014: Select Audio Input Device

**Description:** As a user, I want to choose which microphone to use in settings, in case I have multiple audio input devices.

**Acceptance Criteria:**
- [ ] Settings panel lists all available audio input devices
- [ ] User can select a device from a dropdown
- [ ] Default selection is the system default device
- [ ] Selected device persists across restarts
- [ ] If the selected device is disconnected, fall back to system default and show a notification
- [ ] Typecheck passes

### US-015: Configure Auto-Start

**Description:** As a user, I want the option to have VoxWeave launch automatically when Windows starts, so it's always ready.

**Acceptance Criteria:**
- [ ] Settings → General section has a "Launch on Windows startup" toggle
- [ ] Default is OFF (not auto-starting)
- [ ] When enabled, the app registers itself in the Windows startup registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`) or uses Tauri's autostart plugin
- [ ] When disabled, the startup entry is removed
- [ ] Typecheck passes

## Functional Requirements

- **FR-1:** The app must register a configurable global hotkey (default: `Ctrl+Shift+Space`) that works when any application is focused.
- **FR-2:** The hotkey operates in toggle mode: first press starts recording, second press stops recording and triggers transcription.
- **FR-3:** Audio must be captured from the user-selected microphone (or system default) at 16kHz mono, encoded as Opus for cloud providers or WAV for local whisper.cpp.
- **FR-4:** A floating always-on-top indicator window must appear during recording, showing a real-time audio waveform.
- **FR-5:** The floating indicator must not steal focus from the active application.
- **FR-6:** When recording stops, audio must be sent to the configured transcription engine (active cloud provider or local).
- **FR-7:** Cloud transcription must support three providers: OpenAI, Groq, and OpenRouter, each with their own API key and model configuration.
- **FR-8:** OpenRouter must support non-Whisper speech-to-text models (e.g. Qwen Audio) in addition to Whisper variants.
- **FR-9:** Local transcription must use whisper.cpp Rust bindings running on a background thread.
- **FR-10:** The default injection method is FlashPaste: save clipboard → write transcription → simulate paste keystroke → wait 500ms → restore original clipboard.
- **FR-11:** FlashPaste must detect terminal windows (cmd, PowerShell, Windows Terminal, Git Bash) and use the appropriate paste shortcut (`Ctrl+Shift+V` or `Shift+Insert`) instead of `Ctrl+V`.
- **FR-12:** Simulated keystroke injection must be available as an alternative, using the Windows `SendInput` API with `KEYEVENTF_UNICODE` flags.
- **FR-13:** Keystroke injection speed must be configurable (slow: 10ms/char, normal: 5ms/char, fast: 2ms/char).
- **FR-14:** Keystroke injection must work in terminal/shell windows (cmd, PowerShell, Windows Terminal, Git Bash).
- **FR-15:** Newline characters in transcriptions must be injected as `VK_RETURN` keystrokes (in keystroke mode) to preserve multi-line formatting.
- **FR-16:** Manual clipboard mode must be available: copy transcription to clipboard without auto-pasting.
- **FR-17:** The user must be able to choose between FlashPaste (default), Keystrokes, and Clipboard injection in settings.
- **FR-18:** Fallback chain: if the selected injection method fails, the app must fall back automatically (Keystrokes → FlashPaste → Clipboard; FlashPaste → Clipboard).
- **FR-19:** The app must detect the focused window before injection and restore focus if needed.
- **FR-20:** Before injection (FlashPaste or Keystrokes), the app must check if the target window runs at a higher integrity level (elevated/admin). If so, prompt the user to relaunch VoxWeave as Administrator or fall back to clipboard.
- **FR-21:** Pressing Escape or the recording hotkey during active injection must cancel it immediately.
- **FR-22:** The user's original clipboard content must be preserved and restored after FlashPaste injection (500ms delay before restore).
- **FR-23:** A toast notification must confirm successful injection/copy or display actionable error messages.
- **FR-24:** The app must run in the system tray with a context menu for core actions.
- **FR-25:** First launch must show a setup wizard guiding the user through provider/engine selection, API key entry, and hotkey configuration.
- **FR-26:** All user settings (provider API keys, active provider, hotkey, engine, language, mic device, model size, injection method, injection speed, auto-start) must persist in a local config file.
- **FR-27:** The app must handle error states gracefully: no mic, invalid API key, network failure, missing local model, injection failure, provider failure with fallback offer.
- **FR-28:** Local model files must be downloadable on-demand from within the settings UI with progress indication — NOT bundled with the installer.
- **FR-29:** Auto-start on Windows startup must be available as an opt-in setting (default: off).

## Non-Goals (Out of Scope for v1)

- **No AI auto-editing** — no grammar correction, filler removal, or punctuation enhancement
- **No personal dictionary or custom vocabulary**
- **No tone/context adaptation per application**
- **No streaming/real-time transcription display** — transcription happens after recording stops (batch mode)
- **No multi-language auto-detection** — user sets language manually (auto-detect as a single option is fine)
- **No usage tracking, analytics, or telemetry**
- **No user accounts, subscription, or cloud sync**
- **No Android or macOS support** (see Future Phases below)

## Design Considerations

### Floating Indicator
- Pill-shaped, semi-transparent, ~200×48px
- Contains: red recording dot (pulsing) + real-time waveform bars (5–10 bars reflecting audio amplitude)
- "Processing" state: waveform replaced by a small spinner or indeterminate progress animation
- "Pasting"/"Typing" state: brief visual cue during injection
- Draggable to any screen position; remembers last position
- Dark theme by default, respects system theme if feasible

### Settings Window
- Single-page layout with sections: General, Audio, Transcription, Injection
- **General:** Hotkey config (toggle mode, default `Ctrl+Shift+Space`), launch on startup toggle (default: off), minimize to tray toggle
- **Audio:** Mic device dropdown, optional test recording playback
- **Transcription:** Engine toggle (Cloud / Local)
  - **Cloud sub-section:** Tabbed interface with 3 tabs — OpenAI | Groq | OpenRouter. Each tab shows the provider's logo alongside the label. Each tab contains: API key input (masked, toggle to reveal), model selector dropdown, "Test connection" button, "Set as active" toggle. The active provider's tab is visually highlighted (e.g. colored border or badge)
  - **Local sub-section:** Model selector with download buttons per variant (tiny/base/small/medium), file size labels, quality/speed description, progress bar during download, delete button for downloaded models
- **Injection:** Method selector (FlashPaste / Keystrokes / Clipboard), injection speed selector (slow/normal/fast, only shown for Keystrokes), auto-fallback checkbox

### First-Launch Setup Wizard
- 3-step flow: Engine selection → Provider/API key or model download → Hotkey confirmation
- Clean, focused UI — one decision per step
- "Skip" option on non-critical steps
- Can be re-accessed from Settings → "Re-run setup wizard"

### Toast Notifications
- Appear near the floating indicator or bottom-right of screen
- Minimal: icon + short message + text preview
- Auto-dismiss after 4s

## Technical Considerations

### Tech Stack
- **Framework:** Tauri v2
- **Backend:** Rust
- **Frontend:** Vue 3 + TypeScript + Vite + Tailwind CSS
- **Audio capture:** `cpal` crate (cross-platform audio I/O for Rust)
- **Audio encoding:** `opus` crate for Opus encoding (cloud uploads); raw PCM/WAV for local whisper.cpp
- **Cloud transcription:** `reqwest` crate for HTTP requests to OpenAI, Groq, and OpenRouter APIs
- **Local transcription:** `whisper-rs` crate (Rust bindings for whisper.cpp)
- **Keystroke injection:** Windows `SendInput` API via `winapi` or `windows-sys` crate, using `KEYEVENTF_UNICODE` for full Unicode support
- **Clipboard:** Tauri's built-in clipboard plugin or `arboard` crate (used by FlashPaste and clipboard mode)
- **Global hotkey:** Tauri's global shortcut plugin
- **System tray:** Tauri's tray icon plugin
- **Config storage:** `serde_json` to a local JSON file in the app data directory
- **Auto-start:** Tauri's autostart plugin or direct Windows registry manipulation

### Key Architectural Decisions
- **Audio pipeline:** `cpal` captures raw PCM → buffer in memory → on stop: encode to Opus (cloud) or keep as WAV (local) → pass to transcription engine. Opus encoding at 16kHz mono takes ~50–100ms, negligible vs. the upload time savings (~960KB WAV → ~50–100KB Opus for 30s audio)
- **Waveform data:** Rust computes RMS amplitude per frame chunk, emits to frontend via Tauri event (`audio-level`) at ~30fps
- **Floating window:** Separate Tauri window with `always_on_top: true`, `decorations: false`, `skip_taskbar: true`, `transparent: true`
- **Transcription is async:** Runs on a Tokio task (cloud) or `std::thread` (local whisper.cpp, which is CPU-bound)
- **Multi-provider abstraction:** A `TranscriptionProvider` trait with implementations for OpenAI, Groq, and OpenRouter. Each implementation handles endpoint URL, auth header format, request body shape, and response parsing. OpenRouter's implementation has an additional code path for non-Whisper models that may require a different request format (e.g. chat-completion style with audio input)
- **FlashPaste pipeline:** After transcription → `GetForegroundWindow` to save target handle → check integrity level → read current clipboard via `arboard` and store in memory → write transcription to clipboard → detect if target is a terminal (check window class name against known terminal classes: `ConsoleWindowClass`, `CASCADIA_HOSTING_WINDOW_CLASS`, `mintty`) → simulate appropriate paste shortcut (`Ctrl+Shift+V` for terminals, `Ctrl+V` otherwise) via `SendInput` → `sleep(500ms)` → restore original clipboard content from memory
- **Keystroke injection pipeline:** After transcription → save foreground handle → check integrity level → if elevated, prompt → inject characters via `SendInput` with `KEYEVENTF_UNICODE` → for `\n`, send `VK_RETURN` → configurable inter-key delay → if `SendInput` returns 0, fall back to FlashPaste
- **Injection cancellation:** During keystroke injection, a background listener monitors for Escape key and the global recording hotkey. On detection, the injection loop breaks immediately. A counter tracks characters injected vs total. Already-injected text is not rolled back
- **Admin relaunch flow:** When an elevated target is detected, Rust emits a Tauri event (`elevation-required`) to the frontend, which shows a dialog. "Relaunch" triggers `ShellExecuteW` with `runas` verb to restart VoxWeave elevated. "Copy to clipboard" falls back to clipboard mode for this transcription only (does not change the global setting)
- **Focus management:** Before injection, the app checks if its own window has focus and restores focus to the previously active window via `SetForegroundWindow`
- **Error propagation:** Rust commands return `Result<T, String>` — frontend maps error strings to user-facing messages

### Performance Targets
- Hotkey → recording start: < 200ms
- Opus encoding (30s audio): < 100ms
- Recording stop → text injected (cloud, good connection): < 5 seconds
- Recording stop → text injected (local, base model): < 3 seconds
- FlashPaste injection: < 100ms (near-instant paste + 500ms clipboard restore)
- Keystroke injection throughput: ~200 chars/sec at normal speed (5ms/char)
- Memory usage while idle: < 50MB
- Memory usage while recording: < 150MB

### Dependencies & Risks
- `whisper-rs` requires a C/C++ toolchain for building whisper.cpp — increases build complexity
- Local model files are large (tiny=75MB, base=150MB, small=500MB, medium=1.5GB) — download management and storage considerations
- Global hotkey conflicts with other apps (e.g. Discord, OBS) — need conflict detection
- Tauri v2 transparent window support on Windows can be flaky — may need workarounds
- **Terminal paste shortcut detection** may be fragile — window class name matching is heuristic-based and may miss some terminal emulators. Needs a user-configurable override in future versions
- **FlashPaste clipboard restore race condition** — the 500ms delay should be sufficient for most apps, but very slow paste handlers (e.g. some Electron apps) could read the clipboard after restore. Monitor for user reports
- **Keystroke injection may be blocked** by some apps: UAC-elevated windows, game anti-cheat (EAC, BattlEye), password fields, some Electron apps with custom input handling — FlashPaste or clipboard fallback mitigates this
- **`SendInput` requires matching integrity level** — injection into admin-elevated apps will fail unless VoxWeave is also elevated. Mitigated by auto-detection + relaunch prompt
- **Injection timing sensitivity:** Some apps (e.g. VS Code with autocomplete) may interfere with rapid keystroke injection — the configurable speed setting helps users tune this
- **OpenRouter model diversity** — non-Whisper audio models may have different API contracts, response formats, or latency characteristics. Needs per-model adapter logic
- **Provider API changes** — endpoint URLs and model names may change. Config should allow manual override of API base URL per provider for power users

## Success Metrics

- App installs and runs on Windows 10/11 without errors
- FlashPaste injection works in at least 5 apps (VS Code, Chrome, Word, Notepad, Slack) without user intervention
- FlashPaste works in at least 2 terminal emulators (Windows Terminal, PowerShell)
- Original clipboard content is preserved and restored correctly after FlashPaste
- Keystroke injection works in at least 5 apps as an alternative mode
- Automatic fallback activates correctly when the primary injection method fails
- Cloud transcription latency (stop → text injected) under 5 seconds for recordings up to 60 seconds
- Local transcription latency under 3 seconds for recordings up to 30 seconds (base model)
- Floating indicator renders waveform at ≥ 24fps without visible jank
- App idles in tray at < 50MB memory, < 1% CPU
- First-launch wizard completes in under 60 seconds

---

## Future Phases (Placeholder)

### Phase 2: Android Support
- Background recording via foreground service + Accessibility Service
- Screen-off detection → auto-pause/resume
- Scoped clipboard handling via Accessibility Service
- Mobile-optimized UI (floating bubble trigger instead of hotkey)
- Target: Android 8+ (API 26+)

### Phase 3: macOS Support
- CoreAudio-based microphone capture
- macOS clipboard API integration
- Accessibility API for optional text injection (v3.5)
- macOS permission prompts for mic + accessibility
- Target: macOS 12+

### Phase 4: AI Enhancements (v1.5+)
- Post-transcription AI editing (grammar, punctuation, filler word removal) via GPT-4o-mini
- Per-app tone adaptation (formal for email, casual for chat)
- Personal dictionary / custom vocabulary
- Streaming transcription display (real-time text in overlay)

### Phase 5: Additional Providers & Models
- Dynamic provider registry — add new providers without code changes
- User-configurable API base URL per provider
- Support for self-hosted Whisper endpoints (e.g. whisper-asr-webservice)
