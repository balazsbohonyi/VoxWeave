---
status: resolved
phase: 07-pipeline-integration
source: 07-01-SUMMARY.md, 07-02-SUMMARY.md, 07-03-SUMMARY.md
started: 2026-03-22T15:00:00Z
updated: 2026-03-22T18:00:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

## Current Test

[testing complete]

## Tests

### 1. Injection Success Toast
expected: Complete a dictation (press hotkey, speak, press hotkey again). Text gets injected into the active window. After injection, the floating indicator should remain visible (NOT disappear) and a green success toast should appear next to it showing "Text pasted" (for FlashPaste/clipboard mode) or "Text typed" (for keystroke mode). After ~10 seconds, both the indicator and the toast window should automatically disappear.
result: pass

### 2. Clipboard Fallback Toast
expected: Trigger a scenario where injection falls back to clipboard (e.g., configure injection mode to "Copy to clipboard" in settings). After dictation, a green success toast should appear showing "Copied to clipboard" while the indicator stays visible. Both should auto-dismiss after ~10 seconds.
result: pass

### 3. Cancel Toast — No Chars Typed
expected: Start a recording (press hotkey, speak briefly), then cancel before any keystroke injection begins (cancel button or cancel hotkey). A blue/info toast should appear showing "Paste cancelled". The indicator should remain visible. The toast auto-dismisses after ~10 seconds.
result: issue
reported: "I only see the 'Canceled - x of y characters...' cancel toast"
severity: major

### 4. Cancel Toast — Partial Injection
expected: Start a recording with keystroke injection mode active, let injection begin typing, then cancel mid-injection. A blue/info toast should appear showing "Cancelled — N of M chars typed" (where N is chars typed and M is total). The indicator stays visible and auto-dismisses after ~10 seconds.
result: pass

### 5. Error Toast Persists Until Dismissed
expected: Trigger an injection error (e.g., target window closes between recording and injection). An error toast should appear. Unlike success/cancel toasts, this error toast should NOT auto-dismiss — it should remain visible indefinitely until the user clicks X. After clicking X, it disappears.
result: pass

### 6. Manual Dismiss Cancels Auto-Dismiss Timer
expected: Complete a successful dictation so a success toast appears. Before the 10-second auto-dismiss fires, click the X button on the toast. The toast should disappear immediately. No second dismissal or flicker should occur after the 10s point where the timer would have fired.
result: pass

### 7. New Recording During Toast Window
expected: Complete a dictation so a success toast appears (10s auto-dismiss window). Immediately press the hotkey again to start a new recording within those 10 seconds. The new recording should start normally (indicator goes to recording state). When the 10s timer from the first recording fires, it should NOT hide the indicator (the new recording is still active).
result: issue
reported: "cannot start a new recording, the indicator still shows IDLE after pressing the hotkey while the success toast from the previous indicator is still visible"
severity: major

## Summary

total: 7
passed: 5
issues: 2
pending: 0
skipped: 0

## Gaps

- truth: "When cancel fires before any chars are typed, toast shows 'Paste cancelled'"
  status: resolved
  reason: "User reported: I only see the 'Canceled - x of y characters...' cancel toast"
  severity: major
  test: 3
  root_cause: "injection_cancel_message() checks `total == 0` but total is always > 0 (it's the full text length). Should check `typed == 0` instead. The 'Paste cancelled' branch is unreachable in practice."
  artifacts:
    - path: "src-tauri/src/hotkey/service.rs"
      issue: "Line ~544: condition `total == 0` should be `typed == 0` in injection_cancel_message()"
    - path: "src-tauri/src/hotkey/service.rs"
      issue: "Lines ~598-602: unit test asserts buggy behavior (0, 10) → count format; needs updating"
  missing:
    - "Change `if total == 0` to `if typed == 0` in injection_cancel_message()"
    - "Update unit test to expect 'Paste cancelled' for (typed=0, total=10)"

- truth: "Pressing hotkey during the 10s post-injection toast window starts a new recording normally"
  status: resolved
  reason: "User reported: cannot start a new recording, the indicator still shows IDLE after pressing the hotkey while the success toast from the previous indicator is still visible"
  severity: major
  root_cause: "RecordingState is not reset to Idle until after both sleeps (1s + 10s) complete. During the 10s toast window, state is still Transcribing. The hotkey handler treats hotkey presses in Transcribing state as cancel requests (sets cancel_flag=true, returns). The stale cancel_flag=true then aborts the next real recording attempt too."
  artifacts:
    - path: "src-tauri/src/hotkey/service.rs"
      issue: "Lines ~226-231: Transcribing guard intercepts hotkey as cancel instead of new recording"
    - path: "src-tauri/src/hotkey/service.rs"
      issue: "Lines ~500-501: RecordingState reset to Idle placed after 10s sleep, should be before toast sequence"
    - path: "src-tauri/src/hotkey/service.rs"
      issue: "cancel_flag is not reset at start of new Idle→Recording transition, stale true aborts next transcription"
  missing:
    - "Reset RecordingState to Idle immediately after injection completes, before the 1s+10s toast sleeps"
    - "Reset cancel_flag to false at the start of each new Idle→Recording transition"
    - "Remove or guard the post-sleep state reset so it doesn't overwrite a Recording state already set by a new session"
