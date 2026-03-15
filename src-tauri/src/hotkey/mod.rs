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
    use super::normalize;
    use super::service;
    use crate::state::RecordingState;

    #[test]
    fn normalization_canonicalizes_modifiers() {
        assert_eq!(
            normalize::normalize_hotkey("shift+ctrl+space"),
            "Ctrl+Shift+Space"
        );
    }

    #[test]
    fn next_state_transitions_from_idle() {
        assert_eq!(
            service::next_recording_state(&RecordingState::Idle),
            Some(RecordingState::Recording)
        );
    }

    #[test]
    fn next_state_transitions_from_recording() {
        assert_eq!(
            service::next_recording_state(&RecordingState::Recording),
            Some(RecordingState::Transcribing)
        );
    }

    #[test]
    fn next_state_ignores_transcribing() {
        assert_eq!(
            service::next_recording_state(&RecordingState::Transcribing),
            None
        );
    }

    #[test]
    fn registration_conflict_message_is_human_friendly() {
        assert_eq!(
            service::humanize_registration_error("Ctrl+Alt+Space", "already registered"),
            "Ctrl+Alt+Space is already registered by another app. Choose a different hotkey."
        );
    }
}
