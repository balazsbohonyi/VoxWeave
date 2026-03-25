---
phase: 09-setup-wizard
plan: "01"
subsystem: wizard-scaffold
tags: [tauri, rust, vue, wizard, window-lifecycle]
dependency_graph:
  requires: []
  provides: [wizard-window-scaffold, open_wizard_window-command]
  affects: [lib.rs, tray.rs, tauri.conf.json]
tech_stack:
  added: []
  patterns: [close-to-hide, first-launch-check, multi-window-tauri]
key_files:
  created:
    - wizard.html
    - src/windows/wizard/main.ts
    - src/windows/wizard/App.vue
    - src-tauri/src/commands/wizard.rs
  modified:
    - src-tauri/src/commands/mod.rs
    - src-tauri/tauri.conf.json
    - src-tauri/src/lib.rs
    - src-tauri/src/tray.rs
decisions:
  - Wizard close-to-hide uses same CloseRequested pattern as settings window but omits Destroyed handler (wizard does not drive cleanup_before_exit)
  - MutexGuard stored in intermediate let binding to satisfy borrow checker before block end (same pattern as existing show_on_startup block)
metrics:
  duration: 3 minutes
  completed_date: "2026-03-24"
  tasks: 2
  files: 8
requirements_satisfied: [WIZR-01, WIZR-06]
---

# Phase 9 Plan 01: Wizard Window Scaffold Summary

Wizard window scaffolded as 4th Tauri window: wizard.html entry point, Vue placeholder shell, open_wizard_window Rust command, tauri.conf.json entry (560x450, visible:false), close-to-hide lifecycle handler, first_launch startup check, and quit-loop inclusion.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Vite entry point + Vue shell | a6f4e4e | wizard.html, src/windows/wizard/main.ts, src/windows/wizard/App.vue |
| 2 | Rust command + config + lib.rs + tray.rs wiring | 6be1e17 | commands/wizard.rs, commands/mod.rs, tauri.conf.json, lib.rs, tray.rs |

## Verification

- `npx vue-tsc --noEmit` — PASS
- `cargo build` — PASS (7 pre-existing warnings, 0 errors)
- `wizard.html` exists at project root with `/src/windows/wizard/main.ts` script src
- `tauri.conf.json` has 4th window entry with label "wizard", 560x450, visible:false
- `lib.rs` contains close-to-hide handler for "wizard" and first_launch startup check
- `tray.rs` quit loop includes "wizard"

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] MutexGuard intermediate binding for first_launch read**
- **Found during:** Task 2 (cargo build)
- **Issue:** `state.config.lock().unwrap().first_launch` as a block-final expression creates a temporary MutexGuard that lives too long — E0597 borrow checker error
- **Fix:** Stored guard in `let cfg = state.config.lock().unwrap(); cfg.first_launch` — same pattern already used in the `show_on_startup` block above
- **Files modified:** src-tauri/src/lib.rs
- **Commit:** 6be1e17

## Self-Check: PASSED
