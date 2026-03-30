---
status: complete
phase: 10-local-transcription
source: 10-01-SUMMARY.md, 10-02-SUMMARY.md, 10-03-SUMMARY.md
started: 2026-03-30T00:00:00Z
updated: 2026-03-30T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Settings Local Model Cards
expected: Open Settings → Transcription tab → select Local provider. The local section shows model cards (e.g. tiny, base, small, medium, large). Each card has a Download button. No "coming soon" placeholder.
result: pass

### 2. Download a Model
expected: Click Download on a model card. A progress bar appears showing download percentage as a whole number (no decimals). Progress bar fill and percentage text stay visually in sync throughout the download.
result: pass

### 3. Cancel a Download
expected: While a download is in progress, click Cancel. The download stops, the progress bar disappears, and the card returns to its "Download" button state. No partial file remains visible.
result: pass

### 4. Downloaded Model State
expected: After a download completes, the card shows a "Downloaded" badge and a trash icon (delete button). No Download button remains.
result: pass
note: "Downloaded" badge intentionally removed from UI per design decision; trash icon and state change confirmed correct.

### 5. Delete a Model
expected: Click the trash icon on a downloaded model. The card immediately reverts to the "Download" button state (no flicker of Active badge). The model is gone.
result: issue
reported: "Delete works, but after deleting a model it's not possible to download it again — clicking Download shows no progress bar and nothing happens. Requires app restart to re-download. Same issue expected in the Wizard window."
severity: major

### 6. Set Active Model
expected: After downloading a model, activate it (select it as active). The card shows a blue "Active" badge with white text and a 2px blue border. Only one card is active at a time.
result: pass
note: Design decision: downloaded model is always auto-activated.

### 7. Wizard Step 2 — Local Engine
expected: Start the setup wizard and choose Local as the transcription engine. Step 2 shows a local model picker with the same model cards (Download, Downloaded badge, Active) — not a "coming soon" placeholder.
result: pass

### 8. Wizard Skip
expected: In wizard Step 2 (local model picker), click "Skip for now". The wizard immediately advances to Step 3 — no hanging, no extra click needed.
result: pass

### 9. Wizard Next Gate
expected: In wizard Step 2 (local model picker), the Next button is disabled (grayed out / not clickable) until at least one model has been downloaded. After downloading a model, Next becomes clickable.
result: issue
reported: "Next gate works. Two UI change requests: (1) wizard window shows vertical scrollbar in Step 2 for local models — window should be ~20-30px taller to avoid it; (2) recycle bin icon button should appear before the Set Active button, in both Settings and Wizard."
severity: minor

### 10. Post-Finish Nudge Toast
expected: Complete the wizard with Local engine selected but without downloading any model (use Skip). After the wizard closes, a toast appears nudging you to download a model (something like "No model downloaded — go to Settings to download one").
result: pass

### 11. Pre-Recording Guard
expected: With Local engine active in Settings and no model file downloaded, press the global hotkey. Recording does NOT start (no waveform, no audio capture). Instead, a toast appears saying the model is missing.
result: pass

### 12. Model Missing Toast — Open Settings Action
expected: When the model_missing toast appears (from test 11 or any model_missing scenario), it has an "Open Settings" action button. Clicking it opens the Settings window on the Transcription tab with Local provider visible.
result: pass

## Summary

total: 12
passed: 10
issues: 3
pending: 0
skipped: 0

## Gaps

- truth: "After deleting a model, clicking Download on the same card starts a new download (progress bar appears, download proceeds)"
  status: failed
  reason: "User reported: after deleting a model it's not possible to download it again — clicking Download shows no progress bar and nothing happens. Requires app restart to re-download. Same issue expected in the Wizard window."
  severity: major
  test: 5
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""

- truth: "Wizard Step 2 (local) has no vertical scrollbar; window height is sufficient to show all model cards without scrolling"
  status: failed
  reason: "User reported: wizard window shows a vertical scrollbar in Step 2 for local models — window should be ~20-30px taller to eliminate it"
  severity: minor
  test: 9
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""

- truth: "Recycle bin (delete) icon button appears before the Set Active button on model cards, in both Settings and Wizard"
  status: failed
  reason: "User requested: move recycle bin icon button before the Set Active button in both Settings TranscriptionSection.vue and Wizard Step2Local.vue"
  severity: minor
  test: 9
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
