---
wave: 1
depends_on: []
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/build.rs
  - src-tauri/capabilities/default.json
  - src-tauri/src/main.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/state.rs
  - src-tauri/src/tray.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/config/mod.rs
  - src-tauri/src/platform/mod.rs
  - src-tauri/src/platform/windows/mod.rs
  - src-tauri/tauri.conf.json
  - package.json
  - vite.config.ts
  - src/windows/settings/App.vue
  - src/windows/settings/main.ts
  - src/styles.css
  - index.html
autonomous: true
requirements: []
---

<plan>
<goal>
Create the greenfield Tauri/Vue scaffold, Rust module skeleton, shared app state, and platform seam Phase 1 needs.
</goal>

<must_haves>
- Tray-first startup shape exists; no hotkey/audio/indicator/transcription/injection work.
- One reusable settings window entrypoint exists in codebase.
- Platform traits exist now; Windows details stay behind `platform/windows/`.
- Backend module layout matches project docs so later phases extend, not refactor.
</must_haves>

<tasks>
<task id="01-01-01">
Bootstrap the Tauri v2 + Vue 3 + TypeScript + Tailwind project files and commit the permanent folder layout for `commands`, `config`, `platform`, `windows/settings`, and shared frontend types/composables placeholders.
</task>

<task id="01-01-02">
Create `main.rs`/`lib.rs` startup wiring with managed `AppState`, explicit window labels, and a tray-first bootstrap path that does not open normal UI on launch.
</task>

<task id="01-01-03">
Define `AppState` shells for config, quit intent, and future recording placeholders only as inert state needed by tray/window lifecycle.
</task>

<task id="01-01-04">
Add `platform/mod.rs` traits and `platform/windows/mod.rs` stub provider types aligned to `WindowInfo`, `ElevationChecker`, `InputSimulator`, and `ClipboardAccess`; no real OS behavior yet.
</task>

<task id="01-01-05">
Create a minimal settings window frontend shell that can render, be shown, and later host Phase 8 settings without restructuring entrypoints.
</task>
</tasks>

<verification>
<criteria>
- Project installs/build graph is valid enough for later waves to compile against.
- `main.rs` owns startup/module registration; platform traits are imported from one seam.
- No indicator window, hotkey registration, recording logic, transcription code, or injection code appears.
</criteria>

<commands>
- `cargo test`
- `npx vue-tsc --noEmit`
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
</plan>

