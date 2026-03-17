---
status: complete
phase: 05-cloud-transcription
source: 05-01-SUMMARY.md, 05-02-SUMMARY.md, 05-03-SUMMARY.md, 05-04-SUMMARY.md
started: 2026-03-17T14:30:00Z
updated: 2026-03-17T14:35:00Z
---

## Current Test

[testing complete]

## Tests

### 1. End-to-End Transcription
expected: Press hotkey → speak 3-5 words → press hotkey again. Indicator shows processing dots. Real transcribed text returned from cloud API. Settings window does NOT open at any point.
result: issue
reported: "After the processing state I see the error 'Unexpected status 400: { \"error\": { \"message\": \"Invalid file...' in the indicator, then the pill returns to idle state."
severity: major

### 2. Invalid API Key Error Toast
expected: Set an invalid API key in config (e.g. openai_api_key = "invalid"). Press hotkey, speak, stop. The indicator stays visible and shows an error toast. Toast includes an "Open Settings" button. Clicking "Open Settings" opens the Settings window. The indicator auto-hides after the toast dismisses.
result: issue
reported: "no error toast appearing, but the Settings window still appears after stopping the recording"
severity: major

### 3. Retry Button on Retryable Error
expected: Disconnect network (airplane mode or disable adapter). Press hotkey, speak, stop. After retry attempts complete (~7+ seconds), the indicator stays visible and shows an error toast with a "Retry" button. Clicking Retry re-attempts transcription using the same recorded audio — no re-recording needed.
result: issue
reported: "The toast appears but it's not styled, and it shows a raw error: 'error sending request for url (https://api.openai.com/v1/audio/transcriptions)'. No Retry button visible. The DOM shows <span class='indicator-toast-message'> with the raw error text. Toast disappears after a couple of seconds."
severity: major

### 4. Fallback Provider Button
expected: Configure primary provider with an invalid key, and a fallback provider with a valid key (e.g. openai_api_key = "invalid", groq_api_key = "valid-key", fallback_order starts with openai). Press hotkey, speak, stop. Error toast shows a "Try with Groq?" button. Clicking it retranscribes using the fallback provider.
result: issue
reported: "the toast is not appearing at all. instead the Settings window appears"
severity: major

## Summary

total: 4
passed: 0
issues: 4
pending: 0
skipped: 0

## Gaps

- truth: "Audio is successfully transcribed via OpenAI using the user's API key and returned as text"
  status: failed
  reason: "User reported: After the processing state I see the error 'Unexpected status 400: { \"error\": { \"message\": \"Invalid file...' in the indicator, then the pill returns to idle state."
  severity: major
  test: 1
  artifacts: []
  missing: []

- truth: "On invalid API key, indicator shows error toast with Open Settings button; Settings window does not open automatically"
  status: failed
  reason: "User reported: no error toast appearing, but the Settings window still appears after stopping the recording"
  severity: major
  test: 2
  artifacts: []
  missing: []

- truth: "On retryable network error, indicator shows styled error toast with Retry button and friendly error message"
  status: failed
  reason: "User reported: toast appears but unstyled, shows raw reqwest error string 'error sending request for url (https://api.openai.com/v1/audio/transcriptions)', no Retry button visible, toast disappears after a couple of seconds"
  severity: major
  test: 3
  artifacts: []
  missing: []

- truth: "On fallback provider scenario, indicator shows error toast with Try with [Provider]? button; Settings window does not open automatically"
  status: failed
  reason: "User reported: toast is not appearing at all, instead the Settings window appears"
  severity: major
  test: 4
  artifacts: []
  missing: []
