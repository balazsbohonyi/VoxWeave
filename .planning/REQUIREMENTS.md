# Requirements: VoxFlow

**Defined:** 2026-03-14
**Core Value:** Text lands in any window — terminals, editors, browsers — without friction

## v1 Requirements

Requirements for Windows MVP. Each maps to roadmap phases.

### Hotkey

- [x] **HOTK-01**: User can trigger recording via a global hotkey (default `Ctrl+Shift+Space`) from any application
- [x] **HOTK-02**: Hotkey operates in toggle mode — first press starts recording, second press stops and triggers transcription
- [x] **HOTK-03**: User can change the hotkey in settings and the new binding persists across restarts
- [x] **HOTK-04**: App detects and warns about hotkey conflicts with other applications

### Audio

- [x] **AUDI-01**: App captures audio from the selected microphone (or system default) at 16kHz mono
- [x] **AUDI-02**: Recording starts within 200ms of hotkey press
- [x] **AUDI-03**: Audio is encoded as Opus for cloud providers and WAV for local whisper.cpp
- [x] **AUDI-04**: User can select audio input device from a dropdown in settings
- [x] **AUDI-05**: If the selected device is disconnected, app falls back to system default with a notification
- [x] **AUDI-06**: If no microphone is available, app shows an error notification

### Floating Indicator

- [x] **FLOT-01**: A floating pill-shaped window (~200x48px) appears when recording starts
- [x] **FLOT-02**: Floating indicator is always-on-top, click-through, and does not steal focus
- [x] **FLOT-03**: Indicator displays real-time audio waveform (5-10 bars) reflecting mic input at ≥24fps
- [x] **FLOT-04**: Indicator shows distinct states: recording (red pulsing dot + waveform), processing (spinner), injecting (pasting/typing cue)
- [x] **FLOT-05**: Indicator is draggable and remembers its last position across sessions
- [x] **FLOT-06**: Indicator disappears after text injection completes (or error is shown)

### Cloud Transcription

- [x] **CLOD-01**: App supports two cloud providers: OpenAI and Groq
- [x] **CLOD-02**: Each cloud provider has its own API key and model selection stored under `transcription.providers.<id>`; language hint is a single global field
- [x] **CLOD-03**: OpenAI sends to `/v1/audio/transcriptions` with models: whisper-1, gpt-4o-transcribe, gpt-4o-mini-transcribe
- [x] **CLOD-04**: Groq sends to its transcription endpoint with hardcoded models: whisper-large-v3-turbo (default), whisper-large-v3, distil-whisper-large-v3-en
- ~~**CLOD-05**~~: *(dropped — OpenRouter removed; see Key Decisions in PROJECT.md)*
- [x] **CLOD-06**: Invalid API key errors prompt user to open settings with the offending provider tab highlighted
- [x] **CLOD-07**: Rate limit errors (429) retry with exponential backoff, max 3 retries
- [x] **CLOD-08**: Network errors show a notification with a retry button
- [x] **CLOD-09**: If the active provider fails after retries and another provider is configured, offer to retry with the fallback provider via toast action
- [x] **CLOD-10**: User can configure a language hint per provider (or leave on auto-detect)

### Local Transcription

- [x] **LOCL-01**: App supports local transcription via whisper.cpp (whisper-rs)
- [x] **LOCL-02**: Models are NOT bundled — downloaded on-demand from settings with progress bar and cancel option
- [ ] **LOCL-03**: Available models: tiny (~75MB), base (~150MB), small (~500MB), medium (~1.5GB) with quality/speed descriptions
- [x] **LOCL-04**: Downloaded models stored in `%APPDATA%/VoxFlow/models/`; user can delete models to free space
- [x] **LOCL-05**: Local transcription runs on a background thread without freezing the UI
- [x] **LOCL-06**: Audio is passed as WAV/PCM float32 to whisper.cpp
- [x] **LOCL-07**: If the model file is missing or corrupt, show an error with a prompt to re-download

### Injection

- [x] **INJC-01**: FlashPaste is the default injection method: save clipboard → write text → simulate paste → wait 500ms → restore clipboard
- [x] **INJC-02**: FlashPaste detects terminal windows (ConsoleWindowClass, CASCADIA_HOSTING_WINDOW_CLASS, mintty, VirtualConsoleClass) and uses Ctrl+Shift+V or Shift+Insert instead of Ctrl+V
- [x] **INJC-03**: Simulated keystroke injection is available as an alternative — character-by-character via SendInput with KEYEVENTF_UNICODE
- [x] **INJC-04**: Keystroke injection speed is configurable: slow (10ms/char), normal (5ms/char), fast (2ms/char)
- [x] **INJC-05**: Newline characters are injected as VK_RETURN keystrokes in keystroke mode
- [x] **INJC-06**: Manual clipboard mode copies transcription to clipboard without auto-pasting
- [x] **INJC-07**: Before injection, app checks if the target process runs at higher integrity level; if so, shows dialog with "Relaunch as Admin" and "Copy to clipboard" options
- [x] **INJC-08**: Pressing Escape or the hotkey during keystroke injection cancels immediately; toast shows "X of Y characters typed"
- [x] **INJC-09**: Automatic fallback chain when selected method fails: Keystrokes → FlashPaste → Clipboard; FlashPaste → Clipboard (configurable toggle)
- [x] **INJC-10**: Focus is restored to the target window before injection if VoxFlow's window gained focus
- [x] **INJC-11**: Unicode text (accented characters, symbols) is handled correctly in all injection modes

### System Tray

- [x] **TRAY-01**: App shows a tray icon on launch; icon changes appearance when recording is active
- [x] **TRAY-02**: Right-click tray icon shows context menu: Settings, Start/Stop Recording, separator, Quit
- [x] **TRAY-03**: Double-click tray icon opens the settings window
- [x] **TRAY-04**: Closing the settings window minimizes to tray (does not quit)

### Settings

- [x] **SETT-01**: Settings window has sections: General, Audio, Transcription, Injection
- [x] **SETT-02**: General: hotkey capture input, "Launch on Windows startup" toggle (default OFF), minimize-to-tray toggle
- [x] **SETT-03**: Audio: microphone device dropdown listing all available devices, plus `Auto-stop on silence` toggle and configurable silence-duration input
- [x] **SETT-04**: Transcription: Cloud/Local engine toggle; Cloud has tabbed interface (OpenAI, Groq) each with API key (masked), model dropdown (list sourced from Rust constants via `get_provider_models` command), language hint input, "Test connection", "Set as active"; active provider visually highlighted
- [x] **SETT-05**: Transcription: Local sub-section with model variants, sizes, download/delete buttons, progress bar
- [x] **SETT-06**: Injection: method selector (FlashPaste/Keystrokes/Clipboard), speed selector (shown only for Keystrokes), auto-fallback checkbox
- [x] **SETT-07**: All settings persist immediately (no save button) and are restored on restart

### Setup Wizard

- [x] **WIZR-01**: On first launch (no config), app opens a 3-step setup wizard instead of minimizing to tray
- [x] **WIZR-02**: Step 1: choose engine (Cloud or Local)
- [x] **WIZR-03**: Step 2: configure provider/API key with inline validation (Cloud) or download model (Local)
- [x] **WIZR-04**: Step 3: confirm default hotkey with option to change
- [x] **WIZR-05**: "Finish" saves config and shows "VoxFlow is ready" ~~toast~~ success banner
- [x] **WIZR-06**: Wizard can be re-opened from Settings at any time

### Notifications

- [x] **NOTF-01**: Success toasts confirm injection method and show text preview ("Text pasted", "Text typed", "Copied to clipboard")
- [x] **NOTF-02**: Error toasts show actionable messages (open settings, retry, fallback notification)
- [x] **NOTF-03**: Cancellation toasts show "X of Y characters typed" or "Paste cancelled"
- [x] **NOTF-04**: Toasts auto-dismiss after 4 seconds and can be manually dismissed

### Config

- [x] **CONF-01**: All settings persist in JSON at `%APPDATA%/VoxFlow/config.json`
- [x] **CONF-02**: Config includes: engine, active provider, per-provider API key and model under `transcription.providers`, global language hint, hotkey, mic device, local model path, injection method/speed, auto-fallback, autostart, indicator position, first-launch flag
- [x] **CONF-03**: Missing fields use defaults; unknown fields are ignored (forward/backward compatible)

## v2 Requirements

### Cross-Platform

- **XPLT-01**: macOS support via CoreAudio, CGEvent, NSPasteboard
- **XPLT-02**: Android support via foreground service + Accessibility Service

### AI Enhancements

- **AIEH-01**: Post-transcription AI editing (grammar, punctuation, filler removal)
- **AIEH-02**: Per-app tone adaptation
- **AIEH-03**: Personal dictionary / custom vocabulary

### Advanced

- **ADVN-01**: Streaming/real-time transcription display
- **ADVN-02**: Dynamic provider registry (add providers without code changes)
- **ADVN-03**: User-configurable API base URL per provider

## Out of Scope

| Feature | Reason |
|---------|--------|
| Usage analytics / telemetry | Against BYOK philosophy — zero data collection, ever |
| User accounts / cloud sync | Against BYOK philosophy — personal tool, local config only |
| Hold-to-record mode | Toggle mode is simpler and more standard for dictation |
| Multi-language auto-detection | Manual language selection + auto-detect option is sufficient for v1 |
| Streaming transcription display | Doubles pipeline complexity for marginal UX gain |
| AI post-processing | Adds latency, LLM cost, scope — defer to v2 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| HOTK-01 | Phase 2 | Complete |
| HOTK-02 | Phase 2 | Complete |
| HOTK-03 | Phase 2 | Complete |
| HOTK-04 | Phase 2 | Complete |
| AUDI-01 | Phase 3 | Complete |
| AUDI-02 | Phase 3 | Complete |
| AUDI-03 | Phase 3 | Complete |
| AUDI-04 | Phase 3 | Pending |
| AUDI-05 | Phase 3 | Pending |
| AUDI-06 | Phase 3 | Pending |
| FLOT-01 | Phase 4 | Complete |
| FLOT-02 | Phase 4 | Complete |
| FLOT-03 | Phase 4 | Complete |
| FLOT-04 | Phase 4 | Complete |
| FLOT-05 | Phase 4 | Complete |
| FLOT-06 | Phase 4 | Complete |
| CLOD-01 | Phase 5 | Complete |
| CLOD-02 | Phase 5 | Complete |
| CLOD-03 | Phase 5 | Complete |
| CLOD-04 | Phase 5 | Complete |
| CLOD-05 | Phase 5 | Complete |
| CLOD-06 | Phase 5 | Complete |
| CLOD-07 | Phase 5 | Complete |
| CLOD-08 | Phase 5 | Complete |
| CLOD-09 | Phase 5 | Complete |
| CLOD-10 | Phase 5 | Complete |
| LOCL-01 | Phase 10 | Complete |
| LOCL-02 | Phase 10 | Complete |
| LOCL-03 | Phase 10 | Pending |
| LOCL-04 | Phase 10 | Complete |
| LOCL-05 | Phase 10 | Complete |
| LOCL-06 | Phase 10 | Complete |
| LOCL-07 | Phase 10 | Complete |
| INJC-01 | Phase 6 | Complete |
| INJC-02 | Phase 6 | Complete |
| INJC-03 | Phase 6 | Complete |
| INJC-04 | Phase 6 | Complete |
| INJC-05 | Phase 6 | Complete |
| INJC-06 | Phase 6 | Complete |
| INJC-07 | Phase 6 | Complete |
| INJC-08 | Phase 6 | Complete |
| INJC-09 | Phase 6 | Complete |
| INJC-10 | Phase 6 | Complete |
| INJC-11 | Phase 6 | Complete |
| TRAY-01 | Phase 1 | Complete |
| TRAY-02 | Phase 1 | Complete |
| TRAY-03 | Phase 1 | Complete |
| TRAY-04 | Phase 1 | Complete |
| SETT-01 | Phase 8 | Complete |
| SETT-02 | Phase 8 | Complete |
| SETT-03 | Phase 8 | Complete |
| SETT-04 | Phase 8 | Complete |
| SETT-05 | Phase 8 | Complete |
| SETT-06 | Phase 8 | Complete |
| SETT-07 | Phase 8 | Complete |
| WIZR-01 | Phase 9 | Complete |
| WIZR-02 | Phase 9 | Complete |
| WIZR-03 | Phase 9 | Complete |
| WIZR-04 | Phase 9 | Complete |
| WIZR-05 | Phase 9 | Pending |
| WIZR-06 | Phase 9 | Complete |
| NOTF-01 | Phase 7 | Complete |
| NOTF-02 | Phase 7 | Complete |
| NOTF-03 | Phase 7 | Complete |
| NOTF-04 | Phase 7 | Complete |
| CONF-01 | Phase 1 | Complete |
| CONF-02 | Phase 1 | Complete |
| CONF-03 | Phase 1 | Complete |

**Coverage:**
- v1 requirements: 68 total
- Mapped to phases: 68
- Unmapped: 0

---
*Requirements defined: 2026-03-14*
*Last updated: 2026-03-14 after roadmap creation — all 68 requirements mapped*


