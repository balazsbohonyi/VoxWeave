---
phase: 07-pipeline-integration
plan: "01"
subsystem: indicator, hotkey
tags: [feedback, toast, injection, ux]
dependency_graph:
  requires: []
  provides: [show_toast_window_keep_indicator, injection_success_label, injection_cancel_message]
  affects: [src-tauri/src/indicator/mod.rs, src-tauri/src/hotkey/service.rs]
tech_stack:
  added: []
  patterns: [show_toast_window_keep_indicator variant, RecordingState::Idle guard before deferred hide]
key_files:
  created: []
  modified:
    - src-tauri/src/indicator/mod.rs
    - src-tauri/src/hotkey/service.rs
decisions:
  - show_toast_window_keep_indicator is a thin variant of show_toast_window — identical positioning logic but no hide_indicator_window() calls
  - RecordingState::Idle guard before deferred hide prevents a 10s-delayed hide from interrupting a new recording started during the toast window
  - injection_success_label and injection_cancel_message extracted as free functions for unit testability without AppHandle
  - CopiedToClipboard now shows success toast (not info) and includes the 1s success flash before toast
metrics:
  duration: 214s
  completed_date: "2026-03-22"
  tasks_completed: 2
  files_modified: 2
requirements_satisfied:
  - NOTF-01
  - NOTF-02
  - NOTF-03
---

# Phase 7 Plan 1: Injection Pipeline Toast Feedback Summary

**One-liner:** Added `show_toast_window_keep_indicator` + revised match arms so InjectionResult::Ok/CopiedToClipboard show success toasts and Cancelled shows info toast, all with indicator staying visible and a 10s guarded hide.

## What Was Built

### Task 1: show_toast_window_keep_indicator (indicator/mod.rs)

Added `show_toast_window_keep_indicator` immediately after `show_toast_window`. The function is identical in positioning logic but does NOT call `hide_indicator_window()` at any point — neither in the no-monitor fallback branch nor at the end of the normal path. This allows success/cancel outcomes to show the toast adjacent to the indicator while keeping the indicator visible.

### Task 2: Revised InjectionResult match arms (hotkey/service.rs)

**InjectionResult::Ok:**
- show_success → sleep(1000ms) → show_idle → success toast with mode-derived label ("Text pasted" / "Text typed" / "Copied to clipboard") → sleep(10000ms) → RecordingState::Idle guard → hide indicator + hide toast window

**InjectionResult::CopiedToClipboard:**
- show_success → sleep(1000ms) → show_idle → success toast "Copied to clipboard" → sleep(10000ms) → RecordingState::Idle guard → hide indicator + hide toast window

**InjectionResult::Cancelled:**
- Indicator stays visible (no pre-toast hide) → info toast with message: total==0 → "Paste cancelled", total>0 → "Cancelled — N of M chars typed" → sleep(10000ms) → RecordingState::Idle guard → hide indicator + hide toast window

**InjectionResult::Err and Err(payload):** Unchanged — still call hide then show_toast_window (error paths correctly hide indicator).

## Helper Functions Added

- `injection_success_label(mode: &InjectionMode) -> &'static str` — maps InjectionMode to toast label
- `injection_cancel_message(typed: usize, total: usize) -> String` — formats cancel message string

## Tests Added

```
hotkey::service::tests::success_toast_label    PASS
hotkey::service::tests::cancel_toast_message   PASS
```

## Deviations from Plan

None — plan executed exactly as written.

## Pre-existing Test Failures (Out of Scope)

5 tests in `injection::service::tests` were failing before this plan and remain failing. They are unrelated to indicator/toast logic and were logged as out-of-scope:
- `elevation_check_triggers_only_when_target_il_greater`
- `elevation_copy_to_clipboard_returns_copied_to_clipboard_variant`
- `fallback_flashpaste_to_clipboard_when_flashpaste_fails`
- `fallback_keystroke_chain`
- `focus_restore_order`

## Commits

- `85ba87d` — feat(07-01): add show_toast_window_keep_indicator to indicator/mod.rs
- `fed3d46` — feat(07-01): revise InjectionResult match arms with success/cancel toast feedback

## Self-Check: PASSED

- src-tauri/src/indicator/mod.rs: FOUND
- src-tauri/src/hotkey/service.rs: FOUND
- commit 85ba87d: FOUND
- commit fed3d46: FOUND
