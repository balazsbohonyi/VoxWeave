---
phase: quick-3
plan: 3
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/indicator/events.rs
  - src-tauri/src/indicator/mod.rs
  - src-tauri/src/hotkey/service.rs
  - src/windows/indicator/components/StateBadge.vue
  - src/types/index.ts
  - src/styles.css
  - docs/TODO.md
autonomous: true
requirements: []

must_haves:
  truths:
    - "Indicator never shows green border or DONE text after injection"
    - "After injection, indicator hides and toast appears without a 1-second delay"
    - "Rust compiles without errors (no references to removed Success variant)"
    - "TypeScript passes vue-tsc with no success literal in IndicatorVisualState"
  artifacts:
    - path: "src-tauri/src/indicator/events.rs"
      provides: "IndicatorVisualState without Success variant"
    - path: "src-tauri/src/indicator/mod.rs"
      provides: "No show_success fn; no show_toast_window_keep_indicator calls in success paths"
    - path: "src-tauri/src/hotkey/service.rs"
      provides: "Success injection arms call show_toast_window (hides indicator) without 1s sleep"
    - path: "src/types/index.ts"
      provides: "IndicatorVisualState type without 'success' literal"
    - path: "src/styles.css"
      provides: "No success CSS rules or @keyframes indicator-success-flash"
  key_links:
    - from: "src-tauri/src/hotkey/service.rs"
      to: "indicator::show_toast_window"
      via: "direct call replacing show_success + sleep + show_toast_window_keep_indicator"
---

<objective>
Remove all success state visualization from the indicator: the green flash, DONE badge, and the 1-second delay it introduces between injection and toast.

Purpose: R002 — the green success state was a distinct visual moment that is no longer desired.
Output: Indicator goes directly from Injecting to hidden when injection succeeds; toast appears immediately.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Remove Success variant from Rust backend</name>
  <files>
    src-tauri/src/indicator/events.rs
    src-tauri/src/indicator/mod.rs
    src-tauri/src/hotkey/service.rs
  </files>
  <action>
    **events.rs** — Remove the `Success` variant (and its doc comment) from `IndicatorVisualState`.

    **mod.rs** — Delete the `show_success()` function (lines 67-69). Keep `show_toast_window_keep_indicator` function intact (it may still be used for other purposes — verify callers first; if it has zero callers after this change, delete it too).

    **service.rs** — Update both injection success arms (InjectionResult::Ok ~line 408, InjectionResult::CopiedToClipboard ~line 449):

    Replace the existing block in each arm:
    ```rust
    indicator::show_success(&app_for_inject);
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    let _ = indicator::show_idle(&app_for_inject);
    // ...
    if let Err(e) = indicator::show_toast_window_keep_indicator(&app_for_inject, &payload) { ... }
    tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
    // is_idle check
    indicator::hide(&app_for_inject);
    // toast window hide
    ```

    With the simplified flow (no success flash, no 1s delay, indicator hides immediately with toast):
    ```rust
    let label = injection_success_label(&injection_config.mode); // or "Copied to clipboard"
    let payload = serde_json::json!({ "type": "success", "message": label });
    if let Err(e) = indicator::show_toast_window(&app_for_inject, &payload) {
        log::warn!("Failed to show success toast: {e}");
    }
    tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
    let is_idle = app_for_inject
        .try_state::<AppState>()
        .map(|s| *s.recording_state.lock().unwrap() == RecordingState::Idle)
        .unwrap_or(true);
    if is_idle {
        if let Some(tw) = app_for_inject.get_webview_window("toast") {
            let _ = tw.hide();
        }
    }
    ```

    Note: `show_toast_window` already calls `hide_indicator_window` internally, so no explicit `indicator::hide()` call is needed. The `indicator::show_idle()` and the 1000ms sleep are removed entirely.

    After changes, verify there are zero remaining references to `show_success` and `show_toast_window_keep_indicator` in the codebase. If `show_toast_window_keep_indicator` has no callers, remove it from mod.rs too.

    Run: `cd src-tauri && cargo check` to confirm no compile errors.
  </action>
  <verify>
    <automated>cd D:/develop/projects/VoxWeave/src-tauri && cargo check 2>&1</automated>
  </verify>
  <done>cargo check passes with zero errors; no references to Success variant, show_success, or show_toast_window_keep_indicator remain.</done>
</task>

<task type="auto">
  <name>Task 2: Remove success state from frontend and mark R002 complete</name>
  <files>
    src/types/index.ts
    src/windows/indicator/components/StateBadge.vue
    src/styles.css
    docs/TODO.md
  </files>
  <action>
    **src/types/index.ts** — Remove `| "success"` from the `IndicatorVisualState` union type (line 110).

    **StateBadge.vue** — Remove two lines:
    - The `if (props.state === "success") return "DONE";` branch in the `icon` computed (line 14).
    - The `const isSuccess = computed(() => props.state === "success");` line (line 18).
    - In the template, change `:class="{ 'indicator-badge--success': isSuccess }"` to remove the binding entirely: `class="indicator-badge"` (no dynamic class needed since isSuccess is gone).

    **styles.css** — Delete the three success selector blocks and the keyframe animation:
    - `.indicator-pill[data-state="success"] { ... }` (lines 63-66)
    - `.indicator-pill[data-state="success"] .indicator-record-dot { ... }` (lines 68-71)
    - `.indicator-pill[data-state="success"] .indicator-badge-icon { ... }` (lines 73-75)
    - `@keyframes indicator-success-flash { ... }` (lines 208-220)

    Do NOT remove `.indicator-toast--success` (line 291) — that is for toast window styling, unrelated to indicator success state.

    **docs/TODO.md** — Change `- [ ] R002 | P2: Remove the success state visualization (green border + green REC indicator)` to `- [x] R002 | P2: Remove the success state visualization (green border + green REC indicator)`.

    Run: `cd D:/develop/projects/VoxWeave && npx vue-tsc --noEmit` to confirm TypeScript passes.
  </action>
  <verify>
    <automated>cd D:/develop/projects/VoxWeave && npx vue-tsc --noEmit 2>&1</automated>
  </verify>
  <done>vue-tsc passes; no "success" literal in IndicatorVisualState type; no success CSS rules in styles.css; R002 marked [x] in TODO.md.</done>
</task>

</tasks>

<verification>
After both tasks:
1. `cd src-tauri && cargo check` — zero errors
2. `npx vue-tsc --noEmit` — zero errors
3. Grep confirms no remaining references: `grep -r "show_success\|Success\|success" src-tauri/src/indicator/` returns nothing relevant
4. TODO.md shows `[x] R002`
</verification>

<success_criteria>
- Indicator never enters a green "success" visual state
- Injection success → toast appears immediately (no 1-second delay)
- Indicator hides as soon as toast is shown (show_toast_window hides it internally)
- Rust and TypeScript compile without errors
- R002 marked complete in docs/TODO.md
</success_criteria>

<output>
After completion, create `.planning/quick/3-remove-success-state-visualization/3-SUMMARY.md`
</output>
