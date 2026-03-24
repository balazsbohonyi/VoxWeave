# Phase 9: Setup Wizard - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

A 3-step first-launch wizard that opens automatically when `first_launch=true`. Guides the user through: engine choice (Cloud/Local), API key setup with inline validation (Cloud) or a placeholder (Local), and hotkey confirmation. Sets `first_launch=false` on Finish. Re-openable from Settings at any time. Local transcription download/inference is Phase 10 scope — wizard only shows a placeholder for the Local path.

</domain>

<decisions>
## Implementation Decisions

### Trigger & Dismissal
- Wizard window auto-opens on startup when `first_launch=true` — tray icon still appears immediately alongside it
- Close button (X) is allowed — closing without Finish leaves `first_launch=true`, so wizard re-opens on next app launch
- After Finish: wizard closes, settings window opens (smooth handoff for further configuration)

### Step Layout & Navigation
- Top horizontal stepper: filled dot → line → hollow dot pattern, with step labels (Engine / Key / Hotkey)
- Content area below the stepper; Next/Back/Finish buttons at the bottom
- Back button available on Steps 2 and 3
- Step 2 has a "Skip for now" link — advances to Step 3 without saving API key; user can configure from Settings later
- Window size: 560×450px (same as settings window — no extra tauri.conf.json entry needed beyond width/height)

### Component Reuse
- Step 1: Custom radio card selection (Cloud / Local) — new simple component, not from settings
- Step 2 (Cloud): slim custom component — provider tabs (OpenAI / Groq), API key field (masked + eye toggle), "Test connection" button with inline result; no model dropdown or language hint
- Step 2 (Local): placeholder message — "Local transcription coming soon. You can configure it later in Settings." — step advances to Step 3 immediately (no user input needed)
- Step 3: Reuse `HotkeyCapture.vue` directly — same press-to-capture widget as GeneralSection
- `useConfig()` + `saveConfig()` pattern throughout — same as all other windows

### Re-open from Settings
- Trigger: "Setup Wizard..." button at the bottom of the General section in Settings
- Pre-fills all steps with current config values when re-opened
- Finish when re-opened: closes wizard, sets `first_launch=false`, settings window remains open

### First-Launch Flag
- `first_launch: bool` already exists in `AppConfig` (config/mod.rs) — no new config field needed
- Set to `false` via `saveConfig({ first_launch: false })` on Finish
- Rust-side: check `first_launch` in `.setup()` callback and open wizard window if true (instead of / in addition to normal tray-only startup)

### Claude's Discretion
- Exact stepper dot/line styling and active step color within existing blue-accent, dark-mode palette
- Animation/transition between steps (slide or fade)
- Exact wording of step titles and body copy
- "VoxFlow is ready" toast placement after Finish (indicator area or in-wizard before close)

</decisions>

<specifics>
## Specific Ideas

- Stepper matches the mockup discussed: `●─────────○─────────○` with step labels below
- Wizard feels like a focused modal, not a full app window — no section dividers, no sidebar nav
- Close = dismiss, not finish. Re-prompt on next launch.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `HotkeyCapture.vue`: press-to-capture widget — reuse directly in Step 3
- `useConfig()` composable: `saveConfig(partial)` for immediate persistence — same pattern throughout
- `LanguageSelect.vue`: NOT needed in wizard (language hint is omitted from wizard scope)
- `SectionDivider.vue`: optional reuse for any internal separation within a step

### Established Patterns
- Dark mode: all components use `dark:` Tailwind variants — maintain throughout
- Config persistence: partial update via `saveConfig`, no save button
- Window lifecycle: `visible: false` in tauri.conf.json, opened programmatically from Rust `.setup()` callback
- Three windows already defined in tauri.conf.json (settings, indicator, toast) — wizard is the 4th, needs a new entry with label `wizard` and its own HTML entry point

### Integration Points
- `main.rs` `.setup()` callback: read `config.first_launch`, conditionally open wizard window instead of (or after) normal tray boot
- `tauri.conf.json`: new window entry `wizard` with `wizard.html`, `560×450`, `visible: false`, `decorations: true`
- `vite.config.ts` / `index.html`: new entry point `src/windows/wizard/main.ts`
- `src/windows/wizard/`: new directory with `App.vue` (step router), step components, and stepper UI
- `GeneralSection.vue`: add "Setup Wizard..." button at the bottom, invokes a new `open_wizard_window` Rust command
- `open_wizard_window` Rust command: mirrors existing `show_settings_window` — single-instance check by label

</code_context>

<deferred>
## Deferred Ideas

- None — discussion stayed within phase scope

</deferred>

---

*Phase: 09-setup-wizard*
*Context gathered: 2026-03-24*
