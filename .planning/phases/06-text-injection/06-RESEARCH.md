# Phase 6: Text Injection - Research

**Researched:** 2026-03-21
**Domain:** Windows text injection (SendInput, clipboard, integrity levels, focus management)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- FlashPaste is default (already locked from prior phases)
- Fallback chain if Keystrokes selected: Keystrokes → FlashPaste → Clipboard
- Fallback chain if FlashPaste selected: FlashPaste → Clipboard
- Auto-fallback is configurable (INJC-09 toggle) — include the config field now, Phase 8 exposes UI
- Elevation check only in Keystroke mode (FlashPaste clipboard operations work across integrity levels; SendInput silently fails on elevated targets)
- Dialog: native Windows dialog (MessageBox or TaskDialog) — not a custom overlay
- "Copy to clipboard" path: copy text + show toast confirming "Copied to clipboard — paste manually"
- "Relaunch as Admin": implement using ShellExecuteW "runas"
- After successful injection: indicator flashes brief green/checkmark state (~1 second) then hides
- No full success toast in Phase 6 — that's Phase 7's job
- Indicator stays visible throughout the entire injection (including during character-by-character keystroke mode)
- Injection errors (all fallbacks exhausted): reuse existing toast window with a new InjectionErrorPayload type
- FlashPaste timing: hardcode 500ms between paste and clipboard restore (fixed delay, no event detection)
- Add paste_delay_ms to InjectionConfig now even without Settings UI
- Escape key only cancels injection (not the recording hotkey — hotkey during injection is ignored)
- Toast on cancel: "Cancelled — 47 of 230 chars typed"
- Partial text already typed into the target app: leave as-is, no cleanup
- GetAsyncKeyState polling between char sends (low-level hook is overkill)
- Must capture the foreground window at recording START (before indicator shows), not at injection time
- Add `foreground_window: Arc<Mutex<Option<ForegroundWindowInfo>>>` to AppState
- Focus must be restored to the captured window before injection (INJC-10)

### Claude's Discretion

- Escape detection: polling vs hook — pick the simpler approach (polling recommended)
- Exact green flash color/animation for the success state in the indicator
- InjectionErrorPayload field naming

### Deferred Ideas (OUT OF SCOPE)

- None — discussion stayed within Phase 6 scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INJC-01 | FlashPaste: save clipboard → write text → simulate paste → wait 500ms → restore clipboard | ClipboardAccess trait (arboard), InputSimulator trait (SendInput), 500ms hardcoded delay |
| INJC-02 | FlashPaste detects terminal windows by class name and uses Ctrl+Shift+V or Shift+Insert | TERMINAL_CLASSES constant already defined in platform/mod.rs; send_paste() already has terminal routing stub |
| INJC-03 | Simulated keystroke injection character-by-character via SendInput with KEYEVENTF_UNICODE | send_unicode_string() stub ready; INPUT struct with KEYBDINPUT, wScan=char as u16, dwFlags=KEYEVENTF_UNICODE |
| INJC-04 | Keystroke speed configurable: slow (10ms/char), normal (5ms/char), fast (2ms/char) | New keystroke_speed field in InjectionConfig; std::thread::sleep per char |
| INJC-05 | Newline characters injected as VK_RETURN in keystroke mode | send_return() stub ready; detect '\n' or '\r\n' in text loop |
| INJC-06 | Manual clipboard mode copies to clipboard without auto-pasting | ClipboardAccess::write_text() + no SendInput call |
| INJC-07 | If target process at higher IL, show dialog "Relaunch as Admin" / "Copy to clipboard" | ElevationChecker::current_integrity_level() vs target ForegroundWindowInfo::integrity_level; MessageBoxW dialog |
| INJC-08 | Escape cancels keystroke injection; toast shows "X of Y characters typed" | GetAsyncKeyState(VK_ESCAPE) polling; cancel_flag in AppState; InjectionErrorPayload with char counts |
| INJC-09 | Auto-fallback chain configurable toggle | New auto_fallback bool in InjectionConfig |
| INJC-10 | Focus restored to target window before injection | SetForegroundWindow with saved HWND from recording start; foreground_window in AppState |
| INJC-11 | Unicode text handled correctly in all modes | KEYEVENTF_UNICODE for keystroke; arboard handles Unicode; FlashPaste inherits Unicode clipboard |
</phase_requirements>

---

## Summary

Phase 6 completes the core VoxWeave pipeline: transcribed text lands in the previously-focused window. All three injection modes (FlashPaste, Keystroke, Clipboard) must be implemented behind the platform trait layer that was scaffolded in Phase 1.

The Windows-specific implementations are all stubs in `platform/windows/mod.rs` waiting to be filled. The traits, config enums, and state fields are mostly in place — this phase is about converting those stubs into real Windows API calls using the `windows` crate, implementing the injection orchestration layer in a new `injection/` module, and wiring it into the hotkey pipeline at the "Phase 6 will handle injection here" comment.

The most complex interactions are: the foreground-window capture at recording start (to survive the indicator appearing), the integrity level check before Keystroke injection, and the cancel-on-Escape loop during character-by-character typing. All three have well-understood solutions using standard Windows APIs available directly in the `windows` crate.

**Primary recommendation:** Implement all Windows API calls inside the existing `WindowsProvider` struct in `platform/windows/mod.rs`, then build a thin `injection/service.rs` orchestration layer that runs the fallback chain using those trait impls.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `windows` crate | 0.58+ | GetForegroundWindow, SetForegroundWindow, SendInput, GetAsyncKeyState, GetTokenInformation, ShellExecuteW, GetWindowThreadProcessId, QueryFullProcessImageNameW, GetClassName, MessageBoxW, OpenProcess | Microsoft-maintained, safe wrappers, already decided in CLAUDE.md |
| `arboard` | 3.x | Clipboard read/write | Already decided in CLAUDE.md; synchronous, timing-safe for FlashPaste |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::thread::sleep` | stdlib | Keystroke speed delays + 500ms FlashPaste restore delay | Between SendInput calls in keystroke mode |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `windows` crate direct | `enigo` crate | enigo abstracts platform differences but adds a dependency and may not support all VK_ codes; rejected per CLAUDE.md preference for `windows` crate |
| `arboard` | Tauri clipboard plugin | Plugin is async, breaks tight FlashPaste timing — already decided, non-negotiable |

**Installation:**
```bash
# In src-tauri/Cargo.toml:
arboard = "3"
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_UI_WindowsAndMessaging",
  "Win32_UI_Input_KeyboardAndMouse",
  "Win32_UI_Shell",
  "Win32_System_Threading",
  "Win32_Security",
  "Win32_System_Diagnostics_Debug",
] }
```

---

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/
├── platform/windows/mod.rs   # Fill all stubs with real Windows API calls
├── injection/
│   ├── mod.rs                # pub use, module re-exports
│   └── service.rs            # inject_text() orchestration: method selection, fallback chain
├── commands/
│   └── injection.rs          # Thin command: relaunch_elevated, cancel_injection (if needed)
├── config/mod.rs             # Add keystroke_speed, auto_fallback, paste_delay_ms to InjectionConfig
└── state.rs                  # Add foreground_window: Arc<Mutex<Option<ForegroundWindowInfo>>>
```

### Pattern 1: Foreground Window Capture at Recording Start

**What:** Capture and store the foreground window handle/metadata immediately when the hotkey starts recording, before the indicator window is shown. The indicator showing can steal focus so capture must be first.

**When to use:** In `toggle_recording_state` in `hotkey/service.rs` at the `previous_state == RecordingState::Idle` branch, BEFORE `indicator::show_recording()`.

**Example:**
```rust
// In toggle_recording_state, before show_recording():
{
    let platform = app.state::<PlatformProvider>();
    let state = app.state::<AppState>();
    let fw = platform.get_foreground_window();
    *state.foreground_window.lock().unwrap() = fw;
}
// Then: indicator::show_recording(app)?
```

### Pattern 2: WindowsProvider — GetForegroundWindow + GetClassName + GetWindowThreadProcessId

**What:** Fill the `get_foreground_window()` stub with real Windows API calls.

**When to use:** Called once at recording start.

**Example:**
```rust
// Source: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/WindowsAndMessaging/
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetClassNameW, GetWindowThreadProcessId,
};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_NAME_WIN32};
use windows::Win32::Foundation::CloseHandle;
use windows::core::PWSTR;

fn get_foreground_window(&self) -> Option<ForegroundWindowInfo> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 { return None; }

        let mut class_buf = [0u16; 256];
        GetClassNameW(hwnd, &mut class_buf);
        let class_name = String::from_utf16_lossy(
            &class_buf[..class_buf.iter().position(|&c| c == 0).unwrap_or(0)]
        );

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let hproc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut name_buf = vec![0u16; 260];
        let mut size = name_buf.len() as u32;
        QueryFullProcessImageNameW(hproc, PROCESS_NAME_WIN32, PWSTR(name_buf.as_mut_ptr()), &mut size).ok();
        let _ = CloseHandle(hproc);
        let full_path = String::from_utf16_lossy(&name_buf[..size as usize]);
        let exe_name = full_path.rsplit('\\').next().unwrap_or("").to_string();

        let integrity_level = query_integrity_level(pid).unwrap_or(0);

        Some(ForegroundWindowInfo {
            handle: hwnd.0 as usize,
            class_name,
            exe_name,
            integrity_level,
        })
    }
}
```

### Pattern 3: Integrity Level Query

**What:** Query `TokenIntegrityLevel` from the process token.

**When to use:** Inside `get_foreground_window()` for the target process; inside `current_integrity_level()` for VoxWeave's own token.

**Example:**
```rust
// Source: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Security/fn.GetTokenInformation.html
// Integrity level RID constants (Win32):
// MEDIUM = 0x2000, HIGH = 0x3000, SYSTEM = 0x4000
fn query_integrity_level(pid: u32) -> Option<u32> {
    unsafe {
        use windows::Win32::Security::{
            GetTokenInformation, OpenProcessToken, TOKEN_MANDATORY_LABEL,
            TOKEN_QUERY, TokenIntegrityLevel,
        };
        use windows::Win32::System::Threading::OpenProcess;
        use windows::Win32::Foundation::CloseHandle;

        let hproc = OpenProcess(windows::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut htoken = windows::Win32::Foundation::HANDLE::default();
        OpenProcessToken(hproc, TOKEN_QUERY, &mut htoken).ok()?;
        let _ = CloseHandle(hproc);

        let mut buf = vec![0u8; 256];
        let mut ret_len = 0u32;
        GetTokenInformation(htoken, TokenIntegrityLevel, Some(buf.as_mut_ptr() as _), buf.len() as u32, &mut ret_len).ok()?;
        let _ = CloseHandle(htoken);

        // TOKEN_MANDATORY_LABEL is at the start of buf; SID sub-authority holds the RID
        let tml = &*(buf.as_ptr() as *const TOKEN_MANDATORY_LABEL);
        let sub_auth_count = *windows::Win32::Security::GetSidSubAuthorityCount(tml.Label.Sid) as usize;
        let rid = *windows::Win32::Security::GetSidSubAuthority(tml.Label.Sid, (sub_auth_count - 1) as u32);
        Some(rid)
    }
}
```

### Pattern 4: restore_focus

**What:** SetForegroundWindow with the captured HWND.

**When to use:** In injection service, immediately before any injection attempt.

**Example:**
```rust
// Source: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/WindowsAndMessaging/fn.SetForegroundWindow.html
fn restore_focus(&self, info: &ForegroundWindowInfo) -> Result<(), String> {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{SetForegroundWindow, HWND};
        let hwnd = HWND(info.handle as isize);
        // SetForegroundWindow can fail silently — ignore the bool return
        let _ = SetForegroundWindow(hwnd);
        // Small sleep allows the OS to process the focus change before input
        std::thread::sleep(std::time::Duration::from_millis(50));
        Ok(())
    }
}
```

### Pattern 5: SendInput — Unicode Characters (INJC-03, INJC-11)

**What:** Send each character as a KEYEVENTF_UNICODE INPUT struct (key-down + key-up pair). For supplementary plane characters (code point > U+FFFF), split into surrogate pair.

**When to use:** In `send_unicode_string()` implementation and `send_return()`.

**Example:**
```rust
// Source: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Input/KeyboardAndMouse/
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    VK_RETURN,
};

fn send_unicode_char(ch: u16) -> Result<(), String> {
    let inputs = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0),
                    wScan: ch,
                    dwFlags: KEYEVENTF_UNICODE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0),
                    wScan: ch,
                    dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];
    unsafe {
        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if sent != 2 { return Err("SendInput failed".to_string()); }
    }
    Ok(())
}
```

For characters > U+FFFF, encode as UTF-16 surrogate pair and send both u16 values as separate KEYEVENTF_UNICODE events.

### Pattern 6: send_paste — FlashPaste Paste Shortcut (INJC-01, INJC-02)

**What:** Send Ctrl+V (or Ctrl+Shift+V for terminals). Two-INPUT array: modifier down, V down, V up, modifier up.

**When to use:** In `send_paste()` implementation, after checking `is_terminal(class_name)`.

**Example:**
```rust
// Terminal detection uses existing TERMINAL_CLASSES constant
fn send_paste(&self, class_name: &str) -> Result<(), String> {
    let is_term = self.is_terminal(class_name);
    if is_term {
        // Send Ctrl+Shift+V (most common terminal paste)
        send_key_combo(&[VK_CONTROL, VK_SHIFT, VK_V])
    } else {
        send_key_combo(&[VK_CONTROL, VK_V])
    }
}
```

### Pattern 7: GetAsyncKeyState Polling for Escape Cancel (INJC-08)

**What:** Between each character send in the keystroke loop, check if Escape is held. Return early with cancellation count.

**When to use:** Inside the character loop in `send_unicode_string_with_cancel()`.

**Example:**
```rust
// Source: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Input/KeyboardAndMouse/fn.GetAsyncKeyState.html
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE};

fn is_escape_pressed() -> bool {
    unsafe {
        // High-order bit set = key is down
        GetAsyncKeyState(VK_ESCAPE.0 as i32) & (u16::MAX as i16) < 0
    }
}
```

The cancel check sits between each `send_unicode_char()` + `std::thread::sleep()` call. The cancel_flag in AppState provides a second cancellation path (for future programmatic cancel).

### Pattern 8: ShellExecuteW Elevation Relaunch (INJC-07)

**What:** Relaunch the current executable with "runas" verb to trigger UAC elevation prompt.

**When to use:** In `relaunch_elevated()`, when user picks "Relaunch as Admin" in the elevation dialog.

**Example:**
```rust
// Source: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Shell/fn.ShellExecuteW.html
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::core::w;

fn relaunch_elevated(&self) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_wide: Vec<u16> = exe.as_os_str().encode_wide().chain([0]).collect();
    unsafe {
        let result = ShellExecuteW(
            None,
            w!("runas"),
            windows::core::PCWSTR(exe_wide.as_ptr()),
            None,
            None,
            windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        );
        if result.0 <= 32 {
            return Err(format!("ShellExecuteW failed: {}", result.0));
        }
    }
    // Exit current (non-elevated) process
    std::process::exit(0);
}
```

### Pattern 9: Native Elevation Dialog (INJC-07)

**What:** Use MessageBoxW to show the elevation check dialog — not a custom overlay.

**When to use:** When `target_il > self_il` in the injection service.

**Example:**
```rust
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONWARNING, MB_YESNOCANCEL, IDYES, IDNO};

fn show_elevation_dialog() -> ElevationDialogResult {
    unsafe {
        let result = MessageBoxW(
            None,
            w!("The target window requires elevated privileges.\n\nRelaunch VoxWeave as Administrator, or copy the text to clipboard for manual paste."),
            w!("VoxWeave — Elevation Required"),
            MB_ICONWARNING | MB_YESNOCANCEL,  // Yes=Relaunch, No=Copy, Cancel=Abort
        );
        match result {
            IDYES  => ElevationDialogResult::Relaunch,
            IDNO   => ElevationDialogResult::CopyToClipboard,
            _      => ElevationDialogResult::Cancel,
        }
    }
}
```

### Pattern 10: Injection Service Orchestration

**What:** A new `injection::service::inject_text()` function that wraps the full pipeline: restore focus → check elevation → run method → fallback if needed → emit result.

**When to use:** Called from `hotkey/service.rs` at the "Phase 6 will handle injection here" comment.

**Example structure:**
```rust
// injection/service.rs
pub async fn inject_text<R: Runtime>(app: &AppHandle<R>, text: &str) {
    // 1. Restore focus to captured foreground window
    // 2. Read injection mode and config
    // 3. Check elevation if mode == Keystroke
    // 4. Run injection method (may update indicator to show_injecting)
    // 5. On method failure + auto_fallback enabled: try next in chain
    // 6. On all-fallbacks-exhausted: show_toast_window(InjectionErrorPayload)
    // 7. On success: trigger green flash, then hide indicator
}
```

### Pattern 11: Green Flash on Success

**What:** After successful injection, the indicator needs a brief green/success state (~1 second) before hiding. This requires a new `IndicatorVisualState::Success` variant (or repurposing `Injecting` with a timed hide).

**Recommended approach:** Add `IndicatorVisualState::Success` variant, emit it from Rust, handle it in `StateBadge.vue` with green color. Use `tokio::time::sleep(Duration::from_millis(1000))` in the async injection task then call `indicator::hide()`.

**When to use:** After confirmed successful inject in `injection/service.rs`.

### Anti-Patterns to Avoid

- **Holding MutexGuard across await:** Extract all needed data from AppState before any `.await` or `thread::sleep`. The transcription service already demonstrates this pattern — follow it exactly.
- **SendInput to foreground at wrong time:** Always restore focus with SetForegroundWindow + 50ms sleep before the first SendInput call. Without this, input goes to the indicator window or a different window.
- **Blocking Tokio with thread::sleep in inject loop:** Keystroke injection runs character-by-character with sleep delays — wrap in `tauri::async_runtime::spawn_blocking` or use `std::thread::spawn` to avoid starving the Tokio runtime.
- **arboard Clipboard in async context:** `arboard::Clipboard::new()` opens a clipboard handle on the current thread and must not cross thread boundaries. Create a new Clipboard instance per operation, not a shared instance.
- **500ms FlashPaste delay as async sleep:** Use `std::thread::sleep` inside a `spawn_blocking` — `tokio::time::sleep` in blocking clipboard code may not work correctly.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Terminal window detection | Custom class-name list | `TERMINAL_CLASSES` constant in `platform/mod.rs` | Already defined and correct |
| Unicode char encoding for SendInput | Custom UTF-16 logic | Rust's `char::encode_utf16()` method | Correctly handles supplementary planes via surrogate pairs |
| Clipboard operations | Win32 OpenClipboard/SetClipboardData directly | `arboard` crate | Handles OpenClipboard locking, retries, CF_UNICODETEXT encoding, cleanup |
| Elevation relaunch | Custom process spawning | ShellExecuteW with "runas" | The only correct way to trigger UAC prompt for same executable |
| Process name from HWND | Parsing window title | `GetWindowThreadProcessId` + `QueryFullProcessImageNameW` | Window title is not reliable; exe name is correct |

**Key insight:** The Windows clipboard is a shared OS resource protected by a mutex. `arboard` handles the lock-open-write-close sequence with retries on contention — hand-rolling this in async code is a common source of data corruption and deadlocks.

---

## Common Pitfalls

### Pitfall 1: SetForegroundWindow Silently Fails
**What goes wrong:** `SetForegroundWindow` returns `FALSE` without error when VoxWeave is not the foreground process (Windows restricts which processes can steal focus).
**Why it happens:** Windows Vista+ added restrictions: only a foreground process can call `SetForegroundWindow` successfully without `AllowSetForegroundWindow`.
**How to avoid:** Call `AllowSetForegroundWindow(ASFW_ANY)` at recording START (when VoxWeave is focused) so the later restore succeeds. Alternatively, use `AttachThreadInput` pattern. A 50ms sleep after the call lets the OS process the change.
**Warning signs:** SendInput keystrokes land in VoxWeave's indicator window instead of the target.

### Pitfall 2: FlashPaste Clipboard Race Condition
**What goes wrong:** The target app hasn't rendered the pasted text before clipboard is restored, resulting in empty paste.
**Why it happens:** Some apps (Electron-based: Slack, VS Code, Chrome devtools) process clipboard content asynchronously. 500ms delay is usually sufficient but noted as potentially insufficient in STATE.md blockers.
**How to avoid:** The 500ms hardcoded delay is the locked decision. No fix in Phase 6 — document as known limitation for Phase 8 (configurable paste_delay_ms).
**Warning signs:** Empty paste in Electron apps; works fine in native apps.

### Pitfall 3: KEYEVENTF_UNICODE for Supplementary Plane Characters
**What goes wrong:** Characters above U+FFFF (emoji, some CJK extensions) send as a single u16 truncated to 0xFFFD.
**Why it happens:** `SendInput` with `KEYEVENTF_UNICODE` accepts `wScan: u16` — a single code unit. Characters above U+FFFF need two events (high surrogate + low surrogate).
**How to avoid:** Use `char::encode_utf16()` which returns 1 or 2 u16 values; send each u16 as a separate INPUT pair.
**Warning signs:** Emoji appear as replacement characters (�) in target apps.

### Pitfall 4: Integrity Level Comparison Logic
**What goes wrong:** Elevation check triggers even when target and VoxWeave are at the same level.
**Why it happens:** Comparison must be `target_il > self_il` not `target_il != self_il`. Medium IL (0x2000) processes can inject into other Medium IL processes.
**How to avoid:** Only show the elevation dialog when `target_integrity_level > voxweave_integrity_level`.
**Warning signs:** Spurious elevation dialogs for every injection.

### Pitfall 5: Cancel Flag Not Reset Before Injection
**What goes wrong:** Second recording attempt immediately sees a stale cancel flag = true, cancels immediately with 0 chars.
**Why it happens:** `cancel_flag` was set by Escape during a previous injection and never reset.
**How to avoid:** Reset `cancel_flag` to `false` at the START of each injection attempt, before the character loop.
**Warning signs:** Keystroke injection cancels with "0 of N chars typed" on the second attempt.

### Pitfall 6: arboard on Wrong Thread
**What goes wrong:** Rust panics or clipboard operations silently fail when arboard Clipboard is created on one thread and used on another.
**Why it happens:** Windows clipboard operations are thread-affine — `OpenClipboard` associates with the calling thread.
**How to avoid:** Create `arboard::Clipboard::new()` inline at each use site inside `spawn_blocking`. Do not store Clipboard in AppState or across thread boundaries.
**Warning signs:** Intermittent clipboard failures; arboard errors about clipboard already being open.

### Pitfall 7: Missing `windows` Crate Features
**What goes wrong:** `GetTokenInformation`, `QueryFullProcessImageNameW`, etc. fail to compile with "not found in crate."
**Why it happens:** The `windows` crate uses feature flags for each Win32 module — unused functions are gated out.
**How to avoid:** Add all required feature strings to `Cargo.toml` upfront (see Standard Stack section above).
**Warning signs:** Compile errors about missing items in `windows::Win32::Security`, `windows::Win32::System::Threading`, etc.

---

## Code Examples

### FlashPaste Full Flow
```rust
// In injection/service.rs — FlashPaste mode
pub fn flashpaste(
    clipboard: &dyn ClipboardAccess,
    input: &dyn InputSimulator,
    class_name: &str,
    text: &str,
    paste_delay_ms: u64,
) -> Result<(), String> {
    // 1. Save current clipboard content
    let saved = clipboard.read_text().unwrap_or_default();

    // 2. Write transcription text
    clipboard.write_text(text)?;

    // 3. Send paste shortcut (Ctrl+V or Ctrl+Shift+V for terminals)
    input.send_paste(class_name)?;

    // 4. Wait for target app to process paste
    std::thread::sleep(std::time::Duration::from_millis(paste_delay_ms));

    // 5. Restore clipboard (best-effort — do not fail injection on restore failure)
    let _ = clipboard.write_text(&saved);

    Ok(())
}
```

### Keystroke Loop with Cancel Check
```rust
// In injection/service.rs — Keystroke mode
pub fn keystroke_inject(
    input: &dyn InputSimulator,
    text: &str,
    delay_ms: u64,
    cancel_flag: Arc<Mutex<bool>>,
) -> InjectionResult {
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();

    for (i, ch) in chars.iter().enumerate() {
        // Check cancel before each character
        if *cancel_flag.lock().unwrap() {
            return InjectionResult::Cancelled { typed: i, total };
        }

        if *ch == '\n' || *ch == '\r' {
            if let Err(e) = input.send_return() {
                return InjectionResult::Err(e);
            }
        } else {
            // encode_utf16 returns 1 or 2 u16 values
            let mut buf = [0u16; 2];
            let units = ch.encode_utf16(&mut buf);
            for &unit in units.iter() {
                // send_unicode_string handles single-char at a time
                // implementation sends each u16 as KEYEVENTF_UNICODE INPUT pair
            }
            if let Err(e) = input.send_unicode_char_units(units) {
                return InjectionResult::Err(e);
            }
        }

        // Poll Escape between chars (GetAsyncKeyState)
        if is_escape_pressed() {
            *cancel_flag.lock().unwrap() = false; // reset for next use
            return InjectionResult::Cancelled { typed: i + 1, total };
        }

        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
    }
    InjectionResult::Ok
}
```

### InjectionConfig Fields to Add
```rust
// In config/mod.rs — extend InjectionConfig
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KeystrokeSpeed {
    Slow,    // 10ms/char
    Normal,  // 5ms/char
    Fast,    // 2ms/char
}

impl Default for KeystrokeSpeed {
    fn default() -> Self { Self::Normal }
}

impl KeystrokeSpeed {
    pub fn delay_ms(&self) -> u64 {
        match self {
            Self::Slow   => 10,
            Self::Normal => 5,
            Self::Fast   => 2,
        }
    }
}

pub struct InjectionConfig {
    #[serde(default)]
    pub mode: InjectionMode,

    #[serde(default)]
    pub keystroke_speed: KeystrokeSpeed,

    #[serde(default = "default_true")]
    pub auto_fallback: bool,

    #[serde(default = "default_paste_delay_ms")]
    pub paste_delay_ms: u64,
}

fn default_paste_delay_ms() -> u64 { 500 }
```

### InjectionErrorPayload (new — mirrors TranscriptionErrorPayload pattern)
```rust
// In injection/service.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InjectionErrorPayload {
    pub code: InjectionErrorCode,
    pub message: String,
    /// For cancel: how many chars were typed
    pub typed_chars: Option<usize>,
    /// For cancel: total chars in text
    pub total_chars: Option<usize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionErrorCode {
    Cancelled,
    AllMethodsFailed,
    ElevationRequired,
    FocusLost,
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `winapi` crate for Win32 calls | `windows` crate (Microsoft-maintained) | ~2021 | Safe wrappers, generated from metadata, actively updated |
| Polling `GetKeyState` for cancel | `GetAsyncKeyState` | Long-standing | GetAsyncKeyState works across threads and processes; GetKeyState is thread-local |
| `rust-clipboard` crate | `arboard` (1Password-maintained) | ~2022 | arboard handles Windows Unicode text, image support, active maintenance |

**Deprecated/outdated:**
- `winapi` crate: still works but `windows` crate is the project standard per CLAUDE.md
- Tauri clipboard plugin for FlashPaste: explicitly rejected (async, timing-unsafe) per CLAUDE.md

---

## Open Questions

1. **AllowSetForegroundWindow requirement**
   - What we know: SetForegroundWindow has OS-level restrictions post-Vista
   - What's unclear: Whether Tauri's indicator window counts as "foreground process" at the moment focus is restored
   - Recommendation: Implement without AllowSetForegroundWindow first; add it (called at recording START) if focus restore fails in testing

2. **Shift+Insert for terminal alternate paste**
   - What we know: CONTEXT.md mentions Ctrl+Shift+V OR Shift+Insert; REQUIREMENTS.md says "Ctrl+Shift+V or Shift+Insert"
   - What's unclear: Which terminal variants prefer Shift+Insert vs Ctrl+Shift+V
   - Recommendation: Default to Ctrl+Shift+V first; add Shift+Insert as secondary attempt only if Ctrl+Shift+V fails (or expose as config in Phase 8)

3. **IndicatorVisualState::Success vs reusing Injecting**
   - What we know: A new green flash state is needed after injection; CONTEXT.md says ~1 second then hide
   - What's unclear: Whether to add a new enum variant or drive the flash from the frontend with a timer
   - Recommendation: Add `IndicatorVisualState::Success` to `indicator/events.rs`; frontend handles color + auto-hide timer; keeps Rust side simple

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | none — standard `#[cfg(test)]` modules |
| Quick run command | `cd src-tauri && cargo test injection` |
| Full suite command | `cd src-tauri && cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INJC-01 | FlashPaste: save→write→paste→wait→restore sequence | unit | `cargo test flashpaste` | ❌ Wave 0 |
| INJC-02 | Terminal detection: ConsoleWindowClass → Ctrl+Shift+V path | unit | `cargo test terminal_detection` | ❌ Wave 0 |
| INJC-03 | Keystroke: Unicode chars sent as KEYEVENTF_UNICODE pairs | unit (mock InputSimulator) | `cargo test keystroke_unicode` | ❌ Wave 0 |
| INJC-04 | Keystroke speed: slow=10ms, normal=5ms, fast=2ms | unit | `cargo test keystroke_speed` | ❌ Wave 0 |
| INJC-05 | Newlines sent as VK_RETURN | unit (mock InputSimulator) | `cargo test keystroke_newline` | ❌ Wave 0 |
| INJC-06 | Clipboard mode: write_text called, send_paste NOT called | unit (mock) | `cargo test clipboard_mode` | ❌ Wave 0 |
| INJC-07 | Elevation check: target_il > self_il triggers dialog | unit | `cargo test elevation_check_logic` | ❌ Wave 0 |
| INJC-08 | Cancel: InjectionResult::Cancelled with correct char counts | unit | `cargo test keystroke_cancel` | ❌ Wave 0 |
| INJC-09 | Fallback chain: FlashPaste→Clipboard when FlashPaste fails | unit (mock) | `cargo test fallback_chain` | ❌ Wave 0 |
| INJC-10 | Focus restore called before any injection | unit (mock) | `cargo test focus_restore_order` | ❌ Wave 0 |
| INJC-11 | Supplementary plane chars (>U+FFFF) split into surrogate pairs | unit | `cargo test unicode_surrogate_pairs` | ❌ Wave 0 |

Note: Platform trait mocks (mock WindowsProvider impls) enable unit testing the orchestration logic without real Win32 calls.

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test injection`
- **Per wave merge:** `cd src-tauri && cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/injection/service.rs` — injection orchestration + tests
- [ ] `src-tauri/src/injection/mod.rs` — module declaration
- [ ] Mock implementations of `InputSimulator`, `ClipboardAccess`, `WindowInfo` traits for unit tests (can be in `#[cfg(test)]` blocks inside platform/mod.rs or injection/service.rs)

---

## Sources

### Primary (HIGH confidence)
- [windows::Win32::UI::Input::KeyboardAndMouse](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Input/KeyboardAndMouse/) — SendInput, KEYEVENTF_UNICODE, GetAsyncKeyState, VK_ESCAPE, VK_RETURN
- [windows::Win32::UI::WindowsAndMessaging](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/WindowsAndMessaging/) — GetForegroundWindow, SetForegroundWindow, GetClassNameW, GetWindowThreadProcessId, MessageBoxW
- [windows::Win32::Security::GetTokenInformation](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Security/fn.GetTokenInformation.html) — integrity level query
- [windows::Win32::UI::Shell::ShellExecuteW](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Shell/fn.ShellExecuteW.html) — runas elevation
- [windows::Win32::System::Threading::QueryFullProcessImageNameW](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Threading/fn.QueryFullProcessImageNameW.html) — exe name from process
- [arboard crate (1Password)](https://github.com/1Password/arboard) — clipboard read/write
- Existing codebase: `platform/mod.rs`, `platform/windows/mod.rs`, `state.rs`, `config/mod.rs`, `hotkey/service.rs`, `indicator/mod.rs`, `src/windows/toast/App.vue` — all read directly

### Secondary (MEDIUM confidence)
- [Rust Forum: Sending keyboard input on Windows](https://users.rust-lang.org/t/sending-mouse-and-keyboard-input-on-windows/81078) — verified against windows crate docs
- [Tracking active process in Windows with Rust](https://hellocode.co/blog/post/tracking-active-process-windows-rust/) — GetForegroundWindow + GetWindowThreadProcessId pattern

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — `windows` crate and `arboard` are locked decisions; all APIs verified in official docs
- Architecture: HIGH — all stubs already exist in codebase; patterns follow established transcription service model
- Pitfalls: HIGH for known Windows quirks (clipboard arboard threading, SetForegroundWindow restrictions); MEDIUM for clipboard race in Electron apps (confirmed known issue from STATE.md)

**Research date:** 2026-03-21
**Valid until:** 2026-09-21 (stable APIs, 6-month window)
