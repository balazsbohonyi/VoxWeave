---
phase: 04-floating-indicator
plan: "04-03"
type: execute
wave: 3
depends_on:
  - 04-01-indicator-window-runtime-PLAN.md
  - 04-02-indicator-visual-states-waveform-PLAN.md
files_modified:
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/commands/indicator.rs
  - src-tauri/src/indicator/mod.rs
  - src-tauri/src/indicator/window.rs
  - src-tauri/src/hotkey/service.rs
  - src-tauri/src/audio/mod.rs
  - src-tauri/src/config/mod.rs
  - src/composables/useConfig.ts
  - src/windows/indicator/App.vue
  - src/types/index.ts
autonomous: true
requirements:
  - FLOT-05
  - FLOT-06
must_haves:
  truths:
    - "Indicator is click-through by default and interactive only during explicit drag."
    - "Drag end persists bounded x/y and restores on restart."
    - "First-run default location is bottom-right above taskbar when no valid saved position exists."
    - "Invalid saved coordinates auto-recover to safe fallback."
    - "Hide is deterministic on success, error, and cancellation."
  artifacts:
    - path: "src-tauri/src/commands/indicator.rs"
      provides: "begin/end drag + persist position IPC commands"
    - path: "src-tauri/src/indicator/mod.rs"
      provides: "Drag/hide lifecycle orchestration across terminal paths"
    - path: "src-tauri/src/indicator/window.rs"
      provides: "Monitor-bounded coordinate clamping and policy helpers"
    - path: "src/windows/indicator/App.vue"
      provides: "Frontend drag gesture lifecycle integration"
    - path: "src-tauri/src/config/mod.rs"
      provides: "Indicator position persistence in app config"
  key_links:
    - from: "src/windows/indicator/App.vue"
      to: "src-tauri/src/commands/indicator.rs"
      via: "invoke begin/end drag and persist position commands"
      pattern: "invoke\\(\"(begin_indicator_drag|end_indicator_drag|persist_indicator_position)\""
    - from: "src-tauri/src/hotkey/service.rs"
      to: "src-tauri/src/indicator/mod.rs"
      via: "terminal lifecycle transitions route to indicator hide/show"
      pattern: "indicator::(show_|hide)"
---

<objective>
Complete indicator interaction behaviors: temporary drag mode with bounded persistence and robust hide semantics across completion/error/cancel paths.

Purpose: Satisfy FLOT-05/FLOT-06 interaction and persistence guarantees.
Output: Drag IPC + position persistence + terminal-path hide hardening.
</objective>

<execution_context>
@C:/Users/Balazs/.codex/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@src-tauri/src/commands/mod.rs
@src-tauri/src/commands/indicator.rs
@src-tauri/src/indicator/mod.rs
@src-tauri/src/indicator/window.rs
@src-tauri/src/hotkey/service.rs
@src-tauri/src/audio/mod.rs
@src-tauri/src/config/mod.rs
@src/composables/useConfig.ts
@src/windows/indicator/App.vue
@src/types/index.ts
</context>

<tasks>
<task type="auto">
  <name>Task 1: Add backend drag-mode and position persistence commands</name>
  <files>src-tauri/src/commands/mod.rs, src-tauri/src/commands/indicator.rs, src-tauri/src/indicator/mod.rs, src-tauri/src/indicator/window.rs, src-tauri/src/config/mod.rs</files>
  <action>Expose thin commands for `begin_indicator_drag`, `end_indicator_drag`, and `persist_indicator_position` that toggle click-through safely and clamp coordinates to visible monitor bounds before saving through existing config persistence flow. Implement explicit first-run placement fallback to bottom-right above taskbar (with safe margin) whenever saved coordinates are missing/invalid.</action>
  <verify>
    <automated>cd src-tauri && cargo test indicator::tests::first_run_defaults_bottom_right_above_taskbar -- --exact</automated>
  </verify>
  <done>Drag and persistence mechanics are backend-owned, bounded, and resilient to monitor layout changes.</done>
</task>

<task type="auto">
  <name>Task 2: Implement frontend drag gesture integration</name>
  <files>src/windows/indicator/App.vue, src/types/index.ts, src/composables/useConfig.ts</files>
  <action>Implement press-and-hold gesture to enter drag mode, update window position during drag, and commit persisted position on release. Ensure cancel/blur paths restore click-through mode and avoid leaving indicator interactive.</action>
  <verify>
    <automated>cmd /c npx vue-tsc --noEmit</automated>
  </verify>
  <done>Indicator can be repositioned naturally without compromising non-intrusive default behavior.</done>
</task>

<task type="auto">
  <name>Task 3: Finalize auto-hide and recovery matrix</name>
  <files>src-tauri/src/indicator/mod.rs, src-tauri/src/hotkey/service.rs, src-tauri/src/audio/mod.rs, src/windows/indicator/App.vue</files>
  <action>Harden hide pipeline for all terminal paths (success, backend error, cancellation), add explicit indicator-hidden event handling in frontend cleanup, and verify that stale waveform/state UI never persists after terminal events.</action>
  <verify>
    <automated>cd src-tauri && cargo test indicator::tests::hide_on_complete_or_error -- --exact</automated>
  </verify>
  <done>FLOT-06 is satisfied with deterministic hide behavior under all supported phase scenarios.</done>
</task>
</tasks>

<verification>
<criteria>
- Drag interaction works while preserving click-through-by-default behavior.
- Persisted position restores correctly and remains visible after restart/display changes.
- Indicator always hides after terminal success/error/cancel conditions.
</criteria>

<commands>
- `cd src-tauri && cargo test indicator -- --nocapture`
- `npx vue-tsc --noEmit`
- Manual: `cargo tauri dev`, drag indicator across screen edges/monitors, restart, and validate hide behavior for success and error paths
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
