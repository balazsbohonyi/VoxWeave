# Phase 2: Hotkey - Research

**Researched:** 2026-03-15
**Domain:** Tauri v2 global shortcut registration and toggle-state orchestration
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Hotkey format + normalization
- Default hotkey should be `Ctrl+Shift+Space` (align with ROADMAP; update current default of Alt+Shift+Space).
- Allowed modifiers: `Ctrl`, `Alt`, `Shift`, and `Win`.
- Store hotkey strings in canonical order with consistent casing (e.g., `Ctrl+Alt+Shift+Win+Key`).
- Base key support: allow any key supported by the hotkey library (no extra internal restriction).

### Conflict handling
- Treat conflicts only when registration fails (no preemptive OS-reserved blacklist).
- On conflict: reject new hotkey and keep the last working hotkey active.
- Warning UX: show a toast and bring Settings to foreground.
- If the stored hotkey fails at startup: hotkey remains inactive; warn the user and require a new selection.

### Toggle behavior
- Second hotkey press advances state from `Recording` to `Transcribing` (processing).
- Hotkey presses during `Transcribing` are ignored.
- Tray menu Start/Stop action mirrors the same toggle behavior.
- Keep the state name `Transcribing` (use UI copy like “Processing” later if needed).

### Activation + update sources
- Hotkey is active whenever the app is running (global).
- Hotkey is re-registered on config save when the hotkey value changes.
- No “disabled hotkey” mode (empty string not allowed).
- Tray Start/Stop menu should be enabled in Phase 2.

### Claude's Discretion
- Exact canonical ordering and casing rules for normalization (as long as consistent).
- Warning toast copy and exact UX timing when conflicts occur.

### Deferred Ideas (OUT OF SCOPE)
- Hotkey capture UI and conflict prompts inside settings (Phase 8).
- Cancel behavior during processing (Phase 7+), beyond “ignore hotkey while Transcribing.”
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| HOTK-01 | User can trigger recording via a global hotkey (default `Ctrl+Shift+Space`) from any application | Use `tauri-plugin-global-shortcut` in Rust, register on startup, and handle only `Pressed` events. |
| HOTK-02 | Hotkey operates in toggle mode — first press starts recording, second press stops and triggers transcription | Put all transitions behind one shared toggle function used by both hotkey and tray. |
| HOTK-03 | User can change the hotkey in settings and the new binding persists across restarts | Normalize before save, register new shortcut before unregistering old, then persist canonical string. |
| HOTK-04 | App detects and warns about hotkey conflicts with other applications | Treat plugin registration failure as conflict, reject save, keep last working binding, emit warning event, and focus Settings. |
</phase_requirements>

## Summary

Phase 2 should use Tauri’s official `tauri-plugin-global-shortcut` rather than custom Windows APIs. The plugin supports startup registration, runtime `register` / `unregister`, and per-shortcut handlers via `GlobalShortcutExt`, which is the right fit for VoxFlow’s persisted, user-configurable binding. This keeps the implementation Tauri-v2-native and avoids leaking Windows-specific hotkey code into the repo’s platform seam.

The key planning constraint is not registration itself, but state ownership. Hotkey presses and tray Start/Stop must call the exact same backend toggle function, and that function must remain phase-bounded: only mutate `RecordingState`, update tray/menu labels, and emit backend events. Do not mix audio startup, indicator logic, or transcription work into this phase. The “processing” transition should exist as a seam now, even if Phase 2 completes it with an immediate placeholder return to `Idle`.

Conflict handling should be optimistic and registration-driven. Normalize the new shortcut, attempt to register it while the old shortcut is still active, and only persist/update active state if registration succeeds. On failure, keep the old binding active, reject the save, and warn the user. Startup failure is different: the app should still boot, but with the hotkey inactive until the user picks a valid replacement.

**Primary recommendation:** Add a dedicated `hotkey` backend module that owns normalization, registration lifecycle, and the shared `toggle_recording_state` entrypoint used by both the global shortcut handler and the tray menu.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tauri-plugin-global-shortcut` | `2.x` | Global shortcut registration and runtime re-registration | Official Tauri v2 plugin; supports runtime `register`, `unregister`, and `on_shortcut`. |
| `tauri` | `2.x` | App lifecycle, tray menu mutation, event emission, managed state | Already in repo; Tauri menu items support runtime `set_text` and `set_enabled`. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` / `serde_json` | existing | Persist canonical hotkey string in config | Reuse current config save/load flow; do not create a separate hotkey store. |
| `std::sync::{Arc, Mutex}` | existing | Keep active registered hotkey and recording state in managed state | Matches the repo’s existing `AppState` pattern. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `tauri-plugin-global-shortcut` | Windows `RegisterHotKey` / low-level hooks | Reject. Cross-platform debt, custom parsing, and avoidable OS-specific code. |
| Canonical string storage | Raw user-entered string | Reject. Causes duplicate/false-diff saves and inconsistent re-registration behavior. |

**Installation:**
```bash
cargo add tauri-plugin-global-shortcut@2
```

## Architecture Patterns

### Recommended Project Structure
```text
src-tauri/src/
├── hotkey/
│   ├── mod.rs              # Public hotkey API
│   ├── normalize.rs        # Parse/normalize/canonicalize user strings
│   └── service.rs          # Register/unregister/apply-hotkey logic
├── commands/
│   └── config.rs           # Save hook calls hotkey service when hotkey changes
├── state.rs                # RecordingState + hotkey runtime state
├── tray.rs                 # Menu item text/enabled updates + shared toggle dispatch
└── lib.rs                  # Plugin install + startup registration
```

### Pattern 1: Hotkey Service Owns Registration Lifecycle
**What:** Centralize startup registration, config-change re-registration, and warning event emission in one backend service.
**When to use:** Always; do not spread registration across `lib.rs`, `commands/config.rs`, and `tray.rs`.
**Example:**
```rust
use tauri::AppHandle;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub fn apply_hotkey(app: &AppHandle, new_hotkey: &str) -> Result<(), String> {
    let shortcuts = app.global_shortcut();
    shortcuts.on_shortcut(new_hotkey, move |app, _shortcut, event| {
        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
            let _ = crate::hotkey::toggle_recording_state(app);
        }
    }).map_err(|e| e.to_string())
}
```
Source: https://github.com/tauri-apps/tauri-plugin-global-shortcut/blob/v2/README.md

### Pattern 2: Canonicalize Before Compare, Register, or Persist
**What:** Normalize every hotkey string into a canonical form before diffing or saving.
**When to use:** On startup load, on config save, and in tests.
**Canonical rule:** Modifiers in fixed order `Ctrl+Alt+Shift+Win+Key`, title case for modifiers, keep the plugin-supported key token verbatim once parsed.
**Example:**
```rust
assert_eq!(normalize_hotkey("shift+ctrl+space")?, "Ctrl+Shift+Space");
assert_eq!(normalize_hotkey("Win+Alt+K")?, "Alt+Win+K");
```

### Pattern 3: Shared Toggle Path for Hotkey and Tray
**What:** Both input sources call one pure-ish backend transition function.
**When to use:** Always; tray must never own separate state logic.
**Recommended transition table:**

| Current | Action | Next | Notes |
|--------|--------|------|-------|
| `Idle` | toggle | `Recording` | Update tray text to `Stop Recording`. |
| `Recording` | toggle | `Transcribing` | Emit processing state; Phase 2 may immediately complete placeholder work and return to `Idle`. |
| `Transcribing` | toggle | unchanged | Ignore hotkey/tray action; optionally disable tray item. |

### Pattern 4: Register New Before Unregistering Old
**What:** Preserve last working binding on config changes.
**When to use:** Any persisted hotkey change.
**Safe sequence:**
1. Normalize new hotkey.
2. If canonical string equals active hotkey, persist canonical string only.
3. Try to register new hotkey while old remains active.
4. If success, unregister old hotkey, update runtime state, then persist config.
5. If failure, keep old hotkey active, reject save, emit warning event, and focus Settings.

### Anti-Patterns to Avoid
- **Unregister-first rebind:** A failed new registration leaves the app with no active hotkey.
- **Separate tray state logic:** Creates drift between tray behavior and shortcut behavior.
- **Raw-string comparisons:** `ctrl+shift+space` and `Ctrl+Shift+Space` become false changes.
- **Handling key release too:** The plugin emits stateful events; only act on `Pressed`.
- **Stuffing hotkey logic into `commands/config.rs`:** Commands should stay thin in this repo.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Global shortcut registration | Win32 `RegisterHotKey` layer | `tauri-plugin-global-shortcut` | Official Tauri v2 path; less OS glue and fewer lifecycle bugs. |
| Conflict prediction | Reserved-hotkey blacklist | Actual plugin registration attempt | OS/app conflicts are environment-specific; registration result is the source of truth. |
| Tray state duplication | Separate menu-specific recording toggles | Shared backend `toggle_recording_state` | Ensures hotkey and tray stay behaviorally identical. |
| Hotkey persistence | Separate hotkey config file | Existing `AppConfig.hotkey` + `save_config` flow | Repo already has unknown-field-safe config persistence. |

**Key insight:** The expensive bugs here are lifecycle bugs, not parser bugs. The planner should bias toward a small number of well-owned transitions and registration pathways.

## Common Pitfalls

### Pitfall 1: Default Hotkey Drift
**What goes wrong:** Repo default is still `Alt+Shift+Space`, but Phase 2 requires `Ctrl+Shift+Space`.
**Why it happens:** Current `AppConfig::default()` and persistence tests still encode the old default.
**How to avoid:** Update config default, tests, and any sample configs in the same plan wave.
**Warning signs:** New installs come up on `Alt+Shift+Space`, or tests still assert the old value.

### Pitfall 2: Save Succeeds Even Though Registration Failed
**What goes wrong:** Config file stores a broken/conflicting hotkey while the old binding disappears.
**Why it happens:** Persistence happens before runtime registration succeeds.
**How to avoid:** Registration must happen before mutating in-memory config and before disk save on hotkey changes.
**Warning signs:** After changing hotkey, restart shows the new value but no shortcut works.

### Pitfall 3: Permanent `Transcribing` State in Phase 2
**What goes wrong:** Second press reaches `Transcribing`, then the app is stuck because audio/transcription phases do not exist yet.
**Why it happens:** Phase 2 models future states without a placeholder completion path.
**How to avoid:** Add an explicit seam such as `begin_processing_placeholder()` that transitions to `Transcribing`, emits state, then immediately or near-immediately completes back to `Idle`.
**Warning signs:** Tray item stays disabled forever after one recording cycle.

### Pitfall 4: Warning UX Depends on Future Toast System
**What goes wrong:** HOTK-04 blocks because Phase 7 toast infrastructure is not available yet.
**Why it happens:** The context asks for a toast-like warning, but notifications are a later phase.
**How to avoid:** Emit a focused backend event now, show Settings immediately, and let the frontend render the smallest possible warning surface for Phase 2.
**Warning signs:** Conflict errors exist only in logs or command return strings.

### Pitfall 5: Runtime Menu Item Cannot Reflect State
**What goes wrong:** Tray text stays `Start / Stop Recording` and never reflects actual mode.
**Why it happens:** The code drops the tray/menu handles after setup.
**How to avoid:** Build the tray with a stable ID and use Tauri runtime menu APIs (`set_text`, `set_enabled`) on the existing menu item.
**Warning signs:** Recording state changes but tray label remains static.

## Code Examples

Verified patterns from official sources:

### Register a Shortcut at Runtime
```rust
use tauri_plugin_global_shortcut::GlobalShortcutExt;

fn register_shortcut(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut().on_shortcut("Ctrl+Shift+Space", |app, _shortcut, event| {
        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
            let _ = app.emit("voxflow://hotkey-pressed", ());
        }
    })?;
    Ok(())
}
```
Source: https://github.com/tauri-apps/tauri-plugin-global-shortcut/blob/v2/README.md

### Unregister a Shortcut at Runtime
```rust
use tauri_plugin_global_shortcut::GlobalShortcutExt;

fn unregister_old(app: &tauri::AppHandle, old_hotkey: &str) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut().unregister(old_hotkey)?;
    Ok(())
}
```
Source: https://docs.rs/crate/tauri-plugin-global-shortcut/latest

### Mutate Tray Menu Item State
```rust
fn set_tray_recording_label(item: &tauri::menu::MenuItem<tauri::Wry>, is_recording: bool) -> tauri::Result<()> {
    if is_recording {
        item.set_text("Stop Recording")?;
    } else {
        item.set_text("Start Recording")?;
    }
    item.set_enabled(true)?;
    Ok(())
}
```
Source: https://docs.rs/tauri/2.9.5/tauri/menu/struct.MenuItem

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 system-tray/menu event APIs | Tauri v2 `TrayIconBuilder::on_menu_event` / `on_tray_icon_event` | Tauri v2 | Phase 2 should extend current tray code, not follow v1 examples. |
| Custom OS hotkey APIs | `tauri-plugin-global-shortcut` | Current Tauri v2 ecosystem | Less platform-specific code and better future macOS portability. |
| Static startup-only shortcuts | Runtime `register` / `unregister` via `GlobalShortcutExt` | Current plugin API | Hotkey changes can take effect immediately on config save. |

**Deprecated/outdated:**
- Tauri v1 tray/menu examples: outdated for this repo’s Tauri v2 codebase.

## Open Questions

1. **What is the minimal warning surface for HOTK-04 before Phase 7 toasts exist?**
   - What we know: Backend can emit events and focus the Settings window now.
   - What's unclear: Whether Phase 2 should add a minimal banner/toast shell in the existing settings frontend or rely on a simpler temporary warning.
   - Recommendation: Plan one small frontend task for an event-driven warning surface, but keep it isolated from the full notifications phase.

2. **How visible should the placeholder `Transcribing` state be in Phase 2?**
   - What we know: The state must exist and hotkeys during it must be ignored.
   - What's unclear: Whether the placeholder completion should be immediate or briefly delayed for observability.
   - Recommendation: Implement a backend seam that can be immediate in production but remains unit-testable as a distinct transition.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in unit tests via `cargo test` |
| Config file | none |
| Quick run command | `cargo test hotkey -- --nocapture` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HOTK-01 | Default `Ctrl+Shift+Space` is normalized/registered and `Pressed` advances `Idle -> Recording` | unit + manual smoke | `cargo test hotkey::tests::default_hotkey_press_starts_recording -- --exact` | ❌ Wave 0 |
| HOTK-02 | Shared toggle path advances `Recording -> Transcribing` and ignores input during `Transcribing` | unit | `cargo test hotkey::tests::toggle_respects_state_machine -- --exact` | ❌ Wave 0 |
| HOTK-03 | Changing config hotkey re-registers successfully, persists canonical string, and survives reload | unit | `cargo test hotkey::tests::apply_hotkey_change_persists_canonical_value -- --exact` | ❌ Wave 0 |
| HOTK-04 | Registration failure keeps prior binding active and emits warning path | unit + manual smoke | `cargo test hotkey::tests::conflicting_hotkey_keeps_last_working_binding -- --exact` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test hotkey -- --nocapture`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green plus one Windows manual smoke test from another focused app

### Wave 0 Gaps
- [ ] `src-tauri/src/hotkey/mod.rs` — module to host pure logic and unit tests
- [ ] `src-tauri/src/hotkey/normalize.rs` — normalization/parser tests
- [ ] `src-tauri/src/hotkey/service.rs` — fake registrar tests for rebind/conflict paths
- [ ] `src-tauri/src/tray.rs` tests or helper seams — verify label/enabled state decisions
- [ ] Manual smoke script/checklist for “press hotkey from another app” on Windows

## Sources

### Primary (HIGH confidence)
- `/tauri-apps/tauri-plugin-global-shortcut` - registration, runtime unregister, shortcut handlers, shortcut format
- https://github.com/tauri-apps/tauri-plugin-global-shortcut/blob/v2/README.md - official plugin setup and handler examples
- https://docs.rs/crate/tauri-plugin-global-shortcut/latest - runtime registration API surface
- https://docs.rs/tauri/2.9.5/tauri/menu/struct.MenuItem - runtime menu item mutation (`set_text`, `set_enabled`)
- https://docs.rs/tauri/2.9.5/tauri/struct.AppHandle - tray lookup APIs

### Secondary (MEDIUM confidence)
- https://v2.tauri.app/start/migrate/from-tauri-1 - confirms Tauri v2 tray event model replacing older examples

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - official Tauri plugin and docs cover the needed API directly.
- Architecture: HIGH - recommendations are anchored in current repo structure plus official runtime APIs.
- Pitfalls: MEDIUM - mostly source-backed, with a few repo-specific inferences about future phase interaction.

**Research date:** 2026-03-15
**Valid until:** 2026-04-14
