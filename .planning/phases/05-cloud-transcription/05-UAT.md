---
status: resolved
phase: 05-cloud-transcription
source: 05-01-SUMMARY.md, 05-02-SUMMARY.md, 05-03-SUMMARY.md
started: 2026-03-17T12:00:00Z
updated: 2026-03-17T14:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. End-to-End Transcription
expected: Press hotkey → speak → press hotkey again. Indicator shows processing. Real transcribed text returned from cloud API (not a placeholder). transcription-done event fires with the spoken text.
result: issue
reported: "After setting up API keys in config.json, ellipsis dots appeared (processing state works). However, after pressing the hotkey for the second time to stop the recording, the Settings window appears unexpectedly."
severity: major

### 2. Invalid API Key Error Toast
expected: Set an invalid/wrong API key in config. Trigger a recording. After stopping, a toast appears in the floating indicator with an error message about the invalid key. The toast includes an "Open Settings" button. Clicking it opens the Settings window.
result: skipped
reason: Same Settings window bug from Test 1 blocks testing

### 3. Retry Button on Retryable Error
expected: When a retryable transcription error occurs (network issue, rate limit, server error), the error toast in the indicator shows a "Retry" button. Clicking Retry re-attempts transcription with the same recorded audio — no need to record again.
result: issue
reported: "I disconnected the network, and after recording, I saw the processing ellipsis dots, then for a couple of seconds the pill disappeared and after that reappeared with the IDLE state, and I saw no error toast with a Retry inside the indicator."
severity: major

### 4. Fallback Provider Button
expected: When the primary provider fails and a fallback provider is configured with a valid API key (e.g., primary=OpenAI fails, groq_api_key is set), the error toast shows a "Try with Groq?" button. Clicking it uses the fallback provider with the same audio.
result: skipped
reason: No error toast shown and Settings window appears — same underlying bugs as Tests 1 and 3

## Summary

total: 4
passed: 0
issues: 2
pending: 0
skipped: 2

## Gaps

- truth: "Stopping recording does not open the Settings window; recording stop only triggers transcription"
  status: resolved
  reason: "User reported: after pressing the hotkey for the second time to stop the recording, the Settings window appears unexpectedly."
  severity: major
  test: 1
  root_cause: "emit_hotkey_warning in src-tauri/src/hotkey/service.rs is called with focus_settings=true at lines 97 (startup) and 198 (save/re-register), causing show_settings_window to fire whenever hotkey registration fails. The Settings window opens at startup (or config save), and appears to the user as if triggered by hotkey stop when focus shifts to it."
  artifacts:
    - path: "src-tauri/src/hotkey/service.rs"
      issue: "emit_hotkey_warning called with focus_settings=true at lines 97 and 198; fix: change both to false"
  missing:
    - "Change focus_settings=true to focus_settings=false at line 97 (startup failure)"
    - "Change focus_settings=true to focus_settings=false at line 198 (save/re-register failure)"
  debug_session: ".planning/debug/settings-opens-on-hotkey-stop.md"

- truth: "On transcription error, indicator shows error toast with Retry button"
  status: resolved
  reason: "User reported: I disconnected the network, and after recording, I saw the processing ellipsis dots, then for a couple of seconds the pill disappeared and after that reappeared with the IDLE state, and I saw no error toast with a Retry inside the indicator."
  severity: major
  test: 3
  root_cause: "In src-tauri/src/hotkey/service.rs lines 274-276, the Err(()) match arm calls indicator::hide() synchronously immediately after transcribe_with_retry returns. The transcription-error event is emitted inside transcribe_with_retry but the JS event loop hasn't delivered it yet when the window is hidden. The toast renders in a hidden window and is never seen."
  artifacts:
    - path: "src-tauri/src/hotkey/service.rs"
      issue: "Err(()) branch calls indicator::hide() before JS delivers transcription-error event; fix: call indicator::show_idle() instead, let frontend hide indicator after toast dismissal"
  missing:
    - "Replace indicator::hide() with indicator::show_idle() in the Err(()) branch (hotkey/service.rs ~line 274)"
    - "Add hide_indicator Tauri command or auto-hide after toast duration so indicator closes after user acknowledges error"
  debug_session: ".planning/debug/transcription-error-toast-missing.md"
