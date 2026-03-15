---
wave: gaps
depends_on:
  - 03-01-capture-lifecycle-PLAN.md
  - 03-03-audio-device-settings-PLAN.md
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/src/audio/capture.rs
  - src-tauri/src/audio/mod.rs
  - src-tauri/src/commands/audio.rs
  - src/composables/useConfig.ts
  - src/windows/settings/App.vue
autonomous: true
requirements:
  - AUDI-01
  - AUDI-04
  - AUDI-05
---

<plan>
<goal>
Close Phase 03 UAT gaps by implementing real microphone discovery for production builds and making device-refresh/no-device outcomes visible in settings.
</goal>

<must_haves>
- Recording can start when at least one input device exists.
- Device list refresh returns real hardware devices, not just static UI fallback.
- No-device state is explicit to user (clear warning/error path).
- Existing fallback behavior (selected device unavailable -> fallback warning) remains intact.
</must_haves>

<tasks>
<task type="auto">
  <name>Task 1: Replace production audio snapshot stub with real device enumeration</name>
  <files>src-tauri/Cargo.toml, src-tauri/src/audio/capture.rs</files>
  <action>Add production implementation using `cpal` host APIs to enumerate input devices and detect default input. Keep deterministic dedupe/order, preserve current test-only mock path under `cfg(test)`.</action>
  <verify>`cd src-tauri && cargo test audio::mod::tests::missing_selected_device_falls_back -- --exact`</verify>
  <done>`system_device_snapshot()` returns real device names/default on Windows in non-test builds.</done>
</task>

<task type="auto">
  <name>Task 2: Ensure recording start path uses real snapshot and surfaces no-device errors predictably</name>
  <files>src-tauri/src/audio/mod.rs, src-tauri/src/commands/audio.rs</files>
  <action>Keep start path behavior consistent with UAT expectations: when devices exist, session starts; when none exist, emit `audio-error` with actionable message and return idle without silent failure.</action>
  <verify>`cd src-tauri && cargo test audio::mod::tests::no_device_returns_error -- --exact`</verify>
  <done>Hotkey/tray start transitions succeed with devices present and fail loudly when no devices exist.</done>
</task>

<task type="auto">
  <name>Task 3: Make settings refresh and empty-state behavior observable</name>
  <files>src/composables/useConfig.ts, src/windows/settings/App.vue</files>
  <action>On Refresh, reload device list from backend and render explicit UI state: list of devices when available, and "No microphone devices detected" when empty; keep static System default option but distinguish it from discovered devices.</action>
  <verify>`cmd /c npx vue-tsc --noEmit`</verify>
  <done>Users can tell whether refresh discovered hardware and why only default selection is shown.</done>
</task>

<task type="auto">
  <name>Task 4: Add targeted tests for production snapshot mapping and settings-device flow</name>
  <files>src-tauri/src/audio/capture.rs, src-tauri/src/commands/audio.rs, src/composables/useConfig.ts (if test harness exists)</files>
  <action>Add/adjust tests to cover dedupe + default-device mapping from enumerated devices and command return contract. If frontend unit harness is unavailable, add minimal backend coverage and manual checklist updates.</action>
  <verify>`cd src-tauri && cargo test commands::audio::tests::lists_input_devices -- --exact`</verify>
  <done>Regression coverage exists for the two UAT gaps.</done>
</task>
</tasks>

<verification>
<criteria>
- Start recording works from hotkey and tray when input devices exist.
- Settings refresh shows plugged/unplugged devices after refresh.
- UAT Test 1 and Test 2 scenarios pass on Windows manual validation.
</criteria>

<commands>
- `cd src-tauri && cargo test`
- `cmd /c npx vue-tsc --noEmit`
- Manual: run app, plug/unplug headset, click Refresh, then test hotkey and tray start/stop.
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
</plan>