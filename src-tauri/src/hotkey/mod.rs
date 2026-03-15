pub mod normalize;
pub mod service;
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::ShortcutState;

/// Handle a hotkey press by starting recording.
/// Phase 2 replaces this with the full toggle state machine.
pub fn handle_hotkey_pressed<R: Runtime>(app: &AppHandle<R>) {
    service::handle_shortcut_event(app, ShortcutState::Pressed);
}

#[cfg(test)]
mod tests {
    use super::handle_hotkey_pressed;
    use super::normalize;
    use super::service;
    use super::service::HotkeyHandler;
    use crate::state::{AppState, HotkeyAvailability, RecordingState};
    use tauri::Manager;
    use tauri::test::mock_app;
    use tauri::test::MockRuntime;
    use tauri_plugin_global_shortcut::ShortcutState;
    use std::sync::{Arc, Mutex};

    #[test]
    fn default_hotkey_press_starts_recording() {
        let app = mock_app();
        app.manage(AppState::load());

        let state = app.state::<AppState>();
        assert_eq!(*state.recording_state.lock().unwrap(), RecordingState::Idle);

        handle_hotkey_pressed(&app.handle());

        assert_eq!(*state.recording_state.lock().unwrap(), RecordingState::Recording);
    }

    #[test]
    fn toggle_respects_state_machine() {
        let app = mock_app();
        app.manage(AppState::load());

        let state = app.state::<AppState>();
        assert_eq!(*state.recording_state.lock().unwrap(), RecordingState::Idle);

        service::toggle_recording_state(&app.handle());
        assert_eq!(*state.recording_state.lock().unwrap(), RecordingState::Recording);

        service::toggle_recording_state(&app.handle());
        assert_eq!(*state.recording_state.lock().unwrap(), RecordingState::Idle);

        *state.recording_state.lock().unwrap() = RecordingState::Transcribing;
        service::toggle_recording_state(&app.handle());
        assert_eq!(
            *state.recording_state.lock().unwrap(),
            RecordingState::Transcribing
        );
    }

    #[test]
    fn normalization_canonicalizes_modifiers() {
        assert_eq!(
            normalize::normalize_hotkey("shift+ctrl+space"),
            "Ctrl+Shift+Space"
        );
    }

    #[test]
    fn startup_registration_uses_default_binding() {
        let app = mock_app();
        app.manage(AppState::load());

        let recorded = Arc::new(Mutex::new(Vec::new()));
        let handler_slot: Arc<Mutex<Option<HotkeyHandler<MockRuntime>>>> =
            Arc::new(Mutex::new(None));

        let recorded_clone = recorded.clone();
        let handler_clone = handler_slot.clone();

        service::register_startup_hotkey_with(&app.handle(), move |_app, hotkey, handler| {
            recorded_clone.lock().unwrap().push(hotkey.to_string());
            *handler_clone.lock().unwrap() = Some(handler);
            Ok(())
        });

        assert_eq!(recorded.lock().unwrap().as_slice(), &["Ctrl+Shift+Space"]);

        let handler = handler_slot.lock().unwrap().take().expect("handler missing");
        handler(&app.handle(), ShortcutState::Pressed);

        let state = app.state::<AppState>();
        assert_eq!(*state.recording_state.lock().unwrap(), RecordingState::Recording);
    }

    #[test]
    fn apply_hotkey_change_persists_canonical_value() {
        let app = mock_app();
        app.manage(AppState::load());

        let mut updated = app.state::<AppState>().config.lock().unwrap().clone();
        updated.hotkey = "alt+ctrl+space".to_string();

        let result = service::apply_config_update_with(
            &app.handle(),
            updated,
            |_app, hotkey, _handler| {
                assert_eq!(hotkey, "Ctrl+Alt+Space");
                Ok(())
            },
            |_app, _hotkey| Ok(()),
        )
        .expect("config update failed");

        assert_eq!(result.hotkey, "Ctrl+Alt+Space");
        let state = app.state::<AppState>();
        assert_eq!(
            state.config.lock().unwrap().hotkey,
            "Ctrl+Alt+Space".to_string()
        );
    }

    #[test]
    fn unchanged_canonical_save_short_circuit() {
        let app = mock_app();
        app.manage(AppState::load());

        let mut updated = app.state::<AppState>().config.lock().unwrap().clone();
        updated.hotkey = "shift+ctrl+space".to_string();

        let result = service::apply_config_update_with(
            &app.handle(),
            updated,
            |_app, _hotkey, _handler| panic!("should not re-register"),
            |_app, _hotkey| Ok(()),
        )
        .expect("config update failed");

        assert_eq!(result.hotkey, "Ctrl+Shift+Space");
    }

    #[test]
    fn conflicting_hotkey_keeps_last_working_binding() {
        let app = mock_app();
        app.manage(AppState::load());

        let state = app.state::<AppState>();
        *state.hotkey_binding.lock().unwrap() = "Ctrl+Shift+Space".to_string();
        *state.hotkey_availability.lock().unwrap() = HotkeyAvailability::Registered;

        let mut updated = state.config.lock().unwrap().clone();
        updated.hotkey = "ctrl+alt+space".to_string();

        let result = service::apply_config_update_with(
            &app.handle(),
            updated,
            |_app, _hotkey, _handler| Err("already registered".to_string()),
            |_app, _hotkey| Ok(()),
        )
        .expect("config update failed");

        assert_eq!(result.hotkey, "Ctrl+Shift+Space");
        assert_eq!(
            *state.hotkey_binding.lock().unwrap(),
            "Ctrl+Shift+Space".to_string()
        );
        assert_eq!(
            *state.hotkey_availability.lock().unwrap(),
            HotkeyAvailability::Registered
        );
        assert!(state.hotkey_warning.lock().unwrap().is_some());
    }

    #[test]
    fn startup_conflict_leaves_app_inactive() {
        let app = mock_app();
        app.manage(AppState::load());

        service::register_startup_hotkey_with(&app.handle(), |_app, _hotkey, _handler| {
            Err("startup conflict".to_string())
        });

        let state = app.state::<AppState>();
        assert_eq!(
            *state.hotkey_availability.lock().unwrap(),
            HotkeyAvailability::Unavailable
        );
        assert!(state.hotkey_warning.lock().unwrap().is_some());
    }
}
