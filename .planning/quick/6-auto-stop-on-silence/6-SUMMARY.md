---
phase: quick-6
plan: 1
subsystem: audio
tags: [vad, auto-stop, silence-detection, config]
dependency_graph:
  requires: []
  provides: [vad-silence-auto-stop]
  affects: [audio/mod.rs, commands/audio.rs, lib.rs, indicator/App.vue, AudioSection.vue]
tech_stack:
  added: []
  patterns: [AtomicU32-rms-sampling, tauri-event-bridge, frontend-command-invoke]
key_files:
  created: []
  modified:
    - src-tauri/src/audio/mod.rs
    - src-tauri/src/commands/audio.rs
    - src-tauri/src/lib.rs
    - src/windows/indicator/App.vue
    - src/windows/settings/components/AudioSection.vue
decisions:
  - trigger_stop_recording command guards on RecordingState::Recording before calling toggle_recording_state — safe to call from frontend without race conditions
  - VAD timer accumulates in the emit loop (34ms ticks) — no additional thread needed; reuses existing RMS measurement
  - silence_elapsed_ms resets to 0 on any RMS >= vad_threshold tick — trailing-edge detection (measures contiguous silence)
metrics:
  duration: ~8min
  completed: 2026-04-03
  completed_tasks: 2
  total_tasks: 2
  files_modified: 5
---

# Phase quick-6: VAD Auto-Stop on Silence Summary

**One-liner:** Wired existing config fields `vad_threshold`/`vad_silence_ms` into the audio capture thread's emit loop with a new `trigger_stop_recording` Tauri command as the stop bridge.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | VAD silence timer in capture thread | 63a0134 | src-tauri/src/audio/mod.rs |
| 2 | Wire vad-silence-stop + raise UI max | 941b4ee | commands/audio.rs, lib.rs, App.vue, AudioSection.vue |

## What Was Built

The VAD (voice activity detection) auto-stop feature existed in config and UI but was completely unwired in the backend. This plan closes that gap end-to-end:

**Rust backend (`audio/mod.rs`):**
- `start_realtime_level_capture` now accepts `vad_threshold: f32` and `vad_silence_ms: u32`
- `start_recording_with_snapshot` reads these from config and passes them through
- The emit loop accumulates `silence_elapsed_ms` on each 34ms tick when `rms < vad_threshold`; resets to 0 when audio detected
- When `vad_silence_ms > 0` and `silence_elapsed_ms >= vad_silence_ms`, emits `"vad-silence-stop"` Tauri event and sets `stop_flag` to stop the capture thread

**Rust command (`commands/audio.rs`):**
- New `trigger_stop_recording` command: checks `RecordingState::Recording` before calling `toggle_recording_state` — the same function the global hotkey fires

**Frontend (`indicator/App.vue`):**
- Listens for `"vad-silence-stop"` event and invokes `trigger_stop_recording`
- Unlisten handle cleaned up in `onBeforeUnmount`

**UI (`AudioSection.vue`):**
- Raised silence duration max from 5s to 30s (`max="30.0"`, `Math.min(30000, ...)`)

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `cargo check`: passed
- `cargo test`: 101 tests passed, 0 failed
- `npx vue-tsc --noEmit`: passed (no output = clean)

## Self-Check: PASSED

Files exist:
- src-tauri/src/audio/mod.rs — FOUND (VAD timer in emit loop)
- src-tauri/src/commands/audio.rs — FOUND (trigger_stop_recording command)
- src/windows/indicator/App.vue — FOUND (vad-silence-stop listener)
- src/windows/settings/components/AudioSection.vue — FOUND (max=30.0)

Commits exist:
- 63a0134 — FOUND
- 941b4ee — FOUND
