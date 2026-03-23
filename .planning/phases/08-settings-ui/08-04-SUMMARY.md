---
phase: 08-settings-ui
plan: 04
subsystem: ui
tags: [vue, typescript, tailwind, transcription, settings]

# Dependency graph
requires:
  - phase: 08-02
    provides: get_provider_models and test_connection Tauri commands
  - phase: 08-03
    provides: App.vue shell with transcription-stub placeholder
provides:
  - TranscriptionSection.vue: cloud/local toggle, provider tabs, API key inputs, model dropdowns, test connection flow, set-as-active, language hint, local model stub cards
affects: [08-05-injection-section, 08-settings-ui]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Draft refs pattern: watch(config, ..., { immediate: true }) syncs local drafts on load; never v-model on config directly
    - Blur-to-save for API keys (not input event — avoid saving on every keystroke)
    - Change-to-save for model dropdowns
    - activeTab (view state) vs config.transcription.provider (persisted active provider) are kept strictly separate

key-files:
  created:
    - src/windows/settings/components/TranscriptionSection.vue
  modified:
    - src/windows/settings/App.vue

key-decisions:
  - "activeTab is local UI state only — Set as active button is the only way to update config.transcription.provider"
  - "Language hint rendered outside per-tab block as a global field (one LanguageSelect regardless of active tab)"
  - "Local stub cards rendered in same component under isLocalMode() branch — no separate LocalSection component needed"
  - "saveApiKey called on @blur; clearTestState called on @input so test result disappears as user edits key"

patterns-established:
  - "Per-provider UI state (showApiKey, testConnectionState, testConnectionMessage) stored as Record<'openai'|'groq', T>"
  - "Deep spread pattern for nested provider config: spread transcription, then providers, then individual provider"

requirements-completed: [SETT-04, SETT-05]

# Metrics
duration: 2min
completed: 2026-03-22
---

# Phase 08 Plan 04: TranscriptionSection Summary

**Interactive transcription settings UI with cloud provider tabs (OpenAI/Groq), masked API keys, model dropdowns from Rust, test-connection flow, and local Whisper model stub cards**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-03-22T19:45:46Z
- **Completed:** 2026-03-22T19:48:10Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Built TranscriptionSection.vue (539 lines) covering SETT-04 and SETT-05 in full
- Cloud/local toggle correctly sets provider — switching to Cloud preserves existing openai/groq selection
- Provider tabs with Active badge only on the tab matching config.transcription.provider; activeTab never touches the persisted provider
- Password inputs with eye-icon SVG reveal toggle per provider; test connection state clears on any key edit
- Model dropdowns populated via `invoke('get_provider_models')` on mount — never hardcoded in Vue
- Test connection: spinner during pending, inline green/red result, no navigation
- Set as active: disabled when tab already matches active provider
- Language hint using LanguageSelect.vue rendered as global field below cloud tabs (and in local mode)
- Local stub section: 4 model cards (Tiny/Base/Small/Medium) with disabled Download buttons and "coming in a future update" tooltip
- Wired into App.vue replacing `<div id="transcription-stub">`

## Task Commits

1. **Task 1: TranscriptionSection.vue cloud tabs, API key, model, test connection** - `2da4e95` (feat)
2. **Task 2: Wire into App.vue, replace transcription-stub** - `ce708bc` (feat)

## Files Created/Modified

- `src/windows/settings/components/TranscriptionSection.vue` - Full transcription section (cloud tabs + local stubs)
- `src/windows/settings/App.vue` - Import and render TranscriptionSection, remove stub div

## Decisions Made

- activeTab is view-only state; only the "Set as active" button persists a new provider to config — satisfies CLAUDE.md key decision about keeping these two concerns separate
- Language hint rendered once outside the per-tab block since it is a global config field
- Local model stubs included directly in TranscriptionSection.vue rather than a separate component — kept simple given they are static UI

## Deviations from Plan

None - plan executed exactly as written.

Note: `npm run lint` does not exist in this project (no ESLint script in package.json). This is a pre-existing gap unrelated to this plan. TypeScript check (`npx vue-tsc --noEmit`) passes with zero errors.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- TranscriptionSection.vue is complete and wired into App.vue
- Plan 05 (InjectionSection) can now replace `<div id="injection-stub">` following the same pattern
- The transcription provider/tab distinction is clearly established in UI and code — future plans should not conflate the two

---
*Phase: 08-settings-ui*
*Completed: 2026-03-22*
