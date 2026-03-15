---
wave: 2
depends_on:
  - 01-01-scaffold-platform-PLAN.md
files_modified:
  - src-tauri/src/config/mod.rs
  - src-tauri/src/state.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/commands/config.rs
  - src/types/index.ts
  - src/composables/useConfig.ts
  - src/windows/settings/App.vue
autonomous: true
requirements:
  - CONF-01
  - CONF-02
  - CONF-03
---

<plan>
<goal>
Implement the full v1 config model and persistence layer at `%APPDATA%/VoxFlow/config.json` with stable defaults and unknown-field preservation.
</goal>

<must_haves>
- Full v1 `AppConfig` schema exists now, including future-phase fields.
- Missing fields default from Rust-owned defaults.
- Unknown JSON fields survive load/save round-trips.
- Config path is `%APPDATA%/VoxFlow/config.json`.
</must_haves>

<tasks>
<task id="01-02-01">
Define the complete `AppConfig` Rust schema, nested sections, and defaults covering engine, providers, API keys, models, language hints, hotkey, mic, local model, injection settings, autostart, indicator position, and first-launch flag.
</task>

<task id="01-02-02">
Implement config path resolution, directory creation, load/save helpers, and raw-json merge behavior so typed updates preserve unknown fields.
</task>

<task id="01-02-03">
Store config state in `AppState` using both typed config and whatever raw representation is required for round-trip preservation.
</task>

<task id="01-02-04">
Add thin config commands/composables sufficient to read current config and persist edits from the minimal settings shell without introducing Phase 8 UI scope.
</task>

<task id="01-02-05">
Add Rust tests for: no file -> defaults, partial file -> defaults filled, unknown fields preserved, config dir created, malformed file behavior fixed and documented.
</task>
</tasks>

<verification>
<criteria>
- Config reads/writes the required file path.
- Tests prove missing-field defaults and unknown-field preservation.
- Type mirrors exist for frontend consumption without inventing a second config schema.
</criteria>

<commands>
- `cargo test config`
- `cargo test`
- `npx vue-tsc --noEmit`
</commands>
</verification>

<unresolved_questions>
None. Malformed config should preserve the bad file and fall back via explicit error/default handling, not silently overwrite on read.
</unresolved_questions>
</plan>
