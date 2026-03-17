---
status: diagnosed
phase: 05-cloud-transcription
source: 05-01-SUMMARY.md, 05-02-SUMMARY.md, 05-03-SUMMARY.md, 05-04-SUMMARY.md, 05-05-SUMMARY.md, 05-06-SUMMARY.md
started: 2026-03-18T01:10:00Z
updated: 2026-03-18T01:10:00Z
---

## Current Test

[testing complete]

## Tests

### 1. End-to-End Transcription
expected: Press hotkey → speak 3-5 words → press hotkey again. Indicator shows processing dots. Real transcribed text returned from cloud API. Settings window does NOT open at any point.
result: pass

### 2. Invalid API Key Error Toast
expected: Set an invalid API key in config (e.g. openai_api_key = "invalid"). Press hotkey, speak, stop. Indicator stays visible and shows a styled error toast with an "Open Settings" button. Clicking it opens the Settings window. Settings does NOT open automatically.
result: issue
reported: "The unstyled error is gone, but no toast is displayed either. Indicator cycles IDLE > REC > TRANSCRIBING > IDLE with no error toast shown at any point."
severity: major

### 3. Retry Button on Retryable Error
expected: Disconnect network. Press hotkey, speak, stop. After retries (~7+ seconds), indicator stays visible and shows a styled error toast with a "Retry" button and a friendly message (not a raw URL error). Clicking Retry re-attempts with the same audio.
result: issue
reported: "Stays longer in transcribing state (retries happening), then returns to IDLE. No error toast shown, no Retry button. No raw error displayed either."
severity: major

### 4. Fallback Provider Button
expected: Set primary provider with invalid key, fallback provider with valid key. Press hotkey, speak, stop. Error toast shows a "Try with [Provider]?" button. Clicking it retranscribes using the fallback provider.
result: skipped
reason: Same toast system failure as tests 2 and 3 — no toast renders for any error type

## Summary

total: 4
passed: 1
issues: 2
pending: 0
skipped: 1
skipped: 0

## Gaps

- truth: "On transcription error, indicator shows a styled error toast with the appropriate action button"
  status: failed
  reason: "User reported (tests 2 & 3): no toast displayed for either invalid_key or network errors. Indicator returns to IDLE silently. No raw error shown (CSS loading works), but showTranscriptionErrorToast produces no visible output."
  severity: major
  test: 2
  root_cause: "src/styles.css .indicator-toasts uses 'position: absolute; bottom: calc(100% + 8px)' — this places the toast container above the pill's top edge. The OS compositor clips anything outside the physical Tauri window rectangle (~200×48px). The toast IS in the DOM but occupies pixels the window does not own. Introduced in 05-06 when toast CSS was first added."
  artifacts:
    - path: "src/styles.css"
      issue: "bottom: calc(100% + 8px) positions toasts above the window boundary — OS clips them"
  missing:
    - "Reposition toast to render within the existing window bounds, OR dynamically resize the window when toasts are active"
  debug_session: ".planning/debug/toast-not-rendering-05-06.md"
