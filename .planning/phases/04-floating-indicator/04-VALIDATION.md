---
phase: 04
slug: floating-indicator
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-15
updated: 2026-03-15
---

# Phase 04 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness + Vue/TS checks + manual desktop runs |
| **Config file** | none - existing project setup |
| **Quick run command** | `cd src-tauri && cargo test indicator -- --nocapture` |
| **Full suite command** | `cd src-tauri && cargo test && cd .. && npx vue-tsc --noEmit` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** run focused indicator tests touched by the task, then `cd src-tauri && cargo test indicator -- --nocapture`
- **After every plan wave:** run `cd src-tauri && cargo test`
- **Before `$gsd-verify-work`:** Rust full suite and TypeScript typecheck must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | FLOT-01, FLOT-02 | unit | `cd src-tauri && cargo test indicator::tests::recording_start_shows_indicator -- --exact` | yes after Wave 1 | pending |
| 04-01-02 | 01 | 1 | FLOT-02, FLOT-06 | unit | `cd src-tauri && cargo test indicator::tests::hide_on_complete_or_error -- --exact` | yes after Wave 1 | pending |
| 04-01-03 | 01 | 1 | FLOT-02 | manual | `cargo tauri dev` then verify indicator does not steal focus while typing in another app | yes after Wave 1 | pending |
| 04-02-01 | 02 | 2 | FLOT-03 | unit | `cd src-tauri && cargo test indicator::tests::audio_level_throttle_target_fps -- --exact` | yes after Wave 2 | pending |
| 04-02-02 | 02 | 2 | FLOT-04 | unit + frontend | `cd src-tauri && cargo test indicator::tests::state_event_sequence -- --exact` and `npx vue-tsc --noEmit` | yes after Wave 2 | pending |
| 04-02-03 | 02 | 2 | FLOT-03, FLOT-04 | manual | `cargo tauri dev` then verify 10-bar waveform + state crossfade behavior | yes after Wave 2 | pending |
| 04-03-01 | 03 | 2 | FLOT-05 | unit | `cd src-tauri && cargo test indicator::tests::persisted_position_clamped_to_monitor -- --exact` | yes after Wave 2 | pending |
| 04-03-02 | 03 | 2 | FLOT-05 | manual | `cargo tauri dev` drag indicator across monitors, restart app, verify persisted position | yes after Wave 2 | pending |
| 04-03-03 | 03 | 2 | FLOT-06 | integration | `cd src-tauri && cargo test indicator::tests::hide_on_complete_or_error -- --exact` | yes after Wave 2 | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

Existing test and build infrastructure is sufficient once Wave 1 introduces the indicator module and test seams.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Indicator appears without stealing focus from active app | FLOT-01, FLOT-02 | Requires real desktop focus behavior and cross-app interaction | Run `cargo tauri dev`, focus terminal/editor, trigger recording, confirm typing focus stays in target app |
| Waveform visual quality at target cadence | FLOT-03 | Visual smoothness and pulse behavior are UX characteristics | Trigger recording in quiet and speaking conditions; confirm 10 bars update smoothly and maintain baseline pulse in silence |
| State visuals and transitions | FLOT-04 | Requires real rendering and timing judgment | Trigger record -> stop -> inject simulation; confirm recording/processing/injecting cues with quick crossfades |
| Drag bounds and position persistence | FLOT-05 | Requires monitor geometry and restart behavior | Drag near screen edges and across monitors, restart app, confirm position persists and remains visible |
| Hide behavior on both success and error | FLOT-06 | Depends on runtime success/error flows | Simulate successful completion and audio/injection error events; confirm indicator hides in both cases |

---

## Validation Sign-Off

- [x] All tasks have automated verify or explicit manual verification
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all missing references
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** planned 2026-03-15

---

## Implementation Notes (Final UI/Behavior)

- Indicator finalized as rounded rectangle (`180x70`), always-on-top, draggable from non-dot area.
- Recording control bound to clickable dot: click toggles start/stop recording.
- Waveform finalized as segmented square bars (15 columns) with randomized temporary animation for Phase 4 UX validation.
- Recording label color aligned with active red recording dot.
- Runtime state synchronization is event-driven (`indicator-state` + `audio-level`); temporary 150ms frontend polling fallback was removed after permissions fix.
- `show_on_startup` behavior preserved: indicator remains visible in idle state when configured.
