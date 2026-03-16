---
phase: 04-floating-indicator
plan: "04-05"
type: execute
wave: 2
depends_on:
  - 04-04-indicator-focus-clickthrough-gap-PLAN.md
files_modified:
  - src/windows/indicator/App.vue
  - src-tauri/src/indicator/mod.rs
  - src-tauri/src/indicator/window.rs
  - src-tauri/src/hotkey/service.rs
autonomous: true
requirements:
  - FLOT-05
  - FLOT-06
gap_closure: true
must_haves:
  truths:
    - "Dragged indicator position persists using final drop coordinates."
    - "State transitions do not snap the window back to stale config coordinates."
    - "Position is restored after restart and clamped into visible bounds."
    - "First-run placement defaults to bottom-right above taskbar when no valid saved coordinates exist."
  artifacts:
    - path: "src/windows/indicator/App.vue"
      provides: "Drag-end position persistence timing"
    - path: "src-tauri/src/indicator/mod.rs"
      provides: "State transition placement policy (no stale snap-back)"
    - path: "src-tauri/src/indicator/window.rs"
      provides: "Clamped bounds/fallback utilities used by persistence"
  key_links:
    - from: "src/windows/indicator/App.vue"
      to: "src-tauri/src/indicator/mod.rs"
      via: "persist_indicator_position after final drop"
      pattern: "invoke\\(\"persist_indicator_position\""
    - from: "src-tauri/src/hotkey/service.rs"
      to: "src-tauri/src/indicator/mod.rs"
      via: "state transitions (show_recording/show_processing/hide)"
      pattern: "indicator::show_|indicator::hide"
---

<objective>
Close blocker gap: drag persistence fails and window snaps back on lifecycle transitions.

Purpose: Make position behavior deterministic across drag/stop/restart paths.
Output: final drop persisted, transitions keep latest position, restart restore works.
</objective>

<execution_context>
@C:/Users/Balazs/.codex/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/04-floating-indicator/04-UAT.md
@src/windows/indicator/App.vue
@src-tauri/src/indicator/mod.rs
@src-tauri/src/indicator/window.rs
@src-tauri/src/hotkey/service.rs

<interfaces>
From src-tauri/src/commands/indicator.rs:
`persist_indicator_position(app: AppHandle, x: i32, y: i32) -> Result<(), String>`

From src-tauri/src/indicator/mod.rs:
`pub fn persist_position<R: Runtime>(app: &AppHandle<R>, x: i32, y: i32) -> Result<(), String>`
`pub fn show_recording<R: Runtime>(app: &AppHandle<R>) -> Result<(), String>`
`pub fn show_processing<R: Runtime>(app: &AppHandle<R>)`
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Persist indicator coordinates on drag end (not drag start)</name>
  <files>src/windows/indicator/App.vue</files>
  <action>Move persistence call to drag completion path so saved coordinates use final dropped position. Remove pointerdown-time save. Ensure cleanup paths still restore click-through and do not lose final persist on normal release.</action>
  <verify>
    <automated>cmd /c npx vue-tsc --noEmit</automated>
  </verify>
  <done>Stopping recording after a drag no longer reverts to pre-drag coordinates.</done>
</task>

<task type="auto">
  <name>Task 2: Prevent stale config reposition during visible-state transitions</name>
  <files>src-tauri/src/indicator/mod.rs, src-tauri/src/hotkey/service.rs</files>
  <action>Adjust indicator show/state flow so config-based `place_window_from_config` is not reapplied during intra-session transitions when window is already visible, or ensure config is synced before such transitions. Keep hide behavior deterministic for completion/error.</action>
  <verify>
    <automated>cd src-tauri && cargo test indicator::tests::state_transitions_do_not_snap_to_stale_position -- --exact</automated>
  </verify>
  <done>Runtime transitions preserve current on-screen position instead of snapping to stale saved values.</done>
</task>

<task type="auto">
  <name>Task 3: Enforce first-run bottom-right-above-taskbar fallback</name>
  <files>src-tauri/src/indicator/window.rs, src-tauri/src/indicator/mod.rs</files>
  <action>When indicator position is unset or invalid (including monitor/layout changes), place it at bottom-right above the taskbar with safe margins rather than generic fallback coordinates. Keep this fallback deterministic and compatible with clamp logic.</action>
  <verify>
    <automated>cd src-tauri && cargo test indicator::tests::first_run_defaults_bottom_right_above_taskbar -- --exact</automated>
  </verify>
  <done>Locked decision is implemented: first-run indicator location is bottom-right above taskbar.</done>
</task>

</tasks>

<verification>
- `cd src-tauri && cargo test indicator -- --nocapture`
- `cmd /c npx vue-tsc --noEmit`
- Manual: `cargo tauri dev`; start recording, drag indicator, stop recording, restart app; position remains at drop point or safe clamped fallback.
</verification>

<success_criteria>
- UAT test 7 passes: drag position persists across stop/restart and no snap-back during lifecycle transitions.
</success_criteria>

<output>
After completion, create `.planning/phases/04-floating-indicator/04-05-floating-indicator-04-05-SUMMARY.md`
</output>

<unresolved_questions>
None.
</unresolved_questions>
