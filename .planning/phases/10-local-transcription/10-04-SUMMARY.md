---
phase: 10-local-transcription
plan: "04"
subsystem: ui
tags: [vue, tauri, local-transcription, model-management]

# Dependency graph
requires:
  - phase: 10-local-transcription
    provides: UAT results identifying 3 gaps to close
provides:
  - Re-download after delete works correctly (activeDownloadId reset on delete)
  - Button order is delete-then-set-active in both Settings and Wizard
  - Wizard window height 550px eliminates Step 2 vertical scrollbar
affects: [10-local-transcription]

# Tech tracking
tech-stack:
  added: []
  patterns: []

key-files:
  created: []
  modified:
    - src/windows/settings/components/TranscriptionSection.vue
    - src/windows/wizard/components/Step2Local.vue
    - src-tauri/tauri.conf.json

key-decisions:
  - "activeDownloadId.value = null added as first statement in deleteModel() — ensures Download button re-enables immediately after delete without waiting for any download lifecycle event"

patterns-established: []

requirements-completed: [LOCL-02, LOCL-03]

# Metrics
duration: 2min
completed: 2026-03-30
---

# Phase 10 Plan 04: UAT Gap Closure Summary

**Three UAT gaps closed: re-download after delete re-enabled, button order corrected to delete-before-set-active, wizard window height increased to 550px**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-03-30T13:43:59Z
- **Completed:** 2026-03-30T13:45:35Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Fixed re-download after delete by resetting `activeDownloadId.value = null` as the first statement in `deleteModel()` in both `TranscriptionSection.vue` and `Step2Local.vue`
- Swapped button order in both components so the trash (delete) icon renders before the Set Active button
- Increased wizard window height from 520 to 550 in `tauri.conf.json` to eliminate vertical scrollbar on Step 2 local model picker

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix re-download after delete (Settings + Wizard)** - `2dd7e00` (fix)
2. **Task 2: Swap button order + increase wizard height** - `6db3ac3` (fix)

**Plan metadata:** (final docs commit follows)

## Files Created/Modified

- `src/windows/settings/components/TranscriptionSection.vue` - Added activeDownloadId reset in deleteModel(); swapped button order to delete-then-set-active
- `src/windows/wizard/components/Step2Local.vue` - Same two changes as TranscriptionSection.vue
- `src-tauri/tauri.conf.json` - Wizard window height 520 -> 550

## Decisions Made

None - followed plan as specified. Root cause and fixes were precisely described in the plan.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All 3 UAT gaps are now closed
- Phase 10 (Local Transcription) can be marked complete — all requirements satisfied, UAT issues resolved
- Ready for final PR merge to main

## Self-Check: PASSED

- FOUND: src/windows/settings/components/TranscriptionSection.vue
- FOUND: src/windows/wizard/components/Step2Local.vue
- FOUND: src-tauri/tauri.conf.json
- FOUND: .planning/phases/10-local-transcription/10-04-SUMMARY.md
- FOUND: commit 2dd7e00 (Task 1)
- FOUND: commit 6db3ac3 (Task 2)

---
*Phase: 10-local-transcription*
*Completed: 2026-03-30*
