# Quick Task 2: Remove Injection Cancellation Support

**Date:** 2026-04-02
**Status:** Complete

## What was done

Removed all injection cancellation support (R005):

- **ESC-based cancellation**: Removed `is_escape_pressed()` function and its polling inside `keystroke_inject()`
- **Hotkey-based cancellation**: Removed `cancel_flag` parameter from `keystroke_inject()`, `run_method()`, and `inject_text()` — injection can no longer be cancelled mid-flight
- **`InjectionResult::Cancelled`**: Removed variant from enum; removed all match arms
- **`InjectionErrorCode::Cancelled`**: Removed from enum
- **`typed_chars`/`total_chars`**: Removed fields from `InjectionErrorPayload` (only existed for cancel reporting)
- **`injection_cancel_message()`**: Removed function and its unit tests
- **`show_idle_visual()`**: Removed dead function from indicator (only used in Cancelled arm)
- **Frontend**: Removed `"cancelled"` from `InjectionErrorCode` type, removed `typed_chars`/`total_chars` from `InjectionErrorPayload`, removed `isInjectionPayload()` typed_chars heuristic, removed cancelled toast branch in App.vue
- **TODO.md**: Marked R005 as complete

## Files changed

- `src-tauri/src/injection/service.rs`
- `src-tauri/src/hotkey/service.rs`
- `src-tauri/src/indicator/mod.rs`
- `src/windows/toast/App.vue`
- `src/types/index.ts`
- `docs/TODO.md`

## Verification

- `cargo clippy`: 14 warnings (all pre-existing, none new)
- `cargo test`: 101 passed, 0 failed
- `npx vue-tsc --noEmit`: clean
