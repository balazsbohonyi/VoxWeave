---
phase: 08-settings-ui
plan: 02
subsystem: config
tags: [rust, tauri, reqwest, autostart, commands, ipc]

# Dependency graph
requires:
  - phase: 08-01
    provides: Nested TranscriptionConfig shape with providers.openai/groq sub-structs
provides:
  - get_provider_models Tauri command returning hardcoded OpenAI/Groq model lists
  - test_connection Tauri command for live API key probe with 5s timeout
  - set_launch_at_login Tauri command wrapping tauri-plugin-autostart registry toggle
  - tauri-plugin-autostart initialized in Builder chain
affects: [08-03, 08-04, 08-05]

# Tech tracking
tech-stack:
  added: [tauri-plugin-autostart = "2"]
  patterns: ["Never hold MutexGuard across .await — clone config fields before async block", "Hardcoded model lists as Rust constants returned by command (no API call)"]

key-files:
  created: []
  modified:
    - src-tauri/src/commands/config.rs
    - src-tauri/src/lib.rs
    - src-tauri/Cargo.toml

key-decisions:
  - "test_connection treats any non-401/non-network response as connected — 400 (bad audio) still confirms key validity"
  - "MutexGuard explicitly dropped before .await by scoping config access in a block — borrow checker correctness"
  - "tauri-plugin-autostart MacosLauncher::LaunchAgent arg required by API signature; ignored at runtime on Windows"

patterns-established:
  - "Config lock scope: always clone fields in a braced block before any async operation"
  - "Provider dispatch via match provider.as_str() block — single point for adding future providers"

requirements-completed: [SETT-02, SETT-04]

# Metrics
duration: 4min
completed: 2026-03-22
---

# Phase 8 Plan 02: Backend Commands for Settings UI Summary

**Three new Tauri commands registered: get_provider_models (hardcoded lists), test_connection (reqwest 5s probe), set_launch_at_login (autostart registry) — all wired into invoke_handler**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-22T19:39:54Z
- **Completed:** 2026-03-22T19:43:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added `ProviderModels` struct and `get_provider_models()` command returning 3 hardcoded models each for OpenAI and Groq
- Added `test_connection()` async command that probes transcription endpoints with a 1-byte multipart request, returns clear error on 401 vs. success on 400/2xx
- Added `set_launch_at_login()` command using `tauri_plugin_autostart::ManagerExt` to enable/disable Windows autostart registry key
- Added `tauri-plugin-autostart = "2"` to Cargo.toml and initialized plugin in lib.rs Builder chain
- Registered all three commands in `tauri::generate_handler![]`

## Task Commits

Each task was committed atomically:

1. **Task 1: get_provider_models command + unit tests (TDD)** - `10311b7` (feat)
2. **Task 2: test_connection + set_launch_at_login + autostart plugin wiring** - `6b83165` (feat)

## Files Created/Modified
- `src-tauri/src/commands/config.rs` — Added ProviderModels struct, get_provider_models(), test_connection(), set_launch_at_login(); added reqwest/autostart imports; added cfg(test) block with 2 tests
- `src-tauri/src/lib.rs` — Added tauri_plugin_autostart::init() in Builder chain; registered 3 new commands in invoke_handler
- `src-tauri/Cargo.toml` — Added tauri-plugin-autostart = "2" dependency

## Decisions Made
- test_connection treats 401 as "invalid key" and all other responses as success — 400 from the transcription endpoint confirms key validity (API rejects the dummy audio, not the key)
- MutexGuard is dropped before .await by scoping config access in a block (not storing guard in a variable that outlives the block) — prevents holding lock across await point
- MacosLauncher::LaunchAgent is a required argument to tauri_plugin_autostart::init() even on Windows; it is ignored at runtime on non-macOS

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- 5 pre-existing `injection::service` test failures noted during `cargo test` — unrelated to this plan, pre-existing from prior phases. Already logged in `deferred-items.md`.

## Next Phase Readiness
- Frontend can now call `get_provider_models` to populate model dropdowns without hardcoding in Vue
- Frontend can call `test_connection` with `{ provider: "openai" | "groq" }` to give users live API key feedback
- Frontend can call `set_launch_at_login` with `{ enabled: true/false }` for the General section toggle
- All three commands ready for 08-03/08-04 Transcription and General section UI components

## Self-Check: PASSED

---
*Phase: 08-settings-ui*
*Completed: 2026-03-22*
