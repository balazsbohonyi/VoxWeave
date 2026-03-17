---
status: diagnosed
trigger: "Error toast is unstyled, shows raw reqwest error string, Retry button invisible"
created: 2026-03-18T00:00:00Z
updated: 2026-03-18T00:00:00Z
---

## Symptoms

expected: Styled error toast with friendly message and Retry button
actual: Unstyled span with raw reqwest error string, no Retry button visible
DOM shows: `<span class="indicator-toast-message">error sending request for url (...)</span>`
reproduction: Disconnect network, press hotkey, speak, stop

## Root Cause

Three independent bugs:

### Bug 1 — Missing CSS (causes unstyled toast AND invisible Retry button)

`src/styles.css` (197 lines) contains no toast CSS. All toast classes referenced in App.vue template are missing:
- `.indicator-toasts` (container)
- `.indicator-toast` (per-toast row)
- `.indicator-toast--error` / `--info` / `--success`
- `.indicator-toast-message`
- `.indicator-toast-action`
- `.indicator-toast-dismiss`

DOM nodes render correctly; they just have no visual styling.

### Bug 2 — Raw reqwest error string exposed

`src-tauri/src/transcription/openai.rs` line 79 (and identically in groq.rs/openrouter.rs):
```rust
.map_err(|e| TranscriptionError::Network { message: e.to_string() })?;
```

`service.rs` line 240 and line 310 — both Network match arms pass `message.clone()` directly to the event payload with no sanitization.

### Bug 3 — Retry button invisible (consequence of Bug 1)

The Retry button IS created in the toast object (`action: { label: "Retry", onClick: ... }` at App.vue lines 196-205) and IS rendered into the DOM via `v-if="toast.action"` at line 281. It is invisible solely due to missing `.indicator-toast-action` CSS. No logic change needed.

## Fix

**1. `src/styles.css` — add toast styles:**
Add full toast component CSS: container layout, per-toast card with background/border/padding, message text styling, action button styling, dismiss button styling, error/info/success color variants.

**2. `src-tauri/src/transcription/service.rs` lines 240 and 310 — friendly message:**
```rust
// Before:
TranscriptionError::Network { message } => {
    (TranscriptionErrorCode::Network, message.clone())
}

// After:
TranscriptionError::Network { .. } => {
    (TranscriptionErrorCode::Network, "Network error. Check your connection and try again.".into())
}
```
Apply to both occurrences (transcribe_with_retry and transcribe_with_provider).

verification: not yet applied
files_changed: []
