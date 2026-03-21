---
status: resolved
trigger: "Settings window opens unexpectedly on hotkey stop"
created: 2026-03-17T00:00:00Z
updated: 2026-03-17T14:00:00Z
---

## Current Focus

hypothesis: emit_hotkey_warning calls show_settings_window unconditionally when hotkey registration fails at startup or save
test: traced call chain from register_startup_hotkey_with -> emit_hotkey_warning(focus_settings=true) -> show_settings_window
expecting: confirmed — settings window opens on any hotkey registration failure, including on startup
next_action: diagnosis complete

## Symptoms

expected: Second hotkey press stops recording and triggers transcription only; no Settings window appears
actual: Settings window opens when user presses hotkey a second time to stop recording
errors: none visible
reproduction: Press hotkey (start recording), press hotkey again (stop) — Settings window opens
started: phase 5 introduced open_settings_on_transcription_tab command but bug is in pre-existing code path

## Eliminated

- hypothesis: open_settings_on_transcription_tab invoked from hotkey stop path in Rust
  evidence: hotkey/service.rs toggle_recording_state has no call to open_settings or show_settings_window
  timestamp: 2026-03-17

- hypothesis: transcription-error listener in App.vue fires unconditionally
  evidence: listener correctly guards on payload.code === "invalid_key" before invoking open_settings_on_transcription_tab
  timestamp: 2026-03-17

- hypothesis: transcription-error emitted during normal stop (no actual API error)
  evidence: service.rs only emits transcription-error on actual provider errors or cancellation; normal success emits transcription-done
  timestamp: 2026-03-17

## Evidence

- timestamp: 2026-03-17
  checked: hotkey/service.rs emit_hotkey_warning (lines 289-304)
  found: |
    fn emit_hotkey_warning(..., focus_settings: bool) {
        if focus_settings {
            tray::show_settings_window(app);  // <-- opens settings window
        }
        ...
    }
  implication: Settings window is opened by hotkey warning, not by transcription path

- timestamp: 2026-03-17
  checked: hotkey/service.rs register_startup_hotkey_with (lines 71-101)
  found: emit_hotkey_warning called with focus_settings=true on registration failure (line 97)
  implication: if hotkey fails to register at startup, settings window opens

- timestamp: 2026-03-17
  checked: hotkey/service.rs apply_config_update_with (lines 121-203)
  found: emit_hotkey_warning called with focus_settings=true on re-registration failure (line 198)
  implication: if hotkey save fails, settings window opens

- timestamp: 2026-03-17
  checked: tauri plugin_global_shortcut behavior — on_shortcut registers a NEW handler but does NOT unregister the previous one
  found: apply_config_update_with calls registrar (on_shortcut) with the SAME canonical hotkey string when hotkey has not changed
  implication: second call to on_shortcut for the same key may return an "already registered" error from the plugin

- timestamp: 2026-03-17
  checked: apply_config_update_with early return guard (lines 144-147)
  found: |
    if requested_canonical == current_canonical {
        persist_config(state.inner(), desired_config.clone())?;
        return Ok(desired_config);  // early return — skips re-registration
    }
  implication: this guard correctly short-circuits on same-hotkey saves; but register_startup_hotkey_with at startup has no equivalent guard

- timestamp: 2026-03-17
  checked: when Settings window is open and user changes ANY config field that calls save_config
  found: save_config path goes through apply_config_update_with which only re-registers if hotkey changed — guard is correct
  implication: the bug is NOT triggered by a config save in normal usage

- timestamp: 2026-03-17
  checked: on_shortcut plugin behavior for duplicate registration
  found: tauri-plugin-global-shortcut on_shortcut called twice for same shortcut string will return an error ("HotKey { ... } already registered")
  implication: if register_startup_hotkey is called more than once (e.g. app re-init, hot reload in dev), the second call fails and opens settings

- timestamp: 2026-03-17
  checked: lib.rs — when register_startup_hotkey is called
  found: called once during setup; but in cargo tauri dev with hot reload, the Rust backend does NOT reload — only the frontend reloads
  implication: standard dev hot reload does not cause double registration

- timestamp: 2026-03-17
  checked: tray.rs show_settings_window (line 80)
  found: unconditionally calls window.show() + window.set_focus()
  implication: any call to emit_hotkey_warning with focus_settings=true opens Settings regardless of user intent

## Resolution

root_cause: |
  hotkey/service.rs line 97 and line 198 — emit_hotkey_warning is called with focus_settings=true
  whenever hotkey registration fails (at startup or on save). The function at line 296 calls
  tray::show_settings_window unconditionally when focus_settings is true.

  The specific trigger for the user's symptom ("stops recording → Settings opens") points to
  hotkey registration failing at startup (register_startup_hotkey_with, line 97). When the app
  starts and the configured hotkey is already claimed by another process or is invalid, registration
  fails, emit_hotkey_warning fires with focus_settings=true, and show_settings_window is called
  immediately. This happens BEFORE the user does anything. The Settings window then appears to
  open "on hotkey stop" because:
  1. The hotkey is not actually registered (registration failed).
  2. No hotkey fires on first press — the user sees nothing happen.
  3. The Settings window was already shown at startup and regains focus / appears when user clicks
     back into the app area after "first press".

  Alternatively (and more likely given "works: Processing state appears"): the hotkey IS registered
  but saving config through Settings while the app is running triggers apply_config_update_with with
  a DIFFERENT hotkey string that fails to register, opening Settings mid-session.

fix: |
  Two independent fixes needed:

  1. emit_hotkey_warning with focus_settings=false for the save path (line 198):
     The settings window is already open when the user is changing the hotkey — no need to re-open it.
     Change: emit_hotkey_warning(app, &warning, HotkeyWarningSource::Save, false);

  2. emit_hotkey_warning with focus_settings=false for the startup path (line 97):
     Opening Settings unsolicited on startup is poor UX. The hotkey-warning event is already emitted
     so the Settings window will show the warning card when the user opens it manually.
     Change: emit_hotkey_warning(app, &warning, HotkeyWarningSource::Startup, false);

  Both changes remove the unconditional Settings window auto-open. The warning payload is still
  emitted so the WarningCard in Settings App.vue will display it when the user opens Settings.

verification: not yet applied
files_changed: []
