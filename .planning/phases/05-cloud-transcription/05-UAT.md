---
status: resolved
phase: 05-cloud-transcription
source: 05-01-SUMMARY.md, 05-02-SUMMARY.md, 05-03-SUMMARY.md, 05-04-SUMMARY.md
started: 2026-03-17T14:30:00Z
updated: 2026-03-18T00:30:00Z
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
  status: resolved
  reason: "User reported: After the processing state I see the error 'Unexpected status 400: { \"error\": { \"message\": \"Invalid file...' in the indicator, then the pill returns to idle state."
  severity: major
  test: 1
  root_cause: "openai.rs sends multipart file with filename='audio.ogg' and MIME='audio/ogg'. OpenAI Whisper does not accept .ogg extension and returns 400 immediately. encode.rs format_for_provider routes OpenAI to EncodedFormat::Opus which produces an Ogg container."
  artifacts:
    - path: "src-tauri/src/audio/encode.rs"
      issue: "format_for_provider returns Opus for OpenAI; should return Wav"
    - path: "src-tauri/src/transcription/openai.rs"
      issue: "file_name('audio.ogg') and mime_str('audio/ogg') — not accepted by OpenAI Whisper"
  missing:
    - "Change format_for_provider to return Wav for TranscriptionProvider::Openai"
    - "Update openai.rs file_name to 'audio.wav' and mime_str to 'audio/wav'"
  debug_session: ".planning/debug/openai-audio-format-rejected.md"

- truth: "On invalid API key, indicator shows error toast with Open Settings button; Settings window does not open automatically"
  status: resolved
  reason: "User reported: no error toast appearing, but the Settings window still appears after stopping the recording"
  severity: major
  test: 2
  root_cause: "App.vue lines 172-176: the invalid_key branch calls invoke('open_settings_on_transcription_tab') directly and returns without calling showTranscriptionErrorToast. All other error codes route through showTranscriptionErrorToast correctly."
  artifacts:
    - path: "src/windows/indicator/App.vue"
      issue: "lines 172-176: invalid_key handler invokes open_settings_on_transcription_tab unconditionally instead of showing a toast with an Open Settings action button"
  missing:
    - "Change invalid_key branch to call showTranscriptionErrorToast with action.onClick invoking open_settings_on_transcription_tab"
  debug_session: ".planning/debug/invalid-key-opens-settings-no-toast.md"

- truth: "On retryable network error, indicator shows styled error toast with Retry button and friendly error message"
  status: resolved
  reason: "User reported: toast appears but unstyled, shows raw reqwest error string 'error sending request for url (https://api.openai.com/v1/audio/transcriptions)', no Retry button visible, toast disappears after a couple of seconds"
  severity: major
  test: 3
  root_cause: "Three bugs: (1) styles.css has no toast CSS classes (.indicator-toasts, .indicator-toast, .indicator-toast-message, .indicator-toast-action, .indicator-toast-dismiss) — all toast DOM nodes render unstyled; (2) service.rs Network match arms at lines 240 and 310 pass message.clone() verbatim, exposing raw reqwest error strings; (3) Retry button is in DOM but invisible due to missing .indicator-toast-action CSS (same as bug 1)."
  artifacts:
    - path: "src/styles.css"
      issue: "No toast CSS classes defined — all toast elements render unstyled"
    - path: "src-tauri/src/transcription/service.rs"
      issue: "lines 240 and 310: Network match arm passes raw reqwest error string as message instead of friendly text"
  missing:
    - "Add full toast CSS ruleset to src/styles.css (container, card, message, action button, dismiss button, error/info/success variants)"
    - "Replace message.clone() in both Network match arms in service.rs with a friendly string"
  debug_session: ".planning/debug/toast-unstyled-raw-message-no-retry.md"

- truth: "On fallback provider scenario, indicator shows error toast with Try with [Provider]? button; Settings window does not open automatically"
  status: resolved
  reason: "User reported: toast is not appearing at all, instead the Settings window appears"
  severity: major
  test: 4
  root_cause: "Same root cause as gap 2: invalid_key branch in App.vue auto-opens Settings before the fallback_provider path is evaluated. Fix to gap 2 (show toast with Open Settings button) will unblock the fallback path."
  artifacts:
    - path: "src/windows/indicator/App.vue"
      issue: "invalid_key handler fires first and opens Settings, preventing the fallback_provider toast from ever showing"
  missing:
    - "Fix invalid_key handler (same fix as gap 2) — fallback_provider toast logic is already correct"
  debug_session: ".planning/debug/invalid-key-opens-settings-no-toast.md"
