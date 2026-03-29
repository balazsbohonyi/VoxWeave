---
phase: 10-local-transcription
verified: 2026-03-29T14:00:00Z
status: human_needed
score: 5/5 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: 3/5
  gaps_closed:
    - "Set Active saves absolute model path (Gap 2 from previous VERIFICATION.md)"
    - "Vue ref assignment in @can-proceed handler is correct — Vue 3 template compiler auto-unwraps ref for template inline assignments; Gap 1 from previous VERIFICATION.md was a false positive"
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "End-to-end local transcription with a downloaded model"
    expected: "Press hotkey, speak, press hotkey — local whisper.cpp produces transcribed text with no network call"
    why_human: "Requires CMake + libclang + whisper-rs native build (cargo tauri dev --features local-transcription); cannot compile or exercise this code path without native toolchain"
  - test: "Download progress bar and Cancel flow"
    expected: "Progress bar fills with whole-number percentage; Cancel stops download mid-stream and returns card to idle state; partial file is absent from disk"
    why_human: "Requires live ~75-1500 MB network download; runtime visual behavior cannot be verified programmatically"
  - test: "Wizard Next button gate for Local engine"
    expected: "On Step 2 with Local chosen, Next is disabled initially; becomes enabled when any model reaches 'downloaded' state OR user clicks 'Skip for now'"
    why_human: "Requires running the Tauri app to verify Vue reactivity and event emission chain at runtime"
  - test: "Set Active → restart → Active badge persists"
    expected: "After setting a model active, closing and re-opening Settings, that model card shows the blue Active badge"
    why_human: "Requires config persistence round-trip and UI re-mount hydration verification at runtime"
---

# Phase 10: Local Transcription Verification Report

**Phase Goal:** Users who prefer local, offline transcription can download and use whisper.cpp models of their choice without affecting the cloud pipeline
**Verified:** 2026-03-29
**Status:** HUMAN_NEEDED — all automated checks pass; runtime verification required
**Re-verification:** Yes — previous VERIFICATION.md had `status: gaps_found`; both identified gaps are now resolved

---

## Re-verification Status

| Previous Gap | Previous Status | Current Status |
|---|---|---|
| Gap 1: `@can-proceed="localCanProceed = $event"` alleged Vue ref assignment bug | FAILED | CLOSED — false positive. Vue 3 template compiler auto-unwraps `ref = $event` in inline event handlers to `ref.value = $event`. This is correct behavior. |
| Gap 2: `setActiveModel` saved relative path `ggml-tiny.bin` | FAILED | CLOSED — Fixed. `setActiveModel` now calls `invoke<string>("get_model_path", { modelId })` (TranscriptionSection.vue line 326) and saves the returned absolute path. `get_model_path` command is registered in lib.rs (line 180). |

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Selecting "Local" engine and triggering a recording produces transcription without any network call | VERIFIED | `LocalProvider` in `transcription/local.rs` (lines 62-161) calls whisper-rs exclusively; no HTTP client. `LocalFeatureDisabledProvider` stub in service.rs handles non-feature builds gracefully. Pre-recording guard in `hotkey/service.rs` (lines 248-278) short-circuits before audio capture if no model file exists. |
| 2 | Models (tiny/base/small/medium) downloadable on-demand with live progress bar and cancel option | VERIFIED | `run_download()` streams via `reqwest::bytes_stream()` + `futures_util::StreamExt::next()` emitting `model-download-progress` per chunk. `cancel_model_download` command sets `AtomicBool` flag checked per chunk. Both `TranscriptionSection.vue` and `Step2Local.vue` listen to all 4 download events and render progress bars. `setActiveModel` saves the absolute path returned by `get_model_path` command. |
| 3 | Downloaded models stored in `%APPDATA%/VoxFlow/models/` and deletable from settings | VERIFIED | `models_dir()` returns `dirs_next::config_dir().join("VoxFlow").join("models")`. `delete_model` removes the `.bin` file and clears `config.transcription.providers.local.model_path` when matched. Settings cards show a Delete button when `modelState === "downloaded"`. |
| 4 | Local transcription runs on a background thread and does not freeze the UI | VERIFIED | `LocalProvider::transcribe()` uses `tokio::task::spawn_blocking` for all whisper-rs inference (local.rs line 103). Streaming download runs in `tauri::async_runtime::spawn`. No blocking operations on the main thread. |
| 5 | Missing or corrupt model file shows error toast with re-download prompt | VERIFIED | `TranscriptionError::ModelMissing` and `ModelLoadFailed` both map to `TranscriptionErrorCode::ModelMissing` in `service.rs`. Toast handler in `toast/App.vue:141-153` shows "No local model downloaded." with "Open Settings" action invoking `open_settings_on_transcription_tab` with `provider: "local"`. Pre-recording guard in hotkey service fires this toast at hotkey press before audio starts. |

**Score:** 5/5 truths verified

---

## Required Artifacts

### Plan 01 Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/transcription/local.rs` | LocalProvider + wav_bytes_to_f32 + feature gate | VERIFIED | 266 lines. `wav_bytes_to_f32` always compiled (no native dep). `LocalProvider` in `#[cfg(feature = "local-transcription")] mod provider_impl`. 5 unit tests for WAV conversion and error variants. |
| `src-tauri/src/transcription/provider.rs` | ModelMissing and ModelLoadFailed error variants | VERIFIED | Lines 24-27: both variants with `message: String` field present in `TranscriptionError` enum. |
| `src-tauri/src/transcription/service.rs` | make_provider dispatches Local to LocalProvider; ModelMissing error code | VERIFIED | Lines 65-83: Local arm uses `LocalProvider::new(model_path)` under feature gate; `LocalFeatureDisabledProvider` stub under `not(feature)`. `ModelMissing` in `TranscriptionErrorCode` at line 35. Both new error variants handled in `transcribe_with_retry` and `transcribe_with_provider`. |

### Plan 02 Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/transcription/download.rs` | Download task, progress payloads, model path helpers | VERIFIED | 336 lines. All 4 event name constants, 3 payload structs, `models_dir()`, `model_file_path()`, `model_download_url()`, `get_downloaded_model_ids()`, `run_download()` — all present. 9 unit tests including temp-dir scan test. |
| `src-tauri/src/commands/download.rs` | Four Tauri commands for download management | VERIFIED | 168 lines. `start_model_download` (async, background spawn), `cancel_model_download`, `get_downloaded_models`, `delete_model` — all `#[tauri::command]`. Bonus `get_model_path` command also present. |
| `src-tauri/src/state.rs` | download_cancel and downloading_model in AppState | VERIFIED | Lines 123-126 declare both fields. Both initialized to `Arc::new(Mutex::new(None))` in both `Ok` and `Err` branches of `AppState::load()`. |

### Plan 03 Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/windows/settings/components/TranscriptionSection.vue` | Real download/delete/active UI for local model cards | VERIFIED | `start_model_download` invoked at line 286. All 4 download events listened in `onMounted` (lines 120-153). `onUnmounted` cleanup present. `setActiveModel` uses `get_model_path` for absolute path at line 326. |
| `src/windows/wizard/components/Step2Local.vue` | Full model picker replacing placeholder | VERIFIED | 266 lines. `canProceed` and `navigateNext` emits defined. `startDownload` invokes `start_model_download`. All 4 events listened. Skip button emits `navigateNext`. Replaces prior "coming soon" placeholder. |
| `src/windows/toast/App.vue` | model_missing handler with Open Settings action | VERIFIED | Lines 141-153: `model_missing` code handled with error toast and "Open Settings" action invoking `open_settings_on_transcription_tab` with `provider: "local"`. |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `service.rs` | `local.rs` | `make_provider` Local arm | WIRED | service.rs line 74: `LocalProvider::new(model_path)` under feature gate |
| `local.rs` | whisper-rs crate | `WhisperContext::new_with_params` + `state.full()` | WIRED | local.rs lines 109-151: full whisper-rs API inside `spawn_blocking` closure |
| `commands/download.rs` | `transcription/download.rs` | commands call `run_download` | WIRED | commands/download.rs line 63: `download::run_download(app_clone, ...)` |
| `transcription/download.rs` | reqwest streaming | `bytes_stream()` + `StreamExt::next()` | WIRED | download.rs lines 161, 163: `response.bytes_stream()` + `stream.next().await` |
| `lib.rs` | `commands/download.rs` | `tauri::generate_handler` | WIRED | lib.rs lines 176-180: `start_model_download`, `cancel_model_download`, `get_downloaded_models`, `delete_model`, `get_model_path` all registered |
| `TranscriptionSection.vue` | Tauri commands | invoke model download/delete/get_model_path | WIRED | Lines 119-153, 286, 306, 326: all invocations and event listeners wired |
| `Step2Local.vue` | Tauri events | listen `model-download-*` | WIRED | Step2Local.vue lines 122-157: all 4 events listened in `onMounted`; cleanup in `onUnmounted` |
| `wizard/App.vue` | `Step2Local.vue` | `@can-proceed`, `@navigate-next` | WIRED | App.vue lines 181-182: `@can-proceed="localCanProceed = $event"` (valid — Vue template compiler auto-unwraps) and `@navigate-next="advanceFromStep2"` |
| `hotkey/service.rs` | toast | pre-recording guard via `show_toast_window` | WIRED | hotkey/service.rs lines 248-278: checks model path exists before audio starts |
| `lib.rs` quit path | download cancel flag | `AtomicBool::store(true)` in `WindowEvent::Destroyed` | WIRED | lib.rs lines 80-92: cancel extracted with `.take()` before `cleanup_before_exit()` |

---

## Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|--------------|-------------|--------|----------|
| LOCL-01 | 10-01 | App supports local transcription via whisper.cpp (whisper-rs) | SATISFIED | `LocalProvider` implements `TranscriptionProviderTrait`; whisper-rs 0.16 in Cargo.toml as optional dep; feature gate `local-transcription = ["dep:whisper-rs"]` |
| LOCL-02 | 10-01, 10-02, 10-03 | Models NOT bundled — downloaded on-demand with progress bar and cancel | SATISFIED | Full streaming download in `transcription/download.rs`; progress events; cancel flag; Settings and Wizard UIs both wired |
| LOCL-03 | 10-03 | Models: tiny/base/small/medium with quality/speed descriptions | SATISFIED | `VALID_MODEL_IDS = ["tiny", "base", "small", "medium"]` in download.rs; `LOCAL_MODELS` array with labels, sizes, and quality strings in both `TranscriptionSection.vue` and `Step2Local.vue` |
| LOCL-04 | 10-02 | Models stored in `%APPDATA%/VoxFlow/models/`; deletable from settings | SATISFIED | `models_dir()` resolves via `dirs_next::config_dir()`; `delete_model` command removes file and clears config; Delete button wired in Settings |
| LOCL-05 | 10-01 | Local transcription on background thread without freezing UI | SATISFIED | `tokio::task::spawn_blocking` in `LocalProvider::transcribe()` keeps CPU-bound work off Tokio async executor |
| LOCL-06 | 10-01 | Audio passed as WAV/PCM float32 to whisper.cpp | SATISFIED | `wav_bytes_to_f32()` converts 16-bit PCM WAV → f32 normalized to [-1.0, 1.0]; `LocalProvider` validates `EncodedFormat::Wav` before conversion; existing `AUDI-03` ensures WAV encoding for local provider |
| LOCL-07 | 10-01, 10-03 | Missing or corrupt model → error with re-download prompt | SATISFIED | `ModelMissing` / `ModelLoadFailed` errors surface as toast with "Open Settings" action; pre-recording guard prevents audio capture when model absent; `setActiveModel` now saves absolute path so `.exists()` check is accurate |

**All 7 requirements (LOCL-01 through LOCL-07) satisfied.**

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src-tauri/src/transcription/mod.rs` | 3 | `#![allow(dead_code, unused_imports)]` | INFO | Module re-exports generate unused-import warnings; suppression is intentional |
| `src-tauri/src/commands/download.rs` | 115 | `_state: State<'_>` unused param | INFO | Kept for Tauri command signature consistency; no behavioral impact |

No blockers or warnings found.

---

## Human Verification Required

All automated checks passed. The following items require a human with the appropriate build environment to confirm:

### 1. End-to-End Local Transcription

**Test:** Build with `cargo tauri dev --features local-transcription` (requires CMake, MSVC Build Tools, libclang in PATH). Download the Tiny model (~75 MB) in Settings, click Set Active, press hotkey, speak for 3-5 seconds, press hotkey again.
**Expected:** Transcribed text injected into target window with no outbound network call made during transcription.
**Why human:** whisper-rs requires a native CMake/clang build that is not available in this environment. The code path through `LocalProvider` → `WhisperContext` → `state.full()` → segment extraction cannot be exercised without the compiled native library.

### 2. Download Progress Bar and Cancel

**Test:** In Settings with Local engine selected, click Download on the Base model (~150 MB). While downloading, observe the progress bar and percentage. Then click Cancel.
**Expected:** Progress bar fills; percentage shows whole numbers without decimals; Cancel stops download mid-stream; card returns to idle Download state; no `ggml-base.bin.partial` file remains on disk.
**Why human:** Requires live network download of 75-1500 MB; runtime visual behavior of CSS transitions, percentage rendering, and file cleanup only verifiable at runtime.

### 3. Wizard Next Button Gate (Local Engine)

**Test:** Open Setup Wizard, choose Local in Step 1. On Step 2, verify the Next button is disabled. Download any model and let it complete. Verify Next becomes enabled. Alternatively: instead of downloading, click "Skip for now" and verify Next becomes enabled (and navigates to Step 3 immediately).
**Expected:** Next button respects the `canProceed` state emitted by `Step2Local.vue`.
**Why human:** Requires running the Vue app to confirm the reactivity chain (`canProceed` emit → `localCanProceed = $event` in template → `:disabled` binding re-evaluation) works correctly at runtime.

### 4. Active Badge Persistence Across Settings Reopen

**Test:** Download a model, click Set Active, close the Settings window, reopen it.
**Expected:** The model that was set active shows the blue "Active" badge (2px blue border + blue badge) on reopen.
**Why human:** Requires config persistence round-trip: `setActiveModel` saves absolute path → config written to disk → Settings remounted → `get_downloaded_models` hydration check → `isActiveLocalModel` comparison against `config.transcription.providers.local.model_path`.

---

## Summary

Phase 10 local transcription infrastructure is fully implemented and wired end-to-end. All 5 success criteria and all 7 LOCL requirements have verified implementation evidence in the codebase. The two gaps identified in the previous VERIFICATION.md are both resolved:

- The `setActiveModel` relative-path bug (Gap 2) was fixed — it now calls `get_model_path` to obtain the absolute path before saving config.
- The Vue ref assignment concern (Gap 1) was a false positive — Vue 3's template compiler correctly transforms `ref = $event` in inline event handlers to `ref.value = $event`.

The implementation correctly applies all required patterns:
- `local-transcription` cargo feature gate prevents CMake/libclang requirement for cloud-only builds
- `tokio::task::spawn_blocking` keeps CPU-bound whisper inference off the Tokio executor
- `LocalFeatureDisabledProvider` stub replaces `panic!()` for non-feature builds
- Pre-recording guard fires model-missing toast before opening the audio stream
- Partial-file pattern (`.bin.partial` → `.bin` rename) prevents incomplete downloads from appearing valid
- Quit cleanup cancels in-progress downloads via `AtomicBool` before `cleanup_before_exit()`

---

_Verified: 2026-03-29_
_Verifier: Claude (gsd-verifier)_
