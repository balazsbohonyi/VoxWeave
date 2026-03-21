---
status: resolved
trigger: "On invalid API key, indicator shows error toast with Open Settings button; Settings window does not open automatically — no toast appears, Settings opens automatically instead"
created: 2026-03-18T00:00:00Z
updated: 2026-03-18T00:00:00Z
---

## Current Focus

hypothesis: confirmed — invalid_key branch calls open_settings_on_transcription_tab directly and returns early, bypassing showTranscriptionErrorToast entirely
test: read src/windows/indicator/App.vue lines 172-176
expecting: code calls showTranscriptionErrorToast with an "Open Settings" action button instead
next_action: fix applied (see Resolution)

## Symptoms

expected: When transcription-error fires with code "invalid_key", indicator shows an error toast with the error message and an "Open Settings" action button. User clicks the button to open Settings.
actual: Settings window opens automatically. No toast is shown.
errors: none — silent misbehavior
reproduction: configure an invalid API key, press hotkey, speak, release hotkey
started: introduced in Plan 05-03 when transcription-error listener was added

## Eliminated

- hypothesis: showToast composable is broken or missing action support
  evidence: useToast.ts correctly accepts ShowToastOptions with action field and pushes it to toasts array
  timestamp: 2026-03-18T00:00:00Z

- hypothesis: toast rendering is missing from the template
  evidence: indicator App.vue template has full toast rendering block with action button at lines 270-295
  timestamp: 2026-03-18T00:00:00Z

## Evidence

- timestamp: 2026-03-18T00:00:00Z
  checked: src/windows/indicator/App.vue lines 172-176
  found: |
    if (payload.code === "invalid_key") {
      void invoke("open_settings_on_transcription_tab", { provider: payload.provider });
      return;
    }
  implication: The invalid_key branch invokes open_settings_on_transcription_tab and returns immediately. showTranscriptionErrorToast is never called for this code path.

- timestamp: 2026-03-18T00:00:00Z
  checked: src/windows/indicator/App.vue lines 183-209
  found: all other error codes (rate_limit, network, server, non-retryable) call showTranscriptionErrorToast with appropriate action buttons
  implication: The pattern for showing a toast with an action button is already established; the invalid_key branch just needs to follow the same pattern.

- timestamp: 2026-03-18T00:00:00Z
  checked: src/composables/useToast.ts lines 28-48
  found: showToast accepts ShowToastOptions with optional action: { label, onClick }; action is stored on the toast and rendered by the template
  implication: No composable changes needed. The fix is entirely in the invalid_key branch of App.vue.

## Resolution

root_cause: |
  src/windows/indicator/App.vue lines 172-176.
  The invalid_key branch calls invoke("open_settings_on_transcription_tab") directly and returns early.
  This matches the observed symptom exactly: Settings opens automatically, no toast shown.
  The correct behavior is to show a toast with an "Open Settings" action button so the user
  chooses when to open Settings.

fix: |
  Replace the invalid_key branch (lines 172-176) with a showTranscriptionErrorToast call that
  includes an "Open Settings" action button whose onClick invokes open_settings_on_transcription_tab.

  OLD (lines 172-176):
    if (payload.code === "invalid_key") {
      void invoke("open_settings_on_transcription_tab", { provider: payload.provider });
      return;
    }

  NEW:
    if (payload.code === "invalid_key") {
      const provider = payload.provider;
      showTranscriptionErrorToast({
        message: payload.message,
        type: "error",
        action: {
          label: "Open Settings",
          onClick: () => {
            void invoke("open_settings_on_transcription_tab", { provider });
          },
        },
      });
      return;
    }

verification: root cause confirmed by direct code read; fix aligns with the pattern already used for all other error codes in the same listener block
files_changed:
  - src/windows/indicator/App.vue
