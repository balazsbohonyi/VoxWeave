# VoxWeave VAD, Resampling, Realtime Plan

## Summary

- **Rubato: adopt more effectively.** Already used at [audio/mod.rs](/D:/develop/projects/VoxWeave/src-tauri/src/audio/mod.rs:221), but resamplers are created inside callbacks and integer-rate inputs use unfiltered decimation. Upgrade `0.15 → 0.16`; move stateful resampling to an audio worker using preallocated buffers. Rubato enables streaming-quality sample conversion, not transcription itself. [Rubato documentation](https://docs.rs/rubato/latest/rubato/)
- **Silero: adopt conditionally.** Replace RMS-based speech classification with Silero v6.2.1 through `wavekat-vad = 0.1.16`. It can remove silence/non-speech and define turns, reducing upload/inference duration. It is not noise suppression, echo cancellation, or speaker identification. [Silero VAD](https://github.com/snakers4/silero-vad), [WaveKat VAD](https://github.com/wavekat/wavekat-vad)
- **Realtime: fourth injection mode.** Add opt-in `Realtime`; FlashPaste remains default. Support OpenAI Realtime, Groq turn uploads, and cached local Whisper. OpenAI uses a dedicated `gpt-realtime-whisper` setting; Groq remains file-per-turn with its documented 10-second minimum billing warning. [OpenAI model](https://developers.openai.com/api/docs/models/gpt-realtime-whisper), [Groq STT](https://console.groq.com/docs/speech-to-text)
- No saved recordings/history. Compaction reduces transient payload and inference size only.

## Phased Implementation

### 0. Dependency and Packaging Gate

- Pin Rubato 0.16, WaveKat 0.1.16, Silero model v6.2.1, and compatible CPU-only ONNX Runtime.
- Commit the ~2.3 MB model with source, MIT license, and SHA-256; configure `SILERO_MODEL_PATH` for deterministic offline builds.
- Bundle ONNX Runtime DLL and notices in NSIS and portable ZIP; verify from an isolated directory without system ONNX installation.
- Benchmark on Windows x64:
  - Compressed installer and ZIP growth ≤25 MB.
  - Silero 32 ms frame inference p95 ≤5 ms.
  - Every labeled fixture utterance detected and preserved after padding.
  - No turns from pure silence/non-speech fixtures.
- If any gate fails, stop and publish results; do not continue with RMS or an unapproved replacement.

### 1. Audio Worker and Speech Compaction

- Replace callback-side locking, allocation, resampler construction, and `step_by` decimation with a bounded non-blocking capture queue and dedicated worker.
- Construct Rubato resamplers once per stream; produce canonical 16 kHz mono PCM and a 24 kHz branch only for OpenAI Realtime.
- Never block CPAL. On overflow: drop, count, warn once, mark realtime turn integrity unreliable, and degrade to retained batch audio.
- Add an internal VAD trait with WaveKat/Silero implementation:
  - 512-sample/32 ms frames, fixed threshold `0.5`.
  - 200 ms pre-roll and post-roll.
  - 800 ms silence finalizes a turn.
  - Batch compaction joins speech runs with 150 ms synthetic silence.
  - Existing long-silence auto-stop remains separate and defaults to 15 seconds.
- Apply compaction to FlashPaste, Keystroke, and Clipboard by default; advanced batch-only bypass.
- Preserve untrimmed accepted PCM for error fallback. If no speech is detected—even on manual stop—skip encoding/provider/local inference and show “No speech detected.”
- Deprecate, but continue deserializing and preserving, the old RMS `vad_threshold`; never reinterpret it as Silero’s threshold.

### 2. Realtime Transcription Pipeline

- Extend [InjectionMode](/D:/develop/projects/VoxWeave/src-tauri/src/config/mod.rs:16) with `Realtime`; preserve existing config migrations and defaults.
- Introduce a per-recording realtime session abstraction:
  - OpenAI: persistent Realtime connection, stream 24 kHz PCM, client-side Silero commits, 20-second result timeout.
  - Groq: encode/upload completed VAD turns; show persistent minimum-billing warning.
  - Local: process completed turns through one lazily loaded, app-cached Whisper context; reload only when model changes/disappears.
- Process turns serially and in order. Allow one active plus two queued; the next turn degrades all uncommitted audio to one batch remainder at stop.
- Try only the next configured fallback provider per failed/timed-out turn. First successful fallback becomes active for the rest of that recording.
- Supply the last 50 committed words as rolling context, truncated further for provider limits.
- Normalize whitespace to one space; never synthesize Enter in Realtime. Remove only exact 2–8 word overlaps between adjacent turns.
- Injection behavior:
  - Finalized turns only; no provisional text injection.
  - Keystroke first using existing speed setting, then FlashPaste.
  - Keep original target pinned. If focus changes, pause writes and inject buffered remainder to the original target only at stop.
  - Degraded remainder: FlashPaste once, then Clipboard.
  - Clipboard always contains the complete committed session transcript.
  - Cancel keeps committed text, cancels/discards everything uncommitted.

### 3. Caption Overlay and Settings

- Add a fifth Tauri window: click-through, always-on-top caption overlay, approximately 560×110 px, anchored above and moving with the indicator.
- Show the latest three wrapped lines of finalized text plus listening/transcribing state. No provider-specific provisional words.
- Caption setting defaults on for Realtime and may be disabled for privacy. Clear all caption/session data at completion or cancellation.
- Add mirrored Rust/TypeScript configuration:
  - `InjectionMode::Realtime`
  - `audio.vad_compaction_enabled = true`
  - `audio.vad_turn_pause_ms = 800`
  - `injection.realtime_captions_enabled = true`
  - `providers.openai.realtime_model = "gpt-realtime-whisper"`
- Add typed caption/state/warning events; keep the existing batch `TranscriptionProviderTrait` compatible while adding the internal realtime-session interface.

## Verification

- Unit tests: resampling lengths/continuity/alias rejection, VAD framing/padding/compaction, zero-speech short-circuit, whitespace/dedup/context rules, config migration.
- Fixture tests: synthetic and CC0/public-domain speech, silence, fan, keyboard, and background playback; include provenance/license metadata. Background speech is expected to count as speech.
- Orchestrator tests: ordered commits, sticky failover, 20-second timeout, queue degradation, focus loss, cancellation, clipboard state, provider/VAD failure.
- Packaging tests: isolated NSIS and portable launches, DLL discovery, embedded model inference, ≤25 MB compressed growth.
- Live pre-release smoke tests: OpenAI Realtime, Groq, and cached local Whisper.
- Manual Windows UAT: Notepad, Chromium text field, Windows Terminal; validate typing, no Enter synthesis, focus loss, cancellation, and fallback.
- Performance: boundary-to-provider-dispatch ≤150 ms after the configured 800 ms pause; record provider completion/injection latency separately.
- Run `cargo test`, `cargo clippy`, `npx vue-tsc --noEmit`, and `npm run lint`; update graphify after implementation.

## Assumptions and Unresolved Questions

- Windows x64 first; architecture remains platform-neutral.
- FlashPaste remains default; Realtime is opt-in.
- No telemetry, transcript persistence, saved audio, speaker isolation, or noise suppression.
- Unresolved questions: none.
