## Format

Each item follows the format: `[status] ID | Priority: Description`

- **ID** — unique identifier prefixed by section (`C` = Change, `R` = Removal, `F` = Feature, `T` = Testing)
- **Priority** — `P1` (must do), `P2` (should do), `P3` (nice to have)
- **Status** — `[ ]` open, `[x]` completed

---

## Changes

- [ ] C001 | P1: Show success toast after injection only for the clipboard injection method
- [ ] C002 | P3:  Consider adding the following to the Vue code: linter & prettier
- [x] C003 | P1: Generate a CHANGELOGS.md document
- [ ] C004 | P1: Create and change the logo
- [ ] C005 | P1: Replace the ellipsis for the processing state with an animated spinner
- [ ] C006 | P1: Display a typing cue while injecting the text
- [ ] C007 | P1: Simplify the waveforms - rounded rectangles instead of squares. Also smooth the animation.
- [ ] C008 | P1: simplify the indicator
  - make it smaller
  - display the indicator only when recording and remove the setting that forces to show it on app start
  - remove the red recording indicator circle
- [ ] C009 | P2: display the toasts in the bottom right corner, not relative to the indicator
  - use red / green / dark gray backgrounds with a gradient for warning / success / info toasts and white background
- [ ] C010 | P3: reference the DEV-SETUP.md file in the README.md file

## Removals

- [x] R001 | P1: Remove support for starting/stopping recording on the REC indicator
- [ ] R002 | P2: Remove the success state visualization (green border + green REC indicator)
- [x] R004 | P1: Remove the "Start Recording" / "Stop Recording" tray options
- [x] R005 | P1: Remove support for injection cancellation with ESC, and the toast displayed after

## Testing

- [ ] T001 | P1: Test and fix auto-stop for recording after 5 minutes
  - user should be informed when recording is close to 5 minutes
  - recording should stop after 5 minutes and text should be injected
- [ ] T002 | P2: Test and fix auto-stop on silence
- [ ] T003 | P3: Test the warning toasts inside the Settings window
- [ ] T004 | P2: Test hotkey assignment for registered hotkey combinations (registered by other apps)
- [ ] T005 | P1: Test the "Auto-start on Windows start" option


## New features

- [ ] F001: Add support for push-to-talk - separate hotkey
- [ ] F002: Add support for pausing / resuming recordings
  - only for hands-free mode
- [ ] F003: Add support for Command Mode - separate hotkey
  - use with selected text to trigger voice edits like "make bullets" or "summarize"
  - investigate how would this work with local models