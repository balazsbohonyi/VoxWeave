---
phase: 05-cloud-transcription
plan: "04"
subsystem: ui
tags: [tauri, rust, vue3, typescript, indicator, hotkey, toast]

# Dependency graph
requires:
  - phase: 05-03
    provides: transcription-error event emitted from hotkey pipeline; indicator show/hide functions
provides:
  - focus_settings=false on both hotkey warning call sites (Startup and Save)
  - show_idle instead of hide in transcription Err(()) branch keeps indicator visible for toast
  - hide_indicator Tauri command for frontend-driven indicator close
  - indicator auto-hides after all transcription error toasts are dismissed or expire
affects:
  - phase-06-text-injection
  - phase-05-uat

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Frontend-driven indicator lifecycle: Rust shows idle on error, frontend hides after toast"
    - "Dual auto-hide path: manual dismiss (handleDismissToast) + timer (showTranscriptionErrorToast setTimeout)"

key-files:
  created: []
  modified:
    - src-tauri/src/hotkey/service.rs
    - src-tauri/src/commands/indicator.rs
    - src-tauri/src/lib.rs
    - src/windows/indicator/App.vue

key-decisions:
  - "Indicator stays in show_idle state after transcription error (not hidden) so JS event loop can deliver toast"
  - "hide_indicator is a frontend-invoked command, not auto-called from Rust after error — keeps error UX in frontend control"
  - "showTranscriptionErrorToast wraps showToast + setTimeout(ms+50) buffer to fire after useToast's own auto-dismiss"

patterns-established:
  - "Focus-stealing suppression: emit_hotkey_warning focus_settings=false at all call sites"
  - "Toast lifecycle owns indicator visibility: indicator hides only when toasts.value.length === 0"

requirements-completed:
  - CLOD-06

# Metrics
duration: 4min
completed: 2026-03-17
---

# Phase 05 Plan 04: Gap Closure — Settings Focus and Error Toast Visibility Summary

**Hotkey warning no longer opens Settings window; transcription error toast now stays visible in indicator until dismissed**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-17T12:53:22Z
- **Completed:** 2026-03-17T12:57:13Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Both `emit_hotkey_warning` call sites (Startup and Save) now pass `focus_settings=false`, eliminating uninvited Settings window opens
- `Err(())` branch in transcription async spawn now calls `show_idle` instead of `hide`, keeping indicator visible until the JS event loop delivers the error toast
- New `hide_indicator` Tauri command lets the frontend close the indicator after toast lifecycle ends
- Indicator auto-hides via two paths: manual X dismiss (`handleDismissToast`) and auto-expire timer (`showTranscriptionErrorToast` setTimeout with 50ms buffer)

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix hotkey warning focus and error-branch indicator hide** - `5b28c7b` (fix)
2. **Task 2: Add hide_indicator command and wire frontend auto-hide after toast** - `861b66a` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src-tauri/src/hotkey/service.rs` - focus_settings false at lines 97 and 198; show_idle at Err(()) branch line 275
- `src-tauri/src/commands/indicator.rs` - added hide_indicator command
- `src-tauri/src/lib.rs` - registered hide_indicator in invoke_handler
- `src/windows/indicator/App.vue` - ShowToastOptions import, handleDismissToast, showTranscriptionErrorToast, template dismiss wired to handleDismissToast

## Decisions Made
- Indicator stays in `show_idle` state after transcription error so the indicator window remains visible while the JS event loop delivers the toast. The frontend (not Rust) owns the hide decision.
- `hide_indicator` is invoked from the frontend after toasts clear, not automatically from Rust after error — keeps error UX fully frontend-controlled.
- `showTranscriptionErrorToast` adds `ms + 50` buffer to fire after `useToast`'s own setTimeout, avoiding a race where the timer fires before useToast removes the toast from the array.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `cargo clippy -- -D warnings` reports 19 pre-existing warnings unrelated to this plan's changes. The build compiles cleanly (`cargo build` passes). These warnings are out of scope per deviation scope rules.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Both UAT-diagnosed gaps are closed. UAT re-run can now proceed.
- Settings window no longer opens on startup hotkey conflict.
- Transcription error toast is visible and indicator auto-hides cleanly after acknowledgement.
- Phase 6 text injection is unblocked.

---
*Phase: 05-cloud-transcription*
*Completed: 2026-03-17*
