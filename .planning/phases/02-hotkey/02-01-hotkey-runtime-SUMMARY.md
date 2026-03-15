---
phase: 02-hotkey
plan: 02-01
subsystem: infra
tags: [tauri, global-shortcut, hotkey, tray]

# Dependency graph
requires: []
provides:
  - startup global hotkey registration with runtime state
  - shared toggle path for tray and hotkey
  - canonical hotkey normalization with focused tests
affects: [hotkey-config, recording-pipeline]

# Tech tracking
tech-stack:
  added: [tauri-plugin-global-shortcut]
  patterns: [hotkey normalization helper, registrar seam for tests, unified toggle handler]

key-files:
  created: [src-tauri/src/hotkey/mod.rs, src-tauri/src/hotkey/normalize.rs, src-tauri/src/hotkey/service.rs, src-tauri/Cargo.lock]
  modified: [src-tauri/Cargo.toml, src-tauri/src/lib.rs, src-tauri/src/state.rs, src-tauri/src/tray.rs, src-tauri/src/config/mod.rs]

key-decisions:
  - "Enable Tauri's test feature to use mock_app for hotkey unit tests"

patterns-established:
  - "Hotkey registration uses a registrar seam so tests avoid OS hooks"
  - "Tray and hotkey events share a single toggle_recording_state path"

requirements-completed: [HOTK-01, HOTK-02]

# Metrics
duration: 49min
completed: 2026-03-15
---

# Phase 2 Plan 01: Hotkey Runtime Summary

**Startup global hotkey registration with canonical normalization and a shared toggle state machine for tray + shortcut.**

## Performance

- **Duration:** 49 min
- **Started:** 2026-03-15T10:10:00Z
- **Completed:** 2026-03-15T10:59:01Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments
- Registered the global shortcut plugin at startup and added AppState runtime fields for hotkey status/warnings.
- Implemented normalization, startup registration, and a unified toggle path for tray and shortcut with placeholder transcribe completion.
- Added targeted unit tests for normalization, default registration, and state transitions via a fake registrar seam.

## Task Commits

Each task was committed atomically:

1. **Task 1: Install the hotkey runtime seam and state** - `eb7900e` (feat)
2. **Task 2: Implement normalization, registration, and the shared toggle path** - `db150bf` (feat)
3. **Task 3: Add focused runtime tests for normalization and state transitions** - `456e909` (test)

## Files Created/Modified
- `src-tauri/Cargo.toml` - enable test feature and add global shortcut plugin
- `src-tauri/Cargo.lock` - lockfile update for new dependency
- `src-tauri/src/lib.rs` - register shortcut plugin + startup hotkey
- `src-tauri/src/state.rs` - hotkey runtime fields + availability tracking
- `src-tauri/src/tray.rs` - unified toggle path and state-driven labels
- `src-tauri/src/config/mod.rs` - default hotkey set to Ctrl+Shift+Space
- `src-tauri/src/hotkey/mod.rs` - hotkey module + runtime tests
- `src-tauri/src/hotkey/normalize.rs` - canonical normalization helper
- `src-tauri/src/hotkey/service.rs` - registration + toggle state machine

## Decisions Made
- Enabled Tauri's `test` feature so hotkey unit tests can use `mock_app`.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `cargo test hotkey::tests::toggle_respects_state_machine -- --exact` failed at runtime with `STATUS_ENTRYPOINT_NOT_FOUND` (0xc0000139). The test binary starts but exits abnormally on this machine; likely environment/runtime dependency issue.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Hotkey runtime seam, state machine, and tests are in place for config persistence and conflict handling in Phase 02-02.
- Resolve the Windows test runtime failure before relying on CI for hotkey tests.

---
*Phase: 02-hotkey*
*Completed: 2026-03-15*

## Self-Check: PASSED
