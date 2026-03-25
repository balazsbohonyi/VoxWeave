use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn open_wizard_window(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("wizard") {
        // Re-center on every open so it appears correctly after hiding/re-opening.
        win.center().map_err(|e| e.to_string())?;
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_wizard_window(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("wizard") {
        win.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}
