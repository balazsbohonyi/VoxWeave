---
created: 2026-03-25T20:52:52.033Z
title: Implement wizard Step 2 local model configuration
area: ui
files:
  - src/windows/wizard/components/Step2Local.vue
  - src/windows/wizard/components/WizardStepper.vue
  - src/windows/wizard/App.vue
---

## Problem

When the user selects "Local" engine on Step 1 of the setup wizard, Step 2 currently shows a static "coming soon" placeholder (`Step2Local.vue`). This is intentional for Phase 9 since local transcription (whisper-rs) isn't implemented yet, but it needs real UI once Phase 10 lands.

The stepper Step 2 label was "API key" and has been renamed to "Configure" to be engine-agnostic, but the actual content for the local path is missing.

## Solution

When Phase 10 implements local transcription (`whisper-rs` behind the `local-transcription` cargo feature):

1. Replace `Step2Local.vue` placeholder with a real configuration form:
   - Model selection (tiny/base/small/medium/large) with size/quality hints
   - Model file path picker or download button
   - Device selection (CPU / GPU if available)
   - Match the visual style of `Step2Cloud.vue`

2. The stepper label "Configure" already works for both cloud and local — no further label changes needed.

3. Wire the local model config into `App.vue`'s `finish()` sequence so the selected model/path gets saved to `AppConfig` alongside `first_launch: false`.
