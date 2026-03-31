---
status: complete
phase: 09-setup-wizard
source: 09-01-SUMMARY.md, 09-02-SUMMARY.md, 09-03-PLAN.md
started: 2026-03-25T00:00:00Z
updated: 2026-03-25T21:30:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start — First-Launch Wizard Opens
expected: Delete %APPDATA%\VoxWeave\config.json (or rename it), then run `cargo tauri dev` (or launch the built app). The wizard window should appear automatically — not the settings window, not just a tray icon. App boots without errors or crashes.
result: pass

### 2. WizardStepper Progress Dots
expected: In the open wizard, the stepper at the top shows three dots connected by lines. Step 1 dot is filled blue (active). Steps 2 and 3 dots are hollow/gray. Labels read "Engine", "Configure", "Hotkey" (or similar). Advancing to Step 2 fills Step 1 blue and makes Step 2 the active dot.
result: pass

### 3. Step 1 — Engine Choice
expected: Step 1 shows two clickable radio cards: "Cloud" and "Local". Clicking each selects it (highlighted/checked). The Next button advances to Step 2. Selecting Local leads to a "coming soon" placeholder on Step 2 (not an API key form).
result: pass

### 4. Step 2 (Cloud) — Provider Tabs and API Key
expected: With Cloud selected, Step 2 shows two tabs: OpenAI and Groq. Switching tabs changes the API key input field. The key is masked by default; an eye icon toggle reveals/hides it. A "Test connection" button sends a request (spinner while testing) and shows an inline pass/fail result message. Typing in the key field and tabbing away (blur) saves the draft.
result: pass

### 5. Step 3 — Hotkey Capture
expected: Step 3 shows the HotkeyCapture widget (same as in Settings). Clicking the capture button and pressing a key combination records it. Saving the hotkey works. Back button returns to Step 2.
result: pass

### 6. Finish Sequence — Success Banner + Settings Opens
expected: On Step 3, click Finish. A green success banner appears inside the wizard: "VoxWeave is ready! Opening Settings..." (or similar text). The bottom button bar disappears during this 1.2s window. After ~1.2 seconds, the Settings window opens and the wizard closes/hides.
result: pass

### 7. first_launch=false Written to Config
expected: After clicking Finish, open %APPDATA%\VoxWeave\config.json. The file should contain `"first_launch": false`.
result: pass

### 8. Restart After Finish — Wizard Does Not Re-open
expected: After completing the wizard (Finish clicked), quit VoxWeave completely and relaunch it. Only the tray icon should appear — the wizard must NOT open again automatically.
result: pass

### 9. Close With X — Wizard Re-opens on Next Launch
expected: Delete config.json and restart the app (wizard opens). Close the wizard with the X button WITHOUT clicking Finish. Restart the app — the wizard should open again (first_launch is still true because Finish was never clicked).
result: pass

### 10. Setup Wizard Button in Settings
expected: Open Settings via the tray icon (after a completed first-launch). In the General section, at the bottom, there is a "Setup Wizard..." link/button with a subtitle like "Re-run the first-launch setup guide". Clicking it opens the wizard window.
result: pass

### 11. Wizard Pre-fills From Current Config
expected: With the wizard opened from Settings (via the Setup Wizard button), the wizard starts at Step 1 with the engine choice pre-selected to match the current config (Cloud if OpenAI/Groq is active, Local if local). The provider tab on Step 2 also matches the currently configured provider.
result: pass

## Summary

total: 11
passed: 11
issues: 0
pending: 0
skipped: 0

## Gaps

[none yet]
