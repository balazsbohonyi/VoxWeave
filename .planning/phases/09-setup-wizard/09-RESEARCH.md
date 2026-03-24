# Phase 9: Setup Wizard - Research

**Researched:** 2026-03-24
**Domain:** Tauri v2 multi-window lifecycle + Vue 3 wizard UI + config integration
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Trigger & Dismissal**
- Wizard window auto-opens on startup when `first_launch=true` — tray icon still appears immediately alongside it
- Close button (X) is allowed — closing without Finish leaves `first_launch=true`, so wizard re-opens on next app launch
- After Finish: wizard closes, settings window opens (smooth handoff for further configuration)

**Step Layout & Navigation**
- Top horizontal stepper: filled dot → line → hollow dot pattern, with step labels (Engine / Key / Hotkey)
- Content area below the stepper; Next/Back/Finish buttons at the bottom
- Back button available on Steps 2 and 3
- Step 2 has a "Skip for now" link — advances to Step 3 without saving API key; user can configure from Settings later
- Window size: 560×450px (same as settings window — no extra tauri.conf.json entry needed beyond width/height)

**Component Reuse**
- Step 1: Custom radio card selection (Cloud / Local) — new simple component, not from settings
- Step 2 (Cloud): slim custom component — provider tabs (OpenAI / Groq), API key field (masked + eye toggle), "Test connection" button with inline result; no model dropdown or language hint; whichever tab is active when Next is clicked becomes the active provider (implicit selection, no "Set as active" button)
- Step 2 (Local): placeholder message — "Local transcription coming soon. You can configure it later in Settings." — step advances to Step 3 immediately (no user input needed)
- Step 3: Reuse `HotkeyCapture.vue` directly — same press-to-capture widget as GeneralSection
- `useConfig()` + `saveConfig()` pattern throughout — same as all other windows

**Re-open from Settings**
- Trigger: "Setup Wizard..." button at the bottom of the General section in Settings
- Pre-fills all steps with current config values when re-opened
- Finish when re-opened: closes wizard, sets `first_launch=false`, settings window remains open

**First-Launch Flag**
- `first_launch: bool` already exists in `AppConfig` (config/mod.rs) — no new config field needed
- Set to `false` via `saveConfig({ first_launch: false })` on Finish
- Rust-side: check `first_launch` in `.setup()` callback and open wizard window if true (instead of / in addition to normal tray-only startup)

**Claude's Discretion**
- Exact stepper dot/line styling and active step color within existing blue-accent, dark-mode palette
- Animation/transition between steps (slide or fade)
- Exact wording of step titles and body copy
- "VoxFlow is ready" toast placement after Finish (indicator area or in-wizard before close)

### Deferred Ideas (OUT OF SCOPE)
- None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| WIZR-01 | On first launch (no config), app opens a 3-step setup wizard instead of minimizing to tray | `first_launch` field in AppConfig already exists; Rust `.setup()` callback reads it and calls `show_wizard_window` |
| WIZR-02 | Step 1: choose engine (Cloud or Local) | New `EngineCard.vue` radio card component; saves `transcription.provider` via `saveConfig` |
| WIZR-03 | Step 2: configure provider/API key with inline validation (Cloud) or placeholder (Local) | Slim API key form reusing `test_connection` Rust command; Local path is a static placeholder |
| WIZR-04 | Step 3: confirm default hotkey with option to change | Reuse `HotkeyCapture.vue` directly from settings window |
| WIZR-05 | "Finish" saves config and shows "VoxFlow is ready" toast | `saveConfig({ first_launch: false })` then open settings + show toast |
| WIZR-06 | Wizard can be re-opened from Settings at any time | "Setup Wizard..." button in GeneralSection invokes new `open_wizard_window` Rust command |
</phase_requirements>

---

## Summary

Phase 9 adds a 3-step first-launch setup wizard as a 4th Tauri window. The work is almost entirely frontend (Vue 3 + Tailwind) with a thin Rust layer for window lifecycle. All the hard infrastructure already exists: `first_launch` flag in `AppConfig`, `useConfig()` composable for partial saves, `HotkeyCapture.vue` for Step 3, `test_connection` Rust command for Step 2 API key validation, and the `show_settings_window` function in `tray.rs` as the exact model for `show_wizard_window`.

The wizard window is the 4th Tauri window (joining `settings`, `indicator`, `toast`). It follows the same `visible: false` in `tauri.conf.json` / programmatic show from `.setup()` pattern already used by all other windows. The quit path in `tray.rs` needs the `wizard` label added to its close loop.

**Primary recommendation:** Build as a dedicated Vite entry point (`wizard.html` → `src/windows/wizard/main.ts`) with a single `App.vue` that owns a `currentStep` ref and v-if renders each of three step components.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Vue 3 Composition API | ^3.5.13 (project) | Step routing via `ref<1|2|3>`, per-step components | Already the project's entire frontend |
| Tailwind CSS v4 | ^4.2.1 (project) | Dark-mode stepper dots, card styles, button states | Already used everywhere, no new dep |
| `@tauri-apps/api` | ^2 (project) | `invoke` for `save_config`, `test_connection`, `open_wizard_window` | Already the IPC layer |
| `useConfig()` composable | project | `loadConfig()` + `saveConfig(partial)` | Identical pattern to all other windows |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@tauri-apps/plugin-log` | ^2.8.0 (project) | `attachConsole()` in wizard `main.ts` | Same as `settings/main.ts` entry point |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| v-if step rendering | Vue Router | Router is overkill for a 3-step linear wizard with no URL needs |
| Custom stepper component | Third-party stepper lib | No UI libs allowed per CLAUDE.md key decisions |

**Installation:** No new packages needed — all dependencies already in `package.json`.

---

## Architecture Patterns

### Recommended Project Structure
```
src/windows/wizard/
├── main.ts              # Entry point — attachConsole + createApp
├── App.vue              # Step router — owns currentStep ref, renders step components
└── components/
    ├── WizardStepper.vue  # Top stepper dots + labels (Steps 1/2/3 visual indicator)
    ├── Step1Engine.vue    # Engine radio card selection (Cloud / Local)
    ├── Step2Cloud.vue     # API key + test connection (Cloud path)
    ├── Step2Local.vue     # Static placeholder (Local path)
    └── Step3Hotkey.vue    # Wraps HotkeyCapture.vue for hotkey confirm/change

wizard.html              # New HTML entry point (root)
```

### Pattern 1: Step Router in App.vue
**What:** `App.vue` owns `currentStep: Ref<1|2|3>` and `engineChoice: Ref<'cloud'|'local'>`. Renders step components with v-if. Bottom buttons (Back/Next/Skip/Finish) live in `App.vue` to keep navigation logic centralized.
**When to use:** Always — avoids prop-drilling step state into individual step components.

```typescript
// App.vue
const currentStep = ref<1 | 2 | 3>(1);
const engineChoice = ref<'cloud' | 'local'>('cloud');

function advance() {
  if (currentStep.value === 1) {
    currentStep.value = 2;
  } else if (currentStep.value === 2) {
    currentStep.value = 3;
  }
}
function back() {
  if (currentStep.value > 1) currentStep.value--;
}
async function finish() {
  await saveConfig({ first_launch: false });
  await invoke('open_settings_window');
  await getCurrentWindow().close();
}
```

### Pattern 2: New Vite Entry Point
**What:** A new `wizard.html` file at the project root (same level as `index.html`, `indicator.html`, `toast.html`) that points to `src/windows/wizard/main.ts`. Vite auto-discovers root-level HTML files as entry points — no `vite.config.ts` change needed.
**When to use:** Required — Tauri's `url: "wizard.html"` in `tauri.conf.json` must map to a real Vite-built HTML file.

```html
<!-- wizard.html -->
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>VoxFlow Setup</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/windows/wizard/main.ts"></script>
  </body>
</html>
```

### Pattern 3: New Tauri Window in tauri.conf.json
**What:** Add a 4th window entry with `label: "wizard"`, `url: "wizard.html"`, `560×450`, `visible: false`, `decorations: true`, `resizable: false`. Identical config shape to the `settings` window.
**When to use:** Required — wizard is a separate OS window.

```json
{
  "label": "wizard",
  "title": "VoxFlow Setup",
  "url": "wizard.html",
  "width": 560,
  "height": 450,
  "center": true,
  "visible": false,
  "skipTaskbar": false,
  "decorations": true,
  "resizable": false
}
```

### Pattern 4: Rust show_wizard_window (mirrors show_settings_window)
**What:** Free function in `tray.rs` (or a new `wizard.rs` mod) that does `app.get_webview_window("wizard")` → `.show()` → `.set_focus()`. Thin Tauri command `open_wizard_window` wraps it. Registered in `invoke_handler` in `lib.rs`.
**When to use:** Called from (a) `.setup()` callback when `first_launch=true`, and (b) the new `open_wizard_window` command invoked from `GeneralSection.vue`.

```rust
// commands/wizard.rs  (new file)
#[tauri::command]
pub fn open_wizard_window(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("wizard") {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

### Pattern 5: First-Launch Check in .setup()
**What:** After `tray::setup_tray(app)`, read `config.first_launch`. If true, call `show_wizard_window` instead of (or before) normal startup flow.
**When to use:** Required for WIZR-01.

```rust
// lib.rs .setup() — after tray setup
let first_launch = {
    let state = app.state::<AppState>();
    state.config.lock().unwrap().first_launch
};
if first_launch {
    if let Some(wizard_win) = app.get_webview_window("wizard") {
        let _ = wizard_win.show();
        let _ = wizard_win.set_focus();
    }
}
```

### Pattern 6: Wizard Close Event Handler (prevent re-use issue)
**What:** Register `on_window_event` for the `wizard` window in `.setup()`. On `CloseRequested`, just hide (same as settings). This allows the window to be re-shown from Settings without recreating the WebView. The window is never destroyed until app quit.
**When to use:** Required — mirrors the settings window close-to-hide pattern that is established in `lib.rs`.

### Pattern 7: Quit Path — Add "wizard" to Close Loop
**What:** In `tray.rs` `handle_menu_event` for `"quit"`, the existing loop closes `["settings", "indicator", "toast"]`. Add `"wizard"` to this list.
**When to use:** Required — otherwise the wizard window survives after app quit, causing "Failed to unregister class" errors on Windows.

### Pattern 8: Step 2 Implicit Provider Selection
**What:** The active tab in Step 2 Cloud (`"openai"` | `"groq"`) is local UI state. When the user clicks Next from Step 2, `App.vue` calls `saveConfig({ transcription: { ...config.transcription, provider: activeCloudTab } })`. No "Set as active" button needed.
**When to use:** Required per locked decision — implicit selection on Next click.

### Stepper Visual Pattern
**What:** Pure Tailwind — filled blue circle for completed/active steps, hollow circle with border for upcoming steps, connecting line between circles. Step labels below each dot.

```html
<!-- WizardStepper.vue concept -->
<div class="flex items-center justify-center gap-0 mb-8">
  <!-- Step 1 dot -->
  <div class="flex flex-col items-center gap-1">
    <div class="w-4 h-4 rounded-full"
         :class="currentStep >= 1 ? 'bg-blue-600' : 'border-2 border-gray-400'">
    </div>
    <span class="text-xs text-gray-500">Engine</span>
  </div>
  <!-- Connecting line -->
  <div class="w-16 h-0.5 mb-4"
       :class="currentStep >= 2 ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'">
  </div>
  <!-- Step 2 dot ... Step 3 dot -->
</div>
```

### Anti-Patterns to Avoid
- **Saving on every keypress in Step 2 API key field:** Use draft ref + save on blur, exactly as `TranscriptionSection.vue` does.
- **v-model directly on config fields:** Never bind v-model to `config.value.*` directly — use a draft ref and commit on save/blur.
- **Recreating the wizard window:** Never close+recreate; always hide and re-show. WebView recreations are slow and cause flicker.
- **Blocking `.setup()` with async:** The first_launch check and window show are synchronous — no `await` in `.setup()` callback.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Hotkey capture in Step 3 | New key capture logic | Reuse `HotkeyCapture.vue` directly | Exact same widget already built and tested in Phase 2/8 |
| API key test in Step 2 | Duplicate HTTP call | Reuse `test_connection` Tauri command | Identical endpoint already wired in Phase 8 |
| Config persistence in wizard | Custom save logic | `useConfig()` + `saveConfig(partial)` | Module-level shared state, partial merge, immediate persistence |
| Window single-instance guarantee | Custom tracking state | `app.get_webview_window("wizard")` check | Tauri label lookup is the established pattern — see `show_settings_window` |

---

## Common Pitfalls

### Pitfall 1: vite.config.ts Multi-Page Entry Points
**What goes wrong:** Developer adds `wizard.html` to the project root but Vite can't resolve `/src/windows/wizard/main.ts`. The `vite.config.ts` has no explicit `build.rollupOptions.input` — it relies on Vite's automatic root-level HTML discovery.
**Why it happens:** Vite v6 (used here) auto-discovers `*.html` files at the project root as MPA entry points. No `vite.config.ts` changes are needed. Confusion arises because developers expect to need to configure it.
**How to avoid:** Place `wizard.html` at project root alongside `index.html` and `indicator.html`. Vite discovers it automatically.
**Warning signs:** `404` for `wizard.html` in Tauri dev mode; Vite build missing `wizard.html` in `dist/`.

### Pitfall 2: Close-to-hide Not Registered for Wizard Window
**What goes wrong:** User clicks X on wizard → window is destroyed, not hidden. Next `.show()` call from Settings silently no-ops because the WebviewWindow no longer has a live WebView.
**Why it happens:** The `on_window_event` CloseRequested handler must be explicitly registered per-window label in `.setup()`. It is NOT inherited.
**How to avoid:** Register the same `CloseRequested` + `api.prevent_close()` + `.hide()` handler for the `wizard` window in `lib.rs`, identical to the settings window handler.
**Warning signs:** "Setup Wizard..." button in Settings opens nothing after wizard was closed with X once.

### Pitfall 3: useConfig Module-Level State Shared Across Windows
**What goes wrong:** `useConfig()` uses module-level refs — all calls in the same window share state. This is correct for single-window use. But if Settings and Wizard are both open and both call `loadConfig()`, the last caller wins on the shared `config` ref.
**Why it happens:** The composable is designed for single-window use. Two windows are separate WebView contexts — they don't actually share the same module state. Each window's JS bundle is isolated.
**How to avoid:** No action needed — each Tauri WebviewWindow is its own JS context. Module-level sharing is intra-window only. Confirmed by existing multi-window architecture (settings, indicator, toast all use `useConfig()`).
**Warning signs:** None — this is actually fine by design.

### Pitfall 4: Wizard Window Not Added to Quit Close Loop
**What goes wrong:** App quit via tray menu leaves the wizard window alive, causing WebView2 `Failed to unregister class` error on Windows.
**Why it happens:** The quit loop in `tray.rs` explicitly iterates over window labels: `["settings", "indicator", "toast"]`. The wizard label must be added manually.
**How to avoid:** Add `"wizard"` to the quit label array in `tray.rs`.
**Warning signs:** Console error `Failed to unregister class Chrome_WidgetWin_0` on quit; process doesn't exit cleanly.

### Pitfall 5: Step 2 Local Path Has No Input — Step Advances Without User Action
**What goes wrong:** Local engine was chosen in Step 1. Step 2 renders the placeholder message. User sees no Next button active or the step auto-advances unexpectedly.
**Why it happens:** The Context specifies "step advances to Step 3 immediately (no user input needed)" for Local. This means the Next button is always enabled in Step 2 Local — there's nothing to validate.
**How to avoid:** When engine is `'local'`, render `Step2Local.vue` (static message). Next button remains enabled and advances to Step 3 normally. No auto-advance without explicit Next click.

### Pitfall 6: Finish Sequence Order Matters
**What goes wrong:** Wizard closes before `saveConfig({ first_launch: false })` completes, leaving `first_launch=true` on disk. Wizard re-opens on next launch.
**Why it happens:** `invoke()` is async. If `getCurrentWindow().close()` fires before the `save_config` Rust command resolves, the write may not complete.
**How to avoid:** `await saveConfig(...)` before calling `open_settings_window` or closing the wizard. Chain: save → open settings → close wizard.

---

## Code Examples

### Registering wizard window event handler in lib.rs
```rust
// Source: mirrors existing settings window handler in lib.rs
if let Some(wizard_win) = app.get_webview_window("wizard") {
    let win_clone = wizard_win.clone();
    let quitting = app.state::<AppState>().quitting.clone();
    wizard_win.on_window_event(move |event| {
        match event {
            WindowEvent::CloseRequested { api, .. } => {
                if *quitting.lock().unwrap() {
                    return;
                }
                api.prevent_close();
                let _ = win_clone.hide();
            }
            _ => {}
        }
    });
}
```

### Implicit provider selection on Next in App.vue
```typescript
// Source: established saveConfig pattern from TranscriptionSection.vue
async function advanceFromStep2(): Promise<void> {
  if (engineChoice.value === 'cloud' && config.value) {
    await saveConfig({
      transcription: {
        ...config.value.transcription,
        provider: activeCloudTab.value as TranscriptionProvider,
      },
    });
  }
  currentStep.value = 3;
}
```

### Finish sequence in App.vue
```typescript
// Source: saveConfig pattern from useConfig.ts + invoke from @tauri-apps/api/core
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

async function finish(): Promise<void> {
  await saveConfig({ first_launch: false });
  await invoke('open_wizard_window_finish'); // or reuse open_settings_window
  await getCurrentWindow().close();         // becomes hide() via CloseRequested handler
}
```

Note: `getCurrentWindow().close()` triggers the `CloseRequested` event which calls `.hide()` (not destroy). This is correct — the wizard stays hidden for potential re-open from Settings.

### GeneralSection.vue — Setup Wizard button
```html
<!-- Source: established invoke pattern throughout settings components -->
<button
  type="button"
  class="mt-2 text-sm text-blue-600 dark:text-blue-400 hover:underline"
  @click="invoke('open_wizard_window')"
>
  Setup Wizard...
</button>
```

### HotkeyCapture.vue reuse in Step3Hotkey.vue
```html
<!-- Source: GeneralSection.vue — same props/emit contract -->
<HotkeyCapture
  :model-value="config?.hotkey ?? 'Ctrl+Shift+Space'"
  :warning="hotkeyWarning?.message ?? null"
  @save="(combo) => saveConfig({ hotkey: combo })"
/>
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Pinia store for shared state | Module-level refs in composable | From day 1 (CLAUDE.md) | No Pinia needed — composable pattern is the standard |
| Flat transcription config fields | Nested `providers.<id>` structure | Pre-Phase-8 | Step 2 saves to `transcription.providers.openai.api_key`, not flat `openai_api_key` |
| `show_settings_window` command | Free function in `tray.rs` + thin command | Phase 8 | `open_wizard_window` follows same pattern |

---

## Open Questions

1. **After Finish: how to open the Settings window from the Wizard window**
   - What we know: `tray::show_settings_window` is a free function, not a Tauri command. It's called from tray event handlers and internal Rust code.
   - What's unclear: Is there already a `show_settings_window` Tauri command registered, or does one need to be added for the wizard's finish action?
   - Recommendation: Check `lib.rs` invoke_handler — if `open_settings_on_transcription_tab` is already registered, a simpler `open_settings_window` command can be added alongside it. This is a 3-line addition to `commands/` and `lib.rs`.

2. **"VoxFlow is ready" toast placement**
   - What we know: Context leaves this to Claude's discretion. The existing toast system is the `toast` window (indicator-adjacent, shown by Rust). Showing it from the wizard's `finish()` would require the wizard window to emit a Rust event or the Rust command to emit one.
   - What's unclear: Simplest path is an in-wizard CSS toast (a brief success banner inside the wizard before it closes) rather than triggering the OS-level toast window.
   - Recommendation: In-wizard `<div>` success message that appears for ~1.5 seconds before the finish sequence closes the wizard. Avoids adding an `emit` path for a one-off UX moment.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) — no JS test framework present |
| Config file | `src-tauri/` (run tests from there) |
| Quick run command | `cd src-tauri && cargo test wizard` |
| Full suite command | `cd src-tauri && cargo test` |

No JS unit test framework is installed (no vitest/jest in package.json). Validation for this phase is TypeScript typecheck + manual smoke test.

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WIZR-01 | First-launch flag triggers wizard window open | manual smoke | — | N/A |
| WIZR-02 | Step 1 engine choice saves to config | manual smoke | — | N/A |
| WIZR-03 | Step 2 API key + test connection (Cloud) / placeholder (Local) | manual smoke | — | N/A |
| WIZR-04 | Step 3 hotkey confirm/change via HotkeyCapture | manual smoke | — | N/A |
| WIZR-05 | Finish sets `first_launch=false`, opens Settings | manual smoke | — | N/A |
| WIZR-06 | Re-open wizard from GeneralSection button | manual smoke | — | N/A |
| All | TypeScript types compile without errors | automated | `npx vue-tsc --noEmit` | ✅ already configured |
| All | Rust commands compile without errors | automated | `cd src-tauri && cargo build` | ✅ already configured |

### Sampling Rate
- **Per task commit:** `npx vue-tsc --noEmit` (TS typecheck)
- **Per wave merge:** `cd src-tauri && cargo test && npx vue-tsc --noEmit`
- **Phase gate:** Full build `cargo tauri build` + manual smoke through all 3 steps before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/windows/wizard/main.ts` — wizard entry point (none yet)
- [ ] `src/windows/wizard/App.vue` — step router (none yet)
- [ ] `wizard.html` — Vite MPA entry point (none yet)
- [ ] `src-tauri/src/commands/wizard.rs` — `open_wizard_window` command (none yet)

Note: No new test framework install needed. All validation is `vue-tsc` + `cargo test` + manual.

---

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection: `src-tauri/src/lib.rs` — window lifecycle, `.setup()` callback, invoke_handler registration
- Direct codebase inspection: `src-tauri/src/tray.rs` — `show_settings_window` pattern, quit close loop
- Direct codebase inspection: `src-tauri/src/config/mod.rs` — `first_launch: bool` field confirmed at line 297, `default_true()` at line 309
- Direct codebase inspection: `src-tauri/tauri.conf.json` — 3 existing windows, confirmed `wizard` window not yet present
- Direct codebase inspection: `vite.config.ts` — no explicit rollupOptions.input; Vite root HTML auto-discovery confirmed
- Direct codebase inspection: `src/composables/useConfig.ts` — `saveConfig(partial)` merges and calls `save_config` command
- Direct codebase inspection: `src/windows/settings/components/HotkeyCapture.vue` — props/emit contract confirmed
- Direct codebase inspection: `src/windows/settings/components/TranscriptionSection.vue` — draft ref + save on blur pattern, test_connection usage
- Direct codebase inspection: `src/windows/wizard/` — directory exists but is empty (no files yet)
- Direct codebase inspection: `package.json` — no JS test framework; `vue-tsc` is the only automated check

### Secondary (MEDIUM confidence)
- Tauri v2 docs (knowledge): WebviewWindow `.show()` / `.hide()` / `on_window_event` API is stable in Tauri v2
- Vite v6 MPA docs (knowledge): Root-level HTML files auto-discovered as entry points without `build.rollupOptions.input` config

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies confirmed from package.json; no new deps required
- Architecture: HIGH — all patterns derived directly from existing codebase (lib.rs, tray.rs, settings window)
- Pitfalls: HIGH — each pitfall is derived from established patterns already present in the codebase

**Research date:** 2026-03-24
**Valid until:** 2026-04-24 (stable stack; Tauri v2 API changes rarely affect existing patterns)
