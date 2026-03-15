---
phase: 04-floating-indicator
plan: "04-05"
subsystem: ui
tags: [tauri, vue, indicator, window-position, drag, persistence]
requires:
  - phase: 04-floating-indicator
    provides: click-through drag lifecycle and indicator event flow
provides:
  - Drag-end indicator position persistence using final drop coordinates
  - State transition policy that avoids stale config snap-back while window is visible
  - Deterministic bottom-right-above-taskbar fallback for missing/invalid saved positions
affects: [hotkey, recording-lifecycle, indicator-window]
tech-stack:
  added: []
  patterns: [persist-on-drop, visible-window-no-reposition, invalid-position-fallback]
key-files:
  created: []
  modified:
    - src/windows/indicator/App.vue
    - src-tauri/src/indicator/mod.rs
    - src-tauri/src/indicator/window.rs
key-decisions:
  - "Persist indicator coordinates only after drag completion and cleanup."
  - "Skip config-based window placement when indicator is already visible."
  - "Treat off-screen saved coordinates as invalid and fallback to deterministic bottom-right placement."
patterns-established:
  - "Indicator drag persistence happens at drop finalization, not drag initiation."
  - "Window placement from config is for initial show, not in-session visible transitions."
requirements-completed: [FLOT-05, FLOT-06]
duration: 3min
completed: 2026-03-15
---

# Phase 04 Plan 05: Indicator Position Snapback Gap Summary

**Indicator now persists final drag-drop coordinates, avoids stale-position snap-back during runtime transitions, and restores with deterministic bottom-right fallback when saved coordinates are missing or invalid.**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-15T22:09:22Z
- **Completed:** 2026-03-15T22:11:16Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- Moved frontend position persistence to drag completion so saved coordinates use drop-time location.
- Updated indicator show flow to avoid re-placing from stale config while window is already visible.
- Enforced first-run and invalid-coordinate fallback to bottom-right-above-taskbar with regression coverage.

## Task Commits

Each task was committed atomically:

1. **Task 1: Persist indicator coordinates on drag end (not drag start)** - `2b42d2c` (fix)
2. **Task 2: Prevent stale config reposition during visible-state transitions** - `5d79158` (fix)
3. **Task 3: Enforce first-run bottom-right-above-taskbar fallback** - `35993ec` (fix)

**Plan metadata:** pending

## Files Created/Modified
- `src/windows/indicator/App.vue` - Persists indicator coordinates after `startDragging` completes and drag cleanup runs.
- `src-tauri/src/indicator/mod.rs` - Avoids config-based reposition when indicator window is already visible; adds transition/fallback regression tests.
- `src-tauri/src/indicator/window.rs` - Resolves invalid saved positions to deterministic bottom-right fallback.

## Decisions Made
- Persisted positions at drag completion, not pointerdown path, to capture final dropped coordinates.
- Used window visibility as the guard for config placement to prevent lifecycle snap-back during in-session transitions.
- Classified off-screen saved positions as invalid and resolved them to bottom-right-above-taskbar fallback instead of edge-clamping stale values.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Indicator position lifecycle behavior is deterministic across drag, state transitions, and restart paths.
- Phase 04 can continue with waveform/reactivity gap closure on top of stable placement behavior.

## Self-Check

PASSED
- FOUND: .planning/phases/04-floating-indicator/04-05-floating-indicator-04-05-SUMMARY.md
- FOUND: 2b42d2c
- FOUND: 5d79158
- FOUND: 35993ec

---
*Phase: 04-floating-indicator*
*Completed: 2026-03-15*
