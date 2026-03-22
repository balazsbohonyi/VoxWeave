# Phase 7: Pipeline Integration - Research

**Researched:** 2026-03-22
**Domain:** Rust async state machine + Vue 3 composable / timer management
**Confidence:** HIGH

## Summary

Phase 7 is a focused integration phase. All the underlying pieces (hotkey, audio, transcription, injection, toast window, indicator) already exist. The work is entirely about wiring those pieces together correctly so every injection outcome produces the right user-visible feedback.

The technical scope is narrow: two Rust changes (add success toast, fix success/cancel indicator-hide behavior) and two TypeScript changes (add `autoDismissMs` to the toast composable, add auto-dismiss timer in App.vue). No new libraries, no new windows, no new Tauri commands beyond what Phase 6 left behind.

The primary risk is the indicator visibility timing: keeping the indicator visible through the 1-second success flash, then transitioning to idle (not hidden), and only hiding after the 10-second toast clears — all while honoring `show_on_startup`. Existing code already handles `show_on_startup` in `indicator::hide()`; the only change is calling `hide()` after a delay rather than immediately.

**Primary recommendation:** Surgical edits only. The pipeline is complete; this phase closes the feedback loop without restructuring anything.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Success toast content
- Method label only — no text snippet: "Text pasted", "Text typed", "Copied to clipboard"
- `InjectionResult::Ok` currently has NO success toast (only a green flash then hide) — this is the primary gap
- `InjectionResult::CopiedToClipboard` already shows a toast but is classified as info; reclassify as success with same behavior

#### Auto-dismiss rules
- **Success toasts**: auto-dismiss after **10 seconds** (NOTF-04 updated from 4s to 10s)
- **Cancellation toasts** ("Cancelled — N of M chars typed"): auto-dismiss after **10 seconds**
- **Error toasts**: NO auto-dismiss — manual dismiss only
- All toasts can still be manually dismissed earlier (existing close button behavior unchanged)

#### Indicator visibility during toast
- **Success and cancellation outcomes**: indicator does NOT hide — stays visible throughout the toast duration
  - Brief green flash + "DONE" state fires as normal (~1s), then indicator returns to idle/neutral appearance
  - Toast appears above or below the indicator (separate window, no overlap with pill)
- **Error outcomes**: indicator IS hidden before toast shows — current behavior, keep unchanged
- **Post-toast indicator behavior**: after success/cancel toast auto-dismisses, use existing `show_on_startup` config field (already wired in `indicator/mod.rs:75`) to decide visibility
  - `show_on_startup=true` → indicator stays visible in idle state (persistent widget)
  - `show_on_startup=false` → indicator hides after toast clears

#### Toast classification by outcome
| Outcome | Toast type | Auto-dismiss | Indicator hides? |
|---|---|---|---|
| `InjectionResult::Ok` (FlashPaste / Keystrokes) | success | 10s | No |
| `InjectionResult::CopiedToClipboard` (elevation fallback) | success | 10s | No |
| `InjectionResult::Cancelled` | info/neutral | 10s | No |
| `InjectionResult::Err` / all methods failed | error | Never | Yes (before toast) |
| Transcription errors (invalid_key, rate_limit, network) | error | Never | Yes (current behavior) |
| Transcription cancelled | silent (no toast) | — | Yes (current behavior) |

#### Success toast method label mapping
- `InjectionMode::FlashPaste` → "Text pasted"
- `InjectionMode::Keystrokes` → "Text typed"
- `InjectionMode::Clipboard` → "Copied to clipboard"
- `InjectionResult::CopiedToClipboard` (elevation dialog fallback) → "Copied to clipboard"

#### useToast auto-dismiss implementation
- Add `autoDismissMs?: number` to `ShowToastOptions` and `Toast` interface
- When a toast has `autoDismissMs`, set a `setTimeout` in the toast App.vue to call `handleDismissToast` after the delay
- Success/cancel toasts pass `autoDismissMs: 10000`; error toasts omit the field
- Manual dismiss still works during the countdown (clears the timer)

### Claude's Discretion
- Exact mechanism for hiding indicator after success toast clears (e.g., backend sleep + conditional hide vs. frontend invoke)
- Toast window positioning relative to indicator (above vs. below — whichever avoids going off-screen)
- Whether to clear the success-flash timeout race when indicator is not hiding (minor timing concern)

### Deferred Ideas (OUT OF SCOPE)
- None — discussion stayed within Phase 7 scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| NOTF-01 | Success toasts confirm injection method ("Text pasted", "Text typed", "Copied to clipboard") | `InjectionMode` enum already has the three variants; `injection_config.mode` is in scope at the result match site in `hotkey/service.rs:315-321` |
| NOTF-02 | Error toasts show actionable messages (open settings, retry, fallback) | All error paths already show actionable toasts; no new work needed — existing behavior satisfies this |
| NOTF-03 | Cancellation toasts show "X of Y characters typed" or "Paste cancelled" | Cancellation toast message already formatted by Rust; only the indicator-hide behavior and auto-dismiss need updating |
| NOTF-04 | Toasts auto-dismiss after 4s (updated to 10s per decision) and can be manually dismissed | Requires `autoDismissMs` field in `useToast.ts` + `setTimeout` in `App.vue`; no backend changes needed for timing |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Vue 3 `ref` / `watch` | (project) | Reactive toast list + timer refs | Already in use across all window composables |
| TypeScript `setTimeout` / `clearTimeout` | browser built-in | Auto-dismiss timers | Native; no dependency needed |
| Tokio `time::sleep` | (project) | Rust-side post-success delay before `hide()` | Already used in success path at `hotkey/service.rs:368-371` |
| `serde_json::json!` macro | (project) | Build success toast payload JSON inline | Already used for `CopiedToClipboard` toast at line 378 |

### No New Libraries Required
This phase adds zero new dependencies — all required capabilities exist in the current stack.

## Architecture Patterns

### Current Flow (gap identified)

```
InjectionResult::Ok match arm (hotkey/service.rs:365-373):
  show_success()          ← green flash  [KEEP]
  sleep(1000ms)           ← hold success state  [KEEP]
  hide()                  ← hides indicator  [CHANGE: don't hide yet]
                          ← no toast  [ADD: success toast + 10s then hide]
```

### Target Flow After Phase 7

```
InjectionResult::Ok:
  show_success()              ← green flash (unchanged)
  sleep(1000ms)               ← hold success state (unchanged)
  show_idle()                 ← transition to idle (not hide)
  show_toast_window_no_hide() ← show toast WITHOUT hiding indicator
  sleep(10000ms)              ← hold for auto-dismiss
  hide()                      ← now hide (respects show_on_startup)
```

OR the frontend-driven alternative (Claude's Discretion):

```
InjectionResult::Ok:
  show_success()
  sleep(1000ms)
  show_idle()
  show_toast_window_no_hide()
  [frontend setTimeout(10000) → invoke("hide_indicator")]
```

### Pattern 1: Decoupled Toast Show (No Auto-Hide Indicator)

The existing `show_toast_window()` in `indicator/mod.rs` always calls `hide_indicator_window()` at lines 129 and 146. For success/cancel paths, we need a variant that shows the toast WITHOUT hiding the indicator.

**Option A (Rust sleep):** Add `show_toast_window_keep_indicator()` that omits the `hide_indicator_window()` call, then the Rust match arm sleeps 10s and calls `hide()`.

**Option B (Frontend timer):** The existing `show_toast_window()` hides indicator. Instead, for success/cancel: call `show_idle()` first (indicator stays visible), call a new `show_toast_adjacent()` that only positions+shows the toast window without touching the indicator, and let the frontend auto-dismiss timer later invoke `hide_indicator` Tauri command.

**Recommended: Option A (Rust sleep)** — keeps timing logic in Rust, avoids adding a new Tauri command, consistent with how other paths work. The 10-second `tokio::time::sleep` is the same pattern already used for the 1-second success flash. The existing `hide()` function already handles `show_on_startup`.

### Pattern 2: Auto-Dismiss Timer in useToast

```typescript
// useToast.ts addition
export interface Toast {
  id: number;
  message: string;
  type: "success" | "error" | "info" | "warning";
  action?: ToastAction;
  autoDismissMs?: number;  // NEW
}

export interface ShowToastOptions {
  message: string;
  type?: Toast["type"];
  action?: ToastAction;
  autoDismissMs?: number;  // NEW
}
```

```typescript
// App.vue — in onMounted, after showToast() calls that need auto-dismiss:
// The timer must be tracked and cleared on manual dismiss.
// Pattern: store timers in a Map<id, ReturnType<typeof setTimeout>>

const dismissTimers = new Map<number, ReturnType<typeof setTimeout>>();

function scheduleAutoDismiss(id: number, ms: number) {
  const timer = setTimeout(() => {
    void handleDismissToast(id);
    dismissTimers.delete(id);
  }, ms);
  dismissTimers.set(id, timer);
}

// In handleDismissToast:
async function handleDismissToast(id: number): Promise<void> {
  const timer = dismissTimers.get(id);
  if (timer !== undefined) {
    clearTimeout(timer);
    dismissTimers.delete(id);
  }
  dismissToast(id);
  if (toasts.value.length === 0) {
    await invoke("hide_toast_window");
  }
}
```

### Pattern 3: Success Toast Payload Dispatch

The success toast is a plain `{type, message}` payload, matching the existing `PlainToastPayload` interface already handled by `isPlainToast()` in App.vue. No new interface needed on the frontend — just pass `{ type: "success", message: "Text pasted" }`.

In Rust:
```rust
// hotkey/service.rs — InjectionResult::Ok arm
let label = match injection_config.mode {
    InjectionMode::FlashPaste => "Text pasted",
    InjectionMode::Keystroke  => "Text typed",
    InjectionMode::Clipboard  => "Copied to clipboard",
};
let payload = serde_json::json!({ "type": "success", "message": label });
```

Note: `injection_config` is already cloned and in scope at the match site (line 315-321 of service.rs).

### Anti-Patterns to Avoid

- **Calling `hide()` immediately after success flash:** Current bug in `InjectionResult::Ok` arm. Must be changed to `show_idle()` + toast + deferred hide.
- **Holding `MutexGuard` across `await`:** Pattern well-established in this codebase — always clone config before spawn. Already done correctly; don't break this.
- **Using Tauri events instead of eval for toast delivery:** WebView2 may not deliver events to hidden windows. The `eval()` pattern is already established and must be preserved.
- **Forgetting to clear auto-dismiss timer on manual dismiss:** If user clicks X before 10s, the timer must be cleared to avoid double-dismiss.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Timer management | Custom polling loop | `setTimeout` / `clearTimeout` | Browser-native, zero overhead |
| Toast positioning | Custom geometry math | Existing `window::compute_toast_direction()` + `place_window()` | Already handles above/below, off-screen detection |
| Indicator post-toast visibility | New config field | Existing `show_on_startup` via `indicator::hide()` | Already reads `show_on_startup` at `mod.rs:75` |

## Common Pitfalls

### Pitfall 1: `show_toast_window` Always Hides Indicator
**What goes wrong:** Calling the existing `indicator::show_toast_window()` for success/cancel paths hides the indicator (line 129 and 146 in `indicator/mod.rs`).
**Why it happens:** The function was designed for error paths where indicator must hide first.
**How to avoid:** Either add a `keep_indicator` parameter or a separate `show_toast_adjacent()` function that skips the `hide_indicator_window()` calls.
**Warning signs:** Indicator disappears immediately on success — before the toast or after the 1s flash.

### Pitfall 2: `InjectionMode::Keystroke` vs `InjectionResult::Ok` After Fallback
**What goes wrong:** If the configured mode is `Keystroke` but injection fell back to `FlashPaste`, the `injection_config.mode` still reads `Keystroke` — the toast would say "Text typed" when text was actually pasted.
**Why it happens:** The fallback chain in `inject_text()` returns `InjectionResult::Ok` without indicating which method was actually used.
**How to avoid:** For Phase 7 scope, accept the label reflects the *configured* mode (consistent with what the user set). If accuracy is needed, `InjectionResult::Ok` would need to carry the actual mode used — defer to Phase 8+ unless required by NOTF-01.
**Assessment:** NOTF-01 says "confirm injection method" — the configured method is what the user understands. Acceptable for v1.

### Pitfall 3: Race Between 1s Success Flash and Toast Show
**What goes wrong:** The 1-second `sleep(1000ms)` in the success arm keeps the indicator in `Success` visual state. If toast appears while indicator is still in `Success` state (green), the combined visual is confusing.
**How to avoid:** Call `show_idle()` explicitly after the 1-second sleep, before showing the toast. This transitions indicator to neutral/idle appearance while keeping it visible.

### Pitfall 4: Auto-Dismiss Timer Not Cleared on Manual Dismiss
**What goes wrong:** User clicks X to dismiss toast. Timer fires 10 seconds later, calls `handleDismissToast` on a toast that no longer exists, may call `hide_toast_window` incorrectly.
**How to avoid:** Store timers in a `Map<id, timer>` and clear in `handleDismissToast` before removing from list. Check the pitfall in the pattern section above.

### Pitfall 5: `hide_toast_window` Rust Command Called When Toast Window Already Hidden
**What goes wrong:** `hide_toast_window` in `indicator/mod.rs` calls `show_idle()` which re-shows the indicator. If auto-dismiss fires and the window is already hidden, this incorrectly shows the indicator.
**How to avoid:** The frontend already only calls `hide_toast_window` when `toasts.value.length === 0` in `handleDismissToast`. Ensure this check is preserved in the auto-dismiss path too.

## Code Examples

### Rust: Revised InjectionResult::Ok Arm

```rust
// hotkey/service.rs — replace lines 365-373
Ok(crate::injection::InjectionResult::Ok) => {
    // Green success flash for ~1 second (INJC-08)
    indicator::show_success(&app_for_inject);
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    // Transition to idle (keep indicator visible for toast)
    let _ = indicator::show_idle(&app_for_inject);

    // Show success toast adjacent to indicator (no indicator hide)
    let label = match injection_config.mode {
        crate::config::InjectionMode::FlashPaste => "Text pasted",
        crate::config::InjectionMode::Keystroke  => "Text typed",
        crate::config::InjectionMode::Clipboard  => "Copied to clipboard",
    };
    let payload = serde_json::json!({ "type": "success", "message": label });
    if let Err(e) = indicator::show_toast_window_keep_indicator(&app_for_inject, &payload) {
        log::warn!("Failed to show success toast: {e}");
    }

    // Wait for auto-dismiss period, then hide indicator per show_on_startup
    tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
    indicator::hide(&app_for_inject);
    // Also hide the toast window (frontend may have already dismissed manually)
    if let Some(tw) = app_for_inject.get_webview_window("toast") {
        let _ = tw.hide();
    }
}
```

### Rust: Revised InjectionResult::CopiedToClipboard Arm

```rust
// Replace lines 374-387 — reclassify from "info" to "success", keep indicator
Ok(crate::injection::InjectionResult::CopiedToClipboard) => {
    indicator::show_success(&app_for_inject);
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    let _ = indicator::show_idle(&app_for_inject);

    let payload = serde_json::json!({ "type": "success", "message": "Copied to clipboard" });
    if let Err(e) = indicator::show_toast_window_keep_indicator(&app_for_inject, &payload) {
        log::warn!("Failed to show clipboard success toast: {e}");
    }
    tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
    indicator::hide(&app_for_inject);
    if let Some(tw) = app_for_inject.get_webview_window("toast") {
        let _ = tw.hide();
    }
}
```

### Rust: Revised Cancelled Arm

```rust
// Replace lines 388-407 — keep indicator visible, add 10s sleep before hide
Ok(crate::injection::InjectionResult::Cancelled { typed, total }) => {
    let message = if total == 0 {
        "Paste cancelled".to_string()
    } else {
        format!("Cancelled \u{2014} {typed} of {total} chars typed")
    };
    let payload = serde_json::json!({ "type": "info", "message": message });
    // Keep indicator visible — show toast adjacent without hiding indicator
    if let Err(e) = indicator::show_toast_window_keep_indicator(&app_for_inject, &payload) {
        log::warn!("Failed to show cancel toast: {e}");
    }
    tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
    indicator::hide(&app_for_inject);
    if let Some(tw) = app_for_inject.get_webview_window("toast") {
        let _ = tw.hide();
    }
}
```

### TypeScript: useToast.ts additions

```typescript
export interface Toast {
  id: number;
  message: string;
  type: "success" | "error" | "info" | "warning";
  action?: ToastAction;
  autoDismissMs?: number;  // NEW
}

export interface ShowToastOptions {
  message: string;
  type?: Toast["type"];
  action?: ToastAction;
  autoDismissMs?: number;  // NEW
}

// In showToast() — add autoDismissMs when building the toast object:
toasts.value.push({
  id,
  message: messageOrOptions.message,
  type: messageOrOptions.type ?? "info",
  action: messageOrOptions.action,
  autoDismissMs: messageOrOptions.autoDismissMs,  // NEW
});
```

### TypeScript: App.vue timer management

```typescript
// In <script setup> (add alongside existing const { toasts, showToast, dismissToast }):
const dismissTimers = new Map<number, ReturnType<typeof setTimeout>>();

async function handleDismissToast(id: number): Promise<void> {
  // Clear any pending auto-dismiss timer
  const timer = dismissTimers.get(id);
  if (timer !== undefined) {
    clearTimeout(timer);
    dismissTimers.delete(id);
  }
  dismissToast(id);
  if (toasts.value.length === 0) {
    await invoke("hide_toast_window");
  }
}

// Utility called after each showToast() that has autoDismissMs:
function scheduleAutoDismiss(id: number, ms: number): void {
  const timer = setTimeout(() => {
    void handleDismissToast(id);
  }, ms);
  dismissTimers.set(id, timer);
}
```

In the `__voxflowShowToast` handler, after `showToast()` for success/cancel paths:
```typescript
// After showToast for plain toasts that are success/info type:
const toastId = /* capture id returned from showToast — requires showToast to return id */
// OR: use toasts.value at index toasts.value.length - 1 after push
```

**Note:** `useToast.showToast()` currently does not return the new toast's `id`. It must be updated to return `id` so App.vue can register the auto-dismiss timer. Alternatively, read `toasts.value[toasts.value.length - 1].id` immediately after `showToast()`.

## State of the Art

| Old Approach | Current Approach | Notes |
|--------------|------------------|-------|
| `InjectionResult::Ok` hides indicator immediately after green flash | Phase 7: transition to idle, show success toast, hide after 10s | This is the primary behavioral change |
| `InjectionResult::CopiedToClipboard` shows info toast, hides indicator | Phase 7: success toast, keep indicator, hide after 10s | Reclassify from info to success |
| All toasts: no auto-dismiss | Phase 7: success/cancel auto-dismiss after 10s, errors never auto-dismiss | Asymmetric by design |

## Open Questions

1. **`showToast` return value**
   - What we know: `useToast.ts` does not currently return the new toast `id` from `showToast()`.
   - What's unclear: Whether App.vue should read `toasts.value.slice(-1)[0].id` after calling `showToast()`, or whether `showToast()` should be modified to return `id`.
   - Recommendation: Modify `showToast()` to return `id` — cleaner, avoids race if Vue batches reactivity. One-line change.

2. **"Paste cancelled" vs "N of M chars typed" for FlashPaste cancellation**
   - What we know: `InjectionResult::Cancelled { typed, total }` is only returned by the keystroke path. FlashPaste cannot be mid-paste cancelled (clipboard operations are atomic). `INJC-08` mentions "Paste cancelled" for non-keystroke modes.
   - What's unclear: Does the current code ever return `Cancelled` for FlashPaste? If not, the `total == 0` guard in the example above may be unreachable.
   - Recommendation: Check if `Cancelled` is only produced by keystroke injection. If so, always use "Cancelled — N of M chars typed" format and omit the "Paste cancelled" branch.

3. **Rust-side vs frontend-side auto-dismiss coordination**
   - What we know: The Rust approach (10s `tokio::time::sleep` then `indicator::hide()`) is simpler but means the frontend's `handleDismissToast` is unaware of the pending Rust-side hide.
   - What's unclear: If user manually dismisses the toast at 3s, the frontend hides the toast window and calls `hide_toast_window` (which triggers `show_idle`). But Rust is still sleeping for 7 more seconds and will then call `indicator::hide()`. This is benign if indicator was already in idle/hidden state, but could interrupt a new recording.
   - Recommendation: Check `RecordingState` before the Rust-side deferred hide: `if recording_state == Idle { hide() }`. This prevents the 10s-delayed hide from interrupting a recording that started during the toast window.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (built-in) + Vue TSC (`npx vue-tsc --noEmit`) |
| Config file | `src-tauri/Cargo.toml` (no separate test config) |
| Quick run command | `cd src-tauri && cargo test indicator hotkey injection 2>&1` |
| Full suite command | `cd src-tauri && cargo test && npx vue-tsc --noEmit` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| NOTF-01 | Success toast message matches injection mode | unit | `cd src-tauri && cargo test success_toast_label` | ❌ Wave 0 |
| NOTF-02 | Error toast paths unchanged — existing tests cover | unit | `cd src-tauri && cargo test injection` | ✅ |
| NOTF-03 | Cancellation toast message format correct | unit | `cd src-tauri && cargo test cancel_toast` | ❌ Wave 0 |
| NOTF-04 | `autoDismissMs` field present on success/cancel, absent on error | unit (TS) | `npx vue-tsc --noEmit` | ❌ Wave 0 (type check sufficient) |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test 2>&1 | tail -5`
- **Per wave merge:** `cd src-tauri && cargo test && npx vue-tsc --noEmit`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Unit test for success toast label mapping (NOTF-01) — add to `src-tauri/src/hotkey/` or `injection/service.rs` test module
- [ ] Unit test for cancel toast message format (NOTF-03) — verifies "Paste cancelled" vs "N of M chars typed" branching
- [ ] TypeScript type check passes with new `autoDismissMs` field — covered by `npx vue-tsc --noEmit`, no new test file needed

## Sources

### Primary (HIGH confidence)
- Direct code inspection: `src-tauri/src/indicator/mod.rs` — full implementation read
- Direct code inspection: `src-tauri/src/hotkey/service.rs:340-430` — injection result match arms
- Direct code inspection: `src/windows/toast/App.vue` — full toast dispatch logic
- Direct code inspection: `src/composables/useToast.ts` — full composable
- Direct code inspection: `src-tauri/src/injection/service.rs:1-45` — InjectionResult and InjectionMode types
- Direct code inspection: `src-tauri/src/config/mod.rs:1-35, 185-215` — InjectionMode enum and IndicatorConfig

### Secondary (MEDIUM confidence)
- `.planning/phases/07-pipeline-integration/07-CONTEXT.md` — user decisions (locked)
- `.planning/REQUIREMENTS.md` — NOTF-01 through NOTF-04 definitions

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new libraries, all capabilities verified in existing code
- Architecture: HIGH — all integration points directly inspected in source
- Pitfalls: HIGH — derived from actual code patterns observed (timer clearing, mutex guards, eval vs events)
- Auto-dismiss timing: HIGH — decision locked in CONTEXT.md (10s)

**Research date:** 2026-03-22
**Valid until:** 2026-04-22 (stable domain, no moving dependencies)
