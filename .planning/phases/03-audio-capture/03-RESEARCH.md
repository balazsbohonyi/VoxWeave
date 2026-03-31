# Phase 3: Audio Capture - Research

**Researched:** 2026-03-15
**Domain:** cpal-based capture lifecycle, runtime device fallback, and output encoding contract
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Capture lifecycle
- Start capture immediately when state enters `Recording`; do not gate entry on preflight checks.
- Stop is manual-only in this phase (second hotkey); silence auto-stop remains deferred.
- Drop very short recordings with a warning instead of forwarding to transcription.
- Add a small trailing audio tail on stop to reduce clipped final words.

### Device behavior
- Persist selected mic by device name; if missing, fall back to system default and notify.
- Device list comes directly from capture backend input-device names.
- If selected mic disconnects mid-recording, switch to system default and continue with warning.
- If no device exists at start, stay in `Idle` and show an error notification.

### Encoding contract
- Select output format from active transcription provider at stop time.
- Cloud providers: `Ogg/Opus`.
- Local provider: `16-bit PCM WAV`.
- Retry encode once on failure, then abort with error.

### Carry-forward to Phase 8 settings
- Audio section eventually adds `Auto-stop on silence` toggle and silence-duration control.
- Phase 3 keeps those backend-managed; no full settings surface beyond required mic dropdown.

### Claude's Discretion
- Exact short-recording cutoff.
- Exact trailing-tail duration.
- Exact warning/error copy.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AUDI-01 | Capture from selected/default mic at 16kHz mono | Normalize capture output into a single PCM path at 16kHz mono before encoding. |
| AUDI-02 | Recording starts within 200ms of hotkey press | Transition state first, open stream immediately, push non-critical checks out of critical path. |
| AUDI-03 | Opus for cloud, WAV for local | Resolve provider at stop-time and branch encode pipeline on that value. |
| AUDI-04 | Settings dropdown for mic selection | Add backend list-devices command + minimal settings dropdown bound to `audio.device`. |
| AUDI-05 | Disconnected selected device falls back to default with notification | Re-resolve device on stream errors and when opening stream; emit fallback warning event. |
| AUDI-06 | No microphone shows error notification | Fail fast on start when no input device is available; emit terminal error event and return to `Idle`. |
</phase_requirements>

## Summary

Phase 3 should add a dedicated backend audio service under `src-tauri/src/audio/` and treat hotkey transitions as control-plane triggers, not implementation hosts. The hotkey toggle should call `audio::start_recording()` when entering `Recording` and `audio::stop_and_finalize()` when leaving it. This keeps the phase isolated: capture and encode live in audio modules, while hotkey/tray remain state entrypoints.

Critical-path latency for AUDI-02 comes from avoiding synchronous setup work before state change. The app should enter `Recording` first, then immediately open cpal stream on selected/default device. Device resolution can still emit warnings/errors, but those should not introduce extra UI hops in the start path. If no input device exists, emit an error event and return to `Idle` deterministically.

For AUDI-01/AUDI-03, use one canonical buffer representation in memory (PCM) and separate encode adapters for Opus and WAV. Determine output target from current `transcription.provider` at stop-time to avoid stale mode assumptions from start-time. Encode retry-once behavior should sit behind one function so both formats share the same failure semantics.

For AUDI-04, Phase 3 should add the minimal settings controls needed by the requirement: list input devices and persist selection. Keep this lightweight in the current temporary settings shell and defer full Audio settings architecture to Phase 8.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `cpal` | `0.15.x` | Microphone enumeration + input stream capture | Widely used Rust capture library with device APIs needed for AUDI-01/04/05/06. |
| `hound` | `3.x` | WAV encoding (`16-bit PCM`) | Minimal WAV writer; fits local whisper path contract. |
| `opus` or `audiopus` | current | Ogg/Opus encoding for cloud path | Required output format for cloud provider contract. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tauri` events | existing | Emit audio/device/encoding notifications to UI | Runtime warnings/errors and indicator hooks for later phases. |
| `std::sync` (`Arc`, `Mutex`) | existing | Shared recording session buffers/flags in managed state | Matches current app architecture. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `cpal` | OS-specific WASAPI wrapper | Reject for now: higher Windows lock-in, less reusable seam for future macOS. |
| Manual RIFF writing | custom encoder | Reject: more bug-prone than `hound` with no gain. |

## Architecture Patterns

### Pattern 1: Audio Service Owns Session Lifecycle
Keep start/stop/finalize behavior in `audio` module, not in `hotkey/service.rs`. Hotkey should request transitions only.

### Pattern 2: Device Resolution with Soft Fallback
Resolve selected device by persisted name. If absent, try default and emit `audio-device-fallback` event. If no default, emit `audio-error` and abort start.

### Pattern 3: Late Encoding Decision
Choose encode format at stop/finalize using live config provider, then route through `encode_opus` or `encode_wav`.

### Pattern 4: Fast Start, Defensive Stop
On start: minimal validation only. On stop: trim/append tail, short-audio guard, encode retry-once, final state/event emission.

### Anti-Patterns to Avoid
- Putting cpal callback or buffer ownership directly in hotkey module.
- Encoding on callback thread.
- Treating missing selected device as hard failure without default fallback.
- Building full Phase 8 settings surface in this phase.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WAV serialization | Manual RIFF byte layout | `hound` writer | Lower risk, easier validation. |
| Device list persistence glue in frontend only | Local browser state | Existing `save_config` + Rust config persistence | Keeps single source of truth in backend. |
| Ad-hoc state transitions | scattered `recording_state` writes | one audio lifecycle API called from hotkey toggle | Prevents partial transitions and stuck states. |

## Common Pitfalls

### Pitfall 1: Stream starts only after heavy setup
This breaks AUDI-02. Avoid by opening stream immediately after entering `Recording`.

### Pitfall 2: Sample-rate mismatch leaks downstream
Different devices may provide non-16kHz input. Normalize to 16kHz mono before encode output contract checks.

### Pitfall 3: Device disconnect hard-stops dictation
AUDI-05 requires fallback-to-default when possible; do not abort on first selected-device failure.

### Pitfall 4: Provider decided at start-time only
User can change provider mid-session. Pick format at stop-time for correctness.

### Pitfall 5: Settings dropdown blocks on phase-8 refactor
AUDI-04 is in Phase 3. Add a minimal dropdown now and defer full UX composition later.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust unit tests + Vue unit interaction checks where needed |
| Config file | none |
| Quick run command | `cd src-tauri && cargo test audio -- --nocapture` |
| Full suite command | `cd src-tauri && cargo test && cd .. && npx vue-tsc --noEmit` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AUDI-01 | Capture pipeline outputs 16kHz mono payload contract | unit | `cd src-tauri && cargo test audio::tests::captures_mono_16khz_contract -- --exact` | no - Wave 1 |
| AUDI-02 | Start path transitions to recording and kicks stream quickly | unit + manual | `cd src-tauri && cargo test hotkey::tests::recording_transition_starts_audio_session -- --exact` | no - Wave 1 |
| AUDI-03 | Provider-driven branch outputs Opus for cloud, WAV for local | unit | `cd src-tauri && cargo test audio::tests::selects_encoder_from_provider -- --exact` | no - Wave 2 |
| AUDI-04 | Device list command + settings dropdown persists selected device | unit + frontend | `cd src-tauri && cargo test commands::audio::tests::lists_input_devices -- --exact` and `npx vue-tsc --noEmit` | no - Wave 2 |
| AUDI-05 | Missing selected device falls back to default with warning | unit | `cd src-tauri && cargo test audio::tests::missing_selected_device_falls_back -- --exact` | no - Wave 1 |
| AUDI-06 | No microphone returns error + idle state recovery | unit | `cd src-tauri && cargo test audio::tests::no_device_returns_error -- --exact` | no - Wave 1 |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test audio -- --nocapture`
- **Per wave merge:** `cd src-tauri && cargo test`
- **Before `$gsd-verify-work`:** Rust full suite + TypeScript typecheck green
- **Max feedback latency:** 45 seconds

### Wave 0 Gaps
- [ ] `src-tauri/src/audio/mod.rs` - phase service entrypoints and shared structs
- [ ] `src-tauri/src/audio/capture.rs` - device resolve + stream lifecycle seams
- [ ] `src-tauri/src/audio/encode.rs` - Opus/WAV branch and retry path
- [ ] `src-tauri/src/commands/audio.rs` - list-audio-devices command
- [ ] `src/windows/settings/App.vue` audio-device selector region

## Sources

### Primary (HIGH confidence)
- Existing VoxWeave architecture/docs in `.planning/*`, `CLAUDE.md`, and phase context.
- Current repository code for state/hotkey/config/settings seams.

### Secondary (MEDIUM confidence)
- cpal/hound/opus ecosystem behavior assumptions based on standard Rust usage patterns.

## Metadata

**Research date:** 2026-03-15
**Valid until:** 2026-04-15
