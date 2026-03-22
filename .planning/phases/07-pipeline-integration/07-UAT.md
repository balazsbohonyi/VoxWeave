---
status: complete
phase: 07-pipeline-integration
source: 07-01-SUMMARY.md, 07-02-SUMMARY.md, 07-03-SUMMARY.md
started: 2026-03-22T15:00:00Z
updated: 2026-03-22T15:00:00Z
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
  status: failed
  reason: "User reported: I only see the 'Canceled - x of y characters...' cancel toast"
  severity: major
  test: 3
  artifacts: []
  missing: []

- truth: "Pressing hotkey during the 10s post-injection toast window starts a new recording normally"
  status: failed
  reason: "User reported: cannot start a new recording, the indicator still shows IDLE after pressing the hotkey while the success toast from the previous indicator is still visible"
  severity: major
  test: 7
  artifacts: []
  missing: []
