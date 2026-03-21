// Windows platform implementations of the platform traits.
// Real OS behavior (GetForegroundWindow, SendInput, ShellExecuteW, arboard).

use std::time::Duration;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::Security::{
    GetTokenInformation, GetSidSubAuthority, GetSidSubAuthorityCount,
    TokenElevation, TokenIntegrityLevel,
    TOKEN_ELEVATION, TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, OpenProcess, OpenProcessToken,
    QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
    SW_SHOWNORMAL,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    VIRTUAL_KEY, VK_CONTROL, VK_RETURN, VK_SHIFT, VK_V,
};

use crate::platform::{
    ClipboardAccess, ElevationChecker, ForegroundWindowInfo, InputSimulator, WindowInfo,
};

/// Bundles all Windows platform trait implementations.
/// Constructed once in `lib.rs` setup and stored in `AppState`.
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
// Private helper — query integrity level for a given PID
// ---------------------------------------------------------------------------

/// Read the mandatory integrity level SID sub-authority RID for the given PID.
/// Returns None if any Win32 call fails.
fn query_integrity_level(pid: u32) -> Option<u32> {
    unsafe {
        // Open the process
        let hproc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

        // Open the process token
        let mut htoken = HANDLE::default();
        let token_ok = OpenProcessToken(hproc, TOKEN_QUERY, &mut htoken);
        let _ = CloseHandle(hproc);
        token_ok.ok()?;

        // Query token information size
        let mut return_length: u32 = 0;
        let _ = GetTokenInformation(
            htoken,
            TokenIntegrityLevel,
            None,
            0,
            &mut return_length,
        );

        if return_length == 0 {
            let _ = CloseHandle(htoken);
            return None;
        }

        // Allocate buffer and query
        let mut buffer: Vec<u8> = vec![0u8; return_length as usize];
        let query_ok = GetTokenInformation(
            htoken,
            TokenIntegrityLevel,
            Some(buffer.as_mut_ptr() as *mut _),
            return_length,
            &mut return_length,
        );
        let _ = CloseHandle(htoken);
        query_ok.ok()?;

        // Interpret the TOKEN_MANDATORY_LABEL
        let tml = &*(buffer.as_ptr() as *const TOKEN_MANDATORY_LABEL);
        let sid = tml.Label.Sid;
        let sub_count = *GetSidSubAuthorityCount(sid) as u32;
        if sub_count == 0 {
            return None;
        }
        let rid = *GetSidSubAuthority(sid, sub_count - 1);
        Some(rid)
    }
}

// ---------------------------------------------------------------------------
// WindowInfo
// ---------------------------------------------------------------------------

impl WindowInfo for WindowsProvider {
    fn get_foreground_window(&self) -> Option<ForegroundWindowInfo> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == std::ptr::null_mut() {
                return None;
            }

            // Get class name
            let mut class_buf = [0u16; 256];
            let class_len = GetClassNameW(hwnd, &mut class_buf) as usize;
            let class_name = String::from_utf16_lossy(&class_buf[..class_len]);

            // Get PID
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));

            // Open process and query exe name
            let hproc =
                OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

            let mut exe_buf = [0u16; 1024];
            let mut exe_len = exe_buf.len() as u32;
            let name_ok =
                QueryFullProcessImageNameW(hproc, PROCESS_NAME_WIN32, windows::core::PWSTR(exe_buf.as_mut_ptr()), &mut exe_len);
            let _ = CloseHandle(hproc);
            name_ok.ok()?;

            let full_path = String::from_utf16_lossy(&exe_buf[..exe_len as usize]);
            let exe_name = full_path
                .rsplit('\\')
                .next()
                .unwrap_or(&full_path)
                .to_string();

            let integrity_level = query_integrity_level(pid).unwrap_or(0);

            Some(ForegroundWindowInfo {
                handle: hwnd.0 as usize,
                class_name,
                exe_name,
                integrity_level,
            })
        }
    }

    fn restore_focus(&self, info: &ForegroundWindowInfo) -> Result<(), String> {
        unsafe {
            let hwnd = HWND(info.handle as isize as *mut _);
            // Ignore return value — SetForegroundWindow may silently fail per Windows focus rules
            let _ = SetForegroundWindow(hwnd);
        }
        std::thread::sleep(Duration::from_millis(50));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ElevationChecker
// ---------------------------------------------------------------------------

impl ElevationChecker for WindowsProvider {
    fn is_elevated(&self) -> bool {
        unsafe {
            let mut htoken = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut htoken).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION::default();
            let mut return_length: u32 = 0;
            let ok = GetTokenInformation(
                htoken,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            );
            let _ = CloseHandle(htoken);

            ok.is_ok() && elevation.TokenIsElevated != 0
        }
    }

    fn current_integrity_level(&self) -> u32 {
        let pid = unsafe { GetCurrentProcessId() };
        query_integrity_level(pid).unwrap_or(0)
    }

    fn relaunch_elevated(&self) -> Result<(), String> {
        use std::os::windows::ffi::OsStrExt;

        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get current exe: {e}"))?;

        let wide_path: Vec<u16> = exe_path
            .as_os_str()
            .encode_wide()
            .chain([0u16])
            .collect();

        let verb: Vec<u16> = "runas\0".encode_utf16().collect();

        unsafe {
            let result = ShellExecuteW(
                HWND(std::ptr::null_mut()),
                PCWSTR(verb.as_ptr()),
                PCWSTR(wide_path.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );

            if result.0 as isize <= 32 {
                return Err(format!(
                    "ShellExecuteW failed with code {}",
                    result.0 as isize
                ));
            }
        }

        std::process::exit(0);
    }
}

// ---------------------------------------------------------------------------
// Private helpers for InputSimulator
// ---------------------------------------------------------------------------

/// Build the INPUT structs for a single Unicode character.
/// For BMP chars (1 UTF-16 unit): returns 2 INPUTs (down + up).
/// For supplementary plane chars (2 UTF-16 units): returns 4 INPUTs.
fn build_unicode_inputs(ch: char) -> Vec<INPUT> {
    let mut buf = [0u16; 2];
    let units = ch.encode_utf16(&mut buf);
    let mut inputs = Vec::with_capacity(units.len() * 2);
    for &unit in units.iter() {
        // Key down
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: unit,
                    dwFlags: KEYEVENTF_UNICODE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
        // Key up
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: unit,
                    dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
    }
    inputs
}

/// Build a sequence of key-down + key-up INPUTs for each VK code, in order.
fn build_key_combo_inputs(vk_codes: &[VIRTUAL_KEY]) -> Vec<INPUT> {
    let mut inputs = Vec::with_capacity(vk_codes.len() * 2);
    for &vk in vk_codes {
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
    }
    for &vk in vk_codes.iter().rev() {
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
    }
    inputs
}

// ---------------------------------------------------------------------------
// InputSimulator
// ---------------------------------------------------------------------------

impl InputSimulator for WindowsProvider {
    fn send_paste(&self, class_name: &str) -> Result<(), String> {
        let inputs = if self.is_terminal(class_name) {
            // Terminals use Ctrl+Shift+V
            build_key_combo_inputs(&[VK_CONTROL, VK_SHIFT, VK_V])
        } else {
            // Regular windows use Ctrl+V
            build_key_combo_inputs(&[VK_CONTROL, VK_V])
        };

        unsafe {
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent != inputs.len() as u32 {
                return Err(format!(
                    "SendInput sent {sent} of {} events",
                    inputs.len()
                ));
            }
        }
        Ok(())
    }

    fn send_unicode_string(&self, text: &str) -> Result<(), String> {
        for ch in text.chars() {
            if ch == '\n' || ch == '\r' {
                self.send_return()?;
                continue;
            }

            let inputs = build_unicode_inputs(ch);
            unsafe {
                let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                let expected = inputs.len() as u32;
                if sent != expected {
                    return Err(format!(
                        "SendInput sent {sent} of {expected} events for char U+{:04X}",
                        ch as u32
                    ));
                }
            }
        }
        Ok(())
    }

    fn send_return(&self) -> Result<(), String> {
        let inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_RETURN,
                        wScan: 0,
                        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_RETURN,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];

        unsafe {
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent != 2 {
                return Err(format!("SendInput sent {sent} of 2 events for VK_RETURN"));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ClipboardAccess
// ---------------------------------------------------------------------------

impl ClipboardAccess for WindowsProvider {
    fn read_text(&self) -> Result<String, String> {
        arboard::Clipboard::new()
            .map_err(|e| e.to_string())?
            .get_text()
            .map_err(|e| e.to_string())
    }

    fn write_text(&self, text: &str) -> Result<(), String> {
        arboard::Clipboard::new()
            .map_err(|e| e.to_string())?
            .set_text(text)
            .map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrogate_pair_produces_two_input_structs() {
        // U+10000 encodes to [0xD800, 0xDC00] in UTF-16 (surrogate pair)
        let ch = '\u{10000}';
        let mut buf = [0u16; 2];
        let units = ch.encode_utf16(&mut buf);
        // Each UTF-16 unit generates 2 INPUT events (down + up), so 2 units = 4 events
        assert_eq!(units.len(), 2, "U+10000 must produce 2 UTF-16 units");

        let inputs = build_unicode_inputs(ch);
        // 2 units * 2 events each (down + up) = 4 INPUT structs
        assert_eq!(
            inputs.len(),
            4,
            "surrogate pair must produce 4 INPUT structs (2 down + 2 up)"
        );
    }

    #[test]
    fn bmp_char_produces_two_input_structs() {
        let ch = 'A';
        let inputs = build_unicode_inputs(ch);
        // 1 unit * 2 events (down + up) = 2 INPUT structs
        assert_eq!(
            inputs.len(),
            2,
            "BMP char must produce 2 INPUT structs (1 down + 1 up)"
        );
    }
}
