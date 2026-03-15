---
wave: 1
depends_on: []
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/src/lib.rs
  - src-tauri/src/state.rs
  - src-tauri/src/tray.rs
  - src-tauri/src/hotkey/mod.rs
  - src-tauri/src/hotkey/normalize.rs
  - src-tauri/src/hotkey/service.rs
autonomous: true
requirements:
  - HOTK-01
  - HOTK-02
---

<plan>
<goal>
Add the Phase 2 hotkey runtime: canonical parsing, startup registration, and one shared toggle path for tray + global shortcut.
</goal>

<must_haves>
- Default runtime hotkey is `Ctrl+Shift+Space`; registration is global, app-wide, active at startup.
- Hotkey and tray `Start/Stop Recording` call the same backend toggle path.
- Toggle flow is `Idle -> Recording -> Transcribing`, then placeholder completion returns to `Idle` so Phase 2 never gets stuck.
- Hotkey presses while `Transcribing` are ignored.
- Scope stays phase-bounded: no real audio capture, indicator, transcription provider work, or injection work.
</must_haves>

<tasks>
<task type="auto">
  <name>Task 1: Install the hotkey runtime seam and state</name>
  <files>src-tauri/Cargo.toml, src-tauri/src/lib.rs, src-tauri/src/state.rs, src-tauri/src/hotkey/mod.rs</files>
  <action>Add `tauri-plugin-global-shortcut` to the Rust app, register the plugin during Tauri startup, declare the new `hotkey` module, and extend `AppState` only with runtime hotkey fields needed for the active canonical binding, startup availability/conflict status, and warning payload state. Keep ownership in Rust-managed state so the frontend does not become the source of truth for hotkey lifecycle.</action>
  <verify>`cargo test hotkey::tests::default_hotkey_press_starts_recording -- --exact` passes once the test is added</verify>
  <done>The app has a dedicated hotkey runtime seam, startup can attempt global registration, and state exists for active-binding/warning bookkeeping without introducing audio or transcription implementation.</done>
</task>

<task type="auto">
  <name>Task 2: Implement normalization, registration, and the shared toggle path</name>
  <files>src-tauri/src/hotkey/normalize.rs, src-tauri/src/hotkey/service.rs, src-tauri/src/tray.rs, src-tauri/src/lib.rs</files>
  <action>Create canonical hotkey parsing/normalization with fixed modifier order and consistent casing, register the startup hotkey from config through the service module, and expose one `toggle_recording_state(app_handle)` function. Route only shortcut `Pressed` events and the tray `start_stop_recording` item to that same toggle path. Enable the tray item in Phase 2, update its label from the current recording state, and add a placeholder completion seam that transitions `Transcribing` back to `Idle` so the phase demonstrates the intended state machine without pulling in real pipeline work.</action>
  <verify>`cargo test hotkey::tests::toggle_respects_state_machine -- --exact` passes</verify>
  <done>The default hotkey is globally active at startup, tray and shortcut behavior cannot drift because they share one backend path, and a full `Idle -> Recording -> Transcribing -> Idle` placeholder cycle exists.</done>
</task>

<task type="auto">
  <name>Task 3: Add focused runtime tests for normalization and state transitions</name>
  <files>src-tauri/src/hotkey/mod.rs, src-tauri/src/hotkey/normalize.rs, src-tauri/src/hotkey/service.rs</files>
  <action>Add unit tests around canonical normalization (`shift+ctrl+space` to `Ctrl+Shift+Space`), default startup registration, and state transitions using fake registrar/menu seams instead of real desktop hooks. Keep tests Phase-2-focused: prove the runtime owns hotkey activation and state transitions, not Windows shell behavior.</action>
  <verify>`cargo test hotkey` passes</verify>
  <done>Wave 1 creates the automated verification base for HOTK-01 and HOTK-02 and provides targeted tests the later conflict/persistence work can extend.</done>
</task>
</tasks>

<verification>
<criteria>
- App startup owns hotkey registration and does not rely on tray clicks or settings UI to activate the default binding.
- Tray and hotkey behavior cannot drift because both pass through one backend toggle function.
- Second trigger reaches `Transcribing` without leaving the app permanently busy.
</criteria>

<commands>
- `cargo test hotkey`
- `cargo test`
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
</plan>
