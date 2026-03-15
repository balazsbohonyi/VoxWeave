---
status: complete
phase: 03-audio-capture
source: [03-01-capture-lifecycle-SUMMARY.md, 03-02-encoding-contract-SUMMARY.md, 03-03-audio-device-settings-SUMMARY.md]
started: 2026-03-15T15:44:14.3420843+02:00
updated: 2026-03-15T16:09:49.0248713+02:00
---

## Current Test

[testing complete]

## Tests

### 1. Recording Toggle Lifecycle (Hotkey/Tray)
expected: Trigger recording, then stop recording (via hotkey or tray). The app transitions cleanly between idle and recording states without getting stuck, and returns to idle after stop.
result: issue
reported: "I cannot start recording, nit by using the hotkey, not by using the tray."
severity: major

### 2. Microphone Device List Loads In Settings
expected: Opening Audio settings shows available input devices, and Refresh updates the list without crashing or freezing the settings window.
result: issue
reported: "- it only shows the System default entry (microphone from the laptop), even if I also have a headset plugged in\n- if I click on Refresh the app doesn't freeze or crash, but nothing happens, the list is not refreshed, even after I plugged in another headset, or even if I restart the app"
severity: major

### 3. Selected Microphone Persists
expected: Selecting a microphone and saving persists after reopening settings or restarting the app; the same device remains selected.
result: pass

### 4. Missing Selected Device Falls Back With Warning
expected: If the saved microphone is unavailable, recording still starts using fallback device behavior and a visible audio warning appears in settings.
result: skipped
reason: cannot test this because cannot start recording

### 5. No Input Device Produces Clear Error Recovery
expected: With no input device available, recording does not proceed silently; a clear audio error/warning is surfaced and the app returns to idle.
result: skipped
reason: cannot test this because in Settings I have the single available System default option selected

### 6. Provider Switching Still Finalizes Recording
expected: Recording and stop/finalize completes when using cloud provider mode and local provider mode, without terminal audio-encoding failure.
result: skipped
reason: cannot test this aas I cannot start recording

## Summary

total: 6
passed: 1
issues: 2
pending: 0
skipped: 3

## Gaps

- truth: "Trigger recording, then stop recording (via hotkey or tray). The app transitions cleanly between idle and recording states without getting stuck, and returns to idle after stop."
  status: failed
  reason: "User reported: I cannot start recording, nit by using the hotkey, not by using the tray."
  severity: major
  test: 1
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""

- truth: "Opening Audio settings shows available input devices, and Refresh updates the list without crashing or freezing the settings window."
  status: failed
  reason: "User reported: - it only shows the System default entry (microphone from the laptop), even if I also have a headset plugged in\n- if I click on Refresh the app doesn't freeze or crash, but nothing happens, the list is not refreshed, even after I plugged in another headset, or even if I restart the app"
  severity: major
  test: 2
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""