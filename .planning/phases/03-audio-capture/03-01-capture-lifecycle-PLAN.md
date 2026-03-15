---
wave: 1
depends_on: []
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/src/lib.rs
  - src-tauri/src/state.rs
  - src-tauri/src/hotkey/service.rs
  - src-tauri/src/audio/mod.rs
  - src-tauri/src/audio/capture.rs
  - src-tauri/src/audio/session.rs
  - src-tauri/src/commands/mod.rs
autonomous: true
requirements:
  - AUDI-01
  - AUDI-02
  - AUDI-05
  - AUDI-06
---

<plan>
<goal>
Implement runtime audio session start/stop seams: begin recording quickly, resolve devices correctly, and recover safely on no-device/disconnect paths.
</goal>

<must_haves>
- Entering `Recording` starts audio capture immediately (latency budget first).
- Capture pipeline contract is 16kHz mono for downstream encoding.
- Selected device missing/disconnected falls back to default with warning event.
- No available microphone emits error and keeps state from getting stuck in `Recording`.
- Hotkey module remains control-plane only; capture logic lives in `audio`.
</must_haves>

<tasks>
<task type="auto">
  <name>Task 1: Introduce audio module + session state seam</name>
  <files>src-tauri/src/lib.rs, src-tauri/src/state.rs, src-tauri/src/audio/mod.rs, src-tauri/src/audio/session.rs, src-tauri/src/commands/mod.rs</files>
  <action>Add a dedicated `audio` module with session structs and lifecycle API (`start_recording`, `stop_recording`), wire module imports in `lib.rs`, and extend managed state only with the minimum session handles/flags required for runtime ownership. Keep command handlers thin and avoid encoding concerns in this task.</action>
  <verify>`cd src-tauri && cargo test audio::tests::session_state_transitions -- --exact` passes</verify>
  <done>Runtime has a concrete backend seam for audio lifecycle that later tasks can call without refactoring hotkey/tray architecture.</done>
</task>

<task type="auto">
  <name>Task 2: Implement device resolution + fallback/error behavior</name>
  <files>src-tauri/src/audio/capture.rs, src-tauri/src/audio/mod.rs, src-tauri/src/state.rs</files>
  <action>Implement selected-device lookup by persisted name, fallback to system default when selected device is unavailable, emit a warning event when fallback occurs, and emit terminal error when no input device exists. Ensure failure paths recover recording state to `Idle` deterministically.</action>
  <verify>`cd src-tauri && cargo test audio::tests::missing_selected_device_falls_back -- --exact` and `cd src-tauri && cargo test audio::tests::no_device_returns_error -- --exact` pass</verify>
  <done>AUDI-05 and AUDI-06 behavior is enforced in backend logic with deterministic state recovery and explicit user-facing event hooks.</done>
</task>

<task type="auto">
  <name>Task 3: Connect hotkey toggle to capture start/stop and assert 16kHz mono contract</name>
  <files>src-tauri/src/hotkey/service.rs, src-tauri/src/audio/mod.rs, src-tauri/src/audio/capture.rs</files>
  <action>Replace Phase 2 transcription placeholder transition with calls into audio lifecycle: entering `Recording` starts stream, leaving recording finalizes capture handoff. Ensure capture normalization contract is 16kHz mono and add focused tests for hotkey-triggered session start.</action>
  <verify>`cd src-tauri && cargo test hotkey::tests::recording_transition_starts_audio_session -- --exact` and `cd src-tauri && cargo test audio::tests::captures_mono_16khz_contract -- --exact` pass</verify>
  <done>Phase 3 runtime has real capture lifecycle behavior while preserving the existing state machine entrypoint semantics.</done>
</task>
</tasks>

<verification>
<criteria>
- Trigger from hotkey starts capture path immediately and does not block on non-critical checks.
- Device fallback/no-device behavior emits deterministic events and state outcome.
- Capture output contract is normalized to 16kHz mono before encoding stage.
</criteria>

<commands>
- `cd src-tauri && cargo test audio -- --nocapture`
- `cd src-tauri && cargo test hotkey -- --nocapture`
- `cd src-tauri && cargo test`
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
</plan>
