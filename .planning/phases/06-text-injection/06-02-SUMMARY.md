---
phase: 06-text-injection
plan: 02
subsystem: platform
tags: [windows, win32, sendinput, arboard, clipboard, utf16, surrogate-pairs, integrity-level]

requires:
  - phase: 06-text-injection-01
    provides: InjectionConfig and AppState foreground_window field that this provider will populate

provides:
  - WindowsProvider with fully functional WindowInfo (GetForegroundWindow, restore_focus)
  - WindowsProvider with fully functional ElevationChecker (is_elevated, current_integrity_level, relaunch_elevated)
  - WindowsProvider with fully functional InputSimulator (send_paste, send_unicode_string with surrogate pairs, send_return)
  - WindowsProvider with fully functional ClipboardAccess (read_text, write_text via arboard)
  - build_unicode_inputs() helper with unit tests for BMP and supplementary plane chars

affects:
  - 06-text-injection-03
  - 06-text-injection-04
  - injection/service.rs (calls through these traits)

tech-stack:
  added: []
  patterns:
    - "query_integrity_level() shared by WindowInfo and ElevationChecker — one token query helper"
    - "arboard::Clipboard::new() called per-operation (never stored) for timing safety"
    - "build_unicode_inputs() extracted from send_unicode_string for testability without SendInput"
    - "build_key_combo_inputs() sends all keys down then all keys up in reverse order"
    - "OpenProcessToken is in Win32::System::Threading, not Win32::Security (windows crate 0.58)"

key-files:
  created: []
  modified:
    - src-tauri/src/platform/windows/mod.rs

key-decisions:
  - "OpenProcessToken lives in Win32::System::Threading in windows crate 0.58 (not Win32::Security)"
  - "build_unicode_inputs() extracted to allow unit testing surrogate pair logic without calling SendInput"
  - "query_integrity_level() is a free fn (not trait method) shared by get_foreground_window and current_integrity_level"

patterns-established:
  - "Per-operation arboard::Clipboard::new() — never stored across calls"
  - "encode_utf16() + KEYEVENTF_UNICODE for supplementary plane character support"

requirements-completed:
  - INJC-01
  - INJC-02
  - INJC-03
  - INJC-05
  - INJC-06
  - INJC-07
  - INJC-10
  - INJC-11

duration: 6min
completed: 2026-03-21
---

# Phase 06 Plan 02: Windows Platform Provider Implementation Summary

**Real Win32 implementations for all four platform traits (WindowInfo, ElevationChecker, InputSimulator, ClipboardAccess) replacing stubs, with surrogate-pair-aware UTF-16 input simulation and unit tests.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-21T17:43:44Z
- **Completed:** 2026-03-21T17:49:44Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Replaced all stub implementations in `platform/windows/mod.rs` with real Win32 API calls via the `windows` crate
- Implemented `get_foreground_window()` using GetForegroundWindow, GetClassNameW, QueryFullProcessImageNameW, and a shared `query_integrity_level()` helper
- Implemented `send_unicode_string()` with `build_unicode_inputs()` helper that handles both BMP (1 UTF-16 unit) and supplementary plane chars (2 UTF-16 surrogate units), backed by unit tests
- All four traits fully functional: no "TODO" or "not yet implemented" strings remain

## Task Commits

Both tasks were written in one file write and committed together:

1. **Task 1 + Task 2: All four trait implementations + unit tests** - `9511ccc` (feat)

## Files Created/Modified

- `src-tauri/src/platform/windows/mod.rs` - Complete Win32 implementation of WindowInfo, ElevationChecker, InputSimulator, ClipboardAccess; build_unicode_inputs() helper; unit tests for BMP and surrogate pair chars

## Decisions Made

- `OpenProcessToken` is in `Win32::System::Threading` in the `windows` crate v0.58, not `Win32::Security` where docs might suggest — fixed import path after first compile error.
- `build_unicode_inputs()` extracted as a private free function (not a method) to enable unit testing without calling SendInput.
- `query_integrity_level()` implemented as a shared free function used by both `get_foreground_window()` and `current_integrity_level()` to avoid code duplication.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Corrected OpenProcessToken import path**
- **Found during:** Task 1 (compile check after writing)
- **Issue:** `windows::Win32::Security::OpenProcessToken` does not exist — it lives in `Win32::System::Threading`
- **Fix:** Moved `OpenProcessToken` to the `Win32::System::Threading` import group
- **Files modified:** src-tauri/src/platform/windows/mod.rs
- **Verification:** cargo check passes with no errors
- **Committed in:** 9511ccc

---

**Total deviations:** 1 auto-fixed (1 blocking import path error)
**Impact on plan:** Necessary correction, no scope change.

## Issues Encountered

- Import path for `OpenProcessToken` in windows crate 0.58 differs from documentation examples — it is in `Win32::System::Threading`, resolved by checking crate source.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `WindowsProvider` now fully implements all four platform traits
- Plan 06-03 (injection/service.rs) can call through these traits without further stubs
- arboard clipboard read/write ready for FlashPaste injection mode
- Terminal detection (Ctrl+Shift+V vs Ctrl+V) ready via `send_paste(class_name)`

---
*Phase: 06-text-injection*
*Completed: 2026-03-21*
