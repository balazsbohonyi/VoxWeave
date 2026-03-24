use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn open_wizard_window(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("wizard") {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}
