---
status: diagnosed
phase: 02-hotkey
source: [02-01-hotkey-runtime-SUMMARY.md, 02-02-hotkey-config-conflicts-SUMMARY.md, 02-03-hotkey-settings-input-SUMMARY.md]
started: 2026-03-15T11:57:44.3804830+02:00
updated: 2026-03-15T12:24:20+02:00
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
  root_cause: "Indicator subsystem not implemented in runtime path: only settings window exists in Tauri config, and recording toggle updates tray/state but never shows/hides an indicator window."
  artifacts:
    - path: "src-tauri/tauri.conf.json"
      issue: "Only settings window is declared; no indicator window label/config exists"
    - path: "src-tauri/src/hotkey/service.rs"
      issue: "toggle_recording_state mutates state/tray only; no indicator window show/hide calls"
    - path: "src/windows/indicator/.gitkeep"
      issue: "Indicator frontend window content not implemented"
  missing:
    - "Define indicator window in Tauri app windows config"
    - "Wire indicator show/hide to recording state transitions"
    - "Add indicator UI entry/rendering and verify lifecycle"
  debug_session: ".planning/debug/02-indicator-not-visible.md"
- truth: "Toggling recording from tray and from the currently configured hotkey behaves consistently (same start/stop transitions and no stuck state)."
  status: failed
  reason: "User reported: This works, but still no inidicator visible."
  severity: minor
  test: 2
  root_cause: "Same underlying indicator gap as Test 1: parity of state transitions is present, but indicator visibility path is missing globally."
  artifacts:
    - path: "src-tauri/src/tray.rs"
      issue: "Tray toggle delegates to shared state toggle; no indicator window handling in tray flow"
    - path: "src-tauri/src/hotkey/service.rs"
      issue: "Shared toggle path has no indicator show/hide logic"
  missing:
    - "Attach indicator visibility behavior to shared toggle path used by both tray and hotkey"
    - "Add regression test/manual check to ensure indicator appears for both trigger paths"
  debug_session: ".planning/debug/02-indicator-not-visible.md"