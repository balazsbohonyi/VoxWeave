---
wave: 2
depends_on:
  - 02-01-hotkey-runtime-PLAN.md
files_modified:
  - src-tauri/src/config/mod.rs
  - src-tauri/src/commands/config.rs
  - src-tauri/src/hotkey/mod.rs
  - src-tauri/src/hotkey/service.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/tray.rs
  - src/types/index.ts
  - src/composables/useConfig.ts
  - src/windows/settings/App.vue
  - .planning/phases/02-hotkey/02-VALIDATION.md
autonomous: true
requirements:
  - HOTK-03
  - HOTK-04
---

<plan>
<goal>
Make hotkey changes persist safely: normalize before save, re-register immediately, reject conflicts, and surface the locked warning path with focus + toast-like feedback.
</goal>

<must_haves>
- Persisted default is `Ctrl+Shift+Space`, not the old `Alt+Shift+Space`.
- Saving a changed hotkey re-registers immediately; restart restores the same canonical hotkey.
- Rebind sequence keeps the last working hotkey active if the new one conflicts.
- Startup registration failure leaves the app running, warns the user, and requires choosing a new binding.
- Conflict handling brings Settings to the foreground and emits a minimal toast-like warning path in Phase 2 without dragging in full later notification infrastructure.
</must_haves>

<tasks>
<task type="auto">
  <name>Task 1: Normalize and persist only safe hotkey changes</name>
  <files>src-tauri/src/config/mod.rs, src-tauri/src/commands/config.rs, src-tauri/src/hotkey/service.rs</files>
  <action>Update the config default to `Ctrl+Shift+Space`, then treat hotkey updates in `save_config` as a special flow: canonicalize before compare, no-op if the canonical value is unchanged, attempt to register the new binding before dropping the old one, and persist/mutate in-memory config only after the new binding succeeds. Keep command handlers thin by delegating all rebind logic into the hotkey service.</action>
  <verify>`cargo test hotkey::tests::apply_hotkey_change_persists_canonical_value -- --exact` passes</verify>
  <done>Hotkey saves apply immediately, persist in canonical form, and never replace a working runtime binding with a broken one.</done>
</task>

<task type="auto">
  <name>Task 2: Add conflict warnings that focus Settings and emit a toast-like event path</name>
  <files>src-tauri/src/hotkey/mod.rs, src-tauri/src/hotkey/service.rs, src-tauri/src/lib.rs, src-tauri/src/tray.rs, src/types/index.ts, src/composables/useConfig.ts, src/windows/settings/App.vue</files>
  <action>Extend the hotkey startup/save paths so registration failures become explicit warning events instead of silent logs. On save conflict, keep the last working binding active, show the existing Settings window, bring it to the foreground, and emit a narrow backend event payload that the current settings shell can render immediately as a toast-like warning surface. On startup conflict, mark the hotkey inactive but keep the app usable and surface the same warning path. Keep the implementation minimal and phase-bounded: no full notification center, no Phase 8 hotkey capture UI.</action>
  <verify>`cargo test hotkey::tests::conflicting_hotkey_keeps_last_working_binding -- --exact` passes</verify>
  <done>HOTK-04 is satisfied with the locked UX semantics preserved: warning path exists now, Settings is focused, and the app retains the last working binding or remains safely inactive at startup.</done>
</task>

<task type="auto">
  <name>Task 3: Complete persistence/conflict test coverage and Phase 2 UAT instructions</name>
  <files>src-tauri/src/hotkey/service.rs, .planning/phases/02-hotkey/02-VALIDATION.md</files>
  <action>Add focused tests for canonical persistence, unchanged canonical save short-circuit, conflicting rebind retaining the prior binding, startup conflict leaving the app usable but inactive, and tray/hotkey parity where appropriate. Then update `02-VALIDATION.md` so every real task maps to an automated command or explicit manual smoke step, using only the two actual Phase 2 plans.</action>
  <verify>`cargo test hotkey && cargo test` passes, and `02-VALIDATION.md` references only Phase 2 plans/tasks that actually exist</verify>
  <done>Wave 2 closes HOTK-03/HOTK-04 with executable tests plus a consistent validation contract ready for execute-phase and later verification.</done>
</task>
</tasks>

<verification>
<criteria>
- Changing the hotkey through config save applies immediately and survives restart in canonical form.
- Conflict paths never replace a working binding with a broken one.
- Warning UX exists now with Settings focus and a toast-like signal without dragging in later-phase settings/toast scope.
</criteria>

<commands>
- `cargo test hotkey`
- `cargo test`
- Manual Windows UAT from `.planning/phases/02-hotkey/02-VALIDATION.md`
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
</plan>
