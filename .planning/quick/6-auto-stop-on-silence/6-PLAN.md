---
phase: quick-6
plan: 1
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/audio/mod.rs
  - src/windows/settings/components/AudioSection.vue
autonomous: true
requirements: []

must_haves:
  truths:
    - "When auto-stop is enabled and user stays silent, recording stops automatically after vad_silence_ms"
    - "When auto-stop is disabled (vad_silence_ms=0), recording never auto-stops"
    - "Silence duration can be set up to 30 seconds in the UI"
  artifacts:
    - path: "src-tauri/src/audio/mod.rs"
      provides: "VAD silence timer in capture thread"
      contains: "vad_silence_ms"
    - path: "src/windows/settings/components/AudioSection.vue"
      provides: "Raised max silence duration to 30s"
  key_links:
    - from: "capture thread RMS measurement"
      to: "stop_flag AtomicBool"
      via: "silence duration accumulator in emit loop"
      pattern: "silence_elapsed_ms"
---

<objective>
Implement the VAD (voice activity detection) auto-stop feature that exists in config and UI but was never wired up in the Rust backend.

Purpose: The feature is completely non-functional — users who enable it get no benefit. Recording never auto-stops on silence.
Output: Working auto-stop: when RMS stays below vad_threshold for vad_silence_ms, recording stops as if the user pressed the hotkey. Also raise the UI max from 5s to 30s.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement VAD silence timer in the audio capture thread</name>
  <files>src-tauri/src/audio/mod.rs</files>
  <action>
The bug: `start_realtime_level_capture` never reads `vad_threshold` or `vad_silence_ms` from config. The emit loop checks `stop_for_thread` (the 5-min hard cap flag) but has no silence tracking.

Fix in `start_realtime_level_capture`:

1. Accept two new parameters: `vad_threshold: f32` and `vad_silence_ms: u32` (pass them in from `start_recording_with_snapshot` where the config is already read).

2. In the emitter thread's emit loop (`while !stop_for_thread.load(Ordering::Relaxed)`), add silence tracking:
   - Add `let mut silence_elapsed_ms: u64 = 0;` before the loop.
   - On each loop iteration (which sleeps `AUDIO_LEVEL_EMIT_INTERVAL_MS` ms):
     - Read current RMS from `latest_rms`.
     - If `vad_silence_ms > 0` (feature enabled): if `rms < vad_threshold`, add `AUDIO_LEVEL_EMIT_INTERVAL_MS` to `silence_elapsed_ms`; else reset `silence_elapsed_ms = 0`.
     - If `silence_elapsed_ms >= vad_silence_ms as u64`: set `stop_flag.store(true, Ordering::Relaxed)` and break the loop — this causes `stop_realtime_level_capture` to join cleanly, which eventually leads `stop_recording_and_encode` to run when the hotkey service detects the stop flag.

3. The stop_flag being set to true stops the thread, but that alone does not trigger the recording-stop pipeline (transcription etc). The frontend must be notified. Emit a new Tauri event `"vad-silence-stop"` via `app_for_thread.emit("vad-silence-stop", ())` just before breaking, so the frontend can invoke the same stop path as a hotkey press.

4. In `start_recording_with_snapshot`, read `vad_threshold` and `vad_silence_ms` from the config before calling `start_realtime_level_capture`, and pass them through.

5. Update the `start_realtime_level_capture` function signature accordingly. The `#[cfg(not(test))]` guard means test code is unaffected.

Rust compile check: `cd src-tauri && cargo check` must pass.
  </action>
  <verify>
    <automated>cd D:\develop\projects\VoxWeave\src-tauri && cargo check 2>&1</automated>
  </verify>
  <done>
    `cargo check` passes. VAD silence timer is in the emit loop: when `vad_silence_ms > 0` and RMS stays below `vad_threshold` for that many ms, `stop_flag` is set and `"vad-silence-stop"` event is emitted.
  </done>
</task>

<task type="auto">
  <name>Task 2: Wire vad-silence-stop event in the indicator frontend and raise UI max to 30s</name>
  <files>src/windows/indicator/App.vue, src/windows/settings/components/AudioSection.vue</files>
  <action>
**Part A — Frontend event handler (indicator/App.vue or the file that owns recording event listeners):**

Find where `audio-level` or recording-related Tauri events are listened to in the indicator window. Add a listener for `"vad-silence-stop"` that calls the same `invoke("toggle_recording")` path (or whatever Tauri command triggers the Recording→Transcribing transition in `hotkey/service.rs`).

Check `src/windows/indicator/` for how the hotkey stop is currently triggered from the frontend. The indicator app likely calls `invoke` on some command or re-emits a message. Mirror that exact call for VAD auto-stop.

If the indicator does not own hotkey invocation (the hotkey is registered natively), then `"vad-silence-stop"` should instead invoke a new or existing Tauri command that calls `toggle_recording_state` on the app — the same function that the hotkey fires. The cleanest path: expose a `#[tauri::command] pub fn trigger_stop_recording(app: AppHandle)` in `commands/` that calls `hotkey::service::toggle_recording_state(&app)` if state is Recording, then register it. Call it from the frontend on `"vad-silence-stop"`.

**Part B — UI max (AudioSection.vue):**

1. Change `max="5.0"` → `max="30.0"` on the `<input type="number">` element.
2. Change `Math.min(5000, val * 1000)` → `Math.min(30000, val * 1000)` in `onSilenceDurationInput`.
3. Keep `Math.max(500, ...)` (minimum 0.5s stays).

TypeScript check: `npx vue-tsc --noEmit` must pass.
  </action>
  <verify>
    <automated>cd D:\develop\projects\VoxWeave && npx vue-tsc --noEmit 2>&1 && cd src-tauri && cargo check 2>&1</automated>
  </verify>
  <done>
    Both checks pass. `"vad-silence-stop"` event is listened to in the indicator and triggers recording stop. UI allows silence timeouts up to 30s. Silence of 1.5s (default) with no speech should auto-stop recording.
  </done>
</task>

</tasks>

<verification>
Manual test: enable auto-stop on silence (1.5s default), start recording with hotkey, stay silent — recording should stop and transcription should begin within ~1.5s of silence. Disable auto-stop — recording should wait indefinitely.

`cd src-tauri && cargo test` must pass. `npx vue-tsc --noEmit` must pass.
</verification>

<success_criteria>
- `cargo check` and `cargo test` pass
- `npx vue-tsc --noEmit` passes
- VAD logic present in `start_realtime_level_capture`: silence timer increments when RMS below threshold, fires `"vad-silence-stop"` and sets stop_flag when silence exceeds `vad_silence_ms`
- Frontend handles `"vad-silence-stop"` to trigger the stop-recording pipeline
- UI max silence duration raised from 5s to 30s
</success_criteria>

<output>
After completion, create `.planning/quick/6-auto-stop-on-silence/6-SUMMARY.md` following the summary template.
</output>
