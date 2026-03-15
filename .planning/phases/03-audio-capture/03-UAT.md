---
status: diagnosed
phase: 03-audio-capture
source: [03-01-capture-lifecycle-SUMMARY.md, 03-02-encoding-contract-SUMMARY.md, 03-03-audio-device-settings-SUMMARY.md]
started: 2026-03-15T15:44:14.3420843+02:00
updated: 2026-03-15T16:11:25.6393112+02:00
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
  root_cause: "Production audio device snapshot is still a stub that returns no devices, so start_recording always fails with NoInputDevice and the app immediately returns to idle."
  artifacts:
    - path: "src-tauri/src/audio/capture.rs"
      issue: "system_device_snapshot() returns empty devices/default outside tests"
    - path: "src-tauri/src/audio/mod.rs"
      issue: "start_recording_with_snapshot emits NoInputDevice and errors when snapshot is empty"
    - path: "src-tauri/src/hotkey/service.rs"
      issue: "toggle path resets to Idle on start failure, so tray/hotkey both appear unable to start"
  missing:
    - "Implement real input-device enumeration and default-device detection in system_device_snapshot()"
    - "Keep test-only mock snapshot behind cfg(test) while production path queries real devices"
  debug_session: ".planning/debug/phase-03-test-1-recording-cannot-start.md"

- truth: "Opening Audio settings shows available input devices, and Refresh updates the list without crashing or freezing the settings window."
  status: failed
  reason: "User reported: - it only shows the System default entry (microphone from the laptop), even if I also have a headset plugged in\n- if I click on Refresh the app doesn't freeze or crash, but nothing happens, the list is not refreshed, even after I plugged in another headset, or even if I restart the app"
  severity: major
  test: 2
  root_cause: "Device listing command is wired, but backend returns an empty device snapshot in production; UI shows only the static 'System default' option, so refresh cannot show real devices."
  artifacts:
    - path: "src-tauri/src/audio/capture.rs"
      issue: "system_device_snapshot() does not enumerate hardware devices"
    - path: "src-tauri/src/commands/audio.rs"
      issue: "list_audio_input_devices forwards backend list, which is always empty"
    - path: "src/windows/settings/App.vue"
      issue: "UI always renders static System default option even when backend list is empty"
  missing:
    - "Return real input device names/default from backend enumeration"
    - "Show explicit empty/error state when backend returns no devices so refresh behavior is observable"
  debug_session: ".planning/debug/phase-03-test-2-device-list-not-refreshing.md"