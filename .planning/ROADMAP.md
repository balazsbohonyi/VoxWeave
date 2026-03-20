# Roadmap: VoxFlow

## Overview

VoxFlow is built in 10 phases that follow the natural dependency order of the pipeline: foundation first, then the hotkey trigger, then audio capture, then the visual indicator, then transcription (cloud), then injection — finally integrating all pieces end-to-end before layering on settings UI, first-launch onboarding, and the optional local transcription path. Every phase delivers a coherent, independently testable capability. The result is a working dictation tool that places text into any window, including terminals, using the user's own API keys.

## Phases

**Phase Numbering:**
- Integer phases (1–10): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation** - Project scaffold, config persistence, and system tray presence (completed 2026-03-14)
- [x] **Phase 2: Hotkey** - Global hotkey registration and toggle-mode recording trigger (completed 2026-03-15)
- [ ] **Phase 3: Audio Capture** - Microphone capture, encoding, and device management
- [x] **Phase 4: Floating Indicator** - Always-on-top recording status window with waveform (completed 2026-03-15)
- [x] **Phase 5: Cloud Transcription** - OpenAI, Groq, and OpenRouter providers with error handling (completed 2026-03-17)
- [ ] **Phase 6: Text Injection** - FlashPaste, keystroke, and clipboard injection with fallback chain
- [ ] **Phase 7: Pipeline Integration** - End-to-end hotkey-to-text pipeline with toast notifications
- [ ] **Phase 8: Settings UI** - Full settings window for all configurable parameters
- [ ] **Phase 9: Setup Wizard** - First-launch onboarding flow
- [ ] **Phase 10: Local Transcription** - whisper.cpp integration with on-demand model download

## Phase Details

### Phase 1: Foundation
**Goal**: A launchable Tauri app with config persistence, system tray presence, and platform abstraction scaffolding
**Depends on**: Nothing (first phase)
**Requirements**: CONF-01, CONF-02, CONF-03, TRAY-01, TRAY-02, TRAY-03, TRAY-04
**Success Criteria** (what must be TRUE):
  1. App launches on Windows and appears as a tray icon without a visible window
  2. Right-clicking the tray shows a context menu with Settings, Start/Stop Recording, and Quit
  3. Double-clicking the tray icon opens the settings window
  4. Closing the settings window minimizes to tray rather than quitting
  5. Config is read from and written to `%APPDATA%/VoxFlow/config.json`; missing fields use defaults and unknown fields are preserved
**Plans**: 5 plans
Plans:
- [x] 04-01-indicator-window-runtime-PLAN.md - indicator window runtime + lifecycle wiring
- [x] 04-02-indicator-visual-states-waveform-PLAN.md - indicator UI states + waveform visuals
- [x] 04-03-indicator-drag-persistence-hide-PLAN.md - drag interaction + persistence + hide hardening
- [ ] 04-04-indicator-focus-clickthrough-gap-PLAN.md - close focus theft + default click-through gap
- [ ] 04-05-indicator-position-snapback-gap-PLAN.md - close drag-end persistence + snap-back gap

### Phase 2: Hotkey
**Goal**: A global hotkey that can be triggered from any application and drives a toggle-mode recording state machine
**Depends on**: Phase 1
**Requirements**: HOTK-01, HOTK-02, HOTK-03, HOTK-04
**Success Criteria** (what must be TRUE):
  1. Pressing the default hotkey (`Ctrl+Shift+Space`) from any focused application triggers a state change in VoxFlow
  2. A second hotkey press advances state from recording to processing (toggle mode)
  3. A custom hotkey set in settings takes effect immediately and survives app restart
  4. If the chosen hotkey conflicts with an existing binding, the app warns the user
**Plans**: TBD

### Phase 3: Audio Capture
**Goal**: Audio is captured from the selected microphone, encoded in the correct format for the active transcription mode, within 200ms of the hotkey press
**Depends on**: Phase 2
**Requirements**: AUDI-01, AUDI-02, AUDI-03, AUDI-04, AUDI-05, AUDI-06
**Success Criteria** (what must be TRUE):
  1. Recording begins within 200ms of hotkey press
  2. Audio is captured at 16kHz mono from the selected device (or system default)
  3. Captured audio is encoded as Opus when a cloud provider is active and as WAV when local whisper is active
  4. A microphone device dropdown in settings lists all available input devices and the selection persists
  5. If the selected device is disconnected, the app falls back to system default with a notification; if no device exists, an error notification is shown
**Plans**: TBD

### Phase 4: Floating Indicator
**Goal**: A floating pill-shaped window provides real-time visual feedback for every state in the recording pipeline without interrupting the user's workflow
**Depends on**: Phase 3
**Requirements**: FLOT-01, FLOT-02, FLOT-03, FLOT-04, FLOT-05, FLOT-06
**Note**: Deferred from Phase 2 UAT - indicator visibility checks from .planning/phases/02-hotkey/02-UAT.md were intentionally moved here because indicator delivery is Phase 4 scope.
**Success Criteria** (what must be TRUE):
  1. A pill-shaped (~200x48px) always-on-top, click-through window appears when recording starts and does not steal keyboard focus
  2. The indicator displays a live waveform (5–10 bars) at ≥24fps while recording is active
  3. The indicator transitions through distinct visual states: recording (red pulsing dot + waveform), processing (spinner), and injecting (paste/typing cue)
  4. The indicator can be dragged to any screen position and remembers that position across sessions
  5. The indicator automatically disappears after injection completes or an error is shown
**Plans**: TBD

### Phase 5: Cloud Transcription
**Goal**: Transcribed text is returned from any of the three configured cloud providers, with graceful handling of errors, rate limits, and provider fallback
**Depends on**: Phase 3
**Requirements**: CLOD-01, CLOD-02, CLOD-03, CLOD-04, CLOD-05, CLOD-06, CLOD-07, CLOD-08, CLOD-09, CLOD-10
**Success Criteria** (what must be TRUE):
  1. Audio is successfully transcribed via OpenAI (`/v1/audio/transcriptions`), Groq (transcription endpoint), and OpenRouter (chat completions with `input_audio`) using the user's API keys and chosen models
  2. Each provider's API key, model selection, and language hint are independently configurable and persist across restarts
  3. An invalid API key triggers a notification that opens settings with the offending provider's tab highlighted
  4. Rate-limit errors (429) retry with exponential backoff up to 3 times before surfacing an error
  5. If the active provider fails after retries and another provider is configured, a toast offers to retry with the fallback provider
**Plans**: 7 plans
Plans:
- [ ] 05-01-PLAN.md — provider trait, config extension, and OpenAI/Groq/OpenRouter HTTP impls
- [ ] 05-02-PLAN.md — transcription service: retry loop, fallback logic, error event emission
- [ ] 05-03-PLAN.md — hotkey wiring, retry command, frontend error event handler
- [ ] 05-04-PLAN.md — gap closure: fix hotkey warning focus + error toast indicator visibility
- [ ] 05-05-PLAN.md — gap closure: fix OpenAI audio format (WAV) + friendly Network error message
- [ ] 05-06-PLAN.md — gap closure: fix invalid_key toast handler + add toast CSS
- [ ] 05-07-PLAN.md — gap closure: reposition toast as in-pill overlay to fix compositor clipping

### Phase 05.1: implement real PCM accumulation and Opus encoder (INSERTED)

**Goal:** Replace three audio stubs (synthetic PCM, fake Opus encoder, silent log backend) so recordings contain real microphone audio that cloud transcription providers can decode
**Requirements**: AUDI-01, AUDI-02, AUDI-03
**Depends on:** Phase 5
**Plans:** 7/7 plans complete

Plans:
- [ ] 05.1-01-PLAN.md — Cargo.toml deps + AudioSessionState PCM buffer + real PCM accumulation in capture thread
- [ ] 05.1-02-PLAN.md — Real Opus encoder (audiopus + ogg) replacing DefaultEncoderBackend stub
- [ ] 05.1-03-PLAN.md — tauri-plugin-log wiring (lib.rs + capabilities)

### Phase 6: Text Injection
**Goal**: Transcribed text lands in the target window using the most reliable available method, with terminal-aware shortcuts, elevation checks, Unicode handling, and a resilient fallback chain
**Depends on**: Phase 5
**Requirements**: INJC-01, INJC-02, INJC-03, INJC-04, INJC-05, INJC-06, INJC-07, INJC-08, INJC-09, INJC-10, INJC-11
**Success Criteria** (what must be TRUE):
  1. FlashPaste injects text by saving the clipboard, pasting via the correct shortcut (Ctrl+V for normal windows; Ctrl+Shift+V or Shift+Insert for detected terminal windows), then restoring the clipboard
  2. Keystroke injection types text character-by-character at the configured speed (slow/normal/fast), with newlines sent as VK_RETURN
  3. Manual clipboard mode copies text to clipboard without auto-pasting
  4. Before injection, if the target process runs at a higher integrity level, a dialog offers "Relaunch as Admin" or "Copy to clipboard"
  5. Pressing Escape during keystroke injection cancels immediately and shows a toast with the count of characters typed
  6. The automatic fallback chain (Keystrokes → FlashPaste → Clipboard) engages when the selected method fails
  7. Unicode characters (accented letters, symbols) are injected correctly in all three modes
**Plans**: TBD

### Phase 7: Pipeline Integration
**Goal**: The complete hotkey-to-text pipeline works end-to-end as a seamless user experience, with toast notifications confirming every outcome
**Depends on**: Phase 6
**Requirements**: NOTF-01, NOTF-02, NOTF-03, NOTF-04
**Success Criteria** (what must be TRUE):
  1. A single hotkey press starts recording; a second press produces transcribed text injected into the active window without any manual steps
  2. A success toast confirms the injection method and shows a text preview ("Text pasted", "Text typed", "Copied to clipboard")
  3. Error toasts show actionable messages (open settings, retry, use fallback) rather than raw error strings
  4. Cancellation toasts show the number of characters typed or "Paste cancelled"
  5. All toasts auto-dismiss after 4 seconds and can be manually dismissed earlier
**Plans**: TBD

### Phase 8: Settings UI
**Goal**: A full settings window lets the user configure every aspect of VoxFlow, with changes taking effect immediately and persisting across restarts
**Depends on**: Phase 7
**Requirements**: SETT-01, SETT-02, SETT-03, SETT-04, SETT-05, SETT-06, SETT-07
**Note from Phase 3 context**: Audio settings must expose an `Auto-stop on silence` toggle and silence-duration control; defaults remain backend-managed until this phase.
**Success Criteria** (what must be TRUE):
  1. The settings window has four sections: General, Audio, Transcription, and Injection, each exposing all relevant controls
  2. General section includes the hotkey capture input, a "Launch on Windows startup" toggle (default OFF), and a minimize-to-tray toggle
  3. Transcription section has a Cloud/Local engine toggle; Cloud shows a tabbed interface (OpenAI, Groq, OpenRouter) with API key (masked), model dropdown, "Test connection", and "Set as active" per tab
  4. Local transcription sub-section lists model variants with sizes, download/delete buttons, and a progress bar
  5. Injection section provides method selector (FlashPaste/Keystrokes/Clipboard), speed selector (shown only for Keystrokes), auto-fallback checkbox
  6. All setting changes persist immediately with no save button and are correctly restored on app restart
**Plans**: TBD

### Phase 9: Setup Wizard
**Goal**: A new user completes first-time configuration through a guided 3-step wizard before the app begins minimizing to tray on launch
**Depends on**: Phase 8
**Requirements**: WIZR-01, WIZR-02, WIZR-03, WIZR-04, WIZR-05, WIZR-06
**Success Criteria** (what must be TRUE):
  1. On first launch (no config file), the setup wizard opens instead of the tray icon appearing alone
  2. Step 1 lets the user choose Cloud or Local transcription engine
  3. Step 2 lets the user configure their API key with inline validation (Cloud) or download a model (Local)
  4. Step 3 lets the user confirm or change the default hotkey
  5. Clicking "Finish" saves config and shows "VoxFlow is ready" toast, after which the app behaves normally (tray-only)
  6. The wizard can be re-opened from Settings at any time
**Plans**: TBD

### Phase 10: Local Transcription
**Goal**: Users who prefer local, offline transcription can download and use whisper.cpp models of their choice without affecting the cloud pipeline
**Depends on**: Phase 3
**Requirements**: LOCL-01, LOCL-02, LOCL-03, LOCL-04, LOCL-05, LOCL-06, LOCL-07
**Success Criteria** (what must be TRUE):
  1. Selecting "Local" engine in settings and triggering a recording produces a transcription without any network call
  2. Models (tiny/base/small/medium) can be downloaded on-demand from the settings panel with a live progress bar and a cancel option
  3. Downloaded models are stored in `%APPDATA%/VoxFlow/models/` and can be deleted from within settings to free disk space
  4. Local transcription runs on a background thread and does not freeze the UI during processing
  5. If a model file is missing or corrupt, an error notification prompts the user to re-download
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10
Note: Phase 10 depends on Phase 3 (not Phase 9); it can be executed after Phase 3 is complete if desired, but is scheduled last to keep the cloud pipeline unblocked.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 3/3 | Complete   | 2026-03-14 |
| 2. Hotkey | 0/TBD | Complete    | 2026-03-15 |
| 3. Audio Capture | 0/TBD | Not started | - |
| 4. Floating Indicator | 3/3 | Complete | 2026-03-15 |
| 5. Cloud Transcription | 7/7 | Complete   | 2026-03-20 |
| 5.1. PCM + Opus | 0/3 | Not started | - |
| 6. Text Injection | 0/TBD | Not started | - |
| 7. Pipeline Integration | 0/TBD | Not started | - |
| 8. Settings UI | 0/TBD | Not started | - |
| 9. Setup Wizard | 0/TBD | Not started | - |
| 10. Local Transcription | 0/TBD | Not started | - |
