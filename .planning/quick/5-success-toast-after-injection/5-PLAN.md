---
phase: quick-5
plan: 5
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/hotkey/service.rs
autonomous: true
requirements: []

must_haves:
  truths:
    - "After FlashPaste injection succeeds, indicator hides with no toast"
    - "After Keystroke injection succeeds, indicator hides with no toast"
    - "After Clipboard injection succeeds, success toast is shown"
    - "Elevation-fallback CopiedToClipboard path still shows 'Copied to clipboard' toast"
  artifacts:
    - path: "src-tauri/src/hotkey/service.rs"
      provides: "injection result handling with mode-conditional toast"
  key_links:
    - from: "InjectionResult::Ok arm"
      to: "indicator::hide / indicator::show_toast_window"
      via: "mode check on injection_config.mode"
---

<objective>
Show success toast only when Clipboard mode is used. For FlashPaste and Keystroke, silently hide the indicator after successful injection.

Purpose: FlashPaste and Keystroke inject directly into the active window — the user sees the text appear immediately. A toast is redundant noise. Clipboard mode doesn't inject; the user needs to know the text is ready to paste.
Output: Modified hotkey/service.rs with mode-gated success toast.
</objective>

<context>
@src-tauri/src/hotkey/service.rs
@src-tauri/src/indicator/mod.rs
@src-tauri/src/injection/service.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Gate success toast on Clipboard mode in InjectionResult::Ok arm</name>
  <files>src-tauri/src/hotkey/service.rs</files>
  <action>
In the `InjectionResult::Ok` match arm (around line 408), replace the unconditional `show_toast_window` call with a mode check:

```rust
Ok(crate::injection::InjectionResult::Ok) => {
    if injection_config.mode == crate::config::InjectionMode::Clipboard {
        let label = injection_success_label(&injection_config.mode);
        let payload = serde_json::json!({
            "type": "success",
            "message": label
        });
        if let Err(e) = indicator::show_toast_window(
            &app_for_inject,
            &payload,
        ) {
            log::warn!("Failed to show success toast: {e}");
        }
        tokio::time::sleep(
            std::time::Duration::from_millis(10000),
        )
        .await;
        let is_idle = app_for_inject
            .try_state::<AppState>()
            .map(|s| {
                *s.recording_state.lock().unwrap()
                    == RecordingState::Idle
            })
            .unwrap_or(true);
        if is_idle {
            if let Some(tw) =
                app_for_inject.get_webview_window("toast")
            {
                let _ = tw.hide();
            }
        }
    } else {
        // FlashPaste / Keystroke: text already landed in the target window.
        // Just hide the indicator — no toast needed.
        indicator::hide(&app_for_inject);
    }
}
```

Do NOT change the `InjectionResult::CopiedToClipboard` arm — that elevation-fallback path always shows "Copied to clipboard" toast regardless of configured mode.

Also update the unit test `success_toast_label` (or add a new test) to assert that the label function returns correct strings. The label function itself does not change — only the call site is gated.
  </action>
  <verify>
    <automated>cd src-tauri && cargo test -- --test-thread=1 2>&1 | tail -20</automated>
  </verify>
  <done>
    - cargo test passes with no new failures
    - FlashPaste/Keystroke Ok arm calls indicator::hide (no show_toast_window)
    - Clipboard Ok arm calls show_toast_window with success payload
    - CopiedToClipboard arm unchanged
  </done>
</task>

</tasks>

<verification>
cargo test passes. Code review: grep for show_toast_window in the Ok arm confirms it is inside the `InjectionMode::Clipboard` branch only.
</verification>

<success_criteria>
- FlashPaste injection: indicator hides, no toast window appears
- Keystroke injection: indicator hides, no toast window appears
- Clipboard injection: success toast appears with "Copied to clipboard" message
- Elevation-fallback (CopiedToClipboard variant): toast still appears
- All existing cargo tests pass
</success_criteria>

<output>
After completion, update docs/TODO.md: mark C001 as [x].
</output>
