## Format

Each item follows the format: `[status] ID | Priority: Description`

- **ID** — unique identifier prefixed by section
  - `B` = Bug, `C` = Change, `R` = Removal, `F` = Feature, `T` = Testing
- **Priority** — `P1` (must do), `P2` (should do), `P3` (nice to have)
- **Status** — `[ ]` open, `[x]` completed

---

## Bugs

- [ ] B001 | P1: For local models, when doing a recording with silence (no talk at all), the app will inject: `[BLANK_AUDIO]`
  - sometimes for really long pauses it will inject in those parts of the text something like:
    - `[Pause]` or even multiple ones like `[Pause][Pause][Pause]`
- [x] B002 | P1: Fix the build. After creating a build and starting the .exe file the following happens:
  - indicator shows part of the Settings window - not the waveforms and the indicator states
  - Settings window can be opened from the tray
  - opening the wizard from the Settings window, opens the Settings window again (2 settings windows opened)
  - I suspect this is because we have the following HTML files in the root: `index.html`, `indicator.html`, `wizard.html` and `toast.html`
    - ...and `index.html` is actually for the Settings window, and `indicator.html` is for the indicator window
    - and probably the .exe considers the `index.html` file as the main entry point

## Changes

- [x] C001 | P1: Show success toast after injection only for the clipboard injection method
- [ ] C002 | P3:  Consider adding the following to the Vue code: linter & prettier
- [x] C003 | P1: Generate a CHANGELOGS.md document
- [ ] C004 | P1: Create and change the logo
- [ ] C005 | P1: Replace the ellipsis for the processing state with an animated spinner
- [ ] C006 | P1: Display a typing cue while injecting the text
- [x] C007 | P1: Simplify the waveforms - rounded rectangles instead of squares
  - also use natural smoothened animation and syne waveforms
- [ ] C008 | P1: simplify the indicator
  - make it smaller
  - display the indicator only when recording and remove the setting that forces to show it on app start
  - remove the red recording indicator circle
- [ ] C009 | P2: display the toasts in the bottom right corner, not relative to the indicator
  - use red / green / dark gray backgrounds with a gradient for warning / success / info toasts and white background
- [ ] C010 | P3: reference the DEV-SETUP.md file in the README.md file

## Removals

- [x] R001 | P1: Remove support for starting/stopping recording on the REC indicator
- [x] R002 | P2: Remove the success state visualization (green border + green REC indicator)
- [x] R004 | P1: Remove the "Start Recording" / "Stop Recording" tray options
- [x] R005 | P1: Remove support for injection cancellation with ESC, and the toast displayed after
- [x] R006 | P2: Remove the logging of the transcribed text - not needed anymore

## Testing

- [x] T001 | P1: Test and fix auto-stop for recording after 5 minutes
  - user should be informed when recording is close to 5 minutes
  - recording should stop after 5 minutes and text should be injected
- [x] T002 | P2: Test and fix auto-stop on silence
- [ ] T003 | P3: Test the warning toasts inside the Settings window
- [ ] T004 | P2: Test hotkey assignment for registered hotkey combinations (registered by other apps)
- [ ] T005 | P1: Test the "Auto-start on Windows startup" option

## New features

- [x] F001: Add all transcribed text to the clipboard as well
  - in case the user changes the focus, the transcribed text is also added to the clipboard
- [ ] F002: Snapshot the target where recording started, and in case during recording the target changed, use the snapshot target to inject the text into
- [ ] F003: Add support for push-to-talk - separate hotkey
- [ ] F004: Add support for pausing / resuming recordings
  - only for hands-free mode
- [ ] F005: Add support for Command Mode - separate hotkey
  - use with selected text to trigger voice edits like "make bullets" or "summarize"
  - investigate how would this work with local models