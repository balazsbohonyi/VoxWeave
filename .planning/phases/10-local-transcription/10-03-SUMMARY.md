---
phase: 10-local-transcription
plan: 03
subsystem: ui
tags: [vue, tauri, whisper, download, wizard, toast, typescript]

requires:
  - phase: 10-local-transcription/10-01
    provides: LocalProviderConfig, model_missing TranscriptionErrorCode variant
  - phase: 10-local-transcription/10-02
    provides: start_model_download, cancel_model_download, get_downloaded_models, delete_model commands; model-download-* events

provides:
  - Functional Settings local model cards with download/progress/cancel/delete/set-active UI
  - Step2Local wizard component with full model picker replacing "coming soon" placeholder
  - Wizard Next button gated on model downloaded or Skip clicked
  - Post-finish nudge toast when local engine chosen but no model downloaded
  - model_missing toast handler with Open Settings action in toast/App.vue
  - show_plain_toast Tauri command for programmatic plain toasts

affects:
  - Local transcription end-to-end pipeline (human verify in Task 3)

tech-stack:
  added: []
  patterns:
    - UnlistenFn array pattern for bulk event cleanup in onUnmounted
    - Emit-gate pattern: component emits canProceed(boolean) to parent for Next button gating
    - show_plain_toast Rust command wraps show_toast_window for frontend-invokable plain toasts

key-files:
  created:
    - src/windows/wizard/components/Step2Local.vue
  modified:
    - src/windows/settings/components/TranscriptionSection.vue
    - src/windows/wizard/App.vue
    - src/windows/toast/App.vue
    - src-tauri/src/commands/indicator.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "show_plain_toast added as Rust command to allow wizard finish() to trigger nudge toast after hide_wizard_window — direct window.eval() not possible from wizard to toast window without Rust mediation"
  - "Skip for now in App.vue hidden when engineChoice === local — Step2Local owns its own Skip button that emits canProceed(true)"
  - "updatedConfig hoisted out of try block in wizard finish() so noModelDownloaded check can use it post-save"
  - "model_missing toast follows invalid_key pattern exactly: error type, Open Settings action, invoke open_settings_on_transcription_tab with provider: local"

patterns-established:
  - "canProceed emit pattern: Step2Local emits canProceed(boolean); parent App.vue binds to @can-proceed and disables Next button"
  - "Download event listeners registered in onMounted and cleaned up in onUnmounted via UnlistenFn[] array"

requirements-completed: [LOCL-02, LOCL-03, LOCL-07]

duration: 5min
completed: 2026-03-29
---

# Phase 10 Plan 03: Local Transcription Frontend Wiring Summary

**Settings model cards with real download/delete/active UI, Step2Local wizard model picker replacing placeholder, and model_missing error toast with Open Settings action**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-29T11:39:34Z
- **Completed:** 2026-03-29T11:44:12Z
- **Tasks:** 2 (Task 3 is human-verify checkpoint — awaiting)
- **Files modified:** 6 (1 created, 5 modified)

## Accomplishments

- TranscriptionSection.vue local model cards are fully functional: idle/downloading/downloaded states, progress bar with Cancel, Set Active + Delete buttons, green Active badge
- Step2Local.vue replaced entirely with 4-card model picker, progress tracking via events, Skip for now button emitting canProceed
- Wizard Next button on Step 2 local mode disabled until model downloaded or Skip clicked
- Post-finish nudge toast when user chose local engine but skipped downloading
- toast/App.vue handles model_missing code with "No local model downloaded" error + Open Settings button
- show_plain_toast Rust command added to allow wizard to show toast after window manipulation

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire Settings TranscriptionSection.vue local model cards** - `c6a0e5e` (feat)
2. **Task 2: Replace Step2Local.vue placeholder with model picker + missing model error handling** - `927cbb8` (feat)

**Plan metadata:** (docs commit follows after human verify)

## Files Created/Modified

- `src/windows/settings/components/TranscriptionSection.vue` - Added modelStates/downloadPercent/activeDownloadId refs, event listeners, startDownload/cancelDownload/deleteModel/setActiveModel actions, full idle/downloading/downloaded card templates
- `src/windows/wizard/components/Step2Local.vue` - Full replacement: 4 model cards with download/progress/cancel UI, Skip for now button, canProceed emit
- `src/windows/wizard/App.vue` - localCanProceed ref, @can-proceed handler on Step2Local, Next button disabled when local+!localCanProceed, Skip for now hidden for local mode, noModelDownloaded nudge toast logic
- `src/windows/toast/App.vue` - Added model_missing to TranscriptionErrorPayload code union, added model_missing handler showing Open Settings action
- `src-tauri/src/commands/indicator.rs` - Added show_plain_toast command (PlainToastPayload struct, calls indicator::show_toast_window)
- `src-tauri/src/lib.rs` - Registered show_plain_toast in invoke_handler

## Decisions Made

- Added `show_plain_toast` Rust command: wizard's `finish()` calls `hide_wizard_window` then triggers a nudge toast — direct eval from wizard window to toast window isn't possible, so a Rust command mediating `show_toast_window` is the correct approach
- `updatedConfig` hoisted outside try block in `finish()` so `noModelDownloaded` check is available after the save
- Skip for now in App.vue (for cloud Step 2) is now conditionally shown only for cloud mode — Step2Local has its own Skip inside the component that emits canProceed(true)
- `model_missing` toast follows the same pattern as `invalid_key`: error type, action button invoking `open_settings_on_transcription_tab` with `provider: "local"`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added show_plain_toast Rust command**
- **Found during:** Task 2 (wizard App.vue nudge toast implementation)
- **Issue:** Plan says "after Finish with skip: toast nudges to Settings" but no existing Rust command for showing a plain toast from the wizard window context. The `window.__voxflowShowToast` mechanism requires calling `eval()` on the toast webview, which must be mediated by Rust
- **Fix:** Added `show_plain_toast(toast_type, message)` command to `commands/indicator.rs` that calls `indicator::show_toast_window` with a `PlainToastPayload` struct; registered in `lib.rs`
- **Files modified:** src-tauri/src/commands/indicator.rs, src-tauri/src/lib.rs
- **Verification:** cargo check passes cleanly, vue-tsc passes
- **Committed in:** 927cbb8 (Task 2 commit)

**2. [Rule 1 - Bug] Fixed updatedConfig scope issue in wizard finish()**
- **Found during:** Task 2 (TypeScript typecheck)
- **Issue:** `updatedConfig` was defined inside `try` block but referenced after it for the `noModelDownloaded` check, causing TS2304 error
- **Fix:** Hoisted `updatedConfig` construction before the `try` block; only the `invoke("save_config")` call remains inside try
- **Files modified:** src/windows/wizard/App.vue
- **Verification:** npx vue-tsc --noEmit passes cleanly
- **Committed in:** 927cbb8 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 bug)
**Impact on plan:** Both fixes required for correct operation. No scope creep.

## Issues Encountered

- `npm run lint` script doesn't exist in this project — ESLint not configured as an npm script. TypeScript check via `vue-tsc --noEmit` used instead (matches CLAUDE.md guidance).

## Next Phase Readiness

- All 2 auto tasks complete; awaiting human verification (Task 3 checkpoint)
- Build command: `cargo tauri dev --features local-transcription`
- Human verifies 15-step checklist: download, progress, cancel, set active, delete, recording, error toast, wizard

---
*Phase: 10-local-transcription*
*Completed: 2026-03-29*
