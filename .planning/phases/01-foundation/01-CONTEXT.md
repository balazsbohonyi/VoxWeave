# Phase 1: Foundation - Context

**Gathered:** 2026-03-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the initial Windows Tauri application scaffold for VoxWeave: tray-first app startup, a single reusable settings window that hides to tray on close, config persistence in `%APPDATA%/VoxWeave/config.json`, and the platform abstraction seam that later phases will build on. Recording, the floating indicator, transcription, and injection behavior remain outside this phase.

</domain>

<decisions>
## Implementation Decisions

### Window strategy
- Phase 1 creates only the settings window; the floating indicator window is deferred to Phase 4.
- Normal app startup is tray-only with no visible window.
- Tray actions should open or focus a single reusable settings window instance rather than creating duplicates.
- Closing the settings window always hides it to tray; quitting remains a tray action.

### Config shape
- Define the full v1 `AppConfig` schema in Phase 1 instead of a temporary phase-only subset.
- Future-phase settings should receive concrete default values now so later code can rely on stable reads.
- Unknown JSON fields must survive load/save round-trips to preserve forward/backward compatibility.
- Persist configuration as a single file at `%APPDATA%/VoxWeave/config.json`; do not split config into multiple files yet.

### Tray behavior
- The tray menu should already expose the final Phase 1 menu shape: `Settings`, `Start/Stop Recording`, separator, `Quit`.
- `Start/Stop Recording` should be visible but disabled in Phase 1, since recording is implemented in Phase 2.
- Double-clicking the tray icon should open or focus the settings window.
- Phase 1 only needs an idle tray icon; recording-state icon switching can wait until recording state exists.
- Choosing `Settings` from the tray should always focus and restore the existing settings window if it is already open.

### Claude's Discretion
- Exact naming of config sections and field identifiers, as long as they map cleanly to the full v1 feature set.
- Whether the settings window is created lazily on first open or created once and then reused, as long as the user sees one reusable instance.
- Exact tray implementation details for disabled menu state and focus restoration.

</decisions>

<specifics>
## Specific Ideas

- The product should already feel tray-first in Phase 1, not like a normal desktop app that merely happens to have a tray icon.
- The foundation should establish long-lived patterns now because the repository does not yet contain an existing app scaffold to inherit from.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- No application scaffold exists yet under `src/` or `src-tauri/`; Phase 1 will establish the initial reusable structure.
- Planning documents in `.planning/` already define the intended backend/frontend folder layout and the two-window product model.

### Established Patterns
- Project-level docs lock in Tauri v2 + Rust + Vue 3 + TypeScript + Tailwind as the stack.
- Platform abstraction from day one is non-negotiable; Phase 1 should introduce the trait boundary rather than embedding Windows calls directly into app logic.
- Config persistence must support defaults for missing fields and preserve unknown fields on save.

### Integration Points
- `src-tauri/src/main.rs` should become the root for tray setup, window lifecycle wiring, and managed state registration.
- `src-tauri/src/config/` should own config schema and disk persistence.
- `src-tauri/src/platform/` should define the abstraction seam used by later Windows implementations.
- `src/windows/settings/` should become the only real frontend window in this phase, even if its UI is minimal.

</code_context>

<deferred>
## Deferred Ideas

- Pre-creating the floating indicator window in Phase 1 was considered but deferred to Phase 4 with the rest of indicator behavior.
- Recording behavior behind the tray menu is intentionally deferred to Phase 2; Phase 1 only exposes the menu shape and disabled state.

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-03-14*
