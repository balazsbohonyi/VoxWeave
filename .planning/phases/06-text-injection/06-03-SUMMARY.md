---
phase: 06-text-injection
plan: "03"
subsystem: injection
tags: [rust, injection, platform-abstraction, tdd, unit-tests]
dependency_graph:
  requires: [06-01, 06-02]
  provides: [inject_text, InjectionResult, InjectionErrorPayload, flashpaste, keystroke_inject, clipboard_only]
  affects: [hotkey-service, commands]
tech_stack:
  added: []
  patterns: [thread-local-mock-for-platform-dialog, free-fn-injection-modes, tdd-red-green]
key_files:
  created:
    - src-tauri/src/injection/mod.rs
    - src-tauri/src/injection/service.rs
  modified:
    - src-tauri/src/lib.rs
decisions:
  - "show_elevation_dialog uses thread-local MOCK_ELEVATION_DIALOG_RESULT in cfg(test) to avoid real MessageBoxW calls in tests"
  - "ElevationDialogResultMock defined at module level (not inside tests block) so show_elevation_dialog can reference it in cfg(test) branch"
  - "flashpaste, keystroke_inject, clipboard_only are pub free functions (not methods) matching plan spec"
  - "InjectionResult::CopiedToClipboard is a distinct variant (not Ok) so Plan 04 can match it for the correct toast"
metrics:
  duration_seconds: 366
  completed_date: "2026-03-21"
  tasks_completed: 2
  files_changed: 4
---

# Phase 06 Plan 03: Injection Module — Types, Modes, and Orchestration Summary

Implemented the `injection/` module with three injection mode functions (FlashPaste, Keystroke, Clipboard), the `inject_text()` orchestration function, elevation dialog with test-mockable thread-local, and 13 unit tests covering all INJC requirements.

## What Was Built

- `injection/mod.rs`: Module declaration + pub re-exports of `inject_text`, `InjectionResult`, `InjectionErrorCode`, `InjectionErrorPayload`
- `injection/service.rs`: Complete implementation of:
  - `InjectionResult` enum: `Ok`, `Cancelled { typed, total }`, `CopiedToClipboard`, `Err(String)`
  - `InjectionErrorCode` + `InjectionErrorPayload` (Serialize/Deserialize)
  - `flashpaste()`: save clipboard → write text → send_paste(class_name) → sleep(paste_delay_ms) → restore clipboard
  - `keystroke_inject()`: resets cancel_flag, sends each char via send_unicode_string or send_return for `\n`/`\r`, polls Escape via is_escape_pressed(), returns Cancelled{typed, total} on cancel
  - `clipboard_only()`: write_text only, no send_paste
  - `inject_text()` orchestration: restore_focus first (INJC-10), elevation check (Keystroke only), primary method, fallback chain (Keystroke→FlashPaste→Clipboard; FlashPaste→Clipboard)
  - `show_elevation_dialog()`: native MessageBoxW on Windows production; thread-local mock in cfg(test)

## Tests (13 total)

### Mode-function tests (Task 1)
1. `flashpaste_saves_and_restores_clipboard` — clipboard is restored after paste
2. `flashpaste_sends_ctrl_shift_v_for_terminal` — ConsoleWindowClass passed to send_paste
3. `flashpaste_sends_ctrl_v_for_normal` — Notepad passed to send_paste
4. `keystroke_inject_sends_all_chars` — all non-newline chars via send_unicode_string
5. `keystroke_inject_sends_return_for_newline` — `\n` chars via send_return
6. `keystroke_inject_cancel_returns_correct_counts` — cancel_flag reset + cancel path
7. `clipboard_only_does_not_call_send_paste` — write_text called, no InputSimulator needed

### Orchestration tests (Task 2)
8. `elevation_check_triggers_only_when_target_il_greater` — same IL = no dialog; higher IL + Cancel = Err(ElevationRequired)
9. `elevation_copy_to_clipboard_returns_copied_to_clipboard_variant` — CopyToClipboard dialog choice returns Ok(CopiedToClipboard), NOT Ok(Ok)
10. `fallback_flashpaste_to_clipboard_when_flashpaste_fails` — FlashPaste fails → auto_fallback → Clipboard succeeds
11. `fallback_keystroke_chain` — Keystroke fails → FlashPaste fails → Clipboard succeeds
12. `auto_fallback_false_no_fallback` — FlashPaste fails, auto_fallback=false → Err(AllMethodsFailed)
13. `focus_restore_order` — shared call log verifies `restore_focus` is index 0 before any send_paste/send_unicode_string

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Elevation dialog not mockable in tests**
- **Found during:** Task 2 — the `elevation_check_triggers_only_when_target_il_greater` test called the real Windows `MessageBoxW` dialog
- **Issue:** The plan specified `show_elevation_dialog()` as a free function using `#[cfg(target_os="windows")]`. On Windows in tests, this calls the real system dialog and either blocks or returns an unpredictable result, causing test failure.
- **Fix:** Added `ElevationDialogResultMock` enum and `MOCK_ELEVATION_DIALOG_RESULT` thread-local at module level (gated with `#[cfg(test)]`). The `show_elevation_dialog()` function checks the thread-local first in test builds; production builds call the real `MessageBoxW`. Helper functions `set_elevation_dialog_mock()` and `clear_elevation_dialog_mock()` provided for test setup/teardown.
- **Files modified:** `src-tauri/src/injection/service.rs`
- **Commit:** `00e5521`

**2. [Rule 1 - Bug] MockElevation::new signature simplified**
- **Found during:** Task 2 test setup — plan referenced `dialog_result` field on MockElevation, but MockElevation doesn't control dialog result (that's the thread-local's job)
- **Fix:** Removed `dialog_result` field from `MockElevation`; it now holds only `self_il` and `relaunch_called`. Tests set the dialog mock via `set_elevation_dialog_mock()`.
- **Files modified:** `src-tauri/src/injection/service.rs`
- **Commit:** `00e5521`

**3. [Rule 1 - Bug] MutexGuard comparison in test assertions**
- **Found during:** Task 1 compilation — `assert_eq!(unicode_calls, vec!["a", "b"])` where `unicode_calls` is a `MutexGuard` doesn't compile
- **Fix:** Dereferenced the guard with `*unicode_calls` and used `String` comparisons: `assert_eq!(*unicode_calls, vec!["a".to_string(), "b".to_string()])`
- **Files modified:** `src-tauri/src/injection/service.rs`
- **Commit:** `00e5521`

## Self-Check

### Files exist
- `src-tauri/src/injection/mod.rs`: EXISTS
- `src-tauri/src/injection/service.rs`: EXISTS

### Commits exist
- `00e5521`: EXISTS — feat(06-03): implement injection module

## Self-Check: PASSED
