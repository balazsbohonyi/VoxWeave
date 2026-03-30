---
phase: 10-local-transcription
plan: "01"
subsystem: transcription
tags: [whisper-rs, local-transcription, feature-gate, wav-processing]
dependency_graph:
  requires: []
  provides: [LocalProvider, wav_bytes_to_f32, ModelMissing error variant]
  affects: [transcription/service.rs, transcription/provider.rs]
tech_stack:
  added: [whisper-rs 0.16 (optional), reqwest stream feature, futures-util]
  patterns: [feature-gated optional dependency, spawn_blocking for CPU work, TDD]
key_files:
  created:
    - src-tauri/src/transcription/local.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/transcription/mod.rs
    - src-tauri/src/transcription/provider.rs
    - src-tauri/src/transcription/service.rs
decisions:
  - whisper-rs 0.16 used (latest 0.x, not 0.14 as stated in plan — 0.14 does not exist on crates.io)
  - wav_bytes_to_f32 compiled unconditionally (no whisper-rs dep) so unit tests run without CMake/libclang
  - LocalProvider impl gated in provider_impl submodule re-exported as pub use
  - make_provider signature extended to accept &TranscriptionConfig to pass model_path
  - ModelMissing and ModelLoadFailed short-circuit in transcribe_with_retry (non-retryable)
metrics:
  duration: "9 minutes"
  completed: "2026-03-29T11:37:02Z"
  tasks_completed: 2
  files_changed: 6
---

# Phase 10 Plan 01: LocalProvider Core Implementation Summary

LocalProvider implementing TranscriptionProviderTrait via whisper-rs 0.16, feature-gated behind `local-transcription`, with wav_bytes_to_f32 helper and ModelMissing/ModelLoadFailed error variants wired into the transcription error pipeline.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Add whisper-rs dep, create LocalProvider with wav helper and error variants | 18731a2 |
| 2 | Wire LocalProvider into make_provider dispatch | 425d6fc |

## What Was Built

### transcription/local.rs
- `wav_bytes_to_f32(wav_bytes: &[u8]) -> Result<Vec<f32>, String>`: skips 44-byte WAV header, converts i16 LE pairs to f32 / 32768.0; always compiled (no native dependency)
- `LocalProvider` struct + `TranscriptionProviderTrait` impl: gated behind `#[cfg(feature = "local-transcription")]`
  - Checks model file exists → returns `ModelMissing` if not
  - Converts WAV → f32 → runs inference via `tokio::task::spawn_blocking`
  - Sets language hint from `config.language`, disables progress/timestamp output
  - Collects segments and returns trimmed text

### transcription/provider.rs
- Added `ModelMissing { message: String }` variant to `TranscriptionError`
- Added `ModelLoadFailed { message: String }` variant to `TranscriptionError`

### transcription/service.rs
- Added `ModelMissing` variant to `TranscriptionErrorCode`
- Changed `make_provider(p, config)` signature to accept `&TranscriptionConfig`
- Local arm dispatches to `LocalProvider::new(model_path)` under feature gate; panics with descriptive message without feature
- Both callers updated: `transcribe_with_retry` and `transcribe_with_provider`
- Short-circuit arms for `ModelMissing` and `ModelLoadFailed` in `transcribe_with_retry` (non-retryable, no retry)
- Match arms in `transcribe_with_provider` for both new variants → both map to `ModelMissing` error code

## Verification Results

- `cargo test -- transcription`: 33/33 pass
- `cargo check` (non-feature): passes with only pre-existing warnings
- `npx vue-tsc --noEmit`: no errors
- 5 new unit tests in `transcription::local::tests`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] whisper-rs version 0.14 does not exist**
- **Found during:** Task 1 — Cargo resolves `0.14` to `0.13.1` (whisper-rs-sys), which fails with libclang error
- **Fix:** Updated to `whisper-rs = "0.16"` (latest available 0.x per plan instructions)
- **Files modified:** src-tauri/Cargo.toml
- **Commit:** 18731a2

**2. [Rule 2 - Missing functionality] wav_bytes_to_f32 tests can't run under full feature gate without CMake/libclang**
- **Found during:** Task 1 — `cargo test --features local-transcription` fails (libclang not installed)
- **Issue:** Plan spec says entire local.rs is behind feature gate, but build environment lacks CMake/libclang needed by whisper-rs-sys/bindgen
- **Fix:** Restructured local.rs — `wav_bytes_to_f32` and its tests are compiled unconditionally; `LocalProvider` (which links whisper-rs) stays in a `#[cfg(feature = "local-transcription")]` submodule. This satisfies both "tests pass" and "feature gate" requirements.
- **Files modified:** src-tauri/src/transcription/local.rs, src-tauri/src/transcription/mod.rs
- **Commit:** 18731a2

## Decisions Made

- whisper-rs 0.16 used (latest 0.x; plan said "check crates.io at implementation time")
- wav_bytes_to_f32 extracted outside feature gate for testability without CMake
- make_provider extended to accept &TranscriptionConfig (enables model_path routing to LocalProvider)
- tokio::task::spawn_blocking used for whisper.cpp inference (consistent with CLAUDE.md "don't block Tokio with CPU work")

## Self-Check: PASSED

- src-tauri/src/transcription/local.rs: FOUND
- Commit 18731a2: FOUND
- Commit 425d6fc: FOUND
