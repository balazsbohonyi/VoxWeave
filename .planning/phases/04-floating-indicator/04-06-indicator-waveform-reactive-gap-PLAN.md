---
phase: 04-floating-indicator
plan: "04-06"
type: execute
wave: 3
depends_on:
  - 04-05-indicator-position-snapback-gap-PLAN.md
files_modified:
  - src-tauri/src/audio/mod.rs
  - src-tauri/src/indicator/mod.rs
  - src/windows/indicator/App.vue
autonomous: true
requirements:
  - FLOT-03
  - FLOT-04
gap_closure: true
must_haves:
  truths:
    - "While recording, speaking changes waveform bars in real time from mic amplitude."
    - "While silent, waveform settles smoothly to low baseline."
    - "Processing/injecting visuals stay distinct while waveform behavior remains truthful."
  artifacts:
    - path: "src-tauri/src/audio/mod.rs"
      provides: "Realtime normalized mic level emission on `audio-level` at stable cadence"
    - path: "src/windows/indicator/App.vue"
      provides: "Waveform driven by backend `audio-level` data (no synthetic masking)"
    - path: "src-tauri/src/indicator/mod.rs"
      provides: "Indicator state + hidden lifecycle compatible with live level stream"
  key_links:
    - from: "src-tauri/src/audio/mod.rs"
      to: "src/windows/indicator/App.vue"
      via: "app.emit(\"audio-level\", { rms }) and listen(\"audio-level\")"
      pattern: "audio-level"
    - from: "src/windows/indicator/App.vue"
      to: "indicator visual states"
      via: "recording uses live level, processing/injecting suppress reactive spikes"
      pattern: "state.value|isRecording|level.value"
---

<objective>
Close blocker gap: waveform animates but not mic-reactive (UAT test 4).

Purpose: satisfy true FLOT-03 behavior in runtime, not synthetic fallback.
Output: backend emits real `audio-level`; indicator waveform follows mic input.
</objective>

<execution_context>
@C:/Users/Balazs/.codex/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/04-floating-indicator/04-UAT.md
@src-tauri/src/audio/mod.rs
@src-tauri/src/indicator/mod.rs
@src/windows/indicator/App.vue

<interfaces>
From src/windows/indicator/App.vue:
- listens to `audio-level` and updates `level.value` from `payload.rms`.

From src-tauri/src/audio/mod.rs:
- current public API is recording start/stop; add event emission in active capture path, not synthetic fallback.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Emit real-time mic level events from active capture path</name>
  <files>src-tauri/src/audio/mod.rs, src-tauri/src/indicator/mod.rs</files>
  <action>Add `audio-level` emission from real recording callback using normalized RMS (0..1), throttled to ~24-30fps. Keep warning/error events intact. Add/extend tests proving level events are emitted during active recording flow (not idle, not synthetic encode-only path).</action>
  <verify>
    <automated>cd src-tauri && cargo test audio -- --nocapture</automated>
  </verify>
  <done>Backend provides real mic amplitude updates during recording at stable cadence.</done>
</task>

<task type="auto">
  <name>Task 2: Make waveform strictly data-driven for recording state</name>
  <files>src/windows/indicator/App.vue</files>
  <action>Remove/limit synthetic `Math.max(level, synthetic)` masking so recording waveform reflects backend `audio-level` truth. Keep smoothing/baseline behavior for UX, but it must not fake voice reactivity when no events arrive. Ensure processing/injecting visuals still transition cleanly.</action>
  <verify>
    <automated>cmd /c npx vue-tsc --noEmit</automated>
  </verify>
  <done>Speaking visibly changes bars; silence settles; no false-positive waveform activity from synthetic-only animation.</done>
</task>

</tasks>

<verification>
- `cd src-tauri && cargo test audio -- --nocapture`
- `cd src-tauri && cargo test indicator -- --nocapture`
- `cmd /c npx vue-tsc --noEmit`
- Manual: `cargo tauri dev`; start recording; speak vs silence; waveform reacts to voice amplitude and calms when silent.
</verification>

<success_criteria>
- UAT test 4 passes with live mic-reactive waveform.
- Gap entry for waveform-reactive blocker can be closed.
</success_criteria>

<output>
After completion, create `.planning/phases/04-floating-indicator/04-06-floating-indicator-04-06-SUMMARY.md`
</output>

<unresolved_questions>
None.
</unresolved_questions>
