// Tray icon setup and menu event handling.
// The tray is the primary app chrome - no main window opens on launch.

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Runtime,
};

const TRAY_ID: &str = "main";
const START_STOP_ID: &str = "start_stop_recording";

pub fn setup_tray(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let recording_state = app
        .state::<crate::state::AppState>()
        .recording_state
        .lock()
        .unwrap()
        .clone();
    let menu = build_tray_menu(app, recording_state)?;

    // Load the tray icon from the bundled icon
    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("VoxFlow - Voice to text dictation")
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
            // destroy rather than hiding - WebView2 must tear down before exit.
            if let Some(state) = app.try_state::<crate::state::AppState>() {
                *state.quitting.lock().unwrap() = true;
            }
            if let Some(win) = app.get_webview_window("settings") {
                let _ = win.close();
            }
            if let Some(win) = app.get_webview_window("indicator") {
                let _ = win.close();
            }
            let app_for_exit = app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(250));
                app_for_exit.cleanup_before_exit();
                app_for_exit.exit(0);
            });
        }
        START_STOP_ID => {
            crate::hotkey::service::toggle_recording_state(app);
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
/// Never creates a second instance - Tauri windows are identified by label.
pub fn show_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn build_tray_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
    recording_state: crate::state::RecordingState,
) -> Result<Menu<R>, Box<dyn std::error::Error>> {
    let open_settings = MenuItem::with_id(manager, "open_settings", "Settings", true, None::<&str>)?;

    let start_stop = MenuItem::with_id(
        manager,
        START_STOP_ID,
        recording_menu_label(&recording_state),
        true,
        None::<&str>,
    )?;

    let sep1 = PredefinedMenuItem::separator(manager)?;
    let quit = MenuItem::with_id(manager, "quit", "Quit VoxFlow", true, None::<&str>)?;

    let menu = Menu::with_items(manager, &[&open_settings, &start_stop, &sep1, &quit])?;
    Ok(menu)
}

fn recording_menu_label(state: &crate::state::RecordingState) -> &'static str {
    match state {
        crate::state::RecordingState::Idle => "Start Recording",
        crate::state::RecordingState::Recording | crate::state::RecordingState::Transcribing => {
            "Stop Recording"
        }
    }
}

pub fn update_recording_menu<R: Runtime>(app: &AppHandle<R>, state: crate::state::RecordingState) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Ok(menu) = build_tray_menu(app, state) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}
