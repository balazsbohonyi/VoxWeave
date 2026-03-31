---
status: diagnosed
phase: 04-floating-indicator
source: .planning/phases/04-floating-indicator/04-01-indicator-window-runtime-SUMMARY.md, .planning/phases/04-floating-indicator/04-02-indicator-visual-states-waveform-SUMMARY.md, .planning/phases/04-floating-indicator/04-03-indicator-drag-persistence-hide-SUMMARY.md
started: 2026-03-15T10:48:56.1871314Z
updated: 2026-03-15T21:21:17.2779425Z
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start Smoke Test
expected: Kill any running VoxWeave process. Start the app from scratch. The settings window and backend boot without startup errors, the indicator window remains hidden at idle, and hotkey recording can be started successfully.
result: pass

### 2. Indicator Shows on Recording Start
expected: Triggering the hotkey to start recording shows the floating indicator quickly and keeps it visible while recording is active.
result: pass

### 3. Indicator State Transitions
expected: After stopping recording, the indicator changes from recording to processing; during injection it shows injecting cue/state; when finished it hides automatically.
result: skipped
reason: User cannot fully test because transcription/injection are future phases; confirmed ready -> recording -> ready works.

### 4. Waveform Reacts to Live Audio
expected: While recording, speaking into the microphone causes visible waveform bar movement; when silent, bars settle down smoothly.
result: issue
reported: "Waveform animates, but does not react to actual captured voice amplitude."
severity: blocker
reason: "Confirmed implementation gap in Phase 4 scope (FLOT-03): frontend listens for `audio-level` but backend capture path does not emit `audio-level`; current UI motion is synthetic fallback, not mic-driven."

### 5. Indicator Stays Non-Intrusive
expected: While visible, the indicator does not steal focus from the active app and remains click-through except during explicit drag interaction.
result: issue
reported: "the indicator steals the focus, so this test failed"
severity: blocker

### 6. Drag and Reposition Indicator
expected: Press-and-hold drag on indicator allows moving it; releasing ends drag cleanly without stuck interactive mode.
result: pass

### 7. Position Persistence and Bounds Recovery
expected: After moving indicator and restarting app, indicator reopens at saved position; if saved position is off-screen/invalid, it is clamped back into visible monitor bounds.
result: issue
reported: "this failed. Also, it looks like the position is always reset to what's in the settings file when starting/stopping recordings. So if I start recording, drag it away, stop recording, it will be re-positioned where it was when I started recording."
severity: blocker

## Summary

total: 7
passed: 3
issues: 2
pending: 0
skipped: 2

## Gaps

- truth: "While visible, the indicator does not steal focus from the active app and remains click-through except during explicit drag interaction."
  status: failed
  reason: "User reported: the indicator steals the focus, so this test failed"
  severity: blocker
  test: 5
  root_cause: "Indicator window is configured and shown as focusable/interactive by default (ocus: true and set_ignore_cursor_events(false)), so showing it can activate and steal focus."
  artifacts:
    - path: "src-tauri/tauri.conf.json"
      issue: "Indicator window has focus enabled (ocus: true)."
    - path: "src-tauri/src/indicator/window.rs"
      issue: "Window policy sets interactive mode (set_ignore_cursor_events(false)) for normal state."
    - path: "src-tauri/src/indicator/mod.rs"
      issue: "Show path reapplies interactive policy before window.show()."
    - path: "src/windows/indicator/App.vue"
      issue: "Drag lifecycle does not use begin/end drag IPC toggles to temporarily switch interaction."
  missing:
    - "Make indicator non-focus and click-through by default."
    - "Enable interactivity only during explicit drag start and restore click-through on drag end."
  debug_session: ".planning/debug/indicator-focus-steal.md"

- truth: "After moving indicator and restarting app, indicator reopens at saved position; if saved position is off-screen/invalid, it is clamped back into visible monitor bounds."
  status: failed
  reason: "User reported: this failed. Also, it looks like the position is always reset to what's in the settings file when starting/stopping recordings. So if I start recording, drag it away, stop recording, it will be re-positioned where it was when I started recording."
  severity: blocker
  test: 7
  root_cause: "Frontend persists indicator coordinates at drag start (pointerdown) rather than drag end, while runtime show/hide paths re-place from config on state transitions, causing snap-back to stale coordinates."
  artifacts:
    - path: "src/windows/indicator/App.vue"
      issue: "persist_indicator_position is called during pointerdown; no explicit drag-end final position persist path."
    - path: "src-tauri/src/indicator/mod.rs"
      issue: "show_with_state/show_idle place window from config on transitions, reusing stale saved position."
    - path: "src-tauri/src/hotkey/service.rs"
      issue: "Recording lifecycle transitions call show/hide paths that trigger re-placement."
  missing:
    - "Persist indicator position at drag end using final dropped coordinates."
    - "Avoid config-based repositioning during state changes when window is already visible, or sync config before transition."
  debug_session: ".planning/debug/indicator-position-reset.md"

- truth: "While recording, speaking into the microphone causes visible waveform bar movement; when silent, bars settle down smoothly."
  status: failed
  reason: "Observed behavior is animated but not mic-reactive. This is not a future-phase feature: roadmap/requirements assign waveform-reactive behavior to Phase 4 (FLOT-03)."
  severity: blocker
  test: 4
  root_cause: "Indicator frontend is wired to consume `audio-level`, but backend audio runtime does not emit `audio-level` from real capture; frontend therefore falls back to synthetic waveform motion (`Math.max(level, synthetic)`), which masks missing mic-driven updates."
  artifacts:
    - path: ".planning/ROADMAP.md"
      issue: "Phase 4 explicitly owns live waveform behavior (>=24fps while recording)."
    - path: ".planning/REQUIREMENTS.md"
      issue: "FLOT-03 is marked complete in Phase 4, but runtime evidence contradicts true mic-reactive delivery."
    - path: "src/windows/indicator/App.vue"
      issue: "Waveform includes synthetic baseline animation path (`synthetic`, `Math.max(level.value, synthetic)`) and only updates level when `audio-level` arrives."
    - path: "src-tauri/src/audio/mod.rs"
      issue: "Audio pipeline still uses synthetic PCM for encode contract and emits only `audio-warning`/`audio-error`; no `audio-level` emission path found."
    - path: "src-tauri/src/indicator/mod.rs"
      issue: "Contains FPS/state contract tests only; no integration assertion that real mic amplitude is propagated as `audio-level` events."
  missing:
    - "Emit normalized RMS/level events (`audio-level`) from active recording capture callback at target cadence (~24-30fps)."
    - "Bind indicator waveform strictly to emitted mic level for recording state and reduce/remove synthetic fallback to avoid false positives in UAT."
    - "Add integration test coverage proving backend emits `audio-level` during active recording session and indicator responds."
    - "Update UAT wording: this is an unresolved Phase 4 gap, not deferred to a future phase."
  debug_session: ".planning/debug/indicator-waveform-not-reactive.md"
