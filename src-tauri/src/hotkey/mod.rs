use crate::state::{AppState, RecordingState};
use tauri::{AppHandle, Runtime};

/// Handle a hotkey press by starting recording.
/// Phase 2 replaces this with the full toggle state machine.
pub fn handle_hotkey_pressed<R: Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<AppState>() {
        *state.recording_state.lock().unwrap() = RecordingState::Recording;
    }
}

#[cfg(test)]
mod tests {
    use super::handle_hotkey_pressed;
    use crate::state::{AppState, RecordingState};
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
}
