---
wave: 4
depends_on:
  - 02-01-hotkey-runtime-PLAN.md
  - 02-02-hotkey-config-conflicts-PLAN.md
  - 02-03-hotkey-settings-input-PLAN.md
files_modified:
  - src-tauri/tauri.conf.json
  - src-tauri/src/hotkey/service.rs
  - src-tauri/src/tray.rs
  - src/windows/indicator/*
  - src-tauri/src/lib.rs
autonomous: true
gap_closure: true
requirements:
  - HOTK-01
  - HOTK-02
---

<plan>
<goal>
Close Phase 02 UAT indicator gaps by wiring a real floating indicator window into the existing hotkey/tray state transitions.
</goal>

<must_haves>
- Indicator window exists in Tauri config with transparent always-on-top behavior.
- Shared toggle path controls indicator visibility for both tray and hotkey triggers.
- Indicator show/hide behavior matches recording state transitions (show on recording start, hide when returning idle).
- Add basic regression checks for indicator visibility in hotkey/tray flows.
</must_haves>

<tasks>
<task type="auto">
  <name>Task 1: Define indicator window and bootstrap frontend entry</name>
  <files>src-tauri/tauri.conf.json, src/windows/indicator/*</files>
  <action>Add a dedicated `indicator` window config and minimal frontend mount content so the window can render. Keep it lightweight and non-interactive.</action>
  <verify>Launch app and confirm indicator window can be shown by label without focus steal.</verify>
  <done>Indicator window exists as a runtime target for show/hide calls.</done>
</task>

<task type="auto">
  <name>Task 2: Wire indicator visibility into shared recording toggle path</name>
  <files>src-tauri/src/hotkey/service.rs, src-tauri/src/tray.rs, src-tauri/src/lib.rs</files>
  <action>In shared state transition functions, call indicator show/hide helpers based on state. Ensure tray and hotkey both use the same path and no duplicate transitions occur.</action>
  <verify>Manual UAT: configured hotkey and tray start both show indicator; stop hides indicator.</verify>
  <done>Indicator visibility is deterministic and parity is preserved across triggers.</done>
</task>

<task type="auto">
  <name>Task 3: Add focused verification and docs for gap closure</name>
  <files>.planning/phases/02-hotkey/02-VALIDATION.md, .planning/phases/02-hotkey/02-UAT.md</files>
  <action>Document concrete retest steps for indicator visibility and ensure validation references this plan.</action>
  <verify>Validation checklist explicitly covers both gap truths and pass conditions.</verify>
  <done>Gap-only execution has explicit verification contract before marking phase stable.</done>
</task>
</tasks>

<verification>
<criteria>
- Starting recording from hotkey shows indicator and toggles tray state.
- Starting recording from tray shows indicator with same behavior.
- Stopping recording hides indicator for both trigger paths.
- No regressions in hotkey apply/conflict flows from tests 3-7.
</criteria>

<commands>
- cargo test hotkey -- --nocapture
- npx vue-tsc --noEmit
- Manual UAT from 02-UAT.md (retest tests 1 and 2)
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
</plan>