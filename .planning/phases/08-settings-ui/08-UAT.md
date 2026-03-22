---
status: complete
phase: 08-settings-ui
source: [08-01-SUMMARY.md, 08-02-SUMMARY.md, 08-03-SUMMARY.md, 08-04-SUMMARY.md, 08-05-SUMMARY.md]
started: 2026-03-23T00:00:00Z
updated: 2026-03-23T12:00:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Settings Window Loads
expected: Open the Settings window. All four sections render without errors: General, Audio, Transcription, and Injection. The window is scrollable if content exceeds height.
result: pass

### 2. Hotkey Capture Widget
expected: Click the hotkey input field in the General section. Press a key combo (e.g. Ctrl+Shift+Space). The field shows a live preview of the combo while keys are held. On releasing the non-modifier key, the hotkey is saved. Pressing Escape should blur the field without saving.
result: pass

### 3. Launch at Login Toggle
expected: The General section shows a "Launch at login" checkbox. Toggling it on/off saves immediately (no save button needed). The state reflects correctly after reopening settings.
result: pass

### 4. Audio Device Dropdown
expected: The Audio section shows a microphone dropdown populated with available audio devices. Changing the selection saves immediately.
result: pass

### 5. Silence Detection Toggle
expected: The Audio section shows a silence detection toggle. Toggling it on shows a silence duration input (clamped to 500–5000ms, debounced 300ms). Toggling it off hides or disables the duration input. Changes save immediately.
result: pass

### 6. Transcription Cloud/Local Toggle
expected: The Transcription section shows a Cloud/Local toggle. Switching to Local shows local model content (Whisper stub cards). Switching back to Cloud restores the cloud provider tabs. The active provider persists.
result: pass

### 7. Transcription Provider Tabs and Active Badge
expected: In Cloud mode, OpenAI and Groq tabs are visible. The tab matching the current active provider shows an "Active" badge. Clicking the inactive tab switches the view but does NOT change the active provider (no badge moves) until "Set as active" is clicked.
result: pass

### 8. API Key Input — Masked with Eye Toggle
expected: Each cloud provider tab shows a password-masked API key field. Clicking the eye icon reveals the key. Editing the field clears any existing test-connection result. The key saves on blur (not on every keystroke).
result: pass

### 9. Test Connection
expected: With an API key entered, clicking "Test connection" shows a spinner during the request. On success, a green confirmation message appears inline. On failure (invalid key), a red error message appears. No navigation or page reload occurs.
result: pass

### 10. Set as Active Button
expected: On the inactive provider tab (e.g. Groq when OpenAI is active), a "Set as active" button is enabled. Clicking it updates the active provider — the Active badge moves to the new tab, and the button becomes disabled on that tab.
result: pass

### 11. Model Dropdown Populated from Backend
expected: The model dropdown on each cloud provider tab is populated with models fetched from the Rust backend (not hardcoded in Vue). OpenAI shows ~3 models (e.g. whisper-1, etc.), Groq shows ~3 models. Changing the model saves immediately.
result: pass

### 12. Language Hint Field
expected: A language hint field (e.g. "Language" select or input) appears below the cloud provider tabs (and in local mode). It is a global field — not per-provider. Changing it saves immediately.
result: pass

### 13. Local Whisper Model Stub Cards
expected: Switching to Local mode shows 4 model stub cards (Tiny, Base, Small, Medium). Each card has a "Download" button that is disabled with a tooltip like "coming in a future update." No actual download occurs.
result: pass

### 14. Injection Method Radio Buttons
expected: The Injection section shows three radio buttons: FlashPaste, Keystrokes, Clipboard. The current selection is pre-filled. Changing selection saves immediately. The selected method persists after reopening settings.
result: pass

### 15. Keystroke Speed Selector — Conditional Visibility
expected: In the Injection section, the speed selector is only visible when "Keystrokes" mode is selected. Selecting FlashPaste or Clipboard hides the speed selector entirely (not just grayed out — absent from DOM). Switching back to Keystrokes makes it reappear.
result: pass

### 16. Auto-Fallback Checkbox
expected: The Injection section shows an "Auto-fallback" checkbox. Toggling it saves immediately and the state persists after reopening settings.
result: pass

### 17. Settings Persist Across Restart
expected: Change several settings (hotkey, model selection, injection method). Close the settings window and reopen it (or restart the app). All changed settings are preserved exactly as set.
result: pass

## Summary

total: 17
passed: 17
issues: 0
pending: 0
skipped: 0

## Gaps

[none yet]
