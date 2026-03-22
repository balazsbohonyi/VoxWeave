# Phase 8: Settings UI - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the current flat prototype `App.vue` with a full settings window exposing every configurable parameter. Requires a `TranscriptionConfig` struct migration (flat → nested `providers` map) as the first plan before any UI work. Local transcription sub-section (SETT-05) is UI-only stubs in this phase — actual download/inference is Phase 10.

</domain>

<decisions>
## Implementation Decisions

### Window Layout
- Single scrolling page (no sidebar nav, no tabs)
- Window width: ~480px (widened from current narrow ~384px)
- Section separation: heading label + subtle horizontal divider line beneath it
- Four sections in order: General, Audio, Transcription, Injection

### Hotkey Capture
- Keyboard press-to-capture widget (not a text field)
- Live preview while keys are held (e.g. "Ctrl+Shift+..." as modifier keys accumulate), auto-saves when all keys are released
- Conflict handling: inline warning below the field, keep previous hotkey active until a valid non-conflicting combo is captured
- Backend already emits `hotkey-warning` events — wire to the inline warning display

### Transcription Section
- Tabbed interface within the section: OpenAI tab, Groq tab (top-level Cloud/Local toggle per SETT-04)
- Active provider: explicit "Set as active" button per tab; active tab/provider visually highlighted (badge or accent border)
- "Test connection": button → spinner → inline result below button (success "Connected ✓" or error message); result clears on next edit
- API keys: masked by default (password input), eye icon toggle to reveal
- Language hint: reuse existing `LanguageSelect.vue` component (global field, not per-provider)
- Model dropdown: sourced from `get_provider_models` Rust command (already decided)

### Audio Section
- Microphone device dropdown (existing pattern, already works)
- Auto-stop on silence: toggle + number input in seconds with decimal (e.g. "1.5 s")
  - Duration input enabled only when toggle is ON (disabled/greyed when off)
  - Range: 0.5–5.0 seconds; stored internally as `vad_silence_ms`
  - `vad_threshold` stays hidden (too technical, use backend default)

### Injection Section
- Method selector: FlashPaste / Keystrokes / Clipboard (maps to `injection.mode`)
- Speed selector: only shown when Keystrokes is selected (maps to `injection.keystroke_speed`)
- Auto-fallback checkbox (maps to `injection.auto_fallback`)

### General Section
- Hotkey capture widget (see above)
- "Launch on Windows startup" toggle (maps to `launch_at_login`, default OFF)
- Minimize-to-tray toggle (already behaves this way; expose the control per SETT-02)

### Config Migration (First Plan)
- Migrate `TranscriptionConfig` from flat shape to nested `providers` map before any UI work
- Per-provider: `api_key`, `model` under `providers.<id>`
- Global: `language`, `provider` (active provider id), `fallback_order`
- Remove OpenRouter fields entirely
- Update: `config/mod.rs`, `types/index.ts`, all provider impls, `service.rs`
- Expose model lists via `get_provider_models` command

### Claude's Discretion
- Exact Tailwind spacing, typography scale, and color choices within the existing dark/light palette
- Loading skeleton or spinner while config loads
- Error state design if config fails to load
- Exact eye-icon SVG / toggle button style for API key reveal
- Transition/animation for the silence duration input appearing/disappearing

</decisions>

<specifics>
## Specific Ideas

- No specific UI references given — standard desktop settings aesthetic is fine
- Keep it consistent with the existing indicator/toast visual language (blue accents, gray neutrals, dark mode support)

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `LanguageSelect.vue`: searchable autocomplete for language hint — reuse directly in Transcription section
- `WarningCard.vue`: floating warning card — already used for hotkey/audio warnings, can remain for those
- `useConfig.ts`: `saveConfig(partial)` pattern — all new controls should use this for immediate persistence
- `useConfig.ts`: `loadAudioInputDevices()` + polling already wired — microphone dropdown is ready

### Established Patterns
- Dark mode: all components use `dark:` Tailwind variants — maintain throughout
- IPC: `invoke()` for user-triggered saves, `listen()` for backend-pushed events
- Config persistence: partial update via `saveConfig`, returns saved config, no save button
- Hotkey warning: backend emits `hotkey-warning` event, frontend listens in `useConfig` — wire new press-to-capture widget to this existing flow

### Integration Points
- `TranscriptionConfig` migration touches `src-tauri/src/config/mod.rs`, `src/types/index.ts`, `src-tauri/src/transcription/` provider impls, `service.rs`
- `get_provider_models` Rust command: new command to expose hardcoded model lists to frontend
- `launch_at_login` already in `AppConfig` struct — just needs a UI toggle wired to `saveConfig`
- `vad_silence_ms` and `vad_threshold` already in `AudioConfig` — silence controls just need UI wiring

</code_context>

<deferred>
## Deferred Ideas

- None — discussion stayed within phase scope

</deferred>

---

*Phase: 08-settings-ui*
*Context gathered: 2026-03-22*
