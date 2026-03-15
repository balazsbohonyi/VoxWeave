---
phase: 04-floating-indicator
plan: "04-04"
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/tauri.conf.json
  - src-tauri/src/indicator/window.rs
  - src-tauri/src/indicator/mod.rs
  - src-tauri/src/commands/indicator.rs
  - src/windows/indicator/App.vue
autonomous: true
requirements:
  - FLOT-01
  - FLOT-02
gap_closure: true
must_haves:
  truths:
    - "Indicator appears on recording start without stealing keyboard focus."
    - "Indicator remains click-through except during explicit drag."
  artifacts:
    - path: "src-tauri/tauri.conf.json"
      provides: "Non-focus indicator window config"
    - path: "src-tauri/src/indicator/window.rs"
      provides: "Click-through-by-default policy helpers"
    - path: "src-tauri/src/indicator/mod.rs"
      provides: "Show/hide/drag policy transitions"
    - path: "src/windows/indicator/App.vue"
      provides: "Explicit drag IPC begin/end lifecycle"
  key_links:
    - from: "src/windows/indicator/App.vue"
      to: "src-tauri/src/commands/indicator.rs"
      via: "invoke(begin_indicator_drag/end_indicator_drag)"
      pattern: "invoke\\(\"begin_indicator_drag\"|\"end_indicator_drag\"\\)"
    - from: "src-tauri/src/indicator/mod.rs"
      to: "src-tauri/src/indicator/window.rs"
      via: "apply_window_policy and cursor-event toggles"
      pattern: "set_ignore_cursor_events"
---

<objective>
Close blocker gap: focus theft + wrong default interactivity.

Purpose: Restore non-intrusive indicator behavior promised in Phase 4.
Output: indicator opens non-focus/click-through; drag-only interaction.
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
@src-tauri/tauri.conf.json
@src-tauri/src/indicator/window.rs
@src-tauri/src/indicator/mod.rs
@src/windows/indicator/App.vue

<interfaces>
From src-tauri/src/commands/indicator.rs:
`begin_indicator_drag(app: AppHandle) -> Result<(), String>`
`end_indicator_drag(app: AppHandle) -> Result<(), String>`

From src-tauri/src/indicator/mod.rs:
`pub fn begin_drag<R: Runtime>(app: &AppHandle<R>) -> Result<(), String>`
`pub fn end_drag<R: Runtime>(app: &AppHandle<R>) -> Result<(), String>`
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Enforce non-focus + click-through defaults in backend/window config</name>
  <files>src-tauri/tauri.conf.json, src-tauri/src/indicator/window.rs, src-tauri/src/indicator/mod.rs</files>
  <action>Set indicator window to non-focus in config (`focus: false`). In runtime policy, default to click-through (`set_ignore_cursor_events(true)`) on apply/show/hide idle paths. Ensure drag end restores click-through (not interactive) so normal state never captures mouse input.</action>
  <verify>
    <automated>cd src-tauri && cargo test indicator -- --nocapture</automated>
  </verify>
  <done>Recording start no longer activates indicator; non-drag interaction does not intercept user input.</done>
</task>

<task type="auto">
  <name>Task 2: Make drag lifecycle explicit via begin/end IPC from indicator UI</name>
  <files>src/windows/indicator/App.vue, src-tauri/src/commands/indicator.rs, src-tauri/src/indicator/mod.rs</files>
  <action>Update pointer interaction so frontend calls `begin_indicator_drag` immediately before dragging and always calls `end_indicator_drag` in release/cancel/finally cleanup. Keep default click-through when not dragging.</action>
  <verify>
    <automated>cmd /c npx vue-tsc --noEmit</automated>
  </verify>
  <done>Interactivity is temporary and scoped only to active drag gesture.</done>
</task>

</tasks>

<verification>
- `cd src-tauri && cargo test indicator -- --nocapture`
- `cmd /c npx vue-tsc --noEmit`
- Manual: `cargo tauri dev`, focus external app, hotkey start/stop; typing focus stays in external app.
</verification>

<success_criteria>
- UAT test 5 passes: no focus theft; click-through except during drag.
</success_criteria>

<output>
After completion, create `.planning/phases/04-floating-indicator/04-04-floating-indicator-04-04-SUMMARY.md`
</output>

<unresolved_questions>
None.
</unresolved_questions>

