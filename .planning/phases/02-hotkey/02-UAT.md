---
status: complete
phase: 02-hotkey
source: [02-01-hotkey-runtime-SUMMARY.md, 02-02-hotkey-config-conflicts-SUMMARY.md, 02-03-hotkey-settings-input-SUMMARY.md]
started: 2026-03-15T11:57:44.3804830+02:00
updated: 2026-03-15T12:21:03.9994811+02:00
---

## Current Test

[testing complete]

## Tests

### 1. Configured Hotkey Works After Startup
expected: Launch VoxFlow. Use the hotkey currently shown in Settings. Press it once to enter recording mode (indicator appears / state changes). Press it again to stop (indicator hides / state returns to idle).
result: issue
reported: "Indicator does not appeared, but state changed (can see that in the Start Recording context menu item being changed to Stop Recording). Pressed it again, state changed, context menu item changed back to Start Recording"
severity: major

### 2. Tray Toggle Matches Hotkey Toggle
expected: Toggling recording from tray and from the currently configured hotkey behaves consistently (same start/stop transitions and no stuck state).
result: issue
reported: "This works, but still no inidicator visible."
severity: minor

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
issues: 2
pending: 0
skipped: 0

## Gaps

- truth: "Launch VoxFlow. Use the hotkey currently shown in Settings. Press it once to enter recording mode (indicator appears / state changes). Press it again to stop (indicator hides / state returns to idle)."
  status: failed
  reason: "User reported: Indicator does not appeared, but state changed (can see that in the Start Recording context menu item being changed to Stop Recording). Pressed it again, state changed, context menu item changed back to Start Recording"
  severity: major
  test: 1
  artifacts: []
  missing: []
- truth: "Toggling recording from tray and from the currently configured hotkey behaves consistently (same start/stop transitions and no stuck state)."
  status: failed
  reason: "User reported: This works, but still no inidicator visible."
  severity: minor
  test: 2
  artifacts: []
  missing: []