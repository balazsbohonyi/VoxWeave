---
wave: 2
depends_on:
  - 04-01-indicator-window-runtime-PLAN.md
  - 04-02-indicator-visual-states-waveform-PLAN.md
files_modified:
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/commands/indicator.rs
  - src-tauri/src/indicator/mod.rs
  - src-tauri/src/indicator/window.rs
  - src-tauri/src/config/mod.rs
  - src/composables/useConfig.ts
  - src/windows/indicator/App.vue
  - src/types/index.ts
autonomous: true
requirements:
  - FLOT-05
  - FLOT-06
---

<plan>
<goal>
Complete indicator interaction behaviors: temporary drag mode with bounded persistence and robust hide semantics across completion/error paths.
</goal>

<must_haves>
- Indicator is click-through by default and only interactive during explicit drag gesture.
- Drag end persists bounded x/y into existing indicator config fields and restores on restart.
- Invalid saved coordinates recover automatically to safe bottom-right fallback.
- Indicator hide behavior is consistent across success, error, and cancellation paths.
</must_haves>

<tasks>
<task type="auto">
  <name>Task 1: Add backend drag-mode and position persistence commands</name>
  <files>src-tauri/src/commands/mod.rs, src-tauri/src/commands/indicator.rs, src-tauri/src/indicator/mod.rs, src-tauri/src/indicator/window.rs, src-tauri/src/config/mod.rs</files>
  <action>Expose thin commands for `begin_indicator_drag`, `end_indicator_drag`, and `persist_indicator_position` that toggle click-through safely and clamp coordinates to visible monitor bounds before saving through existing config persistence flow.</action>
  <verify>`cd src-tauri && cargo test indicator::tests::persisted_position_clamped_to_monitor -- --exact` passes</verify>
  <done>Drag and persistence mechanics are backend-owned, bounded, and resilient to monitor layout changes.</done>
</task>

<task type="auto">
  <name>Task 2: Implement frontend drag gesture integration</name>
  <files>src/windows/indicator/App.vue, src/types/index.ts, src/composables/useConfig.ts</files>
  <action>Implement press-and-hold gesture to enter drag mode, update window position during drag, and commit persisted position on release. Ensure cancel/blur paths restore click-through mode and avoid leaving indicator interactive.</action>
  <verify>`npx vue-tsc --noEmit` passes</verify>
  <done>Indicator can be repositioned naturally without compromising non-intrusive default behavior.</done>
</task>

<task type="auto">
  <name>Task 3: Finalize auto-hide and recovery matrix</name>
  <files>src-tauri/src/indicator/mod.rs, src-tauri/src/hotkey/service.rs, src-tauri/src/audio/mod.rs, src/windows/indicator/App.vue</files>
  <action>Harden hide pipeline for all terminal paths (success, backend error, cancellation), add explicit indicator-hidden event handling in frontend cleanup, and verify that stale waveform/state UI never persists after terminal events.</action>
  <verify>`cd src-tauri && cargo test indicator::tests::hide_on_complete_or_error -- --exact` plus manual runtime checks pass</verify>
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
</plan>

