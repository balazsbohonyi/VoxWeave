---
wave: 2
depends_on:
  - 03-01-capture-lifecycle-PLAN.md
files_modified:
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/commands/audio.rs
  - src-tauri/src/lib.rs
  - src/composables/useConfig.ts
  - src/windows/settings/App.vue
  - src/types/index.ts
autonomous: true
requirements:
  - AUDI-04
  - AUDI-05
---

<plan>
<goal>
Deliver Phase 3 microphone selection UI path: list available input devices, persist selection, and keep fallback behavior visible.
</goal>

<must_haves>
- Settings shows a microphone dropdown backed by live device enumeration command.
- Selected device persists through existing `save_config` flow and restores on reload.
- UI handles selected-device-missing fallback notification emitted by backend.
- Scope stays minimal: only controls required for AUDI-04 in the current lightweight settings shell.
</must_haves>

<tasks>
<task type="auto">
  <name>Task 1: Add backend device enumeration command</name>
  <files>src-tauri/src/commands/mod.rs, src-tauri/src/commands/audio.rs, src-tauri/src/lib.rs</files>
  <action>Add a thin Tauri command (`list_audio_input_devices`) returning ordered, deduplicated input-device names from the audio capture backend. Register it in `invoke_handler` and keep command module free of business logic.</action>
  <verify>`cd src-tauri && cargo test commands::audio::tests::lists_input_devices -- --exact` passes</verify>
  <done>Frontend can request device options from backend without touching capture internals directly.</done>
</task>

<task type="auto">
  <name>Task 2: Extend settings shell with microphone dropdown + persistence wiring</name>
  <files>src/windows/settings/App.vue, src/composables/useConfig.ts, src/types/index.ts</files>
  <action>Load input device list on settings mount, render dropdown bound to `config.audio.device`, and persist changes through existing immediate-save config pathway. Keep UX concise and compatible with future Phase 8 layout replacement.</action>
  <verify>`npx vue-tsc --noEmit` passes</verify>
  <done>AUDI-04 persistence path works end-to-end through current settings shell.</done>
</task>

<task type="auto">
  <name>Task 3: Surface fallback state in settings warning region</name>
  <files>src/windows/settings/App.vue, src/types/index.ts</files>
  <action>Reuse existing warning surface to display backend fallback/no-device notifications relevant to audio device selection so users understand when selected device was replaced by default.</action>
  <verify>Manual: `cargo tauri dev`, select removable mic, disconnect mic, verify warning appears and config remains valid</verify>
  <done>Fallback behavior is user-visible and debuggable without waiting for full notification phase.</done>
</task>
</tasks>

<verification>
<criteria>
- Device dropdown lists real input devices and persists selected value.
- Restart restores selected device when still available.
- Fallback warning is visible when selected device is unavailable and default is used.
</criteria>

<commands>
- `cd src-tauri && cargo test commands::audio::tests::lists_input_devices -- --exact`
- `npx vue-tsc --noEmit`
- Manual Windows checklist from `03-VALIDATION.md`
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
</plan>
