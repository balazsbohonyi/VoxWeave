---
status: complete
phase: 02-hotkey
source: [02-01-hotkey-runtime-SUMMARY.md, 02-02-hotkey-config-conflicts-SUMMARY.md, 02-03-hotkey-settings-input-SUMMARY.md]
started: 2026-03-15T11:57:44.3804830+02:00
updated: 2026-03-15T12:38:10+02:00
---

## Current Test

[testing complete]

## Tests

### 1. Configured Hotkey Works After Startup
expected: Launch VoxFlow. Use the hotkey currently shown in Settings. Press it once to enter recording mode (indicator appears / state changes). Press it again to stop (indicator hides / state returns to idle).
result: skipped
reason: Deferred to Phase 4 (Floating Indicator). Hotkey state transitions verified in Phase 2, indicator visibility is out of scope for HOTK requirements.
reported: "Indicator does not appeared, but state changed (can see that in the Start Recording context menu item being changed to Stop Recording). Pressed it again, state changed, context menu item changed back to Start Recording"

### 2. Tray Toggle Matches Hotkey Toggle
expected: Toggling recording from tray and from the currently configured hotkey behaves consistently (same start/stop transitions and no stuck state).
result: skipped
reason: Deferred indicator assertion to Phase 4. Tray/hotkey state parity itself is working in Phase 2.
reported: "This works, but still no inidicator visible."

### 3. Apply Custom Hotkey In Settings
expected: In Settings, edit hotkey text and click Apply. UI shows success feedback and the draft updates to canonical format (for example Ctrl+Shift+Space style ordering/casing).
result: pass

### 4. New Hotkey Takes Effect Immediately
expected: After applying a new hotkey, the old hotkey no longer toggles recording and the new hotkey does, without restarting the app.
result: pass

### 5. Conflict Warning Surface
expected: If the chosen hotkey cannot be bound (conflict/invalid binding), Settings is focused and a warning message is shown in the settings UI.
result: pass

### 6. Last Working Hotkey Is Preserved On Failed Rebind
expected: If Apply fails for a new hotkey, the previously working hotkey still toggles recording.
result: pass

### 7. Hotkey Persists Across Restart
expected: After successfully applying a new hotkey, close and relaunch VoxFlow. The same hotkey remains configured and works.
result: pass

## Summary

total: 7
passed: 5
issues: 0
pending: 0
skipped: 2

## Gaps

- truth: "Indicator appears on recording start and hides when returning idle."
  status: deferred
  reason: "Out of scope for Phase 2 (Hotkey); owned by Phase 4 (Floating Indicator)."
  severity: n/a
  test: 1
  owner_phase: 4
  owner_requirements: [FLOT-01, FLOT-03, FLOT-05]

- truth: "Indicator visibility is consistent when recording is triggered by tray and hotkey paths."
  status: deferred
  reason: "Out of scope for Phase 2 (Hotkey); owned by Phase 4 (Floating Indicator)."
  severity: n/a
  test: 2
  owner_phase: 4
  owner_requirements: [FLOT-01, FLOT-03, FLOT-05]