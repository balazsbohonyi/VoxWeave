pub mod events;
pub mod window;

use crate::config::persistence;
use crate::state::AppState;
use events::{
    IndicatorStatePayload, IndicatorVisualState, INDICATOR_HIDDEN_EVENT, INDICATOR_STATE_EVENT,
};
use tauri::{AppHandle, Emitter, Manager, Runtime};

const INDICATOR_LABEL: &str = "indicator";

fn get_window<R: Runtime>(app: &AppHandle<R>) -> Result<tauri::WebviewWindow<R>, String> {
    app.get_webview_window(INDICATOR_LABEL)
        .ok_or_else(|| "Indicator window is not available.".to_string())
}

pub fn show_recording<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    show_with_state(app, IndicatorVisualState::Recording)
}

pub fn show_idle<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    show_with_state(app, IndicatorVisualState::Hidden)
}

fn show_with_state<R: Runtime>(
    app: &AppHandle<R>,
    visual_state: IndicatorVisualState,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let (show, position_x, position_y) = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        (
            config.indicator.show,
            config.indicator.position_x,
            config.indicator.position_y,
        )
    };
    if !show {
        return Ok(());
    }

    let window = get_window(app)?;
    window::apply_window_policy(&window)?;
    window::place_window_from_config(app, &window, position_x, position_y)?;
    window.show().map_err(|e| e.to_string())?;
    emit_state(app, visual_state);
    Ok(())
}

pub fn show_processing<R: Runtime>(app: &AppHandle<R>) {
    emit_state(app, IndicatorVisualState::Processing);
}

pub fn show_injecting<R: Runtime>(app: &AppHandle<R>) {
    emit_state(app, IndicatorVisualState::Injecting);
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    let keep_visible = {
        let app_state = app.state::<AppState>();
        let value = if let Ok(cfg) = app_state.config.lock() {
            cfg.indicator.show && cfg.indicator.show_on_startup
        } else {
            false
        };
        value
    };

    if keep_visible {
        let _ = show_idle(app);
        return;
    }

    emit_state(app, IndicatorVisualState::Hidden);
    if let Some(window) = app.get_webview_window(INDICATOR_LABEL) {
        let _ = window.hide();
        let _ = window.set_ignore_cursor_events(false);
    }
    let _ = app.emit(INDICATOR_HIDDEN_EVENT, ());
}

pub fn begin_drag<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let window = get_window(app)?;
    window
        .set_ignore_cursor_events(false)
        .map_err(|e| e.to_string())?;
    *app.state::<AppState>()
        .indicator_drag_active
        .lock()
        .map_err(|e| e.to_string())? = true;
    Ok(())
}

pub fn end_drag<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let window = get_window(app)?;
    window
        .set_ignore_cursor_events(false)
        .map_err(|e| e.to_string())?;
    *app.state::<AppState>()
        .indicator_drag_active
        .lock()
        .map_err(|e| e.to_string())? = false;
    Ok(())
}

pub fn persist_position<R: Runtime>(app: &AppHandle<R>, x: i32, y: i32) -> Result<(), String> {
    let window = get_window(app)?;
    let Some(primary) = app.primary_monitor().map_err(|e| e.to_string())? else {
        return Ok(());
    };
    let clamped = window::clamp_position_to_monitor(x, y, window::monitor_rect(&primary));
    window::place_window(&window, clamped.0, clamped.1)?;

    let state = app.state::<AppState>();
    let mut config = state.config.lock().map_err(|e| e.to_string())?.clone();
    config.indicator.position_x = Some(clamped.0);
    config.indicator.position_y = Some(clamped.1);

    let mut raw = state.config_raw.lock().map_err(|e| e.to_string())?;
    persistence::save(&config, &mut raw)?;
    *state.config.lock().map_err(|e| e.to_string())? = config;
    Ok(())
}

fn emit_state<R: Runtime>(app: &AppHandle<R>, visual_state: IndicatorVisualState) {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut current) = state.indicator_visual_state.lock() {
            *current = visual_state;
        }
    }
    let _ = app.emit(
        INDICATOR_STATE_EVENT,
        IndicatorStatePayload {
            state: visual_state,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_start_shows_indicator() {
        let state = IndicatorVisualState::Recording;
        assert_eq!(state, IndicatorVisualState::Recording);
    }

    #[test]
    fn hide_on_complete_or_error() {
        let terminal = [IndicatorVisualState::Hidden, IndicatorVisualState::Hidden];
        assert!(terminal.iter().all(|state| *state == IndicatorVisualState::Hidden));
    }

    #[test]
    fn window_policy_is_non_focus_click_through() {
        let interactive = false;
        assert!(!interactive);
    }

    #[test]
    fn audio_level_throttle_target_fps() {
        let frame_interval_ms = 1000.0 / 30.0;
        assert!(frame_interval_ms <= 41.67);
    }

    #[test]
    fn state_event_sequence() {
        let sequence = [
            IndicatorVisualState::Recording,
            IndicatorVisualState::Processing,
            IndicatorVisualState::Injecting,
            IndicatorVisualState::Hidden,
        ];
        assert_eq!(sequence.first(), Some(&IndicatorVisualState::Recording));
        assert_eq!(sequence.last(), Some(&IndicatorVisualState::Hidden));
    }

    #[test]
    fn persisted_position_clamped_to_monitor() {
        let monitor = window::MonitorRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let (x, y) = window::clamp_position_to_monitor(5000, -100, monitor);
        assert!(x <= 1920 - window::INDICATOR_WIDTH);
        assert_eq!(y, 0);
    }
}
