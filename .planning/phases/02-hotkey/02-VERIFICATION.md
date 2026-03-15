---
phase: 02-hotkey
verified: 2026-03-15T09:27:40Z
status: gaps_found
score: 9/11 must-haves verified
gaps:
  - truth: "User can change the hotkey in Settings UI and the new binding persists across restarts"
    status: failed
    reason: "Settings UI only displays the current hotkey; there is no input or save path to update it."
    artifacts:
      - path: "src/windows/settings/App.vue"
        issue: "No hotkey input/control or save action exists; only read-only display."
    missing:
      - "Settings hotkey control that captures input and calls `save_config` with the new hotkey"
      - "UI feedback for successful hotkey update or validation errors"
---

# Phase 2: Hotkey Verification Report

**Phase Goal:** A global hotkey that can be triggered from any application and drives a toggle-mode recording state machine  
**Verified:** 2026-03-15T09:27:40Z  
**Status:** gaps_found  
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Default runtime hotkey is `Ctrl+Shift+Space` and is registered at startup | ? UNCERTAIN | Default set in `src-tauri/src/config/mod.rs`; startup registers in `src-tauri/src/lib.rs` via `register_startup_hotkey` using global shortcut plugin in `src-tauri/src/hotkey/service.rs`. |
| 2 | Hotkey and tray Start/Stop share one backend toggle path | ✓ VERIFIED | Tray menu calls `hotkey::service::toggle_recording_state` in `src-tauri/src/tray.rs`; hotkey press routes to `handle_shortcut_event` → `toggle_recording_state` in `src-tauri/src/hotkey/service.rs`. |
| 3 | Toggle flow advances `Idle → Recording → Transcribing` on successive triggers | ✓ VERIFIED | State transitions implemented in `src-tauri/src/hotkey/service.rs::toggle_recording_state`. |
| 4 | Hotkey presses while `Transcribing` are ignored | ✓ VERIFIED | `RecordingState::Transcribing` branch returns early in `toggle_recording_state` (`src-tauri/src/hotkey/service.rs`). |
| 5 | Placeholder completion returns to `Idle` so Phase 2 does not get stuck | ✓ VERIFIED | `complete_transcription_placeholder` resets to `Idle` and updates tray (`src-tauri/src/hotkey/service.rs`). |
| 6 | Phase 2 stays bounded: toggle path does not start audio/transcription/injection | ✓ VERIFIED | `toggle_recording_state` only updates state and tray; no audio/transcription/injection modules referenced in `src-tauri/src/hotkey/service.rs`. |
| 7 | Persisted default hotkey is `Ctrl+Shift+Space` | ✓ VERIFIED | `default_hotkey()` returns `Ctrl+Shift+Space` in `src-tauri/src/config/mod.rs`; config defaults/tests align in `src-tauri/src/config/persistence.rs`. |
| 8 | Saving a changed hotkey re-registers immediately and persists in canonical form | ✓ VERIFIED | `save_config` calls `apply_config_update`; normalization + re-register + persistence in `src-tauri/src/hotkey/service.rs` and `src-tauri/src/config/persistence.rs`. |
| 9 | Conflicting rebind keeps last working hotkey active and emits warning | ✓ VERIFIED | On registration error, `apply_config_update_with` leaves binding, sets warning, emits `hotkey-warning` (`src-tauri/src/hotkey/service.rs`). |
| 10 | Startup registration failure leaves app running and warns user | ✓ VERIFIED | Failure path sets `HotkeyAvailability::Unavailable` + warning + `emit_hotkey_warning` (`src-tauri/src/hotkey/service.rs`). |
| 11 | User can change hotkey in Settings UI and the new binding persists | ✗ FAILED | Settings UI is read-only (`src/windows/settings/App.vue`) with no hotkey input or save action. |

**Score:** 9/11 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `src-tauri/src/hotkey/service.rs` | Registration, toggle state machine, conflict handling | ✓ VERIFIED | Contains startup registration, toggle logic, conflict warnings, persistence integration. |
| `src-tauri/src/hotkey/normalize.rs` | Canonical hotkey parsing | ✓ VERIFIED | `normalize_hotkey` normalizes modifiers/keys. |
| `src-tauri/src/lib.rs` | Startup registration of hotkey plugin and binding | ✓ VERIFIED | Registers global shortcut plugin + `register_startup_hotkey`. |
| `src-tauri/src/tray.rs` | Tray Start/Stop uses shared toggle path | ✓ VERIFIED | Start/Stop menu item calls `toggle_recording_state`. |
| `src-tauri/src/commands/config.rs` | Save config routes through hotkey service | ✓ VERIFIED | `save_config` delegates to `apply_config_update`. |
| `src-tauri/src/config/mod.rs` | Default hotkey `Ctrl+Shift+Space` | ✓ VERIFIED | `default_hotkey()` returns `Ctrl+Shift+Space`. |
| `src/composables/useConfig.ts` | Listen for hotkey warning event | ✓ VERIFIED | `listen("hotkey-warning")` updates `hotkeyWarning`. |
| `src/windows/settings/App.vue` | Toast-like warning UI | ✓ VERIFIED | Hotkey warning surface rendered when event payload set. |
| `src/windows/settings/App.vue` | Hotkey change input/control | ✗ MISSING | No hotkey input or save action. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `src-tauri/src/lib.rs` | `register_startup_hotkey` | App setup | ✓ WIRED | Setup registers global shortcut plugin and calls `register_startup_hotkey`. |
| `src-tauri/src/hotkey/service.rs` | Global shortcut runtime | `global_shortcut().on_shortcut` | ✓ WIRED | Registrar hooks shortcut state to handler. |
| `src-tauri/src/tray.rs` | Recording toggle | `toggle_recording_state` | ✓ WIRED | Tray menu Start/Stop uses shared toggle. |
| `src-tauri/src/commands/config.rs` | Hotkey rebind | `apply_config_update` | ✓ WIRED | Save uses hotkey service (normalization + re-register + persist). |
| `src-tauri/src/hotkey/service.rs` | Settings warning UI | `emit("hotkey-warning")` → `useConfig` listener → `App.vue` | ✓ WIRED | Event emitted, listener updates state, UI renders warning. |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| HOTK-01 | 02-01 | User can trigger recording via a global hotkey from any application | ? NEEDS HUMAN | Global shortcut plugin wired; requires OS-level verification. |
| HOTK-02 | 02-01 | Hotkey operates in toggle mode | ✓ SATISFIED | `toggle_recording_state` transitions `Idle → Recording → Transcribing`. |
| HOTK-03 | 02-02 | User can change hotkey in settings and persists across restarts | ✗ BLOCKED | Backend rebind/persist exists; settings UI lacks hotkey change control. |
| HOTK-04 | 02-02 | App detects and warns about hotkey conflicts | ✓ SATISFIED | Conflict sets warning + emits `hotkey-warning` and focuses Settings. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `src-tauri/src/hotkey/service.rs` | 200 | `complete_transcription_placeholder` | ℹ️ Info | Explicit Phase 2 placeholder for later pipeline work. |
| `src-tauri/src/state.rs` | 8 | Placeholder comment | ℹ️ Info | Recording state described as placeholder for Phase 3. |

### Human Verification Required

1. **Global hotkey works from other apps**
   **Test:** Focus another application (terminal, browser), press `Ctrl+Shift+Space` twice.  
   **Expected:** Tray menu label toggles Start/Stop; recording state advances `Idle → Recording → Transcribing` and returns to `Idle`.  
   **Why human:** Requires OS-level global shortcut behavior outside the app.

2. **Conflict warning UX**
   **Test:** Configure a hotkey known to conflict (e.g., system-reserved), restart app.  
   **Expected:** Settings window is focused and warning toast shows conflict details.  
   **Why human:** Requires actual global shortcut registration failure.

### Gaps Summary

The backend hotkey runtime, state machine, persistence, and conflict warnings are wired. The primary gap is the absence of a Settings UI control to change the hotkey. This blocks HOTK-03 and the roadmap success criterion that users can set a custom hotkey in Settings. Adding a hotkey input and wiring it to `save_config` is required to complete Phase 2.

---

_Verified: 2026-03-15T09:27:40Z_  
_Verifier: Claude (gsd-verifier)_
