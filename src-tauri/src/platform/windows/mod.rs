// Windows platform implementations of the platform traits.
// Real OS behavior (GetForegroundWindow, SendInput, ShellExecuteW, arboard)
// is added in later phases. These stubs satisfy the trait seam so the rest
// of the codebase compiles today.

use crate::platform::{
    ClipboardAccess, ElevationChecker, ForegroundWindowInfo, InputSimulator, WindowInfo,
};

/// Bundles all Windows platform trait implementations.
/// Constructed once in `lib.rs` setup and stored in `AppState` (Phase 3+).
pub struct WindowsProvider;

impl WindowsProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// WindowInfo — stub
// ---------------------------------------------------------------------------

impl WindowInfo for WindowsProvider {
    fn get_foreground_window(&self) -> Option<ForegroundWindowInfo> {
        // TODO (Phase 5): implement via GetForegroundWindow + GetClassName +
        // GetWindowThreadProcessId + QueryFullProcessImageName + integrity level query
        None
    }

    fn restore_focus(&self, _info: &ForegroundWindowInfo) -> Result<(), String> {
        // TODO (Phase 5): SetForegroundWindow(info.handle as HWND)
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ElevationChecker — stub
// ---------------------------------------------------------------------------

impl ElevationChecker for WindowsProvider {
    fn is_elevated(&self) -> bool {
        // TODO (Phase 5): OpenProcessToken + GetTokenInformation(TokenElevation)
        false
    }

    fn current_integrity_level(&self) -> u32 {
        // TODO (Phase 5): GetTokenInformation(TokenIntegrityLevel)
        0
    }

    fn relaunch_elevated(&self) -> Result<(), String> {
        // TODO (Phase 5): ShellExecuteW with "runas" verb
        Err("relaunch_elevated not yet implemented".to_string())
    }
}

// ---------------------------------------------------------------------------
// InputSimulator — stub
// ---------------------------------------------------------------------------

impl InputSimulator for WindowsProvider {
    fn send_paste(&self, _class_name: &str) -> Result<(), String> {
        // TODO (Phase 5): SendInput with VK_CONTROL + VK_V (or Ctrl+Shift+V for terminals)
        Err("send_paste not yet implemented".to_string())
    }

    fn send_unicode_string(&self, _text: &str) -> Result<(), String> {
        // TODO (Phase 5): SendInput with KEYEVENTF_UNICODE for each char
        Err("send_unicode_string not yet implemented".to_string())
    }

    fn send_return(&self) -> Result<(), String> {
        // TODO (Phase 5): SendInput with VK_RETURN
        Err("send_return not yet implemented".to_string())
    }
}

// ---------------------------------------------------------------------------
// ClipboardAccess — stub
// ---------------------------------------------------------------------------

impl ClipboardAccess for WindowsProvider {
    fn read_text(&self) -> Result<String, String> {
        // TODO (Phase 5): arboard::Clipboard::new()?.get_text()
        Err("read_text not yet implemented".to_string())
    }

    fn write_text(&self, _text: &str) -> Result<(), String> {
        // TODO (Phase 5): arboard::Clipboard::new()?.set_text(text)
        Err("write_text not yet implemented".to_string())
    }
}
