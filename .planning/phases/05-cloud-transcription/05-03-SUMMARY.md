---
phase: 05-cloud-transcription
plan: "03"
subsystem: transcription
tags: [rust, tauri, vue, typescript, transcription, retry, fallback, events]

# Dependency graph
requires:
  - phase: 05-02
    provides: transcribe_with_retry service, TranscriptionErrorPayload types, provider dispatch
  - phase: 04-floating-indicator
    provides: indicator window App.vue, useToast composable
provides:
  - Hotkey stop path spawns real cloud transcription (no more placeholder)
  - retry_transcription Tauri command for frontend Retry button
  - retry_transcription_with_fallback Tauri command for Try with X? button
  - open_settings_on_transcription_tab Tauri command for invalid key errors
  - transcribe_with_provider single-call provider override in service.rs
  - last_encoded_audio in AppState for retry support
  - transcription-error listener in indicator window with toast routing
affects:
  - 06-injection (uses transcription-done event to trigger injection)
  - 08-settings-ui (open_settings_on_transcription_tab will gain tab navigation)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "tauri::async_runtime::spawn for cloud transcription (not tokio::spawn)"
    - "Store encoded audio before async spawn so retry commands have data"
    - "Explicit MutexGuard intermediate to avoid holding across await"
    - "Single-call provider override by cloning config and mutating the clone"

key-files:
  created:
    - src-tauri/src/commands/transcription.rs
  modified:
    - src-tauri/src/hotkey/service.rs
    - src-tauri/src/transcription/service.rs
    - src-tauri/src/transcription/mod.rs
    - src-tauri/src/state.rs
    - src-tauri/src/config/mod.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src/windows/indicator/App.vue
    - src/composables/useToast.ts

key-decisions:
  - "tauri::async_runtime::spawn used (not tokio::spawn) to avoid reactor panics in Tauri v2"
  - "transcribe_with_provider clones TranscriptionConfig and overrides provider field — no AppState mutation"
  - "last_encoded_audio stored before async spawn so both retry commands get the same audio blob"
  - "useToast composable extended with optional action button for Retry and Try with X? flows"

patterns-established:
  - "Retry flow: store audio in AppState.last_encoded_audio before spawn; commands read it back"
  - "Fallback flow: parse provider string via FromStr, call transcribe_with_provider with cloned config override"
  - "Error routing: invalid_key -> open settings, retryable -> Retry button, fallback_provider -> Try with X? button"

requirements-completed:
  - CLOD-06
  - CLOD-08
  - CLOD-09

# Metrics
duration: 15min
completed: 2026-03-17
---

# Phase 5 Plan 03: Wire Transcription Into Hotkey Pipeline Summary

**Hotkey stop path spawns real cloud transcription with retry/fallback; frontend routes transcription-error events to actionable toast buttons**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-17T11:47:00Z
- **Completed:** 2026-03-17T11:59:23Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments
- Replaced `complete_transcription_placeholder` with `tauri::async_runtime::spawn` calling `transcription::transcribe_with_retry`
- Added `last_encoded_audio: Arc<Mutex<Option<EncodedAudio>>>` to AppState for retry support
- Created `commands/transcription.rs` with `retry_transcription`, `retry_transcription_with_fallback`, and `open_settings_on_transcription_tab`
- Added `transcribe_with_provider` to service.rs for single-call provider override without AppState mutation
- Added `TranscriptionProvider::FromStr` impl so provider strings parse correctly in the fallback command
- Extended `useToast` with optional action buttons; registered transcription-error listener in indicator App.vue

## Task Commits

1. **Task 1: Replace transcription placeholder in hotkey service** - `c889e2d` (feat)
2. **Task 2: retry_transcription and retry_transcription_with_fallback commands** - `7620a82` (feat)
3. **Task 3: Frontend transcription-error event handler** - `07706d1` (feat)

## Files Created/Modified
- `src-tauri/src/commands/transcription.rs` - Three Tauri commands: retry_transcription, retry_transcription_with_fallback, open_settings_on_transcription_tab
- `src-tauri/src/hotkey/service.rs` - Replaced placeholder with real async transcription spawn; stores audio before spawn
- `src-tauri/src/transcription/service.rs` - Added transcribe_with_provider for single-call provider override
- `src-tauri/src/transcription/mod.rs` - Re-exported transcribe_with_provider
- `src-tauri/src/state.rs` - Added last_encoded_audio field with EncodedAudio import
- `src-tauri/src/config/mod.rs` - Added FromStr impl for TranscriptionProvider
- `src-tauri/src/commands/mod.rs` - Added pub mod transcription
- `src-tauri/src/lib.rs` - Registered three new commands in invoke_handler
- `src/windows/indicator/App.vue` - Added transcription-error listener with toast routing
- `src/composables/useToast.ts` - Extended with optional ToastAction support

## Decisions Made
- Used `tauri::async_runtime::spawn` not `tokio::spawn` — avoids "no reactor running" panics in Tauri v2
- `transcribe_with_provider` clones `TranscriptionConfig` and overrides the `provider` field on the clone — AppState config is never mutated for a single fallback call
- `last_encoded_audio` stored before spawning so both retry commands can re-use the same audio blob without asking the user to record again
- `useToast` extended with `ShowToastOptions` interface to support action buttons while keeping backward compatibility with the string overload

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed return type mismatch in retry commands**
- **Found during:** Task 2 (compilation)
- **Issue:** `transcribe_with_retry` returns `Result<String, ()>` but Tauri commands return `Result<(), String>`; also missing `use tauri::Manager` import needed for `app.state()`
- **Fix:** Added `.map(|_| ())` to discard the text result; added `Manager` to imports
- **Files modified:** src-tauri/src/commands/transcription.rs
- **Verification:** cargo build succeeded
- **Committed in:** 7620a82 (Task 2 commit)

**2. [Rule 1 - Bug] Fixed borrow checker error in retry commands**
- **Found during:** Task 2 (compilation)
- **Issue:** `state.last_encoded_audio.lock().unwrap().clone()` at end of block kept MutexGuard alive across block boundary (same pattern already fixed in service.rs)
- **Fix:** Added explicit intermediate `let guard = ...` binding so guard drops before the block ends
- **Files modified:** src-tauri/src/commands/transcription.rs
- **Verification:** cargo build succeeded
- **Committed in:** 7620a82 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1 - Bug)
**Impact on plan:** Both fixes were compile errors in the new commands file. No scope creep.

## Issues Encountered
- The plan's `transcribe_with_provider` code referenced `build_provider` but the actual function in service.rs is `make_provider`. Adjusted accordingly.
- The plan's `transcribe_with_provider` used a simplified error payload but the actual `TranscriptionError` variants needed full pattern matching. Implemented complete match arms.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Hotkey stop path now calls real transcription — second hotkey press starts a cloud API call
- `transcription-done` event fires with transcribed text — Phase 6 (injection) can hook here to inject text
- `retry_transcription` and `retry_transcription_with_fallback` are registered and callable
- Frontend error toast flow complete: invalid key opens settings, retryable shows Retry, fallback shows Try with X?

---
*Phase: 05-cloud-transcription*
*Completed: 2026-03-17*

## Self-Check: PASSED

- src-tauri/src/commands/transcription.rs: FOUND
- src/windows/indicator/App.vue: FOUND
- Commit c889e2d (task 1): FOUND
- Commit 7620a82 (task 2): FOUND
- Commit 07706d1 (task 3): FOUND
