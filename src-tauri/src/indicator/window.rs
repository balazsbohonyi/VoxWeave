use tauri::{AppHandle, LogicalPosition, Monitor, Position, Runtime, WebviewWindow};

pub const INDICATOR_WIDTH: i32 = 180;
pub const INDICATOR_HEIGHT: i32 = 70;
const EDGE_MARGIN_X: i32 = 20;
const EDGE_MARGIN_Y: i32 = 92;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn apply_window_policy<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    window.set_always_on_top(true).map_err(|e| e.to_string())?;
    #[cfg(desktop)]
    let _ = window.set_shadow(false);
    window
        .set_ignore_cursor_events(false)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn monitor_rect(monitor: &Monitor) -> MonitorRect {
    let scale = monitor.scale_factor();
    let pos = monitor.position().to_logical::<f64>(scale);
    let size = monitor.size().to_logical::<f64>(scale);
    MonitorRect {
        x: pos.x.round() as i32,
        y: pos.y.round() as i32,
        width: size.width.round() as i32,
        height: size.height.round() as i32,
    }
}

pub fn clamp_position_to_monitor(x: i32, y: i32, monitor: MonitorRect) -> (i32, i32) {
    let min_x = monitor.x;
    let min_y = monitor.y;
    let max_x = monitor.x + (monitor.width - INDICATOR_WIDTH).max(0);
    let max_y = monitor.y + (monitor.height - INDICATOR_HEIGHT).max(0);
    (x.clamp(min_x, max_x), y.clamp(min_y, max_y))
}

pub fn fallback_bottom_right(monitor: MonitorRect) -> (i32, i32) {
    let x = monitor.x + (monitor.width - INDICATOR_WIDTH - EDGE_MARGIN_X).max(0);
    let y = monitor.y + (monitor.height - INDICATOR_HEIGHT - EDGE_MARGIN_Y).max(0);
    (x, y)
}

pub fn resolve_position(
    monitor: MonitorRect,
    position_x: Option<i32>,
    position_y: Option<i32>,
) -> (i32, i32) {
    match (position_x, position_y) {
        (Some(x), Some(y)) => clamp_position_to_monitor(x, y, monitor),
        _ => fallback_bottom_right(monitor),
    }
}

pub fn place_window<R: Runtime>(window: &WebviewWindow<R>, x: i32, y: i32) -> Result<(), String> {
    window
        .set_position(Position::Logical(LogicalPosition::new(
            f64::from(x),
            f64::from(y),
        )))
        .map_err(|e| e.to_string())
}

pub fn place_window_from_config<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    position_x: Option<i32>,
    position_y: Option<i32>,
) -> Result<(), String> {
    let Some(primary) = app.primary_monitor().map_err(|e| e.to_string())? else {
        return Ok(());
    };
    let rect = monitor_rect(&primary);
    let (x, y) = resolve_position(rect, position_x, position_y);
    place_window(window, x, y)
}
