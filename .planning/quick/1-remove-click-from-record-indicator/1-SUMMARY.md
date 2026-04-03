---
phase: quick-1
plan: 1
subsystem: indicator, tray
tags: [ui, cleanup, hotkey-only-recording]
dependency_graph:
  requires: []
  provides: [indicator-display-only, tray-settings-quit-only]
  affects: [src/windows/indicator/App.vue, src-tauri/src/tray.rs, src-tauri/src/commands/indicator.rs, src-tauri/src/hotkey/service.rs, src-tauri/src/lib.rs]
tech_stack:
  added: []
  patterns: []
key_files:
  modified:
    - src/windows/indicator/App.vue
    - src-tauri/src/tray.rs
    - src-tauri/src/commands/indicator.rs
    - src-tauri/src/hotkey/service.rs
    - src-tauri/src/lib.rs
decisions:
  - Hotkey is the sole recording trigger; indicator circle and tray recording items removed entirely
  - button element converted to span to preserve indicator-record-button CSS class (used by drag guard in onPointerDown)
  - update_recording_menu callers in service.rs retained as no-op tray rebuilds to keep tray valid on state transitions
metrics:
  duration: 7m
  completed_date: "2026-04-02"
  tasks_completed: 2
  files_modified: 5
---

# Quick Task 1: Remove Click-to-Record from Indicator and Tray Summary

**One-liner:** Removed click-to-record from indicator circle and Start/Stop Recording from tray; hotkey is now the sole recording trigger.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Remove click handler from indicator circle + delete toggle_recording_from_indicator command | af51f9b |
| 2 | Remove Start/Stop Recording from tray menu, simplify build_tray_menu and update_recording_menu | d553f30 |

## Changes Made

### Task 1 — Indicator circle click handler removed
- Deleted `onRecordButtonClick` function from `App.vue`
- Removed `@click.stop` binding; converted `<button>` to `<span>` to avoid implicit button semantics while preserving the `indicator-record-button` CSS class that the drag guard in `onPointerDown` relies on
- Deleted `toggle_recording_from_indicator` `#[tauri::command]` from `commands/indicator.rs`
- Removed now-unused `use crate::hotkey::service` import from `indicator.rs`
- Removed the command from `invoke_handler` in `lib.rs`

### Task 2 — Tray menu simplified
- Deleted `START_STOP_ID` constant
- Deleted `start_stop` `MenuItem` construction and `recording_menu_label` helper function
- Removed `START_STOP_ID =>` match arm from `handle_menu_event`
- `build_tray_menu` now takes no `recording_state` parameter; menu is `[Settings, separator, Quit VoxWeave]`
- `update_recording_menu` signature drops the `state: RecordingState` parameter
- Updated all 6 call sites in `hotkey/service.rs` to call `tray::update_recording_menu(app)` without a second argument

## Verification

- `npx vue-tsc --noEmit` — PASS
- `cargo clippy` — PASS (no errors)
- `cargo test` — PASS (103/103 tests)

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- `src/windows/indicator/App.vue` — exists, button converted to span, no click handler
- `src-tauri/src/tray.rs` — exists, no START_STOP_ID, no recording_menu_label, menu has 3 items
- `src-tauri/src/commands/indicator.rs` — exists, toggle_recording_from_indicator removed
- Commit af51f9b — found
- Commit d553f30 — found
