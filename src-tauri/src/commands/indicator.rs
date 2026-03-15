use crate::indicator;
use tauri::AppHandle;

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

