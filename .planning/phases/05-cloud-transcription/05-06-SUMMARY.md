---
phase: 05-cloud-transcription
plan: "06"
subsystem: frontend-indicator
tags: [bug-fix, toast, css, ux, transcription-error]
dependency_graph:
  requires: []
  provides: [toast-visible-on-invalid-key, toast-css-classes]
  affects: [src/windows/indicator/App.vue, src/styles.css]
tech_stack:
  added: []
  patterns: [toast-action-pattern, indicator-palette-css]
key_files:
  modified:
    - src/windows/indicator/App.vue
    - src/styles.css
decisions:
  - invalid_key shows toast with "Open Settings" action button (not auto-opens Settings)
  - Toast CSS appended to styles.css to match existing indicator colour palette
metrics:
  duration: "3m"
  completed_date: "2026-03-18"
  tasks_completed: 2
  files_modified: 2
---

# Phase 5 Plan 6: Frontend Toast Bug Fixes Summary

Fix invalid_key handler to show toast instead of auto-opening Settings, and add missing toast CSS classes to styles.css.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Fix invalid_key handler — show toast instead of auto-opening Settings | b05dd77 | src/windows/indicator/App.vue |
| 2 | Add toast CSS classes to styles.css | 02b1b78 | src/styles.css |

## What Was Built

**Task 1 — App.vue invalid_key fix:**
The `invalid_key` branch in the `transcription-error` listener was directly calling `invoke('open_settings_on_transcription_tab')` instead of showing a toast. This prevented any toast from appearing on invalid API key errors (UAT gap 2) and also blocked the `fallback_provider` toast path (UAT gap 4) because the `return` statement short-circuited execution. The fix replaces the direct invoke with a `showTranscriptionErrorToast` call that includes an "Open Settings" action button whose `onClick` performs the invoke — matching the pattern used by all other error code branches.

**Task 2 — styles.css toast CSS:**
The `src/styles.css` file had zero toast CSS classes despite the App.vue template referencing `.indicator-toasts`, `.indicator-toast`, `.indicator-toast--error`, `.indicator-toast-message`, `.indicator-toast-action`, and `.indicator-toast-dismiss`. All toast DOM nodes were rendering unstyled. Added a complete toast CSS block using the existing indicator colour palette (background `#0b1220`/`#111827`, error accent `rgba(246,79,98,0.55)`, text `#e7eefc`/`#f4f8ff`).

## Verification

- `npx vue-tsc --noEmit` passes with no errors (both tasks)
- `npm run lint` does not exist in this project (no ESLint script configured) — TypeScript typecheck is the applicable gate
- CSS class selectors `.indicator-toast`, `.indicator-toast-action`, `.indicator-toast-dismiss` all present in styles.css

## Deviations from Plan

### Auto-fixed Issues

None.

### Process Deviations

**1. [Rule 1 - Missing script] `npm run lint` does not exist**
- **Found during:** Task 2 verification
- **Issue:** The plan specifies `npm run lint` as the verification command, but the project has no `lint` script in `package.json`. Available scripts: `dev`, `build`, `preview`, `tauri`.
- **Fix:** Used `npx vue-tsc --noEmit` (the project's standard typecheck) as the verification gate instead. CSS files have no linting configured in this project.
- **Impact:** No impact on code quality — the CSS changes are straightforward and correct.

## Success Criteria Check

- [x] `npx vue-tsc --noEmit` passes with no errors
- [x] App.vue invalid_key handler calls `showTranscriptionErrorToast` with `action.onClick` invoking `open_settings_on_transcription_tab`
- [x] App.vue does NOT call `invoke('open_settings_on_transcription_tab')` directly in the `invalid_key` branch
- [x] styles.css contains `.indicator-toasts`, `.indicator-toast`, `.indicator-toast--error`, `.indicator-toast-message`, `.indicator-toast-action`, `.indicator-toast-dismiss`
