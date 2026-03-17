use crate::hotkey::service;
use crate::indicator;
use crate::indicator::events::{IndicatorStatePayload, IndicatorVisualState};
use crate::state::{AppState, RecordingState};
use tauri::AppHandle;
use tauri::State;

#[tauri::command]
pub fn begin_indicator_drag(app: AppHandle) -> Result<(), String> {
    indicator::begin_drag(&app)
}

#[tauri::command]
pub fn end_indicator_drag(app: AppHandle) -> Result<(), String> {
    indicator::end_drag(&app)
}

#[tauri::command]
pub fn persist_indicator_position(app: AppHandle, x: i32, y: i32) -> Result<(), String> {
    indicator::persist_position(&app, x, y)
}

#[tauri::command]
pub fn get_indicator_state(state: State<AppState>) -> Result<IndicatorStatePayload, String> {
    let current = *state
        .indicator_visual_state
        .lock()
        .map_err(|e| e.to_string())?;
    Ok(IndicatorStatePayload {
        state: match current {
            IndicatorVisualState::Recording => IndicatorVisualState::Recording,
            IndicatorVisualState::Processing => IndicatorVisualState::Processing,
            IndicatorVisualState::Injecting => IndicatorVisualState::Injecting,
            IndicatorVisualState::Hidden => IndicatorVisualState::Hidden,
        },
    })
}

#[tauri::command]
pub fn get_recording_state(state: State<AppState>) -> Result<RecordingState, String> {
    let current = state
        .recording_state
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    Ok(current)
}

#[tauri::command]
pub fn toggle_recording_from_indicator(app: AppHandle) -> Result<(), String> {
    service::toggle_recording_state(&app);
    Ok(())
}

#[tauri::command]
pub fn hide_indicator(app: AppHandle) -> Result<(), String> {
    indicator::hide(&app);
    Ok(())
}
