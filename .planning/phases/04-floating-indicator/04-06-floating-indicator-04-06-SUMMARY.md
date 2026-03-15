---
phase: 04-floating-indicator
plan: "04-06"
subsystem: ui
tags: [tauri, rust, cpal, vue, waveform, indicator]
requires:
  - phase: 04-floating-indicator
    provides: Indicator drag/position lifecycle and visual state events
provides:
  - Realtime `audio-level` emission from active microphone capture during recording
  - Recording waveform behavior driven by backend mic amplitude events
  - Recording idle fallback that avoids synthetic false-positive activity when no audio events arrive
affects: [cloud-transcription, injection, uat]
tech-stack:
  added: []
  patterns:
    - Dedicated audio capture thread owns non-Send cpal stream; AppState stores only stop/join controls
    - Indicator recording waveform uses freshness-gated backend level data with light smoothing
key-files:
  created: []
  modified:
    - src-tauri/src/audio/mod.rs
    - src-tauri/src/audio/session.rs
    - src-tauri/src/state.rs
    - src/windows/indicator/App.vue
key-decisions:
  - "Keep cpal::Stream out of AppState (not Send/Sync) and run capture in a dedicated thread."
  - "Recording waveform only reflects recent backend levels; stale event gaps fall back to baseline instead of synthetic spikes."
patterns-established:
  - "Live audio telemetry pattern: callback updates atomic RMS, emitter loop publishes throttled events (~29fps)."
  - "Indicator truthfulness pattern: recording visuals are data-first, processing/injecting remain state-driven."
requirements-completed: [FLOT-03, FLOT-04]
duration: 14min
completed: 2026-03-15
---

# Phase 04 Plan 06: Indicator Waveform Reactive Gap Summary

**Realtime microphone RMS now drives indicator waveform animation during recording with stale-event suppression and preserved processing/injecting visuals**

## Performance

- **Duration:** 14 min
- **Started:** 2026-03-15T22:05:10Z
- **Completed:** 2026-03-15T22:19:49Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Implemented active microphone capture startup that emits normalized `audio-level` payloads on a stable cadence during recording.
- Added session lifecycle stop controls so the live capture/emitter loop shuts down cleanly when recording finalizes.
- Removed synthetic recording-wave masking in the indicator and made recording waveform behavior rely on recent backend level events.

## Task Commits

Each task was committed atomically:

1. **Task 1: Emit real-time mic level events from active capture path** - `36821e8` (feat)
2. **Task 2: Make waveform strictly data-driven for recording state** - `3720ae7` (feat)

## Files Created/Modified
- `src-tauri/src/audio/mod.rs` - Added live cpal capture thread, RMS normalization helpers, throttled `audio-level` emission, and related tests.
- `src-tauri/src/audio/session.rs` - Extended session creation to carry stop/join handles for live capture lifecycle.
- `src-tauri/src/state.rs` - Added runtime capture control fields to `AudioSessionState` while keeping managed state Send/Sync-safe.
- `src/windows/indicator/App.vue` - Recording animation now uses fresh backend levels with smoothing and baseline fallback when events go stale.

## Decisions Made
- Used a dedicated worker thread to own `cpal::Stream` because `cpal::Stream` is not `Send/Sync` and cannot be stored directly in `AppState`.
- Recording waveform now requires fresh `audio-level` events and returns to baseline after a short staleness window, preventing fake “speaking” activity.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] cpal stream ownership in AppState broke Tauri Send/Sync bounds**
- **Found during:** Task 1 (Emit real-time mic level events from active capture path)
- **Issue:** Storing `cpal::Stream` in `AudioSessionState` made `AppState` fail `Send + Sync` constraints required by Tauri command state access.
- **Fix:** Moved stream ownership into a dedicated audio thread and kept only stop/join controls in managed state.
- **Files modified:** `src-tauri/src/audio/mod.rs`, `src-tauri/src/audio/session.rs`, `src-tauri/src/state.rs`
- **Verification:** `cargo test audio -- --nocapture` and `cargo test indicator -- --nocapture`
- **Committed in:** `36821e8` (part of Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for correctness and runtime compatibility; no scope creep.

## Issues Encountered
- Initial implementation compiled locally for audio code but violated Tauri-managed state thread-safety bounds. Resolved by thread-owned capture lifecycle.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Indicator now exposes truthful, live mic reactivity needed for UAT waveform checks.
- Ready for downstream transcription/injection phases that depend on clean indicator state transitions.

---
*Phase: 04-floating-indicator*
*Completed: 2026-03-15*

## Self-Check: PASSED
- Found summary file and task commit hashes (36821e8, 3720ae7).
