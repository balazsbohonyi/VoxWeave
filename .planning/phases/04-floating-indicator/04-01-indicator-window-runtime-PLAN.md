---
wave: 1
depends_on: []
files_modified:
  - src-tauri/tauri.conf.json
  - src-tauri/src/lib.rs
  - src-tauri/src/state.rs
  - src-tauri/src/hotkey/service.rs
  - src-tauri/src/indicator/mod.rs
  - src-tauri/src/indicator/window.rs
  - src-tauri/src/indicator/events.rs
autonomous: true
requirements:
  - FLOT-01
  - FLOT-02
  - FLOT-06
---

<plan>
<goal>
Create the backend/runtime foundation for a dedicated floating indicator window that appears on recording, stays non-intrusive, and hides reliably on completion/error.
</goal>

<must_haves>
- Indicator window exists as a separate Tauri window with transparent, always-on-top, decoration-free policy.
- Show/hide behavior is wired into recording lifecycle entry/exit points without focus theft.
- Default mode is click-through; runtime can temporarily toggle interactivity for drag in later plan.
- Hide executes on both success and error paths so stale indicator windows never remain visible.
</must_haves>

<tasks>
<task type="auto">
  <name>Task 1: Add indicator window definition and runtime module scaffold</name>
  <files>src-tauri/tauri.conf.json, src-tauri/src/lib.rs, src-tauri/src/indicator/mod.rs, src-tauri/src/indicator/window.rs, src-tauri/src/indicator/events.rs</files>
  <action>Add a new `indicator` window entry and a backend `indicator` module that owns window retrieval/creation helpers and strongly-typed event payloads (`indicator-state`, `indicator-hidden`). Keep the indicator hidden at startup and independent from settings window lifecycle.</action>
  <verify>`cd src-tauri && cargo test indicator::tests::recording_start_shows_indicator -- --exact` passes</verify>
  <done>Runtime has a dedicated indicator subsystem and event contract ready for frontend binding.</done>
</task>

<task type="auto">
  <name>Task 2: Wire recording lifecycle to indicator show/hide transitions</name>
  <files>src-tauri/src/hotkey/service.rs, src-tauri/src/audio/mod.rs, src-tauri/src/indicator/mod.rs, src-tauri/src/state.rs</files>
  <action>On transition into recording, show indicator and emit `recording` indicator state. On stop/error completion paths, emit `processing`/`hidden` transitions and hide the window deterministically. Preserve existing recording-state semantics (`Idle/Recording/Transcribing`) while adding indicator-only lifecycle events.</action>
  <verify>`cd src-tauri && cargo test indicator::tests::hide_on_complete_or_error -- --exact` passes</verify>
  <done>Indicator visibility is correctly driven by runtime lifecycle and does not remain stuck after errors.</done>
</task>

<task type="auto">
  <name>Task 3: Enforce non-focus and click-through window policy</name>
  <files>src-tauri/src/indicator/window.rs, src-tauri/src/indicator/mod.rs, src-tauri/src/lib.rs</files>
  <action>Apply always-on-top transparent policy and set ignore-cursor-events by default, ensuring indicator show paths avoid focus-stealing calls. Add guard tests for policy defaults and a manual checklist for cross-app typing focus retention.</action>
  <verify>`cd src-tauri && cargo test indicator::tests::window_policy_is_non_focus_click_through -- --exact` passes</verify>
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
</plan>

