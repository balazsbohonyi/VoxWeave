// Tray icon setup and menu event handling.
// The tray is the primary app chrome — no main window opens on launch.

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Runtime,
};

pub fn setup_tray(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let open_settings = MenuItem::with_id(app, "open_settings", "Settings", true, None::<&str>)?;

    // Recording item is disabled until Phase 3 implements the recording state machine.
    let start_stop = MenuItem::with_id(
        app,
        "start_stop_recording",
        "Start / Stop Recording",
        false, // disabled — placeholder only
        None::<&str>,
    )?;

    let sep1 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit VoxFlow", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&open_settings, &start_stop, &sep1, &quit])?;

    // Load the tray icon from the bundled icon
    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("VoxFlow — voice-to-text")
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
            // Signal quit intent so the close-to-hide handler lets the window
            // destroy rather than hiding — WebView2 must tear down before exit.
            if let Some(state) = app.try_state::<crate::state::AppState>() {
                *state.quitting.lock().unwrap() = true;
            }
            if let Some(win) = app.get_webview_window("settings") {
                let _ = win.close();
            }
            let app_for_exit = app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(120));
                app_for_exit.cleanup_before_exit();
                app_for_exit.exit(0);
            });
        }
        // "start_stop_recording" is disabled; no action needed.
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
/// Never creates a second instance — Tauri windows are identified by label.
pub fn show_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
