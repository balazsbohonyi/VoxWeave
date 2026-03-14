---
wave: 3
depends_on:
  - 01-01-scaffold-platform-PLAN.md
  - 01-02-config-persistence-PLAN.md
files_modified:
  - src-tauri/src/main.rs
  - src-tauri/src/tray.rs
  - src-tauri/src/state.rs
  - src-tauri/src/config/mod.rs
  - src-tauri/tauri.conf.json
  - src/windows/settings/App.vue
  - src/windows/settings/main.ts
autonomous: true
requirements:
  - TRAY-01
  - TRAY-02
  - TRAY-03
  - TRAY-04
---

<plan>
<goal>
Finish the native tray and settings-window lifecycle so the app launches into the tray, reuses one settings window, and hides to tray on close.
</goal>

<must_haves>
- App launches with tray icon and no visible window.
- Tray menu is `Settings`, disabled `Start/Stop Recording`, separator, `Quit`.
- Double-click tray opens/focuses the single settings window.
- Closing settings hides it; quit only exits from tray quit path.
</must_haves>

<tasks>
<task id="01-03-01">
Implement tray creation, menu IDs, disabled recording placeholder item, and event routing in `tray.rs`; keep recording icon switching out of scope until Phase 2 state exists.
</task>

<task id="01-03-02">
Create or eagerly register one hidden `settings` window, then reuse it for tray menu open and tray double-click open/focus behavior.
</task>

<task id="01-03-03">
Intercept settings-window close requests so titlebar close hides the window instead of quitting; ensure tray `Quit` bypasses that path and exits cleanly.
</task>

<task id="01-03-04">
Wire minimal settings-shell actions needed to prove open/focus/hide lifecycle and surface config values from the persistence layer.
</task>

<task id="01-03-05">
Document and run manual Windows validation for tray-only startup, tray menu shape, double-click open, repeated reopen/focus, close-to-tray, and quit behavior.
</task>
</tasks>

<verification>
<criteria>
- Tray behavior matches all Phase 1 success criteria except recording-state icon switching, which is intentionally deferred.
- Reopening settings never creates duplicates.
- Window close and app quit paths are distinct and reliable.
</criteria>

<commands>
- `cargo test`
- `npx vue-tsc --noEmit`
- Manual Windows UAT from `.planning/phases/01-foundation/01-VALIDATION.md`
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
</plan>
