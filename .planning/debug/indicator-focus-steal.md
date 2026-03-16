---
status: diagnosed
trigger: "Gap truth: While visible, the indicator does not steal focus from the active app and remains click-through except during explicit drag interaction."
created: 2026-03-15T00:00:00Z
updated: 2026-03-15T21:24:00Z
---

## Current Focus

hypothesis: Focus steal is caused by indicator being shown as a focusable/interactive window.
test: Validate indicator config and runtime policy in window show path.
expecting: Explicit focus-enabled config and no click-through enable path.
next_action: Return diagnosis (goal is root-cause only).

## Symptoms

expected: Indicator shows without taking focus from current app; click-through by default except during drag.
actual: User reported "the indicator steals the focus, so this test failed".
errors: None reported.
reproduction: Test 5 in UAT.
started: Discovered during UAT.

## Eliminated

## Evidence

- timestamp: 2026-03-15T21:16:00Z
  checked: src-tauri/src/indicator/window.rs::apply_window_policy
  found: Policy sets `set_ignore_cursor_events(false)` every time, which makes the indicator receive pointer events.
  implication: Violates "click-through by default except during drag"; indicator is interactive by default.

- timestamp: 2026-03-15T21:18:00Z
  checked: src-tauri/src/indicator/mod.rs::show_with_state
  found: On every show, code applies window policy then calls `window.show()` with no non-activating guard.
  implication: Show path can present and potentially activate the indicator window when recording starts.

- timestamp: 2026-03-15T21:21:00Z
  checked: src-tauri/tauri.conf.json indicator window config
  found: Indicator window has `"focus": true`.
  implication: When shown, it is allowed to take focus from the active app; this directly matches the UAT report.

- timestamp: 2026-03-15T21:22:00Z
  checked: Indicator interaction toggles in frontend/backend
  found: No code path sets `set_ignore_cursor_events(true)`; drag commands exist but are not invoked from `App.vue` (which directly calls `startDragging()`).
  implication: Indicator is never truly click-through by default, so non-intrusive requirement is structurally unmet.

## Resolution

root_cause: Indicator window is configured and managed as focusable/interactive. `tauri.conf.json` sets `focus: true`, and runtime policy enforces `set_ignore_cursor_events(false)` without any default click-through enable path. When recording shows the window, it can take focus and receive pointer events.
fix:
verification:
files_changed: []
