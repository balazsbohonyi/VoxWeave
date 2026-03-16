# Phase 4: Floating Indicator - Research

**Researched:** 2026-03-15
**Domain:** Tauri secondary window lifecycle, non-focus visual feedback UI, and low-latency audio-level rendering
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### State visuals
- Recording uses red pulsing dot plus live waveform bars.
- Processing uses spinner with muted bars.
- Injecting uses method-specific cue plus short verb label.
- State transitions should crossfade quickly (roughly 120-180ms), not abrupt swaps.

### Waveform behavior
- Render 10 bars.
- Indicator target size ~260x48.
- Smooth but fast-reacting bars, with low baseline pulse during silence.
- On transition to processing, freeze waveform briefly, then fade.

### Positioning and drag behavior
- Indicator default is click-through.
- Press-and-hold temporarily enters drag mode, then returns to click-through on release.
- Drag bounds keep the full indicator visible on nearest monitor.
- Persist exact position; if invalid after display layout change, recover to safe visible location.
- Default first-run location is bottom-right above taskbar.

### Carry-forward constraints
- Keep backend lifecycle naming from prior phases (`Idle -> Recording -> Transcribing`) and map UI copy where needed.
- Preserve tray-first behavior; indicator must never steal focus.

### Claude's Discretion
- Exact injecting icons and labels.
- Exact easing/timing values within the requested transition range.
- Exact monitor-safe margin values.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FLOT-01 | Pill-shaped indicator appears when recording starts | Create dedicated `indicator` window with hidden-by-default startup and runtime show on record start. |
| FLOT-02 | Always-on-top, click-through, no focus steal | Use always-on-top transparent undecorated window config, `set_ignore_cursor_events(true)` by default, and never call focus/show with focus side effects. |
| FLOT-03 | 5-10 bar realtime waveform >=24fps | Emit throttled `indicator-audio-level` payloads from capture path at ~30fps; frontend smooths bars with animation frame interpolation. |
| FLOT-04 | Distinct recording/processing/injecting visuals | Introduce indicator-specific phase event channel independent from backend recording-state enum, allowing injecting cue before Phase 6 completes. |
| FLOT-05 | Draggable + persisted position | Add temporary drag mode bridge and persist x/y into existing `config.indicator.position_x/position_y` immediately after drag end. |
| FLOT-06 | Auto-hide after injection/error | Emit completion/error lifecycle events and hide indicator deterministically in both success and failure paths. |
</phase_requirements>

## Summary

Phase 4 should be implemented as a dedicated indicator subsystem, not bolted onto settings UI. The repository currently has a settings-only webview entry (`index.html -> src/windows/settings/main.ts`) and no indicator frontend files beyond `.gitkeep`. The plan should therefore establish a separate indicator entrypoint and Tauri window wiring before styling/animation work.

To avoid destabilizing the existing phase state machine, indicator rendering should not depend exclusively on `RecordingState`. Instead, add an indicator event stream (`indicator-state`, `indicator-audio-level`, `indicator-hidden`) that can represent `recording`, `processing`, and `injecting` regardless of backend enum timing. This keeps Phase 4 independent from Phase 6 injection internals while still meeting the visual-state requirement.

The config model already contains `indicator.show`, `position_x`, and `position_y`, so drag persistence should reuse current save flow (`save_config`) and raw-json preservation. For click-through drag, backend should expose explicit begin/end drag commands that toggle ignore-cursor-events and enforce bounds before persisting.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri v2 window APIs | existing | Secondary indicator window lifecycle and click-through behavior | Already in project stack and required for desktop-level always-on-top behavior. |
| Vue 3 + TS | existing | Indicator UI state rendering and animation hooks | Existing frontend stack; keeps consistency with settings window. |
| CSS/Tailwind utility layer | existing | Lightweight pill, waveform bars, spinner, transitions | Meets "no heavy UI libraries" project decision. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tauri::Emitter` events | existing | Runtime backend -> indicator updates | For state/audio-level push at target frame rate. |
| `@tauri-apps/api/event` | existing | Indicator frontend event subscriptions | For lifecycle/audio payload consumption. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Separate indicator window | Overlay inside settings window | Rejected: breaks tray-first UX and cannot remain visible while settings hidden. |
| Polling audio levels from frontend | Push events from Rust | Rejected: higher latency/jitter and unnecessary IPC churn. |

## Architecture Patterns

### Pattern 1: Indicator Runtime Facade in Rust
Create a focused module (`src-tauri/src/indicator/`) to own show/hide, click-through toggles, and state event emission. Hotkey/audio modules call facade methods; they should not directly manipulate window properties.

### Pattern 2: Indicator-Specific Lifecycle Events
Use indicator domain states (`recording`, `processing`, `injecting`, `hidden`) so frontend behavior remains stable even while core pipeline state machine evolves in later phases.

### Pattern 3: Bounded Position Persistence
Persist drag end coordinates only after clamping into nearest monitor visible bounds. Validate saved coordinates on launch and fallback to bottom-right when invalid.

### Pattern 4: Decoupled Waveform Rendering
Backend emits normalized RMS values at fixed cadence; frontend performs smoothing and bar projection. This keeps audio callback lightweight and UI animation deterministic.

### Anti-Patterns to Avoid
- Reusing settings window as indicator surface.
- Driving indicator entirely from `RecordingState` transitions.
- Saving drag position continuously on every move event.
- Blocking audio capture thread with heavy event serialization.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Monitor geometry math from scratch in frontend | Custom JS screen probing | Tauri window/monitor APIs in backend | Correctness for multi-monitor bounds and DPI behavior. |
| High-frequency animation timers | ad-hoc setInterval loops | requestAnimationFrame with easing and input smoothing | Better visual stability and frame pacing. |
| Config persistence bypass | Separate indicator storage file | Existing `save_config` / config module | Preserves unknown fields and existing backward compatibility behavior. |

## Common Pitfalls

### Pitfall 1: Focus theft on show
Using APIs that focus the window on show can interrupt typing in the target app. Ensure indicator show path is non-activating and click-through.

### Pitfall 2: Choppy waveform from event burst/jitter
Raw audio callback emits irregular updates. Throttle to stable cadence (~30fps) and smooth on frontend.

### Pitfall 3: Drag mode leaves window interactive
If click-through is not restored after drag, indicator can intercept clicks unexpectedly. Always reset ignore-cursor-events on pointer release and on cancel paths.

### Pitfall 4: Persisted position becomes off-screen
Display layout changes can invalidate x/y. Revalidate on startup and recover gracefully to default anchor.

### Pitfall 5: Hide path only on success
Errors and cancellation must also hide indicator; otherwise stale floating UI remains.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust unit tests + Vue/TS checks + manual desktop validation |
| Config file | none - existing project test stack |
| Quick run command | `cd src-tauri && cargo test indicator -- --nocapture` |
| Full suite command | `cd src-tauri && cargo test && cd .. && npx vue-tsc --noEmit` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FLOT-01 | Recording start shows indicator window | unit + integration | `cd src-tauri && cargo test indicator::tests::recording_start_shows_indicator -- --exact` | no - Wave 1 |
| FLOT-02 | Window stays always-on-top/click-through/non-focus | unit + manual | `cd src-tauri && cargo test indicator::tests::window_policy_is_non_focus_click_through -- --exact` | no - Wave 1 |
| FLOT-03 | Waveform updates at >=24fps with 10 bars | unit + frontend | `cd src-tauri && cargo test indicator::tests::audio_level_throttle_target_fps -- --exact` and `npx vue-tsc --noEmit` | no - Wave 2 |
| FLOT-04 | Recording/processing/injecting visuals mapped correctly | frontend + unit | `cd src-tauri && cargo test indicator::tests::state_event_sequence -- --exact` | no - Wave 2 |
| FLOT-05 | Drag persists bounded position | unit + manual | `cd src-tauri && cargo test indicator::tests::persisted_position_clamped_to_monitor -- --exact` | no - Wave 2 |
| FLOT-06 | Indicator auto-hides on completion/error | unit + integration | `cd src-tauri && cargo test indicator::tests::hide_on_complete_or_error -- --exact` | no - Wave 3 |

### Sampling Rate
- **Per task commit:** run touched focused tests + `cd src-tauri && cargo test indicator -- --nocapture`
- **Per wave merge:** run `cd src-tauri && cargo test`
- **Before `$gsd-verify-work`:** Rust full suite + TypeScript typecheck green
- **Max feedback latency:** 60 seconds

### Wave 0 Gaps
- [ ] `src-tauri/src/indicator/mod.rs` - indicator runtime facade and events
- [ ] `src-tauri/src/indicator/window.rs` - window config/bounds helpers
- [ ] `src-tauri/src/indicator/events.rs` - event payload contracts
- [ ] `src/windows/indicator/main.ts` - indicator bootstrap entrypoint
- [ ] `src/windows/indicator/App.vue` - visual states and waveform UI
- [ ] `src/windows/indicator/components/Waveform.vue` - 10-bar waveform component

## Sources

### Primary (HIGH confidence)
- `.planning/phases/04-floating-indicator/04-CONTEXT.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `CLAUDE.md`
- Existing repository files: `src-tauri/src/lib.rs`, `src-tauri/src/hotkey/service.rs`, `src-tauri/src/audio/mod.rs`, `src-tauri/src/config/mod.rs`, `index.html`, `src/windows/settings/*`

### Secondary (MEDIUM confidence)
- Tauri v2 window/event behavior assumptions aligned with current project architecture.

## Metadata

**Research date:** 2026-03-15
**Valid until:** 2026-04-15

