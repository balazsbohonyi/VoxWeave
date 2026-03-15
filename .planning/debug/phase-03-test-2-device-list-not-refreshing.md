# DEBUG: phase-03-test-2-device-list-not-refreshing

- timestamp: 2026-03-15T16:11:00.7322345+02:00
- source: .planning/phases/03-audio-capture/03-UAT.md (Test 2)
- symptom: Settings shows only "System default"; Refresh never reveals plugged-in devices.

## Investigation
- Device list command: `commands/audio.rs` returns `audio::list_input_device_names()`.
- `audio::list_input_device_names()` also reads `capture::system_device_snapshot()`.
- UI always renders a static `System default` option even when backend list is empty.

## Root Cause
Backend device enumeration is unimplemented in production (`system_device_snapshot` returns empty), so command always returns an empty list; Refresh is effectively a no-op.

## Evidence
- `src-tauri/src/audio/capture.rs`: production snapshot is empty.
- `src-tauri/src/commands/audio.rs`: command is thin wrapper over empty backend list.
- `src/windows/settings/App.vue`: static `<option value="">System default</option>` explains why exactly one entry is visible.

## Suggested Fix Direction
Implement real input-device enumeration + default device lookup and return deduped list; optionally display "No devices found" state when backend list is empty.