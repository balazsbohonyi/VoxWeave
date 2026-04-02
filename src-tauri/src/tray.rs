// Tray icon setup and menu event handling.
// The tray is the primary app chrome - no main window opens on launch.

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, PhysicalPosition, Runtime,
};

const TRAY_ID: &str = "main";

pub fn setup_tray(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(app)?;

    // Load the tray icon from the bundled icon
    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("VoxWeave - Voice to text dictation")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(handle_tray_event)
        .build(app)?;

    Ok(())
}

fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "open_settings" => show_settings_window(app),
        "quit" => {
            // Signal quit intent so close-to-hide handlers let windows destroy.
            // Exit is driven by the settings window's Destroyed event in lib.rs
            // so WebView2 has fully torn down before the process exits.
            if let Some(state) = app.try_state::<crate::state::AppState>() {
                *state.quitting.lock().unwrap() = true;
            }
            for label in ["settings", "indicator", "toast", "wizard"] {
                if let Some(win) = app.get_webview_window(label) {
                    let _ = win.close();
                }
            }
        }
        _ => {}
    }
}

fn handle_tray_event<R: Runtime>(tray: &tauri::tray::TrayIcon<R>, event: TrayIconEvent) {
    // Double-click on tray opens or focuses settings window.
    if let TrayIconEvent::DoubleClick {
        button: MouseButton::Left,
        ..
    } = event
    {
        show_settings_window(tray.app_handle());
    }
}

/// Show the single settings window, creating it if hidden, focusing if already visible.
/// Restores the last user-set position for the current session; centers on first open.
pub fn show_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        if let Some((x, y)) = *app.state::<crate::state::AppState>().settings_position.lock().unwrap() {
            let _ = window.set_position(PhysicalPosition::new(x, y));
        }
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn build_tray_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
) -> Result<Menu<R>, Box<dyn std::error::Error>> {
    let open_settings = MenuItem::with_id(manager, "open_settings", "Settings", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(manager)?;
    let quit = MenuItem::with_id(manager, "quit", "Quit VoxWeave", true, None::<&str>)?;

    let menu = Menu::with_items(manager, &[&open_settings, &sep1, &quit])?;
    Ok(menu)
}

pub fn update_recording_menu<R: Runtime>(app: &AppHandle<R>) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Ok(menu) = build_tray_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}
