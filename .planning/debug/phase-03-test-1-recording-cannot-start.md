# DEBUG: phase-03-test-1-recording-cannot-start

- timestamp: 2026-03-15T16:11:00.7322345+02:00
- source: .planning/phases/03-audio-capture/03-UAT.md (Test 1)
- symptom: Cannot start recording via hotkey or tray.

## Investigation
- Traced recording start path: `hotkey/service.rs` -> `audio::start_recording`.
- `audio::start_recording` depends on `capture::system_device_snapshot()`.
- In non-test builds, `system_device_snapshot()` returns empty device list and no default device.

## Root Cause
`system_device_snapshot()` is still a test stub in production path, so runtime always reports no input devices.

## Evidence
- `src-tauri/src/audio/capture.rs`: non-test return is `input_devices: Vec::new(), default_input: None`.
- `src-tauri/src/audio/mod.rs`: empty snapshot leads to `NoInputDevice` error and `start_recording` returns Err.
- `src-tauri/src/hotkey/service.rs`: start failure resets state to idle, making both hotkey/tray start attempts fail.

## Suggested Fix Direction
Implement real microphone enumeration/default resolution in `system_device_snapshot()` (cpal or platform abstraction), and keep stub only for test cfg.