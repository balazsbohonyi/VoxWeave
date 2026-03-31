# Phase 1 Research: Foundation

## Objective

Plan Phase 1 so it establishes the permanent app skeleton for VoxWeave without pulling future pipeline work forward. This phase should end with a tray-first Windows Tauri app, a single reusable settings window, config persistence at `%APPDATA%/VoxWeave/config.json`, and the platform abstraction seam that later phases build on.

## Scope Lock

Implement now:
- Tauri v2 app scaffold
- Rust backend module layout for `main.rs`, `tray.rs`, `state.rs`, `config/`, `platform/`
- One settings window that is hidden by default and reused
- Tray icon + menu + double-click behavior
- Config load/save with defaults for missing fields and preservation of unknown JSON fields
- Windows platform module stubs behind traits

Do not implement now:
- Global hotkey registration
- Recording state machine beyond a placeholder enum/value
- Floating indicator window
- Any transcription, injection, notifications, wizard, or full settings UI
- Dynamic tray icon state switching for recording

## Requirements Translation

Phase 1 covers:
- `CONF-01`: persist config in JSON at `%APPDATA%/VoxWeave/config.json`
- `CONF-02`: define the full v1 config schema now, not a temporary subset
- `CONF-03`: missing fields default; unknown fields survive round-trip
- `TRAY-01`: tray icon exists on launch
- `TRAY-02`: tray menu shape exists now
- `TRAY-03`: double-click tray opens settings
- `TRAY-04`: closing settings hides to tray

Important nuance:
- `TRAY-01` says icon changes during recording, but Phase 1 context explicitly defers recording-state icon switching. Plan against the phase context, not the raw requirement wording.
- `TRAY-02` menu includes `Start/Stop Recording`, but Phase 1 context says it should be present and disabled.
- `CONF-02` means the config type must already include future-phase fields, even if many are unused by current UI.

## Standard Stack

- Tauri v2 Rust APIs for tray, menu, path resolution, window management, and lifecycle hooks.
- Vue 3 + TypeScript + Vite for the settings webview, but only a minimal shell is needed in this phase.
- Tailwind CSS if frontend styling is touched, but avoid full settings information architecture in Phase 1.
- `serde` + `serde_json` for config serialization.
- `std::fs` for config file IO; no need for a persistence plugin.
- `windows` crate reserved for future Windows implementations, but Phase 1 only needs trait definitions and possibly placeholder Windows structs.

## Architecture Patterns

## 1. Backend-first foundation

Phase 1 should be planned as a backend-heavy phase with only a minimal frontend shell. The durable architecture lives in Rust:
- `main.rs`: builder setup, state registration, window/tray wiring
- `tray.rs`: tray creation and menu event routing
- `config/mod.rs`: schema, defaults, load/save, path resolution
- `state.rs`: `AppState` holding config and any minimal app status needed by tray/menu logic
- `platform/mod.rs`: trait definitions
- `platform/windows/`: Windows stub implementations or placeholders

Reason: later phases depend on Rust-owned lifecycle and platform services more than on frontend structure.

## 2. Single hidden settings window, reused

Use one named webview window, hidden at startup, then show/focus it from tray actions. Do not create a new window on each tray interaction.

Why:
- matches phase context exactly
- simplifies close-to-tray behavior
- avoids duplicate window bugs
- gives later phases a stable window label for IPC and window restoration

Tauri v2 docs support this pattern through `app.get_webview_window(label)`, `show()`, `unminimize()`, and `set_focus()`, and tray event handlers can call this directly.

## 3. Intercept close, do not destroy

Closing the settings window should prevent actual close and hide instead. Planning should assume a per-window close-request handler plus an app-level exit path for `Quit`.

Needed distinction:
- window close from titlebar: prevent close, hide window
- tray quit action: allow real app exit

If this distinction is not explicit in the plan, the app will either quit unexpectedly or become impossible to exit cleanly.

## 4. Config service owns round-trip compatibility

Do not scatter config logic across commands or frontend composables. Plan a dedicated config service with:
- full `AppConfig`
- `Default` for stable missing-field behavior
- load path resolution from app config directory
- directory creation if missing
- parse/merge behavior that preserves unknown fields
- atomic-ish write pattern appropriate for a single JSON file

Unknown field preservation is the nontrivial part. A straightforward typed deserialize/serialize will drop fields the current struct does not know about.

Recommended model:
- deserialize raw JSON into `serde_json::Value`
- deserialize typed config from that value
- when saving, serialize typed config back to `Value`
- merge typed value into original raw object so known fields update while unknown fields remain
- write merged JSON back to disk

This implies the config manager should retain raw JSON alongside typed state, or return a structure that contains both.

## 5. Trait seam before Windows details

`platform/mod.rs` should define the interfaces now even if most methods are unused in Phase 1. This is cheaper than retrofitting later when hotkey, injection, and elevation concerns appear.

At minimum align the trait surface with project docs:
- `WindowInfo`
- `ElevationChecker`
- `InputSimulator`
- `ClipboardAccess`

Phase 1 does not need real implementations for all methods, but it should decide:
- trait names
- module boundaries
- constructor/ownership pattern
- how these services will enter app logic later

Best planning choice: define traits and a `PlatformServices` aggregate or provider pattern, but avoid implementing unused behavior until the dependent phase.

## 6. Thin commands pattern should exist even if barely used

The project docs require thin `commands/` handlers. Phase 1 may only need a minimal config get/update command or none at all if the settings shell is static, but the module boundary should still be laid down so later phases do not need structural refactors.

## Planning Implications

The clean decomposition is:
1. Create Tauri/Vue scaffold and app identifiers.
2. Establish backend module skeleton and `AppState`.
3. Implement config schema/defaults/path IO/round-trip preservation.
4. Create hidden reusable settings window and close-to-tray behavior.
5. Build tray icon, menu, double-click handler, and quit path.
6. Add platform traits and Windows stub module structure.
7. Add minimal frontend shell only sufficient to verify window open/hide behavior.
8. Verify with Rust tests for config logic and manual Windows tray/window validation.

This order matters because tray and window wiring should consume a working state/config layer, not invent temporary state that gets deleted in Phase 2.

## Key Technical Decisions To Lock During Planning

## Config representation

Lock these early:
- full v1 `AppConfig` exists in Phase 1
- defaults live in Rust, not only frontend
- config file location is resolved from Tauri path APIs, not string-built manually
- unknown-field preservation strategy is explicit in implementation tasks

If the plan leaves unknown-field preservation vague, it will almost certainly be skipped.

## Window creation strategy

Choose one and stick to it:
- preferred: create the settings window during setup as hidden, then reuse it
- acceptable: lazy-create on first open, then reuse

For Phase 1 planning, eager hidden creation is simpler to validate because tray actions only need to show/focus an existing labeled window.

## Quit path

The plan must separate:
- `hide window`
- `close window`
- `exit app`

This is the main lifecycle trap in tray apps.

## Start/Stop Recording menu behavior

Treat it as a disabled placeholder in Phase 1. Do not wire fake recording toggles just to make the menu item appear.

## App startup behavior

Normal startup should be tray-only with no visible window. Since the first-launch wizard is Phase 9, do not add wizard branching now.

## Don�t Hand-Roll

- Do not hand-roll tray behavior in the frontend. Keep tray/menu logic in Rust with Tauri APIs.
- Do not invent a custom config DSL or split config into multiple files.
- Do not hand-roll platform conditionals across core logic; use the trait seam immediately.
- Do not build a temporary phase-only config struct. That creates migration work in every later phase.
- Do not use a Tauri clipboard plugin for future clipboard work; project docs already reject that path.
- Do not create/destroy settings windows repeatedly. Reuse one window.

## Common Pitfalls

## 1. Confusing hide-to-tray with exit prevention

If only window close is intercepted, the app may still exit when the last window is hidden or closed depending on wiring. The quit flow must be intentional.

## 2. Losing unknown config fields

Plain typed deserialize/serialize will remove unknown JSON properties. This directly violates `CONF-03` as interpreted by project docs.

## 3. Letting frontend own defaults

Defaults must be canonical in Rust because config load occurs before any frontend is needed and later backend logic depends on stable values.

## 4. Overbuilding the settings UI

Phase 1 needs a window shell, not the Phase 8 settings product. Planning should keep frontend work intentionally thin.

## 5. Baking Windows APIs into non-platform modules

Even small convenience imports from `windows` in `tray.rs` or `main.rs` weaken the future macOS seam. Keep OS-specific code inside `platform/windows/`.

## 6. Treating tray double-click as generic click

Tauri v2 tray APIs distinguish click and double-click events. The plan should target explicit double-click handling for `TRAY-03`.

## Code Examples

## Tray and menu pattern

Official Tauri v2 docs show `TrayIconBuilder` with:
- `.menu(&menu)`
- `.on_menu_event(...)`
- `.on_tray_icon_event(...)`

This is the correct shape for Phase 1. Use menu event IDs like `settings`, `recording`, `quit`.

## Show/focus existing window pattern

Tauri docs show the stable sequence:
- `app.get_webview_window("settings")`
- `unminimize()`
- `show()`
- `set_focus()`

Use this for both tray menu `Settings` and tray double-click behavior.

## Path resolution pattern

Tauri v2 docs moved path access behind the manager path API. Plan around `app.path()` / `app_handle.path()` for config directory resolution instead of deprecated v1 path helpers.

## Exit prevention pattern

Tauri docs show app-level exit prevention via lifecycle hooks. Even if Phase 1 only needs window close interception plus tray quit, plan the lifecycle flow with a clear quit flag or equivalent so hide-to-tray does not block intentional exit.

## Validation Architecture

Phase 1 warrants explicit validation architecture because the hardest requirement is behavioral, not algorithmic.

## Automated validation

Rust unit tests in `config/` should cover:
- no config file -> defaults returned
- partial config file -> missing fields filled from defaults
- config with unknown fields -> unknown fields remain after save
- config directory creation
- malformed JSON -> defined failure path or fallback behavior, whichever the plan chooses

These tests provide the only reliable proof for `CONF-01` to `CONF-03`.

## Manual validation on Windows

Phase 1 also needs manual UAT steps because tray behavior is OS-integrated:
1. Launch app; verify no visible window appears and tray icon exists.
2. Right-click tray; verify menu entries and disabled `Start/Stop Recording`.
3. Double-click tray; verify settings window opens/focuses.
4. Close settings via titlebar; verify window hides and process remains alive in tray.
5. Reopen settings from tray repeatedly; verify same window instance behavior.
6. Quit from tray; verify process exits fully.
7. Edit config file to add unknown keys; relaunch/save config; verify unknown keys remain.

## Recommended Plan Slices

Use small plans with these boundaries:

### Slice A: Scaffold + state
- Tauri/Vue project init
- Rust module skeleton
- `AppState` shell

### Slice B: Config subsystem
- full `AppConfig`
- defaults
- path resolution
- load/save
- unknown field preservation
- tests

### Slice C: Window lifecycle
- settings window creation/reuse
- hidden startup
- close-to-tray interception

### Slice D: Tray integration
- tray icon
- menu
- double-click open
- quit flow
- disabled recording item

### Slice E: Platform seam
- trait definitions
- Windows stub modules
- dependency injection choice

This decomposition minimizes rework and gives each slice a clear verification target.

## Open Questions To Resolve In Planning

- Should malformed config fall back to defaults silently, or preserve the bad file and surface an error path? Pick one behavior explicitly.
- Is the settings window created eagerly during setup or lazily on first open? Eager is simpler for Phase 1.
- Will `AppState` store only typed config, or typed config plus original raw JSON needed for unknown-field preservation? The latter is safer.
- Will Phase 1 expose any config IPC yet, or is a static settings shell sufficient until Phase 8?

## Sources

- Provided project docs: `.planning/phases/01-foundation/01-CONTEXT.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md`, `CLAUDE.md`
- Official Tauri v2 docs via Context7: `/tauri-apps/tauri-docs` on tray APIs, tray event handling, existing window show/focus flow, and manager path APIs
