# Phase 4: Floating Indicator - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the floating indicator window behavior for the recording pipeline: visible always-on-top pill during recording, live waveform, distinct state visuals (recording/processing/injecting), draggable positioning with persistence, and automatic hide after completion/error. This phase clarifies how the indicator behaves; it does not add new pipeline capabilities beyond indicator UX.

</domain>

<decisions>
## Implementation Decisions

### State visuals
- Recording state uses a red pulsing dot plus live waveform bars.
- Processing state uses a spinner plus muted bars for continuity.
- Injecting state uses method-specific icon plus short verb label (for example, "Pasting" / "Typing" / "Copied").
- State transitions use a quick crossfade (roughly 120-180ms), not instant swaps or slide motion.

### Waveform behavior
- Waveform renders 10 bars.
- Indicator target size is ~260x48 (wider than the original ~200px baseline to fit 10 bars cleanly).
- Bar movement uses smoothed fast response (light smoothing; quick rise/decay).
- During recording silence, bars keep a low baseline pulse instead of going fully flat.
- On transition to processing, waveform freezes briefly and then fades.

### Positioning and drag behavior
- Indicator is click-through by default.
- Press-and-hold on the indicator temporarily enters drag mode; on release, it returns to click-through.
- Drag bounds keep the indicator fully visible on the nearest monitor.
- Position persistence restores exact saved coordinates; if invalid (monitor/layout changed), fallback to a safe visible position.
- First-run default location is bottom-right, above taskbar.

### Carry-forward constraints
- Keep backend state naming and transitions from prior phases (`Idle -> Recording -> Transcribing`) and map UI copy as needed.
- Respect existing tray-first app behavior and avoid focus-stealing interactions.

### Claude's Discretion
- Exact icon set for injecting methods.
- Exact easing/timing values inside the chosen quick crossfade range.
- Exact safe-margin values used for bottom-right default and drag bounds.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src-tauri/src/config/mod.rs` already has `indicator.show`, `indicator.position_x`, and `indicator.position_y` fields for persistence.
- `src/windows/indicator/` exists as a window slot but has no implemented UI yet.
- `src/composables/useConfig.ts` and config IPC paths already persist settings immediately.

### Established Patterns
- Backend state machine currently transitions via hotkey service (`Idle -> Recording -> Transcribing`) and returns to `Idle` after placeholder completion.
- Rust emits frontend events for runtime warnings/errors (`audio-warning`, `audio-error`, `hotkey-warning`) and this pattern should be reused for indicator state/audio-level updates.
- Single managed `AppState` is the runtime source of truth for recording lifecycle.

### Integration Points
- Tauri setup/window lifecycle hooks in `src-tauri/src/lib.rs` are where indicator window creation/show/hide wiring should be added.
- Hotkey/audio lifecycle in `src-tauri/src/hotkey/service.rs` and `src-tauri/src/audio/mod.rs` are the trigger points for indicator state transitions and waveform event emission.
- Frontend indicator implementation belongs in `src/windows/indicator/` and should consume emitted state/audio-level events.

</code_context>

<specifics>
## Specific Ideas

- Keep the indicator visible and informative without stealing focus from the target app.
- Preserve continuity across states instead of abrupt visual resets.
- Default placement should stay out of the main work area (bottom-right above taskbar).

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within phase scope.

</deferred>

---

*Phase: 04-floating-indicator*
*Context gathered: 2026-03-15*