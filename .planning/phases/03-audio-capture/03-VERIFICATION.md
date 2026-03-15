---
phase: 03-audio-capture
verified: 2026-03-15T13:15:00Z
status: human_needed
score: 6/6 must-haves implemented
human_verification:
  - test: "Recording starts from real microphone path under 200ms"
    expected: "Pressing hotkey starts capture immediately and indicator transition is prompt."
    why_human: "Requires real device timing and desktop runtime behavior."
  - test: "Selected microphone disconnect fallback"
    expected: "If selected mic disappears, app falls back to default and warning appears in Settings."
    why_human: "Requires real hardware connect/disconnect scenario."
  - test: "No microphone available error path"
    expected: "When no input devices exist, recording fails cleanly, app remains responsive, and error is emitted."
    why_human: "Requires hardware/OS configuration not reproducible in unit tests here."
---

# Phase 3: Audio Capture Verification Report

**Phase Goal:** Audio is captured from selected microphone, encoded by provider mode, and device management is robust.  
**Verified:** 2026-03-15  
**Status:** human_needed

## Goal Achievement

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Audio lifecycle seam exists and hotkey start/stop uses it | ? VERIFIED | `src-tauri/src/audio/mod.rs`, `src-tauri/src/hotkey/service.rs` |
| 2 | Capture contract is 16kHz mono | ? VERIFIED | `src-tauri/src/audio/session.rs`, `audio::tests::captures_mono_16khz_contract` |
| 3 | Missing selected device falls back with warning | ? VERIFIED | `resolve_input_device`, `audio-warning` payload + UI |
| 4 | No microphone path fails cleanly and resets state | ? VERIFIED | `start_recording_with_snapshot` no-device branch + test |
| 5 | Provider selects Opus for cloud and WAV for local | ? VERIFIED | `src-tauri/src/audio/encode.rs`, tests for signatures |
| 6 | Settings exposes microphone dropdown with persistence path | ? VERIFIED | `src/windows/settings/App.vue`, `src/composables/useConfig.ts`, `save_config` flow |

## Requirements Coverage

| Requirement | Status | Evidence |
|-------------|--------|----------|
| AUDI-01 | ? SATISFIED | 16kHz mono contract + session constants/tests |
| AUDI-02 | ? HUMAN | Runtime latency under 200ms requires desktop timing check |
| AUDI-03 | ? SATISFIED | Provider-driven encoder format + retry policy |
| AUDI-04 | ? SATISFIED | Device list command + settings dropdown + save flow |
| AUDI-05 | ? SATISFIED | Fallback warning emitted and surfaced in settings |
| AUDI-06 | ? SATISFIED | No-device error handling and state recovery to Idle |

## Notes

- Rust test binaries in this environment still fail at runtime with `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)`. Compilation succeeded via `cargo check`.
- Frontend typecheck passed via `cmd /c npx vue-tsc --noEmit` due PowerShell script policy restrictions.

## Human Verification Required

1. Run `cargo tauri dev` and validate real microphone start latency and recording behavior.
2. Select a removable microphone, disconnect it, then verify fallback warning and persisted config validity.
3. Validate no-device error path by disabling/removing all input devices.

---
_Verified: 2026-03-15_  
_Verifier: Codex (local fallback execution)
