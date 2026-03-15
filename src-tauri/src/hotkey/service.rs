use crate::config::{persistence, AppConfig};
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
            let warning = HotkeyWarning {
                hotkey: canonical,
                message: err.clone(),
            };
            *state.hotkey_availability.lock().unwrap() = HotkeyAvailability::Unavailable;
            *state.hotkey_warning.lock().unwrap() = Some(warning.clone());
            log::warn!("Failed to register global hotkey: {err}");
        }
    }
}

pub fn apply_config_update<R: Runtime>(
    app: &AppHandle<R>,
    config: AppConfig,
) -> Result<AppConfig, String> {
    apply_config_update_with(
        app,
        config,
        |app, hotkey, handler| {
            app.global_shortcut()
                .on_shortcut(hotkey, move |app, _shortcut, event| {
                    handler(app, event.state);
                })
                .map_err(|err| err.to_string())
        },
        |app, hotkey| app.global_shortcut().unregister(hotkey).map_err(|e| e.to_string()),
    )
}

pub(crate) fn apply_config_update_with<R: Runtime, FRegister, FUnregister>(
    app: &AppHandle<R>,
    config: AppConfig,
    registrar: FRegister,
    unregister: FUnregister,
) -> Result<AppConfig, String>
where
    FRegister: FnOnce(&AppHandle<R>, &str, HotkeyHandler<R>) -> Result<(), String>,
    FUnregister: FnOnce(&AppHandle<R>, &str) -> Result<(), String>,
{
    let state = app.state::<AppState>();
    let current_config = state
        .config
        .lock()
        .map_err(|e| e.to_string())?
        .clone();

    let current_canonical = normalize_hotkey(&current_config.hotkey);
    let requested_canonical = normalize_hotkey(&config.hotkey);

    let mut desired_config = config.clone();
    desired_config.hotkey = requested_canonical.clone();

    if requested_canonical == current_canonical {
        persist_config(state.inner(), desired_config.clone())?;
        return Ok(desired_config);
    }

    let previous_binding = state
        .hotkey_binding
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let previous_availability = state
        .hotkey_availability
        .lock()
        .map_err(|e| e.to_string())?
        .clone();

    let handler: HotkeyHandler<R> = Box::new(|app, event| handle_shortcut_event(app, event));

    match registrar(app, requested_canonical.as_str(), handler) {
        Ok(_) => {
            if previous_availability == HotkeyAvailability::Registered
                && previous_binding != requested_canonical
            {
                if let Err(err) = unregister(app, previous_binding.as_str()) {
                    log::warn!("Failed to unregister old hotkey: {err}");
                }
            }

            *state
                .hotkey_binding
                .lock()
                .map_err(|e| e.to_string())? = requested_canonical.clone();
            *state
                .hotkey_availability
                .lock()
                .map_err(|e| e.to_string())? = HotkeyAvailability::Registered;
            *state
                .hotkey_warning
                .lock()
                .map_err(|e| e.to_string())? = None;

            persist_config(state.inner(), desired_config.clone())?;
            Ok(desired_config)
        }
        Err(err) => {
            let warning = HotkeyWarning {
                hotkey: requested_canonical,
                message: err.clone(),
            };
            *state
                .hotkey_warning
                .lock()
                .map_err(|e| e.to_string())? = Some(warning.clone());

            Ok(current_config)
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

fn persist_config(state: &AppState, config: AppConfig) -> Result<(), String> {
    let mut raw = state.config_raw.lock().map_err(|e| e.to_string())?;
    persistence::save(&config, &mut raw)?;

    let mut current = state.config.lock().map_err(|e| e.to_string())?;
    *current = config;
    Ok(())
}
