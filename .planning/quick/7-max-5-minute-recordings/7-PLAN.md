---
phase: quick-7
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/audio/mod.rs
  - src/windows/indicator/App.vue
autonomous: true
requirements: []
must_haves:
  truths:
    - "Recording stops automatically after exactly 5 minutes"
    - "A warning toast appears at 4.5 minutes informing the user recording is near the limit"
    - "After auto-stop at 5 min, transcription and injection proceed as normal"
  artifacts:
    - path: "src-tauri/src/audio/mod.rs"
      provides: "Emits recording-limit-stop event when hard cap is hit"
    - path: "src/windows/indicator/App.vue"
      provides: "Listens for recording-near-limit (shows toast) and recording-limit-stop (triggers stop)"
  key_links:
    - from: "src-tauri/src/audio/mod.rs accumulate_pcm_chunk"
      to: "indicator App.vue listener"
      via: "app.emit(recording-limit-stop)"
    - from: "indicator App.vue"
      to: "commands::audio::trigger_stop_recording"
      via: "invoke(trigger_stop_recording)"
---

<objective>
Fix the 5-minute recording auto-stop feature. The Rust hard cap already truncates the PCM buffer and sets stop_flag at 4,800,000 samples, and the near-limit event is emitted at 4,320,000 samples — but neither event is wired to actually stop the recording or notify the user.

Purpose: Users dictating long passages would record indefinitely past 5 minutes with no feedback, then get a garbled or oversized transcript.
Output: Recording stops at 5 min, user warned at 4.5 min via toast.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md

Key findings from investigation:

RUST (src-tauri/src/audio/mod.rs):
- `accumulate_pcm_chunk` sets `stop_flag.store(true, Ordering::Relaxed)` when buf.len() > HARD_CAP (4,800,000)
- `stop_flag` is the same AtomicBool the emitter loop polls (`while !stop_for_thread.load(...)`) — so the loop exits, dropping the cpal stream. Good.
- BUT: setting stop_flag does NOT emit any event. The frontend never learns the hard cap was hit. Recording state machine stays in Recording on the Rust side — it waits for the next hotkey press or explicit stop command.
- `app.emit("recording-near-limit", ())` IS emitted at 4,320,000 samples. Good.

FRONTEND (src/windows/indicator/App.vue):
- Listens for `"vad-silence-stop"` → calls `invoke("trigger_stop_recording")`. This is the pattern to follow.
- Does NOT listen for `"recording-near-limit"` → no warning shown to user.
- Does NOT listen for any hard-cap event → stop never triggered.

TOAST MECHANISM:
- Toast window uses `window.__voxweaveShowToast` called via Rust's `show_plain_toast` command.
- BUT for a warning emitted from inside the audio capture thread, the simpler path is: emit the event to the indicator window, and the indicator calls `invoke("show_plain_toast", ...)` or uses the `show_toast_window` command.
- Actually simplest: emit `"recording-near-limit"` is already done; indicator can call `invoke("show_plain_toast", { toastType: "warning", message: "..." })` when it receives it. This matches the existing show_plain_toast Tauri command signature: `(toast_type: String, message: String)`.

PLAN:
1. Rust: in accumulate_pcm_chunk, after setting stop_flag, also emit `"recording-limit-stop"` event (mirrors vad-silence-stop pattern).
2. Frontend: add two listeners in onMounted:
   a. `"recording-near-limit"` → invoke show_plain_toast with warning "Recording will stop in 30 seconds (5-minute limit)"
   b. `"recording-limit-stop"` → invoke trigger_stop_recording (same as vad-silence-stop)
   Also add the two unlisten refs and clean up in onBeforeUnmount.
</context>

<tasks>

<task type="auto">
  <name>Task 1: Emit recording-limit-stop from Rust hard cap</name>
  <files>src-tauri/src/audio/mod.rs</files>
  <action>
In `accumulate_pcm_chunk` (around line 289-294), after `stop_flag.store(true, Ordering::Relaxed)`, add:

```rust
let _ = app.emit("recording-limit-stop", ());
```

The block currently looks like:
```rust
if buf.len() > HARD_CAP {
    buf.truncate(HARD_CAP);
    // Signal the recording to stop at the hard cap.
    stop_flag.store(true, Ordering::Relaxed);
}
```

Change to:
```rust
if buf.len() > HARD_CAP {
    buf.truncate(HARD_CAP);
    // Signal the capture thread to exit.
    stop_flag.store(true, Ordering::Relaxed);
    // Notify the frontend to trigger the full stop pipeline.
    let _ = app.emit("recording-limit-stop", ());
}
```

No other changes to this file. Verify the function signature still has `app: &AppHandle<R>` (it does — it's the last parameter).
  </action>
  <verify>
    <automated>cd src-tauri && cargo check 2>&1 | grep -E "^error" | head -20; echo "exit:$?"</automated>
  </verify>
  <done>cargo check passes with no errors. The emit call compiles.</done>
</task>

<task type="auto">
  <name>Task 2: Wire recording-near-limit and recording-limit-stop in indicator frontend</name>
  <files>src/windows/indicator/App.vue</files>
  <action>
Make three changes to App.vue:

**1. Add two unlisten refs** (after the existing `unlistenVadSilenceStop` ref, around line 25):
```ts
let unlistenNearLimit: UnlistenFn | null = null;
let unlistenLimitStop: UnlistenFn | null = null;
```

**2. Add two listeners in onMounted** (after the existing `unlistenVadSilenceStop` block, around line 218-220):
```ts
  unlistenNearLimit = await listen("recording-near-limit", () => {
    void invoke("show_plain_toast", { toastType: "warning", message: "Recording will stop in 30 seconds (5-minute limit reached)." });
  });

  unlistenLimitStop = await listen("recording-limit-stop", () => {
    void invoke("trigger_stop_recording");
  });
```

**3. Clean up in onBeforeUnmount** (after the existing `if (unlistenVadSilenceStop)` line, around line 249):
```ts
  if (unlistenNearLimit) unlistenNearLimit();
  if (unlistenLimitStop) unlistenLimitStop();
```

The `show_plain_toast` Tauri command signature is `(toast_type: String, message: String)` — Tauri snake_case → camelCase mapping means invoke args are `{ toastType, message }`.
  </action>
  <verify>
    <automated>npx vue-tsc --noEmit 2>&1 | grep -E "error TS" | head -20; echo "exit:$?"</automated>
  </verify>
  <done>vue-tsc passes with no type errors. Both listeners are registered and cleaned up.</done>
</task>

</tasks>

<verification>
After both tasks:
1. `cd src-tauri && cargo check` — no errors
2. `npx vue-tsc --noEmit` — no errors
3. Manual smoke test: start recording, wait 4.5 min → warning toast appears; wait until 5 min → recording auto-stops and transcription runs
</verification>

<success_criteria>
- Recording stops automatically at 5 minutes without requiring hotkey press
- Warning toast "Recording will stop in 30 seconds..." appears at the 4.5-minute mark
- After auto-stop, transcription proceeds normally (same pipeline as hotkey-stop)
- cargo check and vue-tsc both pass
</success_criteria>

<output>
After completion, create `.planning/quick/7-max-5-minute-recordings/7-SUMMARY.md`
</output>
