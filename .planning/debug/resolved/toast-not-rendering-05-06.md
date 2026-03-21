---
status: diagnosed
trigger: "Error toast not visible — indicator returns to IDLE silently after transcription error"
created: 2026-03-18T01:30:00Z
updated: 2026-03-18T01:30:00Z
---

## Symptoms

expected: Styled error toast appears in indicator with action button
actual: No toast visible. Indicator cycles IDLE > REC > TRANSCRIBING > IDLE silently.
reproduction: Set invalid API key or disconnect network, press hotkey, speak, stop

## Root Cause

`src/styles.css` — `.indicator-toasts` uses `position: absolute; bottom: calc(100% + 8px)` which places the toast container 8px *above* the pill's top edge.

The Tauri indicator window is a fixed-size OS window (~200×48px). The OS compositor clips any content outside the physical window rectangle. CSS `overflow: visible` only allows painting outside the *element box* within the WebView, not outside the OS window frame.

The toast IS pushed to `toasts` reactive array and IS rendered by Vue into the DOM. It is invisible because it occupies pixels above the window boundary.

This regression was introduced in 05-06 when the toast CSS was first added with `bottom: calc(100% + 8px)`. Before that, toasts rendered in normal document flow inside the window bounds.

## Fix

Render the toast as an in-pill overlay that replaces/extends the pill content while active. Two approaches:

**Option A (dynamic window resize):**
Use a `watch(toasts)` in App.vue that calls `getCurrentWindow().setSize(new LogicalSize(width, newHeight))` when toasts become active (expand window upward or downward), shrink back on empty.

**Option B (in-pill overlay, recommended):**
Change `.indicator-toasts` positioning to render inside the existing window bounds — either as an overlay on top of the pill content, or by conditionally showing toast content in place of the normal pill state. This avoids any Tauri window resize API calls.

Concrete minimal fix: change `bottom: calc(100% + 8px)` to a layout that renders within the existing window. The exact approach depends on the desired UX — overlay vs extended window.

verification: not yet applied
files_changed: []
