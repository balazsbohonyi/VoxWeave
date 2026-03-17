---
status: diagnosed
trigger: "Invalid API key opens Settings window automatically instead of showing error toast"
created: 2026-03-18T00:00:00Z
updated: 2026-03-18T00:00:00Z
---

## Symptoms

expected: Error toast with "Open Settings" button appears in indicator; user clicks to open Settings
actual: Settings window opens automatically; no toast shown in indicator
reproduction: Set invalid API key, press hotkey, speak, stop — Settings opens immediately, no toast

## Root Cause

`src/windows/indicator/App.vue` lines 172-176: the `invalid_key` branch calls `invoke("open_settings_on_transcription_tab")` directly and immediately `return`s, bypassing `showTranscriptionErrorToast` entirely.

```ts
// CURRENT (broken):
if (payload.code === "invalid_key") {
  const provider = payload.provider;
  await invoke("open_settings_on_transcription_tab", { provider });
  return;
}
```

All other error codes (rate_limit, network, server) correctly route through `showTranscriptionErrorToast`. The toast infrastructure and action button support in `useToast.ts` are correct — only the `invalid_key` branch is wrong.

## Fix

**`src/windows/indicator/App.vue` lines 172-176 — show toast, move invoke to onClick:**
```ts
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
```

verification: not yet applied
files_changed: []
