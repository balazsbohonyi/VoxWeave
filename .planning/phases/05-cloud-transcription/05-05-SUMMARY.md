---
phase: 05-cloud-transcription
plan: 05
subsystem: transcription
tags: [rust, openai, whisper, audio-encoding, wav, error-handling]

# Dependency graph
requires:
  - phase: 05-cloud-transcription
    provides: "transcription pipeline wired to hotkey, service.rs retry/fallback logic"
provides:
  - "OpenAI Whisper audio upload now sends WAV (not Ogg) — fixes 400 Invalid file error"
  - "Network error toasts show friendly human-readable message instead of raw reqwest URL"
affects: [05-cloud-transcription, UAT]

# Tech tracking
tech-stack:
  added: []
  patterns: [TDD red-green for audio routing, friendly error messages over raw error propagation]

key-files:
  created: []
  modified:
    - src-tauri/src/audio/encode.rs
    - src-tauri/src/transcription/openai.rs
    - src-tauri/src/transcription/service.rs
    - src-tauri/src/audio/mod.rs

key-decisions:
  - "OpenAI requires WAV (not Opus/Ogg) — format_for_provider routes Openai to EncodedFormat::Wav"
  - "Network error arms in service.rs emit friendly string, not raw reqwest error URL"

patterns-established:
  - "Format routing: provider-specific audio formats declared in encode.rs format_for_provider"
  - "Error messages: never forward raw HTTP client error strings to the frontend"

requirements-completed: [CLOD-01, CLOD-09]

# Metrics
duration: 15min
completed: 2026-03-18
---

# Phase 5 Plan 5: UAT Gap Closure — Audio Format + Network Error Summary

**Fixed OpenAI Whisper 400 error (WAV routing) and replaced raw reqwest network error URLs with a friendly user-facing message**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-18T00:00:00Z
- **Completed:** 2026-03-18T00:15:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- `format_for_provider(Openai)` now returns `EncodedFormat::Wav` — OpenAI Whisper API accepts audio
- `openai.rs` multipart file part uses `audio.wav` filename and `audio/wav` MIME type
- Both Network error match arms in `service.rs` emit "Network error. Check your connection and try again."
- 4 new unit tests in `encode.rs` covering all provider-to-format mappings (TDD)
- All 63 Rust tests pass, clippy clean

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix audio format routing for OpenAI** - `1941490` (feat)
2. **Task 2: Replace raw Network error message** - `5675090` (fix)
3. **Deviation: Update audio/mod.rs tests** - `80c1a10` (fix)

## Files Created/Modified
- `src-tauri/src/audio/encode.rs` - `format_for_provider` routes Openai to Wav; 4 new unit tests added
- `src-tauri/src/transcription/openai.rs` - file_name changed to "audio.wav", mime_str changed to "audio/wav"
- `src-tauri/src/transcription/service.rs` - Both Network arms emit friendly message
- `src-tauri/src/audio/mod.rs` - Updated 3 tests that were asserting old Openai→Opus behavior

## Decisions Made
- OpenAI Whisper requires WAV or MP3 — routing OpenAI to WAV (not Opus) is the correct fix per API contract
- Network errors should never expose raw HTTP client error strings (URLs, internal details) to end users

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated audio/mod.rs tests asserting old incorrect Openai→Opus behavior**
- **Found during:** Task 1 verification (full `cargo test`)
- **Issue:** Three tests in `audio/mod.rs` were asserting `Openai → Opus` (the old wrong behavior): `selects_encoder_from_provider`, `opus_output_contract`, `encode_retries_once_on_transient_failure`
- **Fix:** Updated `selects_encoder_from_provider` to expect `Openai → Wav`; switched `opus_output_contract` and `encode_retries_once_on_transient_failure` to use `Groq` provider for testing Opus path
- **Files modified:** `src-tauri/src/audio/mod.rs`
- **Verification:** All 63 tests pass after fix
- **Committed in:** `80c1a10`

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** Required to keep test suite passing after correct behavior was introduced. No scope creep.

## Issues Encountered
- None beyond the deviation documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Both UAT gaps (gap 1: OpenAI 400 error, gap 3: unfriendly network error toast) are now fixed at the Rust layer
- Ready for end-to-end re-validation with a real OpenAI API key

## Self-Check: PASSED

- FOUND: src-tauri/src/audio/encode.rs
- FOUND: src-tauri/src/transcription/openai.rs
- FOUND: src-tauri/src/transcription/service.rs
- FOUND: .planning/phases/05-cloud-transcription/05-05-SUMMARY.md
- FOUND commit 1941490: feat(05-05): fix audio format routing for OpenAI
- FOUND commit 5675090: fix(05-05): replace raw Network error message
- FOUND commit 80c1a10: fix(05-05): update audio/mod.rs tests

---
*Phase: 05-cloud-transcription*
*Completed: 2026-03-18*
