---
phase: 10-local-transcription
plan: 02
subsystem: transcription
tags: [rust, tauri, whisper, download, reqwest, futures-util, streaming]

requires:
  - phase: 10-local-transcription/10-01
    provides: LocalProviderConfig struct in config, feature gate scaffold

provides:
  - transcription::download module with streaming download, cancel, and model path helpers
  - Four Tauri commands: start_model_download, cancel_model_download, get_downloaded_models, delete_model
  - AppState download_cancel and downloading_model fields
  - Quit path cancel of in-progress downloads

affects:
  - 10-03 (frontend wiring for download commands)
  - local transcription service (uses model_file_path from this module)

tech-stack:
  added:
    - futures-util = "0.3" (StreamExt for response byte streaming)
    - reqwest stream feature enabled
  patterns:
    - run_download uses partial file (.bin.partial) during download, renames to final on success
    - cancel check per chunk via AtomicBool::load(Ordering::Relaxed)
    - Tauri commands release MutexGuard before .await via explicit clone pattern
    - Background task clears state (download_cancel, downloading_model) in all outcome branches

key-files:
  created:
    - src-tauri/src/transcription/download.rs
    - src-tauri/src/commands/download.rs
  modified:
    - src-tauri/src/transcription/mod.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/state.rs
    - src-tauri/src/lib.rs
    - src-tauri/Cargo.toml

key-decisions:
  - "run_download uses ggml-<id>.bin.partial during streaming — prevents half-written files from appearing as downloaded"
  - "VALID_MODEL_IDS is a compile-time &[&str] constant — validated in both model_file_path and start_model_download"
  - "start_model_download returns Ok() immediately; frontend tracks progress via model-download-progress events"
  - "get_downloaded_models returns Vec<String> only (no in-progress state) — frontend tracks that via events"
  - "Quit cleanup in WindowEvent::Destroyed takes cancel flag via .take() before spawning exit thread — avoids borrow-checker lifetime issue"

patterns-established:
  - "Download state cleared in all outcome branches (success/error/cancel) — no stale state"
  - "Model path helpers in transcription::download are not feature-gated — download works without whisper-rs compiled"

requirements-completed: [LOCL-02, LOCL-04]

duration: 6min
completed: 2026-03-29
---

# Phase 10 Plan 02: Model Download Infrastructure Summary

**Streaming whisper model downloader with progress events, cancel support, and four Tauri commands wired into AppState and quit cleanup**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-29T08:47:51Z
- **Completed:** 2026-03-29T08:53:51Z
- **Tasks:** 2
- **Files modified:** 6 (+ 2 created)

## Accomplishments

- Full streaming download via reqwest + futures-util with per-chunk progress events and cancel support
- Partial file pattern prevents incomplete downloads from appearing as valid models in get_downloaded_models
- Four Tauri commands registered and callable from frontend
- App quit path cancels any in-progress download before cleanup_before_exit

## Task Commits

Each task was committed atomically:

1. **Task 1: Download module + AppState fields + model path helpers** - `90b3d73` (feat)
2. **Task 2: Tauri download commands + quit cleanup + command registration** - `82d8c1e` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src-tauri/src/transcription/download.rs` - VALID_MODEL_IDS, payload structs, models_dir, model_file_path, model_download_url, get_downloaded_model_ids, run_download async fn; 9 unit tests
- `src-tauri/src/commands/download.rs` - Four Tauri commands: start_model_download, cancel_model_download, get_downloaded_models, delete_model
- `src-tauri/src/transcription/mod.rs` - Added `pub mod download;`
- `src-tauri/src/commands/mod.rs` - Added `pub mod download;`
- `src-tauri/src/state.rs` - Added download_cancel and downloading_model fields to AppState (both branches)
- `src-tauri/src/lib.rs` - Registered four commands in invoke_handler; added quit-cleanup cancel logic
- `src-tauri/Cargo.toml` - Added futures-util = "0.3"; reqwest stream feature already present

## Decisions Made

- run_download uses `.bin.partial` extension during streaming and renames to final on success only — prevents get_downloaded_model_ids from returning an incomplete model
- start_model_download returns Ok(()) immediately; background task emits progress/done/error/cancelled events; frontend tracks in-progress state via events rather than AppState polling
- Quit cleanup extracts cancel flag with `.take()` before spawning the exit thread — avoids lifetime issue with temporary `State<AppState>` reference

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added `use tauri::Emitter;` import to download.rs**
- **Found during:** Task 1 (download module compilation)
- **Issue:** `app.emit()` requires `tauri::Emitter` trait in scope; not included in initial implementation
- **Fix:** Added `use tauri::Emitter;` at top of download.rs
- **Files modified:** src-tauri/src/transcription/download.rs
- **Verification:** cargo test passed 9/9 after fix
- **Committed in:** 90b3d73 (Task 1 commit)

**2. [Rule 3 - Blocking] Restructured quit-cleanup code to satisfy borrow checker**
- **Found during:** Task 2 (lib.rs quit path)
- **Issue:** `app_handle.state::<AppState>()` temporary didn't live long enough when MutexGuard held across block end
- **Fix:** Extract cancel flag with `let maybe_cancel = ...; if let Some(cancel) = maybe_cancel { ... }` pattern
- **Files modified:** src-tauri/src/lib.rs
- **Verification:** cargo check passes cleanly
- **Committed in:** 82d8c1e (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were mandatory for compilation. No scope creep.

## Issues Encountered

- `cargo check --features local-transcription` fails as expected — whisper-rs requires CMake/clang (LIBCLANG_PATH not set). This is the intended behavior per the feature-gate architecture decision.

## Next Phase Readiness

- All four download commands are registered and callable from frontend
- Progress events (model-download-progress, model-download-done, model-download-cancelled, model-download-error) are defined and emitted
- Plan 03 can wire the frontend UI to these commands and listen to these events

---
*Phase: 10-local-transcription*
*Completed: 2026-03-29*
