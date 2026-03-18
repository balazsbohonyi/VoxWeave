---
created: 2026-03-18T22:19:15.866Z
title: Implement real PCM accumulation and Opus encoder
area: general
files:
  - src-tauri/src/audio/mod.rs:157-189
  - src-tauri/src/audio/encode.rs:24-29
  - src-tauri/src/audio/capture.rs
  - src-tauri/src/transcription/service.rs
---

## Problem

Phase 3 audio capture was implemented as a stub seam. Two critical pieces are missing before cloud transcription receives real audio:

1. **No PCM buffer accumulation**: The cpal stream only updates `latest_rms` for the level meter animation. The actual microphone samples are never stored. `stop_recording_and_encode()` calls `synthetic_capture_pcm()` which returns `vec![0.0; 16_000]` — 1 second of hardcoded silence — instead of the real recorded audio. OpenAI Whisper receives silence and returns an empty string.

2. **Fake Opus encoder**: `DefaultEncoderBackend::encode_opus()` writes `OggSOpusHead` + 4-byte length (not valid Opus). Groq and OpenRouter receive garbage bytes and would return a 400 error. Only the WAV encoder is real and valid.

3. **Bonus: no log backend**: `log = "0.4"` facade is in Cargo.toml but no backend (env_logger, tauri-plugin-log) is initialized. All `log::info!` / `log::warn!` calls are silently discarded. Discovered while trying to trace transcription success.

Phase 5 cloud transcription context assumed Phase 3 would produce real audio. This gap must be closed before transcription is meaningful.

## Solution

1. Add a `Arc<Mutex<Vec<f32>>>` PCM accumulation buffer to `AppState` (or `AudioSessionState`). In the cpal data callback, append samples to this buffer in addition to updating the RMS atomic. In `stop_recording_and_encode`, drain the buffer instead of calling `synthetic_capture_pcm()`.

2. Replace the stub `encode_opus` with a real Opus encoder. Options: `opus` crate (requires libopus native dep) or `audiopus` crate. Alternatively, encode WAV for all providers in the short term (Groq accepts WAV too) and defer real Opus encoding.

3. Add `env_logger` (dev) or `tauri-plugin-log` and call `env_logger::init()` (or equivalent) in `main.rs` so Rust logs appear in `cargo tauri dev` console.
