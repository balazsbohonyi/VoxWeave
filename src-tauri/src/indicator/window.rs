use tauri::{AppHandle, LogicalPosition, Monitor, Position, Runtime, WebviewWindow};

pub const INDICATOR_WIDTH: i32 = 150;
pub const INDICATOR_HEIGHT: i32 = 38;
pub const TOAST_HEIGHT: i32 = 108;
pub const TOAST_GAP: i32 = 6;
const EDGE_MARGIN_X: i32 = 20;
const EDGE_MARGIN_Y: i32 = 120;

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
    let work_area = monitor.work_area();
    let pos = work_area.position.to_logical::<f64>(scale);
    let size = work_area.size.to_logical::<f64>(scale);
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
        (Some(x), Some(y)) if saved_position_is_valid(x, y, monitor) => {
            clamp_position_to_monitor(x, y, monitor)
        }
        _ => fallback_bottom_right(monitor),
    }
}

fn saved_position_is_valid(x: i32, y: i32, monitor: MonitorRect) -> bool {
    let min_x = monitor.x;
    let min_y = monitor.y;
    let max_x = monitor.x + (monitor.width - INDICATOR_WIDTH).max(0);
    let max_y = monitor.y + (monitor.height - INDICATOR_HEIGHT).max(0);
    x >= min_x && x <= max_x && y >= min_y && y <= max_y
}

/// Determines whether the toast should appear above or below the indicator.
/// Default is below; falls back to above if there is not enough space.
pub fn compute_toast_direction(indicator_y: i32, monitor: MonitorRect) -> &'static str {
    let space_below = (monitor.y + monitor.height) - (indicator_y + INDICATOR_HEIGHT);
    if space_below >= TOAST_GAP + TOAST_HEIGHT {
        "below"
    } else {
        "above"
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
