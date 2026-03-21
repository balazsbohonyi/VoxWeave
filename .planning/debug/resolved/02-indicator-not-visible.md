# Debug Session: indicator-not-visible

## Scope
Phase 02 UAT gaps 1 and 2.

## Symptoms
- Recording state changes via hotkey/tray (menu label toggles Start/Stop).
- Floating indicator never appears.

## Evidence
- `src-tauri/tauri.conf.json` defines only one window: `settings`.
- No `indicator` window label/config exists.
- `src-tauri/src/hotkey/service.rs::toggle_recording_state` only mutates state + tray menu.
- No show/hide calls for an indicator window in hotkey/tray flow.
- `src/windows/indicator/` contains only `.gitkeep` (no mounted indicator UI).

## Root Cause
Indicator subsystem is not implemented/wired in this phase: no Tauri indicator window exists and no runtime show/hide transitions are invoked when recording state changes.

## Missing
1. Add indicator window definition and properties in `tauri.conf.json`.
2. Add indicator window show/hide wiring on recording state transitions.
3. Add indicator frontend entry/render assets.
4. Add tests/UAT coverage for indicator visibility transitions.