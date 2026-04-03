use crate::audio;
use crate::state::{AppState, RecordingState};
use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
pub fn list_audio_input_devices() -> Result<Vec<String>, String> {
    Ok(audio::list_input_device_names())
}

/// Trigger recording stop programmatically (e.g. from a VAD silence event).
/// Only acts when the current state is Recording — ignores other states.
#[tauri::command]
pub fn trigger_stop_recording<R: Runtime>(app: AppHandle<R>) {
    let state = app.state::<AppState>();
    let is_recording = state
        .recording_state
        .lock()
        .map(|s| *s == RecordingState::Recording)
        .unwrap_or(false);
    if is_recording {
        crate::hotkey::service::toggle_recording_state(&app);
    }
}

#[cfg(test)]
mod tests {
    use super::list_audio_input_devices;

    #[test]
    fn lists_input_devices() {
        let devices = list_audio_input_devices().expect("command failed");
        assert_eq!(devices, vec!["Mock Microphone".to_string()]);
    }
}
