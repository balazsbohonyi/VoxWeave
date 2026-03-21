---
status: diagnosed
trigger: "recording too short error not shown as toast"
created: 2026-03-20T00:00:00Z
updated: 2026-03-20T00:00:00Z
---

## Current Focus

hypothesis: The "too short" error is returned as Err(String) from stop_recording_and_encode
           but the Err arm in toggle_recording_state only logs and hides the indicator —
           it never calls show_toast_window or emits any event.
test: Code trace from audio/mod.rs:167 → hotkey/service.rs:250-257
expecting: Confirmed — no toast path exists for this error code path
next_action: DONE — root cause identified, see Resolution

## Symptoms

expected: User sees a toast/error notification saying "Recording too short"
actual: Nothing visible. Error is logged with log::warn! and indicator is hidden.
errors: No error in UI; console/log shows "Failed to finalize recording: Recording too short (under 0.5 seconds)..."
reproduction: Press hotkey to start recording, immediately press hotkey again (< 0.5 s)
started: Always been broken; no toast path was ever wired for this code path

## Eliminated

- hypothesis: Error is swallowed inside stop_recording_and_encode before returning
  evidence: audio/mod.rs:167-170 returns Err(...) cleanly — no swallowing inside that function
  timestamp: 2026-03-20

- hypothesis: Frontend receives the error and drops it
  evidence: useRecording.ts is a stub (Phase 3 placeholder); it does not listen to any events.
            But the real path is hotkey/service.rs (Rust side), not the frontend invoke path.
  timestamp: 2026-03-20

- hypothesis: audio-error event is emitted for the too-short case like it is for EncodeFailed
  evidence: audio/mod.rs:173-186 shows emit_audio_error is only called for the Err branch of
            encode_with_retry_once. The too-short check at line 167 returns Err early BEFORE
            reaching encode_with_retry_once, so emit_audio_error is never called for it.
  timestamp: 2026-03-20

## Evidence

- timestamp: 2026-03-20
  checked: src-tauri/src/audio/mod.rs lines 135-187
  found: stop_recording_and_encode returns Err("Recording too short...") at line 168.
         The EncodeFailed branch (lines 176-184) calls emit_audio_error before returning Err.
         The too-short branch (lines 167-170) returns Err with NO event emission — it is a naked return.
  implication: No Tauri event is emitted for the too-short case at the audio layer.

- timestamp: 2026-03-20
  checked: src-tauri/src/hotkey/service.rs lines 249-257
  found: The Err arm of audio::stop_recording_and_encode is:
           log::warn!("Failed to finalize recording: {err}");
           indicator::hide(app);
           *state.recording_state.lock().unwrap() = RecordingState::Idle;
           tray::update_recording_menu(app, RecordingState::Idle);
         No call to indicator::show_toast_window or any event emission.
  implication: The error string is logged and then discarded. Nothing surfaces it to the user.

- timestamp: 2026-03-20
  checked: src-tauri/src/indicator/mod.rs show_toast_window
  found: show_toast_window exists and is used for transcription errors (via emit_transcription_error
         in transcription/service.rs). It takes any Serialize payload and delivers it via WebView eval.
  implication: The toast mechanism works and is reachable from Rust synchronous code — it just
               is not called from the too-short error path.

- timestamp: 2026-03-20
  checked: src-tauri/src/transcription/service.rs emit_transcription_error
  found: Uses indicator::show_toast_window(app, &payload). The payload type is
         TranscriptionErrorPayload which is specific to transcription errors.
         There is no equivalent AudioErrorPayload toast path for audio-layer errors.
  implication: A new toast payload type or a shared simple message payload is needed,
               OR show_toast_window can be called with an ad-hoc struct, since it accepts
               any Serialize type.

## Resolution

root_cause: |
  src-tauri/src/hotkey/service.rs, lines 250-257.

  When audio::stop_recording_and_encode returns Err, the match arm only calls log::warn!
  and hides the indicator. It never calls indicator::show_toast_window (or emits any event).

  The too-short error path in audio/mod.rs (line 167-170) also does not emit an audio-error
  event (unlike the EncodeFailed path at lines 176-184 which does call emit_audio_error).
  So there are TWO missing pieces, but the PRIMARY gap is in hotkey/service.rs — even if an
  event were emitted, nothing in the frontend is currently wired to display it as a toast.
  The transcription error path (transcription/service.rs) demonstrates the correct pattern:
  indicator::show_toast_window is called directly from Rust with the error payload.

fix: Not applied (goal: find_root_cause_only)

verification: N/A

files_changed: []
