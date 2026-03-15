use crate::audio;
use crate::config::{persistence, AppConfig};
use crate::hotkey::normalize::normalize_hotkey;
use crate::state::{AppState, HotkeyAvailability, HotkeyWarning, RecordingState};
use crate::tray;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub(crate) type HotkeyHandler<R> =
    Box<dyn Fn(&AppHandle<R>, ShortcutState) + Send + Sync + 'static>;

const HOTKEY_WARNING_EVENT: &str = "hotkey-warning";

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyWarningSource {
    Startup,
    Save,
}

#[derive(Debug, Clone, Serialize)]
pub struct HotkeyWarningPayload {
    pub hotkey: String,
    pub message: String,
    pub source: HotkeyWarningSource,
}

fn pretty_hotkey(hotkey: &str) -> String {
    hotkey
        .split('+')
        .map(|part| match part {
            "Super" | "Meta" | "Command" | "Cmd" => "Win".to_string(),
            value if value.starts_with("Key") && value.len() > 3 => value[3..].to_string(),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join("+")
}

pub(crate) fn humanize_registration_error(hotkey: &str, err: &str) -> String {
    let pretty = pretty_hotkey(hotkey);
    let lower = err.to_ascii_lowercase();

    if lower.contains("already registered")
        || lower.contains("already in use")
        || err.contains("HotKey {")
    {
        return format!("{pretty} is already registered by another app. Choose a different hotkey.");
    }

    if lower.contains("invalid") || lower.contains("unsupported") {
        return format!("{pretty} is not a valid global hotkey on this system.");
    }

    format!("Could not register {pretty}. {err}")
}

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
                message: humanize_registration_error(&configured, &err),
            };
            *state.hotkey_availability.lock().unwrap() = HotkeyAvailability::Unavailable;
            *state.hotkey_warning.lock().unwrap() = Some(warning.clone());
            emit_hotkey_warning(app, &warning, HotkeyWarningSource::Startup, true);
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
                hotkey: requested_canonical.clone(),
                message: humanize_registration_error(&requested_canonical, &err),
            };
            *state
                .hotkey_warning
                .lock()
                .map_err(|e| e.to_string())? = Some(warning.clone());

            emit_hotkey_warning(app, &warning, HotkeyWarningSource::Save, true);

            Ok(current_config)
        }
    }
}

pub fn handle_shortcut_event<R: Runtime>(app: &AppHandle<R>, event: ShortcutState) {
    if event == ShortcutState::Pressed {
        toggle_recording_state(app);
    }
}

pub(crate) fn next_recording_state(current: &RecordingState) -> Option<RecordingState> {
    match current {
        RecordingState::Idle => Some(RecordingState::Recording),
        RecordingState::Recording => Some(RecordingState::Transcribing),
        RecordingState::Transcribing => None,
    }
}

pub fn toggle_recording_state<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let previous_state = {
        let mut recording_state = state.recording_state.lock().unwrap();
        let previous = recording_state.clone();
        if let Some(next) = next_recording_state(&previous) {
            *recording_state = next;
        } else {
            return;
        }
        previous
    };

    let next_state = state.recording_state.lock().unwrap().clone();
    tray::update_recording_menu(app, next_state.clone());

    if previous_state == RecordingState::Idle {
        if let Err(err) = audio::start_recording(app) {
            log::warn!("Failed to start recording: {err}");
            *state.recording_state.lock().unwrap() = RecordingState::Idle;
            tray::update_recording_menu(app, RecordingState::Idle);
        }
        return;
    }

    if previous_state == RecordingState::Recording {
        if let Err(err) = audio::stop_recording_and_encode(app) {
            log::warn!("Failed to finalize recording: {err}");
        }
        complete_transcription_placeholder(app);
    }
}

fn complete_transcription_placeholder<R: Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<AppState>() {
        *state.recording_state.lock().unwrap() = RecordingState::Idle;
    }
    tray::update_recording_menu(app, RecordingState::Idle);
}

fn emit_hotkey_warning<R: Runtime>(
    app: &AppHandle<R>,
    warning: &HotkeyWarning,
    source: HotkeyWarningSource,
    focus_settings: bool,
) {
    if focus_settings {
        tray::show_settings_window(app);
    }
    let payload = HotkeyWarningPayload {
        hotkey: warning.hotkey.clone(),
        message: warning.message.clone(),
        source,
    };
    let _ = app.emit(HOTKEY_WARNING_EVENT, payload);
}

fn persist_config(state: &AppState, config: AppConfig) -> Result<(), String> {
    let mut raw = state.config_raw.lock().map_err(|e| e.to_string())?;
    persistence::save(&config, &mut raw)?;

    let mut current = state.config.lock().map_err(|e| e.to_string())?;
    *current = config;
    Ok(())
}
