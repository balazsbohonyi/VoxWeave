# Phase 7: Pipeline Integration - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Connect the existing hotkey → audio → transcription → injection pipeline into a seamless end-to-end user experience by adding proper toast notifications for every outcome. The pipeline itself works; this phase closes the feedback loop: success, error, and cancellation all produce user-visible toasts with correct auto-dismiss and indicator visibility behavior.

</domain>

<decisions>
## Implementation Decisions

### Success toast content
- Method label only — no text snippet: "Text pasted", "Text typed", "Copied to clipboard"
- `InjectionResult::Ok` currently has NO success toast (only a green flash then hide) — this is the primary gap
- `InjectionResult::CopiedToClipboard` already shows a toast but is classified as info; reclassify as success with same behavior

### Auto-dismiss rules
- **Success toasts**: auto-dismiss after **10 seconds** (NOTF-04 updated from 4s to 10s)
- **Cancellation toasts** ("Cancelled — N of M chars typed"): auto-dismiss after **10 seconds**
- **Error toasts**: NO auto-dismiss — manual dismiss only
- All toasts can still be manually dismissed earlier (existing close button behavior unchanged)

### Indicator visibility during toast
- **Success and cancellation outcomes**: indicator does NOT hide — stays visible throughout the toast duration
  - Brief green flash + "DONE" state fires as normal (~1s), then indicator returns to idle/neutral appearance
  - Toast appears above or below the indicator (separate window, no overlap with pill)
- **Error outcomes**: indicator IS hidden before toast shows — current behavior, keep unchanged
- **Post-toast indicator behavior**: after success/cancel toast auto-dismisses, use existing `show_on_startup` config field (already wired in `indicator/mod.rs:75`) to decide visibility
  - `show_on_startup=true` → indicator stays visible in idle state (persistent widget)
  - `show_on_startup=false` → indicator hides after toast clears

### Toast classification by outcome
| Outcome | Toast type | Auto-dismiss | Indicator hides? |
|---|---|---|---|
| `InjectionResult::Ok` (FlashPaste / Keystrokes) | success | 10s | No |
| `InjectionResult::CopiedToClipboard` (elevation fallback) | success | 10s | No |
| `InjectionResult::Cancelled` | info/neutral | 10s | No |
| `InjectionResult::Err` / all methods failed | error | Never | Yes (before toast) |
| Transcription errors (invalid_key, rate_limit, network) | error | Never | Yes (current behavior) |
| Transcription cancelled | silent (no toast) | — | Yes (current behavior) |

### Success toast method label mapping
- `InjectionMode::FlashPaste` → "Text pasted"
- `InjectionMode::Keystrokes` → "Text typed"
- `InjectionMode::Clipboard` → "Copied to clipboard"
- `InjectionResult::CopiedToClipboard` (elevation dialog fallback) → "Copied to clipboard"

### useToast auto-dismiss implementation
- Add `autoDismissMs?: number` to `ShowToastOptions` and `Toast` interface
- When a toast has `autoDismissMs`, set a `setTimeout` in the toast App.vue to call `handleDismissToast` after the delay
- Success/cancel toasts pass `autoDismissMs: 10000`; error toasts omit the field
- Manual dismiss still works during the countdown (clears the timer)

### Claude's Discretion
- Exact mechanism for hiding indicator after success toast clears (e.g., backend sleep + conditional hide vs. frontend invoke)
- Toast window positioning relative to indicator (above vs. below — whichever avoids going off-screen)
- Whether to clear the success-flash timeout race when indicator is not hiding (minor timing concern)

</decisions>

<specifics>
## Specific Ideas

- The green flash + "DONE" text that currently fires before hiding should be preserved — just no longer followed by an immediate hide
- The key gap in Phase 6: `InjectionResult::Ok` path only does `show_success()` + `hide()` with no toast at all — this must be added
- Cancellation treated symmetrically with success for indicator visibility (user chose to stop, not an error)

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `useToast.ts` — `ShowToastOptions` and `Toast` interfaces need `autoDismissMs?: number` added; otherwise reusable as-is
- `src/windows/toast/App.vue` — `handleDismissToast` + `__voxflowShowToast` already handles all payload types; add auto-dismiss timer here
- `indicator/mod.rs:show_success()` — green flash already implemented; keep it, just don't call `hide()` immediately after for success path
- `indicator/mod.rs:show_toast_window()` — reused for success toast; currently always hides indicator before showing (need to separate these)
- `src-tauri/src/config/mod.rs:202` — `show_on_startup: bool` exists with default `true`; already wired in `indicator/mod.rs:75`

### Established Patterns
- Toast payloads delivered via `window.__voxflowShowToast(payload)` eval from Rust — established in Phase 5, continue
- `tauri::async_runtime::spawn` for the injection task in `hotkey/service.rs` — success/cancel post-processing runs inside this spawn
- `indicator::hide()` called explicitly by Rust after each outcome — need to conditionally skip for success/cancel paths

### Integration Points
- `hotkey/service.rs:364–420` — the match on `InjectionResult` variants is where success/cancel toast calls go; currently `Ok` has no toast
- `hotkey/service.rs:274–290` — transcription error path already hides indicator, keep unchanged
- `src/windows/toast/App.vue:__voxflowShowToast` — entry point for all toast payloads; auto-dismiss logic goes here
- `src/types/index.ts` — may need a success toast payload type or reuse the existing PlainToastPayload `{type, message}` pattern

### Key gap identified
`InjectionResult::Ok` branch (line 365–373 in hotkey/service.rs) currently:
```rust
indicator::show_success(&app_for_inject);  // green flash
tokio::time::sleep(Duration::from_millis(1000)).await;
indicator::hide(&app_for_inject);           // hides — no toast!
```
Needs: keep green flash, keep the ~1s in success state, then transition to idle (not hide), then show success toast, then conditionally hide after 10s based on `show_on_startup`.

</code_context>

<deferred>
## Deferred Ideas

- None — discussion stayed within Phase 7 scope

</deferred>

---

*Phase: 07-pipeline-integration*
*Context gathered: 2026-03-22*
