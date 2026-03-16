---
status: diagnosed
trigger: "Gap truth: After moving indicator and restarting app, indicator reopens at saved position; if saved position is off-screen/invalid, it is clamped back into visible monitor bounds."
created: 2026-03-15T21:04:49.3222621Z
updated: 2026-03-15T21:05:43.5728504Z
---

## Current Focus

hypothesis: Indicator position is persisted before drag completes, so show/hide transitions reapply the pre-drag coordinates.
test: Follow pointer event -> drag call -> persist call ordering and compare with backend re-positioning points.
expecting: Persist call runs immediately after initiating drag (not at release), storing old position; next show/hide reads config and snaps window back.
next_action: return root-cause diagnosis with evidence and affected files

## Symptoms

expected: Dragged indicator position persists and is used consistently; invalid/off-screen positions get clamped.
actual: Position resets to value from settings file when starting/stopping recording; drag during recording is lost on stop.
errors: None reported.
reproduction: UAT Test 7 - start recording, drag indicator, stop recording, observe it jump back to initial position.
started: Discovered during UAT.

## Eliminated

- hypothesis: In-memory config stays stale after persistence and causes reset.
  evidence: `indicator::persist_position` writes clamped coordinates to disk and then replaces `state.config` with updated config.
  timestamp: 2026-03-15T21:05:43.5728504Z

## Evidence

- timestamp: 2026-03-15T21:05:43.5728504Z
  checked: src/windows/indicator/App.vue pointer handler
  found: `onPointerDown` calls `win.startDragging()` and then immediately reads `win.outerPosition()` and invokes `persist_indicator_position`.
  implication: Persistence timing is tied to drag start path, with no explicit drag-end persistence event.

- timestamp: 2026-03-15T21:05:43.5728504Z
  checked: src/windows/indicator/App.vue + command usage search
  found: `begin_indicator_drag` / `end_indicator_drag` commands exist but are never called; only `startDragging()` path is used.
  implication: There is no backend-recognized drag lifecycle where final drop coordinates are persisted on release.

- timestamp: 2026-03-15T21:05:43.5728504Z
  checked: src-tauri/src/indicator/mod.rs show/hide flow
  found: `show_with_state` always calls `place_window_from_config(...)`; `hide()` with `show_on_startup=true` calls `show_idle()`, which also routes through `show_with_state` and repositions from config.
  implication: Any mismatch between actual dragged location and persisted config will be corrected back to config on start/stop transitions.

- timestamp: 2026-03-15T21:05:43.5728504Z
  checked: src-tauri/src/hotkey/service.rs recording transitions
  found: Recording start calls `indicator::show_recording`; stop path ends in `indicator::hide` (which may call `show_idle`).
  implication: The user-reported snap-back on start/stop is directly explained by repeated config-based placement.

## Resolution

root_cause: Indicator drag persistence is wired to pointer-down/start-drag flow instead of a reliable drag-end event. The saved coordinates used by backend repositioning can remain at the pre-drag position, and start/stop recording transitions reapply that stale config position via `show_with_state`.
fix: Not applied (goal: find_root_cause_only).
verification: Root cause established by call-path analysis of frontend drag handler and backend show/hide placement flow.
files_changed: []
