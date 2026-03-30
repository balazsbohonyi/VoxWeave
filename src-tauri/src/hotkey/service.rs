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
        // Pre-recording guard: if Local provider is selected but no model is
        // available, emit an error toast immediately and never start audio capture.
        // This prevents the indicator from entering processing state when transcription
        // would fail anyway (Issue 6).
        {
            let cfg = state.config.lock().unwrap();
            if cfg.transcription.provider == crate::config::TranscriptionProvider::Local {
                let model_path = cfg.transcription.providers.local.model_path.clone();
                let has_model = model_path.as_deref().map(|p| {
                    if p.is_empty() { return false; }
                    let path = std::path::Path::new(p);
                    if path.is_absolute() {
                        path.exists()
                    } else {
                        // Legacy relative path (e.g. "ggml-tiny.bin") — reconstruct absolute
                        p.strip_prefix("ggml-")
                            .and_then(|s| s.strip_suffix(".bin"))
                            .and_then(|id| crate::transcription::download::model_file_path(id).ok())
                            .map(|abs| abs.exists())
                            .unwrap_or(false)
                    }
                }).unwrap_or(false);
                if !has_model {
                    drop(cfg);
                    // Reset state back to Idle (it was advanced to Recording above)
                    *state.recording_state.lock().unwrap() = RecordingState::Idle;
                    tray::update_recording_menu(app, RecordingState::Idle);
                    let payload = TranscriptionErrorPayload {
                        code: TranscriptionErrorCode::ModelMissing,
                        message: "No local model downloaded.".to_string(),
                        provider: Some("local".to_string()),
                        fallback_provider: None,
                        retryable: false,
                    };
                    if let Err(e) = indicator::show_toast_window(app, &payload) {
                        log::warn!("Failed to show model-missing toast: {e}");
                    }
                    return;
                }
            }
        }

        // Reset cancel flag so a stale true from a previous cancel does not
        // abort the new transcription before it starts.
        *state.cancel_flag.lock().unwrap() = false;
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

                                // Reset to Idle immediately after injection so the hotkey handler
                                // accepts a new recording during the toast window.
                                if let Some(st) = app_for_inject.try_state::<AppState>() {
                                    *st.recording_state.lock().unwrap() = RecordingState::Idle;
                                }
                                tray::update_recording_menu(&app_for_inject, RecordingState::Idle);

                                match result {
                                    Ok(crate::injection::InjectionResult::Ok) => {
                                        // Green success flash for ~1 second, transition to idle,
                                        // then show toast without hiding indicator (INJC-08, NOTF-01).
                                        indicator::show_success(&app_for_inject);
                                        tokio::time::sleep(
                                            std::time::Duration::from_millis(1000),
                                        )
                                        .await;
                                        let _ = indicator::show_idle(&app_for_inject);
                                        let label = injection_success_label(&injection_config.mode);
                                        let payload = serde_json::json!({
                                            "type": "success",
                                            "message": label
                                        });
                                        if let Err(e) = indicator::show_toast_window_keep_indicator(
                                            &app_for_inject,
                                            &payload,
                                        ) {
                                            log::warn!("Failed to show success toast: {e}");
                                        }
                                        tokio::time::sleep(
                                            std::time::Duration::from_millis(10000),
                                        )
                                        .await;
                                        // Only hide if a new recording has not started.
                                        let is_idle = app_for_inject
                                            .try_state::<AppState>()
                                            .map(|s| {
                                                *s.recording_state.lock().unwrap()
                                                    == RecordingState::Idle
                                            })
                                            .unwrap_or(true);
                                        if is_idle {
                                            indicator::hide(&app_for_inject);
                                        }
                                        if let Some(tw) =
                                            app_for_inject.get_webview_window("toast")
                                        {
                                            let _ = tw.hide();
                                        }
                                    }
                                    Ok(crate::injection::InjectionResult::CopiedToClipboard) => {
                                        // Elevation dialog: user chose "Copy to clipboard".
                                        // Show success toast without hiding indicator (NOTF-02).
                                        indicator::show_success(&app_for_inject);
                                        tokio::time::sleep(
                                            std::time::Duration::from_millis(1000),
                                        )
                                        .await;
                                        let _ = indicator::show_idle(&app_for_inject);
                                        let payload = serde_json::json!({
                                            "type": "success",
                                            "message": "Copied to clipboard"
                                        });
                                        if let Err(e) = indicator::show_toast_window_keep_indicator(
                                            &app_for_inject,
                                            &payload,
                                        ) {
                                            log::warn!("Failed to show clipboard toast: {e}");
                                        }
                                        tokio::time::sleep(
                                            std::time::Duration::from_millis(10000),
                                        )
                                        .await;
                                        let is_idle = app_for_inject
                                            .try_state::<AppState>()
                                            .map(|s| {
                                                *s.recording_state.lock().unwrap()
                                                    == RecordingState::Idle
                                            })
                                            .unwrap_or(true);
                                        if is_idle {
                                            indicator::hide(&app_for_inject);
                                        }
                                        if let Some(tw) =
                                            app_for_inject.get_webview_window("toast")
                                        {
                                            let _ = tw.hide();
                                        }
                                    }
                                    Ok(crate::injection::InjectionResult::Cancelled {
                                        typed,
                                        total,
                                    }) => {
                                        // Keep indicator visible — show info toast (NOTF-03).
                                        let message = injection_cancel_message(typed, total);
                                        let payload = serde_json::json!({
                                            "type": "info",
                                            "message": message
                                        });
                                        if let Err(e) = indicator::show_toast_window_keep_indicator(
                                            &app_for_inject,
                                            &payload,
                                        ) {
                                            log::warn!("Failed to show cancel toast: {e}");
                                        }
                                        // Return indicator to neutral state — clears the Injecting visual.
                                        indicator::show_idle_visual(&app_for_inject);
                                        tokio::time::sleep(
                                            std::time::Duration::from_millis(10000),
                                        )
                                        .await;
                                        let is_idle = app_for_inject
                                            .try_state::<AppState>()
                                            .map(|s| {
                                                *s.recording_state.lock().unwrap()
                                                    == RecordingState::Idle
                                            })
                                            .unwrap_or(true);
                                        if is_idle {
                                            indicator::hide(&app_for_inject);
                                        }
                                        if let Some(tw) =
                                            app_for_inject.get_webview_window("toast")
                                        {
                                            let _ = tw.hide();
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

/// Maps injection mode to user-facing success toast label.
pub(crate) fn injection_success_label(mode: &crate::config::InjectionMode) -> &'static str {
    match mode {
        crate::config::InjectionMode::FlashPaste => "Text pasted",
        crate::config::InjectionMode::Keystroke => "Text typed",
        crate::config::InjectionMode::Clipboard => "Copied to clipboard",
    }
}

/// Formats the cancel toast message based on how many chars were typed.
pub(crate) fn injection_cancel_message(typed: usize, total: usize) -> String {
    if typed == 0 {
        "Paste cancelled".to_string()
    } else {
        format!("Cancelled \u{2014} {typed} of {total} chars typed")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::InjectionMode;

    #[test]
    fn success_toast_label() {
        assert_eq!(injection_success_label(&InjectionMode::FlashPaste), "Text pasted");
        assert_eq!(injection_success_label(&InjectionMode::Keystroke), "Text typed");
        assert_eq!(injection_success_label(&InjectionMode::Clipboard), "Copied to clipboard");
    }

    #[test]
    fn cancel_toast_message() {
        // typed == 0, total == 0 → generic message
        assert_eq!(injection_cancel_message(0, 0), "Paste cancelled");
        // typed > 0 → detailed message with em-dash
        assert_eq!(
            injection_cancel_message(5, 20),
            "Cancelled \u{2014} 5 of 20 chars typed"
        );
        // typed == 0 but total > 0 → "Paste cancelled" (guard is typed == 0, not total == 0)
        assert_eq!(
            injection_cancel_message(0, 10),
            "Paste cancelled"
        );
    }
}
