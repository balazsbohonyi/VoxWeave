mod audio;
mod commands;
mod config;
mod hotkey;
mod indicator;
mod injection;
mod platform;
mod state;
mod transcription;
mod tray;

use platform::PlatformProvider;
use state::AppState;
use tauri::{Manager, WindowEvent};
use tauri_plugin_log::{Builder as LogBuilder, Target, TargetKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    tauri::Builder::default()
        .plugin(
            LogBuilder::new()
                .level(log_level)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // Managed state — single source of truth across commands and tray
            let app_state = AppState::load();
            app.manage(app_state);
            app.manage(PlatformProvider::default());

            // Global shortcut plugin (hotkey runtime relies on this in Phase 2).
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;

            // Build tray icon; tray drives all window visibility (tray-first bootstrap).
            tray::setup_tray(app)?;

            // Register the startup hotkey from config.
            hotkey::service::register_startup_hotkey(&app.handle());

            // Settings window is defined in tauri.conf.json with `visible: false`.
            // Register the close-to-hide handler so titlebar close hides rather than destroys.
            if let Some(settings_win) = app.get_webview_window("settings") {
                let win_clone = settings_win.clone();
                let quitting = app.state::<AppState>().quitting.clone();
                settings_win.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        if *quitting.lock().unwrap() {
                            // App is quitting — allow WebView2 to destroy cleanly.
                            return;
                        }
                        // Normal close: hide instead of destroy so the window is reusable.
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            // Indicator window is pre-defined in tauri.conf and starts hidden.
            // Enforce non-focus/click-through defaults. Toast is now a separate window
            // so there is no expanded height to reset — never call resize_window here
            // (set_resizable(true) triggers WS_THICKFRAME minimum size enforcement on Windows).
            if let Some(indicator_win) = app.get_webview_window("indicator") {
                let _ = indicator::window::apply_window_policy(&indicator_win);
            }

            // Optional startup visibility (default true) so users can keep the
            // indicator pinned even before recording starts.
            let show_on_startup = {
                let app_state = app.state::<AppState>();
                let cfg = app_state.config.lock().unwrap();
                cfg.indicator.show && cfg.indicator.show_on_startup
            };
            if show_on_startup {
                let _ = indicator::show_idle(&app.handle());
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::save_config,
            commands::config::get_provider_models,
            commands::config::test_connection,
            commands::config::set_launch_at_login,
            commands::audio::list_audio_input_devices,
            commands::indicator::begin_indicator_drag,
            commands::indicator::end_indicator_drag,
            commands::indicator::persist_indicator_position,
            commands::indicator::get_indicator_state,
            commands::indicator::get_recording_state,
            commands::indicator::toggle_recording_from_indicator,
            commands::indicator::hide_indicator,
            commands::transcription::retry_transcription,
            commands::transcription::retry_transcription_with_fallback,
            commands::transcription::open_settings_on_transcription_tab,
            commands::indicator::hide_toast_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VoxFlow");
}
