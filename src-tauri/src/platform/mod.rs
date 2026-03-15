// Platform abstraction layer.
// All OS-specific code lives behind these traits. Core logic (injection
// pipeline, hotkey handler, etc.) depends ONLY on these traits — never on
// concrete types. Adding macOS requires only new trait implementations.

pub mod windows;

/// Information about the window that currently holds focus.
#[derive(Debug, Clone)]
pub struct ForegroundWindowInfo {
    /// Raw OS window handle (cast to usize for cross-platform carry).
    pub handle: usize,
    /// Window class name (e.g. "CASCADIA_HOSTING_WINDOW_CLASS" for Windows Terminal).
    pub class_name: String,
    /// Executable name without path (e.g. "code.exe").
    pub exe_name: String,
    /// Process integrity level as a numeric IL (Windows MANDATORY_LABEL_RID).
    /// 0 on non-Windows or when unavailable.
    pub integrity_level: u32,
}

/// Terminal window class names that need alternative paste shortcuts.
pub const TERMINAL_CLASSES: &[&str] = &[
    "ConsoleWindowClass",
    "CASCADIA_HOSTING_WINDOW_CLASS",
    "mintty",
    "VirtualConsoleClass",
];

// ---------------------------------------------------------------------------
// Trait definitions
// ---------------------------------------------------------------------------

/// Foreground window detection, terminal identification, and focus restore.
pub trait WindowInfo: Send + Sync {
    /// Return information about the currently focused window, or `None` if
    /// no window is focused or the info cannot be determined.
    fn get_foreground_window(&self) -> Option<ForegroundWindowInfo>;

    /// Restore focus to a previously captured window.
    fn restore_focus(&self, info: &ForegroundWindowInfo) -> Result<(), String>;

    /// Return `true` if the window with the given class name is a terminal
    /// that requires `Ctrl+Shift+V` / `Shift+Insert` instead of `Ctrl+V`.
    fn is_terminal(&self, class_name: &str) -> bool {
        TERMINAL_CLASSES.contains(&class_name)
    }
}

/// Process elevation / integrity level checks and relaunch helpers.
pub trait ElevationChecker: Send + Sync {
    /// Return `true` if VoxFlow itself is running elevated (admin).
    fn is_elevated(&self) -> bool;

    /// Return the current process integrity level (Windows MANDATORY_LABEL_RID).
    /// Returns 0 on platforms where this concept does not apply.
    fn current_integrity_level(&self) -> u32;

    /// Relaunch VoxFlow with elevated privileges (ShellExecuteW "runas" on Windows).
    fn relaunch_elevated(&self) -> Result<(), String>;
}

/// Keyboard/mouse input simulation (SendInput on Windows).
pub trait InputSimulator: Send + Sync {
    /// Send a paste shortcut appropriate for the target window class.
    /// Uses `Ctrl+V` for regular windows, `Ctrl+Shift+V` for terminals.
    fn send_paste(&self, class_name: &str) -> Result<(), String>;

    /// Type a string character-by-character via unicode input events.
    fn send_unicode_string(&self, text: &str) -> Result<(), String>;

    /// Send a Return/Enter key press.
    fn send_return(&self) -> Result<(), String>;
}

/// Clipboard read/write via `arboard` (NOT the Tauri clipboard plugin).
/// arboard is used directly to ensure synchronous, timing-safe operations
/// required by the FlashPaste injection mode.
pub trait ClipboardAccess: Send + Sync {
    /// Read the current clipboard text content.
    fn read_text(&self) -> Result<String, String>;

    /// Write text to the clipboard.
    fn write_text(&self, text: &str) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// Platform provider — selects concrete implementations at compile time
// ---------------------------------------------------------------------------

/// The Windows platform provider that implements all platform traits.
/// Future: add `#[cfg(target_os = "macos")]` arm for a macOS provider.
#[cfg(target_os = "windows")]
pub type PlatformProvider = windows::WindowsProvider;
