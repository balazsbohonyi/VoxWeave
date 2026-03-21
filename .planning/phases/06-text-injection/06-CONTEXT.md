# Phase 6: Text Injection - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Text returned from transcription is injected into the previously-focused window using the configured method (FlashPaste, Keystroke, or Clipboard). Includes elevation checks, fallback chain, Escape-cancel for keystroke mode, and a success flash on the indicator.

</domain>

<decisions>
## Implementation Decisions

### Injection methods
- FlashPaste is default (already locked from prior phases)
- Fallback chain if Keystrokes selected: Keystrokes → FlashPaste → Clipboard
- Fallback chain if FlashPaste selected: FlashPaste → Clipboard
- Auto-fallback is configurable (INJC-09 toggle) — include the config field now, Phase 8 exposes UI

### Elevation check
- Only triggered in Keystroke mode (FlashPaste clipboard operations work across integrity levels; SendInput silently fails on elevated targets)
- Dialog: native Windows dialog (MessageBox or TaskDialog) — not a custom overlay
- "Copy to clipboard" path: copy text + show toast confirming "Copied to clipboard — paste manually"
- "Relaunch as Admin": implement per spec using ShellExecuteW "runas"

### Success feedback (Phase 6 minimal UX)
- After successful injection: indicator flashes a brief green/checkmark state (~1 second) then hides
- No full success toast in Phase 6 — that's Phase 7's job
- Indicator stays visible throughout the entire injection (including during character-by-character keystroke mode)
- Injection errors (all fallbacks exhausted): reuse existing toast window with a new InjectionErrorPayload type (same pattern as TranscriptionErrorPayload)

### FlashPaste timing
- Hardcode 500ms between paste and clipboard restore
- Fixed delay (no event detection)
- Phase 8 Settings UI will expose paste_delay_ms as a configurable field — add it to InjectionConfig now even without UI

### Keystroke cancellation
- Escape key only cancels injection (not the recording hotkey — hotkey during injection is ignored)
- Toast on cancel: "Cancelled — 47 of 230 chars typed" (show actual char counts)
- Partial text already typed into the target app: leave as-is, no cleanup
- Escape detection approach: Claude's discretion (GetAsyncKeyState polling between char sends is simplest; low-level hook is overkill)

### Foreground window capture
- Must capture the foreground window at recording START (before indicator shows), not at injection time
- Add `foreground_window: Arc<Mutex<Option<ForegroundWindowInfo>>>` to AppState
- Focus must be restored to the captured window before injection (INJC-10)

### Claude's Discretion
- Escape detection: polling vs hook — pick the simpler approach (polling recommended)
- Exact green flash color/animation for the success state in the indicator
- InjectionErrorPayload field naming

</decisions>

<specifics>
## Specific Ideas

- Prior discussion confirmed keystroke injection is feasible (Whisper Flow uses it); FlashPaste is preferred for speed
- Toast format confirmed: "Cancelled — 47 of 230 chars typed" (not just "Cancelled")
- Native Windows dialog for elevation check — consistent with OS style

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `platform/mod.rs` — InputSimulator, ClipboardAccess, WindowInfo, ElevationChecker traits fully defined; Windows stubs ready to implement
- `src/windows/toast/App.vue` — existing toast window receives payloads via `window.__voxflowShowToast(payload)`; add InjectionErrorPayload type alongside TranscriptionErrorPayload
- `state.rs` — `cancel_flag: Arc<Mutex<bool>>` already exists; reuse for keystroke cancellation
- `config/mod.rs` — InjectionMode enum already defined (FlashPaste/Keystroke/Clipboard); extend InjectionConfig with `keystroke_speed`, `auto_fallback`, `paste_delay_ms`

### Established Patterns
- Error/fallback pattern from `transcription/service.rs` — emit error event with optional fallback, frontend handles toast
- `tauri::async_runtime::spawn` (not tokio::spawn) for async injection task
- Toast payloads delivered via eval() not events (already working for transcription errors)

### Integration Points
- `hotkey/service.rs:283–290` — explicit "Phase 6 will handle injection here" comment; add foreground window capture at recording start in the same function
- `types/index.ts` — InjectionDonePayload already defined (line 101–104); extend with InjectionErrorPayload
- Indicator visual state `Injecting` already defined in `indicator/events.rs`; hide → green flash → hide sequence goes here

### Missing dependencies (add to Cargo.toml)
- `windows` crate — GetForegroundWindow, SendInput, integrity level queries, ShellExecuteW
- `arboard` — direct clipboard access (already specified in CLAUDE.md as non-negotiable)

</code_context>

<deferred>
## Deferred Ideas

- None — discussion stayed within Phase 6 scope

</deferred>

---

*Phase: 06-text-injection*
*Context gathered: 2026-03-21*
