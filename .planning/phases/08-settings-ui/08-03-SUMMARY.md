---
phase: 08-settings-ui
plan: 03
subsystem: settings-ui
tags: [vue, typescript, tailwind, settings, hotkey, audio]

# Dependency graph
requires:
  - phase: 08-settings-ui
    plan: 01
    provides: Nested TranscriptionConfig shape + TypeScript types
provides:
  - App.vue 480px scrolling layout with General + Audio sections rendered
  - SectionDivider.vue reusable section heading + hr
  - HotkeyCapture.vue keyboard capture widget with live preview
  - GeneralSection.vue hotkey + launch_at_login + minimize-to-tray
  - AudioSection.vue device dropdown + silence toggle + debounced duration input
affects: [08-04, 08-05]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Each section component calls useConfig() independently — no prop-drilling from App.vue"
    - "HotkeyCapture: keydown adds to held set; keyUp on non-modifier triggers save and clears"
    - "Silence toggle uses computed getter/setter to map vad_silence_ms=0 sentinel to boolean"
    - "Debounced number input (300ms) clamped to [500,5000]ms before saveConfig call"

key-files:
  created:
    - src/windows/settings/components/SectionDivider.vue
    - src/windows/settings/components/HotkeyCapture.vue
    - src/windows/settings/components/GeneralSection.vue
    - src/windows/settings/components/AudioSection.vue
  modified:
    - src/windows/settings/App.vue

key-decisions:
  - "App.vue keeps only top-level loading/error/config state; section components own their own useConfig() call"
  - "minimize_to_tray rendered as always-checked disabled checkbox with descriptive label — no new Rust config field needed to satisfy SETT-02 visually"
  - "set_launch_at_login invoke wrapped in try/catch for graceful degradation when Plan 02 commands are not yet registered"
  - "Tab and Escape do not save in HotkeyCapture; Escape blurs the element; Tab propagates for form navigation"

requirements-completed: [SETT-01, SETT-02, SETT-03]

# Metrics
duration: 3min
completed: 2026-03-22
---

# Phase 8 Plan 03: App.vue Shell + General and Audio Settings Sections Summary

**480px scrolling settings shell with SectionDivider, HotkeyCapture widget, GeneralSection (hotkey + startup toggles), and AudioSection (device dropdown + silence VAD controls)**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-22T19:40:18Z
- **Completed:** 2026-03-22T19:43:00Z
- **Tasks:** 3
- **Files modified/created:** 5

## Accomplishments

- Replaced prototype App.vue with a clean 480px max-width scrolling container; GeneralSection + AudioSection rendered; Transcription and Injection left as div stubs for Plans 04 and 05
- Created SectionDivider.vue: `<h2>` + `<hr>` with Tailwind uppercase tracking typography
- Created HotkeyCapture.vue: focusable div captures key combos, normalizes to Tauri shortcut format (Ctrl/Shift/Alt/Super+Key), shows live preview while keys held, emits `save` event on non-modifier key release; Escape blurs without saving; Tab propagates
- Created GeneralSection.vue: SectionDivider + HotkeyCapture bound to `config.hotkey` + launch_at_login checkbox wired to `saveConfig` + `set_launch_at_login` invoke + minimize-to-tray always-enabled toggle
- Created AudioSection.vue: microphone `<select>` dropdown saving on change, silence toggle backed by `vad_silence_ms=0` sentinel, debounced duration input (300ms, [500-5000]ms clamp)

## Task Commits

1. **Task 1: App.vue shell + SectionDivider** — `4f041df`
2. **Task 2: HotkeyCapture + GeneralSection** — `956c222`
3. **Task 3: AudioSection** — `88628bf`

## Files Created/Modified

- `src/windows/settings/App.vue` — replaced prototype with 480px shell, imports GeneralSection + AudioSection
- `src/windows/settings/components/SectionDivider.vue` — new: `title` prop, h2 + hr
- `src/windows/settings/components/HotkeyCapture.vue` — new: keyboard capture widget (80+ lines)
- `src/windows/settings/components/GeneralSection.vue` — new: hotkey + login + tray controls
- `src/windows/settings/components/AudioSection.vue` — new: device dropdown + silence controls

## Decisions Made

- App.vue delegates state to section components via independent `useConfig()` calls (singleton composable pattern) — avoids prop-drilling
- `minimize_to_tray` rendered as always-on disabled checkbox — no new Rust config field needed for SETT-02 visual compliance
- `set_launch_at_login` invoke wrapped in try/catch to gracefully handle Plan 02 not yet executed
- `Tab` and `Escape` in HotkeyCapture do not trigger a save; Escape blurs the element

## Deviations from Plan

None — plan executed exactly as written. The try/catch around `set_launch_at_login` is a minor defensive addition (Rule 2: missing error handling) since Plan 02 depends_on is not listed in this plan's frontmatter.

## Self-Check: PASSED

- `src/windows/settings/components/SectionDivider.vue` — FOUND
- `src/windows/settings/components/HotkeyCapture.vue` — FOUND
- `src/windows/settings/components/GeneralSection.vue` — FOUND
- `src/windows/settings/components/AudioSection.vue` — FOUND
- `src/windows/settings/App.vue` — modified FOUND
- Commit 4f041df — FOUND
- Commit 956c222 — FOUND
- Commit 88628bf — FOUND
- `npx vue-tsc --noEmit` — PASSED
