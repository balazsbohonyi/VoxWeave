---
status: diagnosed
trigger: "On retryable network error, toast appears but is completely unstyled, message is raw reqwest error string, no Retry button"
created: 2026-03-18T00:00:00Z
updated: 2026-03-18T00:00:00Z
---

## Current Focus

hypothesis: Three independent bugs — missing CSS rules, raw Network message passthrough, Retry button invisible due to CSS absence.
test: static code analysis complete
next_action: return diagnosis to caller

## Symptoms

expected: styled error toast with friendly message and Retry button for retryable network errors
actual: unstyled span with raw reqwest error string, no Retry button visible
errors: DOM shows `<span class="indicator-toast-message">error sending request for url (...)</span>`
reproduction: trigger a network-level transcription failure (disconnect network, attempt transcription)
started: always (never worked)

## Eliminated

- hypothesis: useToast action field not populated when retryable=true
  evidence: App.vue lines 196-205 correctly sets action:{label:"Retry",onClick:...} when payload.retryable===true AND payload.fallback_provider is falsy. Toast object structure is correct.
  timestamp: 2026-03-18

- hypothesis: toast template not rendering action button
  evidence: App.vue lines 278-285 correctly gates the action button on v-if="toast.action". Template logic is correct.
  timestamp: 2026-03-18

## Evidence

- timestamp: 2026-03-18
  checked: src/styles.css — full file, 197 lines
  found: Classes .indicator-toasts, .indicator-toast, .indicator-toast--error, .indicator-toast--info, .indicator-toast--success, .indicator-toast-message, .indicator-toast-action, .indicator-toast-dismiss are entirely absent. File ends at line 197 with only pill/waveform/record-button/animation rules.
  implication: BUG 1 — Toast DOM nodes exist but have zero visual styling. Browser default styles apply (transparent background, zero padding, inline text flow). This is why the toast looks like a raw unstyled span.

- timestamp: 2026-03-18
  checked: src-tauri/src/transcription/openai.rs lines 71-80
  found: reqwest .send().await error mapped with `message: e.to_string()`. reqwest Display for a connection-level failure produces the string "error sending request for url (https://api.openai.com/v1/audio/transcriptions): ..."
  implication: BUG 2 — Raw reqwest internal error string stored in TranscriptionError::Network { message }.

- timestamp: 2026-03-18
  checked: src-tauri/src/transcription/service.rs line 240
  found: `TranscriptionError::Network { message } => (TranscriptionErrorCode::Network, message.clone())` — no transformation, raw string emitted verbatim into TranscriptionErrorPayload.message.
  implication: Confirms BUG 2. Same pattern is also present in transcribe_with_provider (service.rs line 310) for the fallback path.

- timestamp: 2026-03-18
  checked: App.vue lines 184-208 (toast dispatch logic)
  found: Branch order is: (1) fallback_provider truthy -> "Try with X?" button; (2) retryable===true -> "Retry" button; (3) else -> no button. When no fallback provider is configured, retryable path fires and action IS added to the toast object. Button IS rendered by template (lines 278-285).
  implication: BUG 3 is CSS-only. The Retry button is added to the toast object and rendered in DOM but invisible because .indicator-toast-action has no CSS.

## Resolution

root_cause: |
  BUG 1 — Missing CSS (primary cause of "unstyled" appearance):
  src/styles.css has zero rules for any toast class. These classes are referenced
  in App.vue but never defined:
    .indicator-toasts         (container div, lines 270)
    .indicator-toast          (per-toast row div, line 272)
    .indicator-toast--error   (error type modifier, line 275)
    .indicator-toast--info    (info type modifier)
    .indicator-toast--success (success type modifier)
    .indicator-toast-message  (text span, line 277)
    .indicator-toast-action   (Retry/fallback button, line 281)
    .indicator-toast-dismiss  (dismiss x button, line 287)
  Fix file: src/styles.css — add toast layout and colour rules.

  BUG 2 — Raw reqwest error string exposed to user (Rust service.rs):
  All three provider impls construct TranscriptionError::Network { message: e.to_string() }
  from the reqwest send error. service.rs line 240 passes this through verbatim:
    TranscriptionError::Network { message } => (TranscriptionErrorCode::Network, message.clone())
  The same verbatim passthrough exists at service.rs line 310 in transcribe_with_provider.
  Fix file: src-tauri/src/transcription/service.rs lines 239-241 and lines 309-311.
  Change both Network match arms to emit a fixed friendly string instead of message.clone(),
  e.g. "Network error. Check your connection and try again."

  BUG 3 — Retry button not visible (consequence of BUG 1 only, no logic fix needed):
  App.vue lines 196-205 correctly populates the action object when retryable=true and
  fallback_provider is None. App.vue lines 278-285 correctly renders the button element.
  The button exists in DOM but is invisible solely because .indicator-toast-action has no
  CSS. Fixing BUG 1 (add CSS) is the only required change for BUG 3.

fix: not applied (diagnose-only mode)
verification: n/a
files_changed: []
