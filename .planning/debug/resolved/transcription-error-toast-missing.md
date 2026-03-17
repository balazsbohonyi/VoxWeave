---
status: resolved
trigger: "No error toast on transcription failure — indicator disappears silently"
created: 2026-03-17T00:00:00Z
updated: 2026-03-17T14:00:00Z
---

## Current Focus

hypothesis: indicator::hide() is called BEFORE the transcription-error event reaches the indicator window, causing the window to hide (and tear down its Vue instance's event listener context) before the toast can render.
test: trace the exact order of operations in hotkey/service.rs Err(()) branch
expecting: confirmed — hide fires synchronously, transcription-error arrives after window is hidden/JS context paused
next_action: DONE — root cause confirmed

## Symptoms

expected: transcription-error toast appears in the floating indicator with Retry / Try with [Provider]? / Open Settings button
actual: indicator disappears silently, no toast, no error shown anywhere
errors: none in logs (all emit calls use let _ = ... so failures are swallowed)
reproduction: disconnect network, press hotkey, speak, press again, wait 3+ retry attempts
started: Phase 5 implementation

## Eliminated

- hypothesis: emit() uses wrong event name
  evidence: event name constant TRANSCRIPTION_ERROR_EVENT = "transcription-error" matches listener "transcription-error" in App.vue exactly
  timestamp: 2026-03-17

- hypothesis: App.vue listener routing logic is wrong
  evidence: listener code correctly handles all codes (invalid_key, cancelled, and the fallback/retry toast paths) — logic is sound
  timestamp: 2026-03-17

- hypothesis: useToast is broken
  evidence: useToast implementation is correct — showToast pushes to reactive ref, toasts render in template
  timestamp: 2026-03-17

- hypothesis: wrong AppHandle scope (emit goes to wrong window)
  evidence: app.emit() in Tauri v2 broadcasts to ALL windows — both indicator and settings receive every event — not the cause
  timestamp: 2026-03-17

## Evidence

- timestamp: 2026-03-17
  checked: hotkey/service.rs lines 267-283 (tauri::async_runtime::spawn block)
  found: on Err(()), the code calls indicator::hide(&app_clone) FIRST (line 275), THEN falls through to reset recording state. transcription::transcribe_with_retry emits transcription-error synchronously before returning Err(()), but the hide() call follows immediately in the same async task continuation.
  implication: the window.hide() WebviewWindow call makes the indicator window invisible and pauses its WebView rendering pipeline before the transcription-error event is processed by the JS event loop inside that window.

- timestamp: 2026-03-17
  checked: indicator/mod.rs hide() function (lines 66-88)
  found: hide() calls window.hide() which hides the native OS window. It also emits indicator-hidden event. The indicator-hidden listener in App.vue (line 137-141) resets state to "hidden" — this fires on the same JS microtask queue that would process transcription-error.
  implication: even if the window is kept alive, the indicator-hidden handler fires and sets state to "hidden", but that does not prevent toast rendering since toasts are independent of state.

- timestamp: 2026-03-17
  checked: Tauri v2 WebviewWindow.hide() behavior
  found: hiding a webview window does NOT destroy the JS context or event listeners. The window is merely visually hidden; the WebView process keeps running and can still receive and process IPC events.
  implication: the window being hidden is NOT what prevents the toast. The toast would render inside the hidden window, which is then never seen.

- timestamp: 2026-03-17
  checked: hotkey/service.rs Err(()) branch sequence (lines 274-276)
  found: exact order is: (1) transcribe_with_retry emits transcription-error, (2) returns Err(()), (3) match arm calls indicator::hide(&app_clone). So indicator::hide is called AFTER the event is emitted.
  implication: the event emission happens before hide() — but hide() hides the window before the JS event loop in the indicator WebView processes the event, because hide() is a synchronous native call while the IPC event delivery is async.

- timestamp: 2026-03-17
  checked: timing relationship between app.emit() and WebView JS event loop
  found: app.emit() in Tauri queues an IPC message to the WebView's JS event loop. It does NOT block until JS processes it. indicator::hide() calls window.hide() which is a synchronous OS-level call that fires immediately. Result: OS hides the window BEFORE the JS event loop delivers the transcription-error event to the listener.
  implication: THIS IS THE ROOT CAUSE. The toast data arrives in the JS listener after the window is already hidden. The toast is rendered but invisible — it exists in the DOM of a hidden window that is never shown again because no code re-shows the indicator after an error.

## Resolution

root_cause: |
  In src-tauri/src/hotkey/service.rs lines 274-276, the Err(()) branch calls
  indicator::hide(&app_clone) immediately after transcribe_with_retry returns.

  transcribe_with_retry emits the transcription-error IPC event before returning,
  but app.emit() is asynchronous from the JS perspective — it merely enqueues
  the message to the WebView's JS event loop. indicator::hide() then calls the
  OS-level window.hide() synchronously, hiding the native window before JS
  delivers the event to the listener in App.vue.

  The toast IS created (the JS listener fires, showToast runs, the DOM updates),
  but it renders inside a window that is now hidden and never re-shown.
  The user never sees it.

fix: |
  Two complementary changes required:

  1. src-tauri/src/hotkey/service.rs — On transcription error, do NOT call
     indicator::hide() immediately. Instead, leave the indicator visible so
     the toast can be seen. The indicator should only hide after the user
     dismisses the toast OR after a timeout. This can be implemented as:
     - Keep indicator visible (show_idle state) on Err(()) instead of hide().
     - Add a "dismiss" IPC command or emit a "toast-dismissed" event from
       the frontend that triggers indicator hide.
     - Alternatively: delay the hide by the toast duration (5000ms) using
       tokio::time::sleep before calling hide, so the window stays visible
       long enough for the user to see and interact with the toast.

  2. src/windows/indicator/App.vue — The transcription-error listener should
     ensure the indicator window is visible when showing a toast. Currently
     it only calls showToast; it should also call a command or emit an event
     to keep the indicator shown (or re-show it if already hidden).

  Minimal targeted fix (least invasive):
  In hotkey/service.rs, replace:
    Err(()) => {
        indicator::hide(&app_clone);
    }
  With:
    Err(()) => {
        indicator::show_idle(&app_clone);  // keep visible for toast
    }
  And add a timer (tokio::time::sleep ~6s) before the final hide, or let
  the frontend dismiss by invoking a "hide_indicator" command from the
  toast dismiss handler.

verification: not yet applied
files_changed: []
