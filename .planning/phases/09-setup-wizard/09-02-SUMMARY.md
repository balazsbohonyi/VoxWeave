---
phase: 09-setup-wizard
plan: "02"
subsystem: wizard-ui
tags: [vue, tailwind, wizard, hotkey-capture, tauri, step-router]

requires:
  - phase: 09-01
    provides: [wizard-window-scaffold, open_wizard_window-command]
provides:
  - WizardStepper component (currentStep 1|2|3 → filled/hollow dots + labels)
  - Step1Engine component (cloud/local radio cards, v-model)
  - Step2Cloud component (provider tabs, masked API key, eye toggle, test_connection IPC, draft-on-blur)
  - Step2Local component (static placeholder for coming-soon local engine)
  - Step3Hotkey component (HotkeyCapture wrapper with saveConfig on save)
  - App.vue full step router (engineChoice, activeCloudTab, openaiKey, groqKey, back/next/skip/finish)
affects: [09-03]

tech-stack:
  added: []
  patterns:
    - draft-on-blur for API key inputs (local ref, emit on blur — mirrors TranscriptionSection.vue)
    - v-model pass-through on Step2Cloud (activeTab, openaiKey, groqKey as separate v-model props)
    - HotkeyCapture reuse across settings and wizard windows without modification

key-files:
  created:
    - src/windows/wizard/components/WizardStepper.vue
    - src/windows/wizard/components/Step1Engine.vue
    - src/windows/wizard/components/Step2Cloud.vue
    - src/windows/wizard/components/Step2Local.vue
    - src/windows/wizard/components/Step3Hotkey.vue
  modified:
    - src/windows/wizard/App.vue

key-decisions:
  - "showSuccessBanner ref deferred to Plan 03 — unused ref fails vue-tsc noUnusedLocals; Plan 03 adds it when wiring finish()"
  - "Step2Cloud uses internal draft refs (openaiDraft/groqDraft) and emits on blur — consistent with settings TranscriptionSection pattern"
  - "Step3Hotkey calls loadConfig() in onMounted independently — wizard is a separate window with its own JS context"

patterns-established:
  - "WizardStepper: dots use bg-blue-600 for active/done, border-2 border-gray-400 bg-transparent for future steps"
  - "Step2Cloud tab switching resets testResult and showKey to prevent stale state from prior provider"

requirements-completed: [WIZR-02, WIZR-03, WIZR-04]

duration: 3min
completed: "2026-03-24"
---

# Phase 9 Plan 02: Wizard Step Components Summary

**5-component wizard UI with step router, radio cards, tabbed API key entry with test_connection, and HotkeyCapture reuse from the settings window**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-24T20:10:28Z
- **Completed:** 2026-03-24T20:13:05Z
- **Tasks:** 2 auto + 1 checkpoint (awaiting human verification)
- **Files modified:** 6

## Accomplishments

- WizardStepper renders three connected dots with step labels; active/completed = filled blue, future = hollow gray
- Step1Engine provides accessible radio card selection (Cloud / Local) with v-model binding
- Step2Cloud provides provider tabs (OpenAI/Groq), masked key input with eye toggle, test connection IPC with spinner and inline result messages
- Step2Local is a static placeholder for the coming-soon local engine path
- Step3Hotkey wraps the existing HotkeyCapture widget from settings — zero code duplication
- App.vue is the full step router: WizardStepper + step content switching + bottom button bar (back/skip/next/finish)

## Task Commits

1. **Task 1: WizardStepper + Step1Engine + Step2Local** - `8df3fc6` (feat)
2. **Task 2: Step2Cloud + Step3Hotkey + App.vue full** - `eea9727` (feat)

## Files Created/Modified

- `src/windows/wizard/components/WizardStepper.vue` - Pure display: currentStep prop, three dots + lines + labels
- `src/windows/wizard/components/Step1Engine.vue` - Cloud/Local radio cards, v-model, sr-only hidden inputs
- `src/windows/wizard/components/Step2Cloud.vue` - Provider tabs, masked API key, eye toggle, test_connection IPC, draft-on-blur
- `src/windows/wizard/components/Step2Local.vue` - Static "coming soon" placeholder
- `src/windows/wizard/components/Step3Hotkey.vue` - HotkeyCapture wrapper with saveConfig on save emit
- `src/windows/wizard/App.vue` - Full step router replacing placeholder shell

## Decisions Made

- `showSuccessBanner` ref deferred to Plan 03: `noUnusedLocals` in vue-tsc flags it as an error; Plan 03 will add it when the `finish()` function is wired
- Step3Hotkey calls `loadConfig()` in its own `onMounted` — the wizard window has a separate JS context from settings, so composable state is not shared across windows
- Tab switch in Step2Cloud resets `testResult` and `showKey` to avoid stale state bleed between OpenAI and Groq tabs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed showSuccessBanner to satisfy noUnusedLocals**
- **Found during:** Task 2 (vue-tsc verification)
- **Issue:** Plan specified `showSuccessBanner: Ref<boolean>` stub for Plan 03, but vue-tsc `TS6133` errors on unused locals — eslint-disable comments do not suppress this error
- **Fix:** Removed `showSuccessBanner` declaration; Plan 03 will add it when it actually uses the value in `finish()`
- **Files modified:** src/windows/wizard/App.vue
- **Verification:** `npx vue-tsc --noEmit` passes
- **Committed in:** eea9727 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — TS unused variable)
**Impact on plan:** No functionality lost; Plan 03 will add showSuccessBanner when it wires the finish sequence.

## Issues Encountered

None beyond the auto-fixed deviation above.

## Next Phase Readiness

- All 5 step components and the step router are complete and TypeScript-clean
- Human checkpoint (Task 3) is next: `cargo tauri dev`, delete config.json, verify wizard opens and is navigable
- Plan 03 implements the `finish()` function: save config, close wizard, and show success state

## Self-Check: PASSED

All 6 files verified on disk. Commits 8df3fc6 and eea9727 verified in git log.

---
*Phase: 09-setup-wizard*
*Completed: 2026-03-24*
