use crate::hotkey::normalize::normalize_hotkey;
use crate::state::{AppState, HotkeyAvailability, HotkeyWarning, RecordingState};
use crate::tray;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub(crate) type HotkeyHandler<R> =
    Box<dyn Fn(&AppHandle<R>, ShortcutState) + Send + Sync + 'static>;

pub fn register_startup_hotkey<R: Runtime>(app: &AppHandle<R>) {
    register_startup_hotkey_with(app, |app, hotkey, handler| {
        app.global_shortcut()
            .on_shortcut(hotkey, move |app, _shortcut, event| {
                handler(app, event.state);
            })
            .map_err(|err| err.to_string())
    });
}

pub(crate) fn register_startup_hotkey_with<R: Runtime, F>(
    app: &AppHandle<R>,
    registrar: F,
) where
    F: FnOnce(&AppHandle<R>, &str, HotkeyHandler<R>) -> Result<(), String>,
{
    let state = app.state::<AppState>();
    let configured = state.config.lock().unwrap().hotkey.clone();
    let canonical = normalize_hotkey(&configured);

    *state.hotkey_binding.lock().unwrap() = canonical.clone();

    let handler: HotkeyHandler<R> = Box::new(|app, event| handle_shortcut_event(app, event));

    match registrar(app, canonical.as_str(), handler) {
        Ok(_) => {
            *state.hotkey_availability.lock().unwrap() = HotkeyAvailability::Registered;
            *state.hotkey_warning.lock().unwrap() = None;
        }
        Err(err) => {
            let message = err.clone();
            *state.hotkey_availability.lock().unwrap() = HotkeyAvailability::Unavailable;
            *state.hotkey_warning.lock().unwrap() = Some(HotkeyWarning {
                hotkey: canonical,
                message: err,
            });
            log::warn!("Failed to register global hotkey: {message}");
        }
    }
}

pub fn handle_shortcut_event<R: Runtime>(app: &AppHandle<R>, event: ShortcutState) {
    if event == ShortcutState::Pressed {
        toggle_recording_state(app);
    }
}

pub fn toggle_recording_state<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let next_state = {
        let mut recording_state = state.recording_state.lock().unwrap();
        match *recording_state {
            RecordingState::Idle => {
                *recording_state = RecordingState::Recording;
                RecordingState::Recording
            }
            RecordingState::Recording => {
                *recording_state = RecordingState::Transcribing;
                RecordingState::Transcribing
            }
            RecordingState::Transcribing => {
                return;
            }
        }
    };

    tray::update_recording_menu(app, next_state.clone());

    if next_state == RecordingState::Transcribing {
        complete_transcription_placeholder(app);
    }
}

fn complete_transcription_placeholder<R: Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<AppState>() {
        *state.recording_state.lock().unwrap() = RecordingState::Idle;
    }
    tray::update_recording_menu(app, RecordingState::Idle);
}
