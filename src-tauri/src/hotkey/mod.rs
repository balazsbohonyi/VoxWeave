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
    use crate::state::{AppState, RecordingState};
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
}
