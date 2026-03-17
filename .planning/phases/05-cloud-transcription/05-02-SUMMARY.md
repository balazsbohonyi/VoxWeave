---
phase: 05-cloud-transcription
plan: "02"
subsystem: transcription
tags: [rust, async, retry, backoff, tauri-events, error-handling, tokio]
dependency_graph:
  requires:
    - 05-01 (TranscriptionProviderTrait, TranscriptionError, OpenAiProvider, GroqProvider, OpenRouterProvider)
  provides:
    - transcribe_with_retry (public async fn, hotkey pipeline entry point)
    - TranscriptionErrorPayload (Tauri event payload struct)
    - TranscriptionErrorCode (enum: invalid_key, rate_limit, network, server, cancelled)
    - run_with_retry_inner (pure testable retry helper)
    - find_fallback_provider (pure testable fallback selector)
    - TRANSCRIPTION_ERROR_EVENT / TRANSCRIPTION_DONE_EVENT constants
  affects:
    - src-tauri/src/transcription/mod.rs (service module declared + re-exported)
tech_stack:
  added: []
  patterns:
    - Pure helper extraction (run_with_retry_inner, find_fallback_provider) for unit testing without AppHandle
    - Explicit MutexGuard intermediate to avoid holding across .await points
    - Exponential backoff via 1000 * 2^attempt ms (1s/2s/4s for attempts 0/1/2)
key_files:
  created:
    - src-tauri/src/transcription/service.rs
  modified:
    - src-tauri/src/transcription/mod.rs
decisions:
  - "Pure helper run_with_retry_inner extracted so retry logic is unit-testable without AppHandle mocking"
  - "Explicit let guard = ...; guard.field.clone() pattern chosen to satisfy borrow checker without holding MutexGuard across await"
  - "cancel_flag cleared (set to false) immediately on detection before emitting Cancelled event"
metrics:
  duration_seconds: 540
  completed_date: "2026-03-17"
  tasks_completed: 3
  files_created: 1
  files_modified: 1
requirements_satisfied:
  - CLOD-06
  - CLOD-07
  - CLOD-08
  - CLOD-09
---

# Phase 5 Plan 2: Transcription Service Retry Loop and Event Emission Summary

**One-liner:** Retry orchestration layer with 1s/2s/4s exponential backoff, cancel-flag checking, immediate InvalidKey short-circuit, and Tauri event emission on every outcome path.

## What Was Built

A single file `src-tauri/src/transcription/service.rs` implementing the full orchestration layer between the hotkey pipeline and cloud providers:

- **Event constants:** `TRANSCRIPTION_ERROR_EVENT` ("transcription-error"), `TRANSCRIPTION_DONE_EVENT` ("transcription-done")
- **Payload types:** `TranscriptionErrorCode` (5 snake_case variants), `TranscriptionErrorPayload` (code, message, provider, fallback_provider, retryable)
- **`make_provider()`:** Dispatches `TranscriptionProvider` enum to concrete `OpenAiProvider`, `GroqProvider`, `OpenRouterProvider` instances; panics with `unimplemented!` for Local (phase 10)
- **`find_fallback_provider()`:** Walks `config.fallback_order`, skips the current provider and any with an empty API key, returns the first eligible provider name as `Option<String>`
- **`run_with_retry_inner()`:** Pure async helper accepting a closure; drives up to `max_retries` retries with 1000ms × 2^attempt backoff; short-circuits immediately on `InvalidKey` and `Cancelled`
- **`transcribe_with_retry()`:** Public entry point for the hotkey pipeline; clones config before first `.await`, checks cancel flag before each attempt, emits typed Tauri events on every outcome path

## Task Commits

| Task | Description | Commit |
|------|-------------|--------|
| 1 + 2 | Event types, provider dispatch, fallback finder, retry loop, all tests | 70264af |
| Fix | Remove redundant let binding flagged by clippy in config clone | 7da4588 |

## Test Results

```
cargo test transcription::service
running 6 tests — all passed

test_find_fallback_skips_current        — fallback_order=[Openai, Groq], current=Openai, groq key set → Some("groq")
test_find_fallback_skips_empty_key      — groq key empty, openrouter key set → Some("openrouter")
test_find_fallback_none_available       — all keys empty → None
test_retry_three_times_on_rate_limit    — RateLimit x4 → 4 total calls, returns Err(RateLimit)
test_success_on_second_attempt          — RateLimit x1 then Ok → 2 calls, returns Ok("hello world")
test_invalid_key_no_retry               — InvalidKey → exactly 1 call, immediate Err(InvalidKey)

cargo test — full suite: 59 passed, 0 failed
cargo clippy — 0 new warnings in service.rs
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing `tauri::Manager` import for `app.state()`**
- **Found during:** Task 2 implementation (first compile attempt)
- **Issue:** `app.state::<AppState>()` requires `tauri::Manager` in scope; compiler error E0599
- **Fix:** Added `use tauri::Manager;` inside `transcribe_with_retry` function body
- **Files modified:** src-tauri/src/transcription/service.rs
- **Commit:** 70264af (folded in before commit)

**2. [Rule 1 - Bug] Borrow checker rejected one-liner config clone**
- **Found during:** Task 2 implementation
- **Issue:** `state.config.lock().unwrap().transcription.clone()` as last expression in a block creates a temporary `MutexGuard` that the compiler cannot prove is dropped before the block ends; E0597
- **Fix:** Explicit `let guard = state.config.lock().unwrap(); guard.transcription.clone()` — guard drops at end of statement, satisfying the borrow checker
- **Files modified:** src-tauri/src/transcription/service.rs
- **Commit:** 70264af (folded in before commit)

**3. [Rule 1 - Bug] Clippy: needless intermediate `let cloned = ...`**
- **Found during:** Post-task clippy check
- **Issue:** After switching to explicit guard, the pattern `let cloned = guard.transcription.clone(); cloned` triggered clippy's `clippy::needless_let_underscore` lint
- **Fix:** Collapsed to `guard.transcription.clone()` as direct block tail expression (the two-line pattern without the redundant variable)
- **Files modified:** src-tauri/src/transcription/service.rs
- **Commit:** 7da4588

### Note: Task 3 included in Task 1 commit

The plan specified Task 3 as a separate step to add `pub mod service` to mod.rs. This was done as part of Task 1's commit because service.rs required mod.rs to be updated to compile for the first test run. No behavioral difference — all three tasks' done criteria are satisfied.

## Self-Check: PASSED

Files created:
- src-tauri/src/transcription/service.rs — FOUND
- .planning/phases/05-cloud-transcription/05-02-SUMMARY.md — FOUND (this file)

Commits verified:
- 70264af — FOUND (feat(05-02): add transcription service...)
- 7da4588 — FOUND (fix(05-02): remove unnecessary let binding...)

Success criteria verified:
- transcribe_with_retry is pub and re-exported from transcription::transcribe_with_retry — YES
- InvalidKey path: no retry, provider name in payload, retryable=false — YES
- Rate limit / network / server: 3 retries with backoff, then error with fallback_provider — YES
- cancel_flag checked before each attempt and reset after handling — YES
- All 6 unit tests pass — YES
- Full cargo test suite (59 tests) passes — YES
