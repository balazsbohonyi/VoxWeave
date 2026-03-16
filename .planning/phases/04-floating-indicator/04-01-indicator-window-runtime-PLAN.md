---
phase: 04-floating-indicator
plan: "04-01"
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/tauri.conf.json
  - src-tauri/src/lib.rs
  - src-tauri/src/state.rs
  - src-tauri/src/hotkey/service.rs
  - src-tauri/src/audio/mod.rs
  - src-tauri/src/indicator/mod.rs
  - src-tauri/src/indicator/window.rs
  - src-tauri/src/indicator/events.rs
autonomous: true
requirements:
  - FLOT-01
  - FLOT-02
  - FLOT-06
must_haves:
  truths:
    - "Indicator window exists as a separate transparent always-on-top surface."
    - "Recording lifecycle shows indicator on start and hides it on completion/error."
    - "Default indicator mode is click-through and avoids focus theft."
  artifacts:
    - path: "src-tauri/tauri.conf.json"
      provides: "Indicator window registration and baseline window policy"
    - path: "src-tauri/src/indicator/window.rs"
      provides: "Window policy helpers (non-focus/click-through)"
    - path: "src-tauri/src/indicator/mod.rs"
      provides: "Indicator show/hide lifecycle API"
    - path: "src-tauri/src/hotkey/service.rs"
      provides: "Recording lifecycle hooks into indicator visibility"
    - path: "src-tauri/src/indicator/events.rs"
      provides: "Typed indicator-state and indicator-hidden payload contracts"
  key_links:
    - from: "src-tauri/src/hotkey/service.rs"
      to: "src-tauri/src/indicator/mod.rs"
      via: "recording start/stop/error transitions call indicator lifecycle"
      pattern: "indicator::(show|hide|show_recording|show_processing)"
    - from: "src-tauri/src/indicator/mod.rs"
      to: "src-tauri/src/indicator/window.rs"
      via: "window policy and visibility operations"
      pattern: "(set_ignore_cursor_events|show|hide)"
---

<objective>
Create backend/runtime foundation for a dedicated floating indicator window that appears during recording, stays non-intrusive, and hides reliably on completion/error.

Purpose: Establish FLOT-01/FLOT-02/FLOT-06 runtime contract before UI states.
Output: Indicator window module + lifecycle/event wiring in Rust backend.
</objective>

<execution_context>
@C:/Users/Balazs/.codex/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@src-tauri/tauri.conf.json
@src-tauri/src/lib.rs
@src-tauri/src/state.rs
@src-tauri/src/hotkey/service.rs
@src-tauri/src/audio/mod.rs
@src-tauri/src/indicator/mod.rs
@src-tauri/src/indicator/window.rs
@src-tauri/src/indicator/events.rs
</context>

<tasks>
<task type="auto">
  <name>Task 1: Add indicator window definition and runtime module scaffold</name>
  <files>src-tauri/tauri.conf.json, src-tauri/src/lib.rs, src-tauri/src/indicator/mod.rs, src-tauri/src/indicator/window.rs, src-tauri/src/indicator/events.rs</files>
  <action>Add a new `indicator` window entry and a backend `indicator` module that owns window retrieval/creation helpers and strongly-typed event payloads (`indicator-state`, `indicator-hidden`). Keep the indicator hidden at startup and independent from settings window lifecycle.</action>
  <verify>
    <automated>cd src-tauri && cargo test indicator::tests::recording_start_shows_indicator -- --exact</automated>
  </verify>
  <done>Runtime has a dedicated indicator subsystem and event contract ready for frontend binding.</done>
</task>

<task type="auto">
  <name>Task 2: Wire recording lifecycle to indicator show/hide transitions</name>
  <files>src-tauri/src/hotkey/service.rs, src-tauri/src/audio/mod.rs, src-tauri/src/indicator/mod.rs, src-tauri/src/state.rs</files>
  <action>On transition into recording, show indicator and emit `recording` indicator state. On stop/error completion paths, emit `processing`/`hidden` transitions and hide the window deterministically. Preserve existing recording-state semantics (`Idle/Recording/Transcribing`) while adding indicator-only lifecycle events.</action>
  <verify>
    <automated>cd src-tauri && cargo test indicator::tests::hide_on_complete_or_error -- --exact</automated>
  </verify>
  <done>Indicator visibility is correctly driven by runtime lifecycle and does not remain stuck after errors.</done>
</task>

<task type="auto">
  <name>Task 3: Enforce non-focus and click-through window policy</name>
  <files>src-tauri/src/indicator/window.rs, src-tauri/src/indicator/mod.rs, src-tauri/src/lib.rs</files>
  <action>Apply always-on-top transparent policy and set ignore-cursor-events by default, ensuring indicator show paths avoid focus-stealing calls. Add guard tests for policy defaults and a manual checklist for cross-app typing focus retention.</action>
  <verify>
    <automated>cd src-tauri && cargo test indicator::tests::window_policy_is_non_focus_click_through -- --exact</automated>
  </verify>
  <done>FLOT-02 baseline behavior is guaranteed before frontend visuals are built.</done>
</task>
</tasks>

<verification>
<criteria>
- Starting recording makes indicator visible without stealing keyboard focus.
- Indicator remains click-through by default.
- Success and error completion paths hide indicator deterministically.
</criteria>

<commands>
- `cd src-tauri && cargo test indicator -- --nocapture`
- `cd src-tauri && cargo test`
- Manual: `cargo tauri dev` focus another app, trigger recording, confirm typing focus is retained
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
