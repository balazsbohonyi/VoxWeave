use crate::audio;
use crate::config::{persistence, AppConfig};
use crate::transcription;
use crate::transcription::service::{TranscriptionErrorCode, TranscriptionErrorPayload};
use crate::hotkey::normalize::normalize_hotkey;
use crate::indicator;
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
            emit_hotkey_warning(app, &warning, HotkeyWarningSource::Startup, false);
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

            emit_hotkey_warning(app, &warning, HotkeyWarningSource::Save, false);

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

    // Hotkey pressed while transcribing or injecting: cancel in-flight work.
    // The injection/transcription task polls cancel_flag and will clean up state itself.
    {
        let recording_state = state.recording_state.lock().unwrap();
        if *recording_state == RecordingState::Transcribing {
            *state.cancel_flag.lock().unwrap() = true;
            return;
        }
    }

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
        // Capture foreground window BEFORE indicator shows so we record the
        // correct target window (INJC-10). The indicator show must not steal focus.
        {
            let platform = app.state::<crate::platform::PlatformProvider>();
            use crate::platform::WindowInfo as _;
            let fw = platform.get_foreground_window();
            *state.foreground_window.lock().unwrap() = fw;
        }
        if let Err(err) = indicator::show_recording(app) {
            log::warn!("Failed to show indicator: {err}");
        }
        if let Err(err) = audio::start_recording(app) {
            log::warn!("Failed to start recording: {err}");
            *state.recording_state.lock().unwrap() = RecordingState::Idle;
            tray::update_recording_menu(app, RecordingState::Idle);
            indicator::hide(app);
            return;
        }
        return;
    }

    if previous_state == RecordingState::Recording {
        match audio::stop_recording_and_encode(app) {
            Err(err) => {
                log::warn!("Failed to finalize recording: {err}");
                let state = app.state::<AppState>();
                *state.recording_state.lock().unwrap() = RecordingState::Idle;
                tray::update_recording_menu(app, RecordingState::Idle);
                // Show the error as a toast in the indicator overlay.
                // show_toast_window positions, shows, and hides the indicator itself.
                // Use TranscriptionErrorPayload because that is what the toast JS expects.
                let payload = TranscriptionErrorPayload {
                    code: TranscriptionErrorCode::TooShort,
                    message: err.clone(),
                    provider: None,
                    fallback_provider: None,
                    retryable: false,
                };
                if let Err(toast_err) = indicator::show_toast_window(app, &payload) {
                    log::warn!("Failed to show error toast: {toast_err}");
                    // Fallback: hide the indicator so it is not left dangling.
                    indicator::hide(app);
                }
            }
            Ok(encoded) => {
                // Store audio for retry commands before spawning
                {
                    let state = app.state::<AppState>();
                    *state.last_encoded_audio.lock().unwrap() = Some(encoded.clone());
                }
                // Show processing state on indicator while transcription runs
                indicator::show_processing(app);
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    match transcription::transcribe_with_retry(&app_clone, &encoded).await {
                        Ok(text) => {
                            // Show injecting state while injection runs
                            indicator::show_injecting(&app_clone);

                            // Extract all needed data from AppState BEFORE spawn_blocking
                            // (never hold MutexGuard across thread boundary)
                            let fw_info = app_clone
                                .state::<crate::state::AppState>()
                                .foreground_window
                                .lock()
                                .unwrap()
                                .clone();
                            let injection_config = app_clone
                                .state::<crate::state::AppState>()
                                .config
                                .lock()
                                .unwrap()
                                .injection
                                .clone();
                            let cancel_flag = app_clone
                                .state::<crate::state::AppState>()
                                .cancel_flag
                                .clone();
                            let app_for_inject = app_clone.clone();

                            tauri::async_runtime::spawn(async move {
                                // Use tauri::async_runtime::spawn_blocking (not tokio::task::spawn_blocking)
                                // so blocking injection work does not block the Tokio runtime.
                                let result = tauri::async_runtime::spawn_blocking({
                                    let app_inner = app_for_inject.clone();
                                    let text_clone = text.clone();
                                    let fw_clone = fw_info.clone();
                                    let cfg_clone = injection_config.clone();
                                    let cancel_clone = cancel_flag.clone();
                                    move || {
                                        // Retrieve PlatformProvider from Tauri managed state
                                        // inside the closure. AppHandle is Clone + Send + 'static.
                                        let platform = app_inner
                                            .state::<crate::platform::PlatformProvider>();
                                        // WindowsProvider implements all four platform traits directly.
                                        let p: &crate::platform::PlatformProvider = &*platform;
                                        crate::injection::inject_text(
                                            p,
                                            p,
                                            p,
                                            p,
                                            fw_clone.as_ref(),
                                            &text_clone,
                                            &cfg_clone,
                                            cancel_clone,
                                        )
                                    }
                                })
                                .await
                                .unwrap_or(Err(crate::injection::InjectionErrorPayload {
                                    code: crate::injection::InjectionErrorCode::AllMethodsFailed,
                                    message: "Injection task panicked".to_string(),
                                    typed_chars: None,
                                    total_chars: None,
                                }));

                                match result {
                                    Ok(crate::injection::InjectionResult::Ok) => {
                                        // Green success flash for ~1 second then hide (INJC-08)
                                        indicator::show_success(&app_for_inject);
                                        tokio::time::sleep(
                                            std::time::Duration::from_millis(1000),
                                        )
                                        .await;
                                        indicator::hide(&app_for_inject);
                                    }
                                    Ok(crate::injection::InjectionResult::CopiedToClipboard) => {
                                        // Elevation dialog: user chose "Copy to clipboard".
                                        // Show info toast, NOT the green success flash.
                                        indicator::hide(&app_for_inject);
                                        let plain_toast = serde_json::json!({
                                            "type": "info",
                                            "message": "Copied to clipboard \u{2014} paste manually"
                                        });
                                        if let Err(e) =
                                            indicator::show_toast_window(&app_for_inject, &plain_toast)
                                        {
                                            log::warn!("Failed to show clipboard toast: {e}");
                                        }
                                    }
                                    Ok(crate::injection::InjectionResult::Cancelled {
                                        typed,
                                        total,
                                    }) => {
                                        // Show cancelled info toast with char counts
                                        let payload = crate::injection::InjectionErrorPayload {
                                            code: crate::injection::InjectionErrorCode::Cancelled,
                                            message: format!(
                                                "Cancelled \u{2014} {typed} of {total} chars typed"
                                            ),
                                            typed_chars: Some(typed),
                                            total_chars: Some(total),
                                        };
                                        indicator::hide(&app_for_inject);
                                        if let Err(e) =
                                            indicator::show_toast_window(&app_for_inject, &payload)
                                        {
                                            log::warn!("Failed to show cancel toast: {e}");
                                        }
                                    }
                                    Ok(crate::injection::InjectionResult::Err(msg)) => {
                                        log::warn!("Injection error (non-payload): {msg}");
                                        indicator::hide(&app_for_inject);
                                    }
                                    Err(payload) => {
                                        indicator::hide(&app_for_inject);
                                        if let Err(e) =
                                            indicator::show_toast_window(&app_for_inject, &payload)
                                        {
                                            log::warn!("Failed to show injection error toast: {e}");
                                        }
                                    }
                                }

                                // Injection complete — reset recording state here so the
                                // Transcribing state stays live during injection and the hotkey
                                // cancel path (which checks for Transcribing) works correctly.
                                if let Some(st) = app_for_inject.try_state::<AppState>() {
                                    *st.recording_state.lock().unwrap() = RecordingState::Idle;
                                }
                                tray::update_recording_menu(&app_for_inject, RecordingState::Idle);
                            });
                            // Return early so the outer spawn's unconditional reset below is
                            // skipped — the inner spawn owns the reset for the injection path.
                            return;
                        }
                        Err(()) => {
                            // show_toast_window already showed the toast and hid the indicator.
                            // Only restore idle indicator if the toast is not currently visible
                            // (e.g. cancelled errors skip the toast entirely).
                            let toast_visible = app_clone
                                .get_webview_window("toast")
                                .and_then(|w| w.is_visible().ok())
                                .unwrap_or(false);
                            if !toast_visible {
                                let _ = indicator::show_idle(&app_clone);
                            }
                        }
                    }
                    // Always reset state after transcription attempt completes
                    if let Some(state) = app_clone.try_state::<AppState>() {
                        *state.recording_state.lock().unwrap() = RecordingState::Idle;
                    }
                    tray::update_recording_menu(&app_clone, RecordingState::Idle);
                });
            }
        }
    }
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
