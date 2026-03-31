# Phase 2: Hotkey - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver global hotkey registration and a toggle-mode recording trigger that changes VoxWeave’s recording state machine from any focused application. This includes conflict detection and immediate application of config hotkey changes. Audio capture, indicator UI, transcription, and injection remain out of scope.

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<specifics>
## Specific Ideas

- No specific UI references; focus is on reliable global toggle behavior.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src-tauri/src/config/mod.rs` already defines `AppConfig.hotkey` with a default (currently `Alt+Shift+Space`) and persistence via `config::persistence`.
- `src-tauri/src/state.rs` already defines `RecordingState` with `Idle`, `Recording`, `Transcribing`.
- Tray menu item `start_stop_recording` exists in `src-tauri/src/tray.rs` but is disabled.

### Established Patterns
- Config changes are saved via `commands::config::save_config` and stored in managed `AppState`.
- Tray-first app lifecycle; settings window can be shown programmatically.

### Integration Points
- Hotkey registration should be initialized in `src-tauri/src/lib.rs` during app setup.
- Config save handler is the hook point for hotkey re-registration.
- Tray menu handler should dispatch the same state toggle as the hotkey.

</code_context>

<deferred>
## Deferred Ideas

- Hotkey capture UI and conflict prompts inside settings (Phase 8).
- Cancel behavior during processing (Phase 7+), beyond “ignore hotkey while Transcribing.”

</deferred>

---

*Phase: 02-hotkey*
*Context gathered: 2026-03-15*
