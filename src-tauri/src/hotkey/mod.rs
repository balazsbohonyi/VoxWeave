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
    use super::service;
    use crate::state::{AppState, RecordingState};
    use tauri::Manager;
    use tauri::test::mock_app;

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
}
