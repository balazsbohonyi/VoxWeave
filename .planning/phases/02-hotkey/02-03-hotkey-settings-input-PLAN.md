---
wave: 3
depends_on:
  - 02-02-hotkey-config-conflicts-PLAN.md
files_modified:
  - src/windows/settings/App.vue
  - src/composables/useConfig.ts
  - .planning/phases/02-hotkey/02-VALIDATION.md
autonomous: true
gap_closure: true
requirements:
  - HOTK-03
---

<plan>
<goal>
Close the Phase 2 verification gap by adding a minimal Settings hotkey edit flow that saves through existing backend rebind/persistence logic.
</goal>

<must_haves>
- Settings UI has an editable hotkey field (not read-only) and a clear apply action.
- Applying a new hotkey calls existing `save_config` flow, so canonicalization/rebind safety stays backend-owned.
- Success and failure feedback are visible in Settings without introducing full Phase 8 settings architecture.
- Existing warning surface remains intact for conflict/startup warning events.
</must_haves>

<tasks>
<task type="auto">
  <name>Task 1: Add minimal hotkey edit + apply controls in Settings</name>
  <files>src/windows/settings/App.vue</files>
  <action>Replace the read-only hotkey line with a focused edit surface: local draft field, dirty-state detection, and an explicit Apply action. Keep scope narrow to Phase 2 by editing only hotkey (no full tabbed settings UI). Preserve existing loading/error rendering and warning toast block.</action>
  <verify>Manual smoke: with settings open, edit hotkey text and apply; UI shows saving state and returns to non-dirty state on success.</verify>
  <done>Users can initiate a hotkey change from Settings instead of read-only display.</done>
</task>

<task type="auto">
  <name>Task 2: Wire apply flow through existing composable save path</name>
  <files>src/composables/useConfig.ts, src/windows/settings/App.vue</files>
  <action>Use `saveConfig` with `{ hotkey: draftHotkey }` from the Settings component and keep backend command contract unchanged. Ensure the UI reflects command errors from Rust (conflict/validation) and keeps draft value behavior predictable after failed saves.</action>
  <verify>Manual smoke: force a conflict and confirm Settings shows failure while current active config remains unchanged.</verify>
  <done>Frontend uses the backend-owned hotkey rebind/persist pipeline without duplicating business logic.</done>
</task>

<task type="auto">
  <name>Task 3: Update validation doc for the new gap-closure plan</name>
  <files>.planning/phases/02-hotkey/02-VALIDATION.md</files>
  <action>Add this plan's manual verification steps and command references so HOTK-03 now has explicit UI-level coverage in Phase 2 validation.</action>
  <verify>Validation doc references `02-03-hotkey-settings-input-PLAN.md` and includes a clear pass/fail manual checklist.</verify>
  <done>Gap-closure execution and later verification have an explicit validation contract.</done>
</task>
</tasks>

<verification>
<criteria>
- Settings supports changing hotkey via UI and applying it through existing save command.
- Successful apply updates displayed hotkey and persists across restart (manual check).
- Failed apply (conflict) surfaces actionable feedback while keeping previous working binding.
</criteria>

<commands>
- Manual Windows UAT from `.planning/phases/02-hotkey/02-VALIDATION.md`
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
</plan>
