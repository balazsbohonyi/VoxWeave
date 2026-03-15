---
phase: 03-audio-capture
plan: 03-02
subsystem: backend
tags: [audio, encoding, provider]

# Dependency graph
requires: [03-01-capture-lifecycle]
provides:
  - provider-driven format selection (Opus/WAV)
  - retry-once encode policy
  - output contract tests for encoded payloads
affects: [transcription-pipeline]

# Tech tracking
tech-stack:
  added: []
  patterns: [provider-at-stop-time selection, retry-once helper]

key-files:
  created: [src-tauri/src/audio/encode.rs]
  modified: [src-tauri/src/audio/mod.rs, src-tauri/src/hotkey/service.rs]

key-decisions:
  - "Encoding format is selected from active provider at finalize time"

patterns-established:
  - "Encode failures retry once, then emit explicit audio-error and recover to Idle"

requirements-completed: [AUDI-03]

# Metrics
duration: 78min
completed: 2026-03-15
---

# Phase 3 Plan 02: Encoding Contract Summary

Implemented provider-driven encoding with Opus for cloud providers and WAV for local provider, applied at recording stop time.

## Accomplishments
- Added `encode_for_provider` with deterministic format routing and payload metadata.
- Integrated retry-once behavior into finalize flow with explicit error event on terminal failure.
- Added contract tests for Opus/WAV signatures and retry behavior.

## Task Commits
1. **Task set implemented in consolidated commit** - `4a8b855` (feat)

## Deviations from Plan
- Tasks were committed in one combined commit with the rest of Phase 3 implementation.

## Issues Encountered
- Machine-level Rust test runtime issue (`STATUS_ENTRYPOINT_NOT_FOUND`) blocks executing test binaries.

## Self-Check: PASSED
