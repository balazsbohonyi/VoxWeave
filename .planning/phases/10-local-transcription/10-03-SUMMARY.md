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
  - Wizard Next button gated on model downloaded or Skip clicked; Skip auto-advances to Step 3
  - Post-finish nudge toast when local engine chosen but no model downloaded
  - model_missing toast handler with Open Settings action in toast/App.vue
  - show_plain_toast Tauri command for programmatic plain toasts
  - Pre-recording guard emitting model_missing toast at hotkey press (before audio capture)
  - LocalFeatureDisabledProvider stub replacing panic when feature not compiled in

affects:
  - Local transcription end-to-end pipeline

tech-stack:
  added: []
  patterns:
    - UnlistenFn array pattern for bulk event cleanup in onUnmounted
    - Emit-gate pattern: component emits canProceed(boolean) to parent for Next button gating
    - show_plain_toast Rust command wraps show_toast_window for frontend-invokable plain toasts
    - Immediate state reset before async invoke to prevent UI badge flicker on delete
    - navigateNext emit from child component to trigger parent navigation on skip
    - LocalFeatureDisabledProvider stub: graceful error return instead of panic for disabled features

key-files:
  created:
    - src/windows/wizard/components/Step2Local.vue
  modified:
    - src/windows/settings/components/TranscriptionSection.vue
    - src/windows/wizard/App.vue
    - src/windows/toast/App.vue
    - src-tauri/src/commands/indicator.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/transcription/service.rs
    - src-tauri/src/hotkey/service.rs

key-decisions:
  - "show_plain_toast added as Rust command to allow wizard finish() to trigger nudge toast after hide_wizard_window — direct window.eval() not possible from wizard to toast window without Rust mediation"
  - "Skip for now in App.vue hidden when engineChoice === local — Step2Local owns its own Skip button that emits canProceed(true) and navigateNext"
  - "updatedConfig hoisted out of try block in wizard finish() so noModelDownloaded check can use it post-save"
  - "model_missing toast follows invalid_key pattern exactly: error type, Open Settings action, invoke open_settings_on_transcription_tab with provider: local"
  - "LocalFeatureDisabledProvider stub instead of panic(): app never crashes when local-transcription feature not compiled in"
  - "Pre-recording guard uses std::path::Path::new().exists() at hotkey time to check model availability before audio stream opens"
  - "modelState set to idle immediately on delete (before invoke) so Active badge never appears on a card being deleted"
  - "CSS transition removed from progress bar fill so percentage text and bar fill stay in visual sync"

patterns-established:
  - "canProceed + navigateNext emit pattern: child emits canProceed(boolean) for Next button gate; also emits navigateNext for skip-triggered navigation"
  - "Download event listeners registered in onMounted and cleaned up in onUnmounted via UnlistenFn[] array"
  - "Pre-recording guards: check requirements at hotkey press, emit toast immediately, never open audio stream if check fails"

requirements-completed: [LOCL-02, LOCL-03, LOCL-07]

duration: 40min
completed: 2026-03-29
---

# Phase 10 Plan 03: Local Transcription Frontend Wiring Summary

**Settings model cards with real download/delete/active UI, Step2Local wizard model picker, pre-recording model availability guard, and LocalFeatureDisabledProvider stub replacing panic — 8 post-UAT bugs fixed**

## Performance

- **Duration:** 40 min (initial 5 min + 35 min bug fix continuation)
- **Started:** 2026-03-29T11:39:34Z
- **Completed:** 2026-03-29T12:35:00Z
- **Tasks:** 3 (2 auto + 1 human-verify with 8 bug fixes)
- **Files modified:** 8 (1 created, 7 modified)

## Accomplishments

- TranscriptionSection.vue local model cards fully functional with correct styling (blue Active badge, 2px border, trash icon delete, Downloaded badge)
- Step2Local.vue wizard model picker with auto-advance on Skip
- App never panics when local-transcription feature not compiled — LocalFeatureDisabledProvider stub emits model_missing toast
- Pre-recording guard prevents audio capture when no local model file exists on disk
- All 8 post-UAT bugs fixed and committed

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire Settings TranscriptionSection.vue local model cards** - `c6a0e5e` (feat)
2. **Task 2: Replace Step2Local.vue placeholder with model picker** - `927cbb8` (feat)
3. **Task 3 bug fixes — Frontend UI (Issues 1-4, 7, 8)** - `dfa87bb` (fix)
4. **Task 3 bug fixes — Rust panic + pre-recording guard (Issues 5-6)** - `8bf127c` (fix)

## Files Created/Modified

- `src/windows/settings/components/TranscriptionSection.vue` - Full download/delete/active cards; blue Active badge, 2px border, Downloaded badge, trash icon; immediate state reset on delete; whole-number % without CSS transition
- `src/windows/wizard/components/Step2Local.vue` - Full model picker; navigateNext emit; same badge/border/percentage fixes
- `src/windows/wizard/App.vue` - localCanProceed ref, @can-proceed + @navigate-next handlers on Step2Local
- `src/windows/toast/App.vue` - model_missing handler with Open Settings action
- `src-tauri/src/commands/indicator.rs` - Added show_plain_toast command
- `src-tauri/src/lib.rs` - Registered show_plain_toast
- `src-tauri/src/transcription/service.rs` - LocalFeatureDisabledProvider stub replacing panic!()
- `src-tauri/src/hotkey/service.rs` - Pre-recording guard for local provider with no model

## Decisions Made

- `LocalFeatureDisabledProvider` implements `TranscriptionProviderTrait` returning `ModelMissing` — no behavioral change for builds with the feature; safe fallback otherwise
- Pre-recording model check uses `std::path::Path::new(&path).exists()` before touching audio hardware
- Active badge: `bg-blue-600 text-white` with `border-radius: 4px`; active card: `border-2 border-blue-600`
- Delete pre-clears `modelState` to "idle" before async invoke to prevent badge flicker
- CSS transition removed from progress bar fill to keep text and bar visually synchronized

## Deviations from Plan

### Auto-fixed Issues (initial tasks)

**1. [Rule 2 - Missing Critical] Added show_plain_toast Rust command**
- **Found during:** Task 2
- **Fix:** `show_plain_toast(toast_type, message)` command calls `indicator::show_toast_window` with PlainToastPayload
- **Committed in:** 927cbb8

**2. [Rule 1 - Bug] Fixed updatedConfig scope in wizard finish()**
- **Found during:** Task 2 (TypeScript typecheck)
- **Fix:** Hoisted `updatedConfig` before try block
- **Committed in:** 927cbb8

### Post-UAT Bug Fixes (Task 3 checkpoint response)

**3. [Rule 1 - Bug] Progress bar percentage showed decimal places** — `dfa87bb`

**4. [Rule 1 - Bug] Delete flashed Active badge** — Immediate state reset before invoke — `dfa87bb`

**5. [Rule 1 - Bug] Progress bar visual out of sync** — Removed `transition-all duration-300` — `dfa87bb`

**6. [Rule 1 - Bug] Active badge wrong styling** — Blue bg, white text, 4px radius, 2px border — `dfa87bb`

**7. [Rule 1 - Bug] App panics when local-transcription feature not compiled** — LocalFeatureDisabledProvider stub — `8bf127c`

**8. [Rule 2 - Missing Critical] No pre-recording check for missing model** — Pre-recording guard in hotkey/service.rs — `8bf127c`

**9. [Rule 1 - Bug] Settings missing "Downloaded" status text; text Delete button** — Downloaded badge + trash SVG icon — `dfa87bb`

**10. [Rule 1 - Bug] Skip for now didn't auto-advance** — navigateNext emit wired to advanceFromStep2 — `dfa87bb`

---

**Total deviations:** 10 auto-fixed (8 Rule 1 bugs, 2 Rule 2 missing critical)
**Impact on plan:** All fixes required for correct UX and safety. No scope creep.

## Issues Encountered

- `cargo check --features local-transcription` requires libclang (whisper-rs bindgen) which is not in PATH — confirms panic scenario when user's build lacks deps; stub fix resolves both cases.
- `npm run lint` script not configured in this project — TypeScript check via `vue-tsc --noEmit` used instead.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All 3 plans of Phase 10 complete (10-01, 10-02, 10-03)
- Phase 10 ready for PR and merge to main
- To run with local transcription: install CMake + MSVC BuildTools + libclang, rebuild with `--features local-transcription`

---
*Phase: 10-local-transcription*
*Completed: 2026-03-29*
