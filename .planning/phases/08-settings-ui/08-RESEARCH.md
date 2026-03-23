# Phase 8: Settings UI - Research

**Researched:** 2026-03-22
**Domain:** Vue 3 / Tauri v2 Settings UI + Rust config struct migration
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Window Layout**
- Single scrolling page (no sidebar nav, no tabs)
- Window width: ~480px (widened from current narrow ~384px)
- Section separation: heading label + subtle horizontal divider line beneath it
- Four sections in order: General, Audio, Transcription, Injection

**Hotkey Capture**
- Keyboard press-to-capture widget (not a text field)
- Live preview while keys are held (e.g. "Ctrl+Shift+..." as modifier keys accumulate), auto-saves when all keys are released
- Conflict handling: inline warning below the field, keep previous hotkey active until a valid non-conflicting combo is captured
- Backend already emits `hotkey-warning` events — wire to the inline warning display

**Transcription Section**
- Tabbed interface within the section: OpenAI tab, Groq tab (top-level Cloud/Local toggle per SETT-04)
- Active provider: explicit "Set as active" button per tab; active tab/provider visually highlighted (badge or accent border)
- "Test connection": button → spinner → inline result below button (success "Connected ✓" or error message); result clears on next edit
- API keys: masked by default (password input), eye icon toggle to reveal
- Language hint: reuse existing `LanguageSelect.vue` component (global field, not per-provider)
- Model dropdown: sourced from `get_provider_models` Rust command (already decided)

**Audio Section**
- Microphone device dropdown (existing pattern, already works)
- Auto-stop on silence: toggle + number input in seconds with decimal (e.g. "1.5 s")
  - Duration input enabled only when toggle is ON (disabled/greyed when off)
  - Range: 0.5–5.0 seconds; stored internally as `vad_silence_ms`
  - `vad_threshold` stays hidden (too technical, use backend default)

**Injection Section**
- Method selector: FlashPaste / Keystrokes / Clipboard (maps to `injection.mode`)
- Speed selector: only shown when Keystrokes is selected (maps to `injection.keystroke_speed`)
- Auto-fallback checkbox (maps to `injection.auto_fallback`)

**General Section**
- Hotkey capture widget (see above)
- "Launch on Windows startup" toggle (maps to `launch_at_login`, default OFF)
- Minimize-to-tray toggle (already behaves this way; expose the control per SETT-02)

**Config Migration (First Plan)**
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

### Deferred Ideas (OUT OF SCOPE)
- None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SETT-01 | Settings window has sections: General, Audio, Transcription, Injection | Single scrolling page with four section components; section ordering locked |
| SETT-02 | General: hotkey capture input, "Launch on Windows startup" toggle (default OFF), minimize-to-tray toggle | `launch_at_login` field already in `AppConfig`; hotkey capture replaces current text input; `saveConfig` pattern applies to all three |
| SETT-03 | Audio: microphone device dropdown + Auto-stop on silence toggle + silence-duration input | `vad_silence_ms` / `vad_threshold` already in `AudioConfig`; device polling in `useConfig` already works |
| SETT-04 | Transcription: Cloud/Local toggle; Cloud tabs (OpenAI, Groq) each with API key, model dropdown, language, "Test connection", "Set as active" | Requires config migration first; `get_provider_models` command; test_connection is a new Tauri command |
| SETT-05 | Transcription: Local sub-section with model variants, sizes, download/delete buttons, progress bar | UI stubs only this phase — no download/inference logic; local provider already returns `unimplemented!()` |
| SETT-06 | Injection: method selector (FlashPaste/Keystrokes/Clipboard), speed selector (Keystrokes only), auto-fallback checkbox | `InjectionConfig` struct complete; all fields already mapped to serializable enums |
| SETT-07 | All settings persist immediately (no save button) and are restored on restart | `saveConfig(partial)` + serde defaults pattern already established and working |
</phase_requirements>

---

## Summary

Phase 8 replaces the prototype `App.vue` (a narrow debug view) with a full settings window. The work has two distinct parts: a Rust-side config struct migration that must land first, and the full Vue 3 UI build that follows.

The migration changes `TranscriptionConfig` from a flat struct (`openai_api_key`, `groq_api_key`, `openai_model`, `groq_model`) to a nested `providers` map keyed by provider id (`providers.openai.api_key`, `providers.openai.model`, etc.). This touches `config/mod.rs`, `types/index.ts`, both provider impls (`openai.rs`, `groq.rs`), and `service.rs` (which reads flat fields in several places today). All tests that construct `TranscriptionConfig` with flat field names will need updating. Config files on disk from Phase 7 use the old shape — serde's `#[serde(default)]` and the existing unknown-field round-trip mechanism (`config_raw` + `merge_into`) handle the migration transparently.

The UI build decomposes `App.vue` into focused section components inside `src/windows/settings/`. All four sections use the established `saveConfig(partial)` pattern for immediate persistence. The hotkey section requires a new press-to-capture composable (keyboard event capture, modifier accumulation, release-to-save). The transcription section requires a new `test_connection` Tauri command and the new `get_provider_models` command. The local transcription sub-section (SETT-05) is UI stubs only — no download or inference.

**Primary recommendation:** Execute the config migration as Plan 1 before any UI work. The migration is a breaking change to the IPC contract between frontend and Rust; doing it first means the UI never has to target the old shape.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Vue 3 (Composition API) | existing | Component logic | Already used throughout |
| Tailwind CSS | existing | Styling | Already used, `dark:` variants throughout |
| `@tauri-apps/api/core` invoke | existing | Rust command calls | Established IPC pattern |
| `@tauri-apps/api/event` listen | existing | Backend event subscription | Established event pattern |
| TypeScript | existing | Type safety | Required by project |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `useConfig.ts` composable | existing | Config load/save/events | All settings controls |
| `LanguageSelect.vue` | existing | BCP-47 language autocomplete | Transcription section language hint |
| `WarningCard.vue` | existing | Inline warning display | Hotkey conflict, audio warnings |
| serde `#[serde(rename_all = "snake_case")]` | existing | Enum serialization | All new Rust enums exposed to frontend |
| `serde_json::Value` raw round-trip | existing | Unknown field preservation | Config migration compatibility |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom tab component | Headless UI / Radix | Overkill; CLAUDE.md explicitly prohibits heavy UI libraries |
| Direct DOM keyboard events | Vue `v-on:keydown` | Same thing; Vue's binding is cleaner |
| Pinia for component state | Local `ref`s + props | CLAUDE.md: no Pinia |

**Installation:** No new dependencies required. All needed libraries are already in the project.

---

## Architecture Patterns

### Recommended Project Structure

The existing `src/windows/settings/` directory becomes the home for all new components:

```
src/windows/settings/
├── App.vue                        # Root — replaced with full scrolling settings page
├── main.ts                        # Unchanged entry point
└── components/
    ├── WarningCard.vue            # Existing — reuse unchanged
    ├── LanguageSelect.vue         # Existing — reuse unchanged
    ├── SectionDivider.vue         # New — heading + hr visual separator
    ├── GeneralSection.vue         # New — hotkey capture + toggles
    ├── AudioSection.vue           # New — device dropdown + silence controls
    ├── TranscriptionSection.vue   # New — cloud/local toggle + provider tabs
    ├── InjectionSection.vue       # New — method/speed/fallback controls
    └── HotkeyCapture.vue          # New — press-to-capture keyboard widget
```

On the Rust side, two new commands land in `commands/config.rs`:

```
src-tauri/src/
├── config/
│   └── mod.rs                    # TranscriptionConfig migrated to nested providers map
├── commands/
│   └── config.rs                 # + get_provider_models command
│                                 # + test_connection command
└── transcription/
    ├── openai.rs                 # Read from providers.openai.* instead of flat fields
    └── groq.rs                   # Read from providers.groq.* instead of flat fields
```

### Pattern 1: Config Migration — Nested Providers Map

**What:** Replace flat `openai_api_key` / `groq_api_key` / `openai_model` / `groq_model` fields with a `HashMap<String, ProviderConfig>` (or a dedicated struct per provider) stored as `providers` in JSON.

**Design choice — struct vs HashMap:** A fixed struct with named fields (`openai: ProviderConfig`, `groq: ProviderConfig`) is preferable to `HashMap<String, ProviderConfig>` because it keeps the serde schema explicit and avoids runtime key lookups. The two known providers can have typed fields; Local stays separate.

**Proposed new shape:**

```rust
// config/mod.rs

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct CloudProviderConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "...")]      // provider-specific default model
    pub model: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct TranscriptionProviders {
    #[serde(default)]
    pub openai: CloudProviderConfig,
    #[serde(default)]
    pub groq: CloudProviderConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionConfig {
    #[serde(default)]
    pub provider: TranscriptionProvider,         // active provider
    #[serde(default)]
    pub providers: TranscriptionProviders,       // per-provider config
    #[serde(default)]
    pub language: String,                        // global language hint
    #[serde(default)]
    pub local_model_path: Option<String>,
    #[serde(default = "default_fallback_order")]
    pub fallback_order: Vec<TranscriptionProvider>,
}
```

**TypeScript mirror:**

```typescript
// types/index.ts

export interface CloudProviderConfig {
  api_key: string;
  model: string;
}

export interface TranscriptionProviders {
  openai: CloudProviderConfig;
  groq: CloudProviderConfig;
}

export interface TranscriptionConfig {
  provider: TranscriptionProvider;   // "openai" | "groq" | "local"
  providers: TranscriptionProviders;
  language: string;
  local_model_path: string | null;
  fallback_order: TranscriptionProvider[];
}

// Remove: openrouter from TranscriptionProvider union
export type TranscriptionProvider = "openai" | "groq" | "local";
```

**Migration concern — on-disk config compatibility:** Existing configs on disk have the flat shape. Serde with `#[serde(default)]` will produce empty strings/defaults for `providers.openai.api_key` etc. when loading an old config. The existing `merge_into()` unknown-field round-trip does not help here because the old keys (`openai_api_key`) map to a *different* path in the new schema. **Action required:** The migration plan must include a one-time field migration step in `AppState::load()` that checks for old flat keys in `config_raw` and promotes them to the new nested path before deserializing.

### Pattern 2: New Rust Commands

**`get_provider_models` command:**

```rust
// commands/config.rs
#[derive(serde::Serialize)]
pub struct ProviderModels {
    pub openai: Vec<String>,
    pub groq: Vec<String>,
}

#[tauri::command]
pub fn get_provider_models() -> ProviderModels {
    ProviderModels {
        openai: vec![
            "whisper-1".to_string(),
            "gpt-4o-transcribe".to_string(),
            "gpt-4o-mini-transcribe".to_string(),
        ],
        groq: vec![
            "whisper-large-v3-turbo".to_string(),
            "whisper-large-v3".to_string(),
            "distil-whisper-large-v3-en".to_string(),
        ],
    }
}
```

**`test_connection` command:**

```rust
#[tauri::command]
pub async fn test_connection(
    state: State<'_, AppState>,
    provider: String,
) -> Result<(), String> {
    // Send a minimal transcription request (e.g. 0-byte silent WAV or a known-good tiny sample)
    // Return Ok(()) on HTTP 200, Err(message) on failure.
    // Must NOT use the recording pipeline — make a direct HTTP probe.
}
```

The simplest implementation: send a request with no audio bytes. OpenAI and Groq will respond with a 400 (bad request) or 401 (invalid key). A 401 → "Invalid API key". A 400 → "Connected (key valid)". A network error → connection failure message. This avoids sending real audio just to verify the key.

### Pattern 3: Hotkey Press-to-Capture Widget

**What:** `HotkeyCapture.vue` — a focusable div that captures keyboard events, accumulates modifier keys, previews the combo live, and saves on all-keys-released.

```typescript
// HotkeyCapture.vue composable logic (pseudo)
const MODIFIER_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta']);

function onKeyDown(e: KeyboardEvent) {
  e.preventDefault();
  pressedKeys.add(e.key === 'Control' ? 'Ctrl' : e.key);
  updatePreview();
}

function onKeyUp(e: KeyboardEvent) {
  if (!MODIFIER_KEYS.has(e.key)) {
    // A non-modifier was released — the combo is complete
    triggerSave(currentCombo.value);
    pressedKeys.clear();
  }
}
```

**Format rule:** Combo string must match Tauri's global shortcut format: `Ctrl+Shift+Space`, `Alt+F4`, etc. The widget must normalize key names to Tauri's expected casing (e.g. `" "` → `Space`, `ArrowUp` → `Up`).

**Conflict handling:** The backend already emits `hotkey-warning` events (listened in `useConfig`). The widget shows `hotkeyWarning` inline and does NOT persist the conflicting combo. The previous hotkey remains active until a valid combo is accepted.

### Pattern 4: Silence Duration UI ↔ Config Mapping

`vad_silence_ms` is stored as milliseconds (integer). The UI shows seconds with one decimal place.

```typescript
// display: vad_silence_ms / 1000  →  "1.5 s"
// save:    parseFloat(inputValue) * 1000  →  stored as vad_silence_ms
// clamp:   min=500 (0.5s), max=5000 (5.0s), step=0.1
```

The toggle (`auto_stop_enabled`) is a UI-only concept — the backend uses `vad_silence_ms === 0` as "disabled" OR the toggle writes a sentinel value. **Design decision for planner:** Use `vad_silence_ms = 0` to mean "disabled" (backend already treats 0ms as never-stop), so toggling off sets `vad_silence_ms = 0` and the previous value is held locally until toggle is restored. Alternatively, keep a separate `vad_enabled: bool` field. The simpler path (sentinel 0) avoids adding a new config field and keeps Rust unchanged.

### Pattern 5: Immediate Persistence (No Save Button)

The `saveConfig(partial)` pattern from `useConfig.ts` is established and works for all fields. Every interactive control fires a save on change:
- Toggle: `@change="saveConfig({ launch_at_login: $event.target.checked })"`
- Select: `@change="saveConfig({ injection: { ...config.injection, mode: $event.target.value } })"`
- Number input (silence duration): debounce 300ms to avoid saving on every keystroke

### Anti-Patterns to Avoid

- **Two-way binding on config directly:** Never `v-model="config.hotkey"` on the reactive config ref. Always use a local draft ref synced from config, save explicitly. Direct mutation breaks the save/restore guarantee.
- **Saving nested config sections with spread only:** `saveConfig({ transcription: { ...config.transcription, language: 'en' } })` is correct. Forgetting the spread drops other fields in the section.
- **Holding MutexGuard across await in new Rust commands:** `test_connection` must clone the config before any `.await` (same pattern as `transcribe_with_retry`).
- **Using `openrouter` anywhere:** Remove from `TranscriptionProvider` enum in both Rust and TypeScript. Any match arms or references will cause compile errors to catch stragglers.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Language autocomplete | Custom dropdown | `LanguageSelect.vue` (existing) | Already built, tested, searchable |
| Warning display | New alert component | `WarningCard.vue` (existing) | Consistent amber styling |
| Config persistence | Custom file I/O | `saveConfig()` + `useConfig.ts` | Already handles merge, error, loading state |
| Dark mode | Manual color toggling | Tailwind `dark:` variants | Already wired project-wide |
| Audio device list | Custom device enumeration | `list_audio_input_devices` Rust command (existing) | Already implemented and polled |
| Model list fetching | API call to provider | `get_provider_models` Rust command (hardcoded) | CLAUDE.md: hardcoded model lists, no dynamic API calls |

**Key insight:** The majority of Phase 8 is assembly work — wiring existing primitives into a structured layout. The net-new code is: `HotkeyCapture.vue`, `test_connection` command, `get_provider_models` command, config migration logic, and the section decomposition of `App.vue`.

---

## Common Pitfalls

### Pitfall 1: Config Migration Breaks Existing Tests

**What goes wrong:** `openai.rs` and `groq.rs` tests construct `TranscriptionConfig { openai_api_key: "...", groq_api_key: "...", openai_model: "...", ... }`. After migration, these fields don't exist — compile errors.

**Why it happens:** The migration is a struct-level breaking change. Tests that use struct literal syntax hit every removed field.

**How to avoid:** Update all `TranscriptionConfig` construction sites in tests immediately after changing the struct. Run `cargo test` after migration, before any UI work.

**Warning signs:** `cargo check` errors on `openai_api_key` field access.

### Pitfall 2: On-Disk Config Loses API Keys After Migration

**What goes wrong:** User had keys in `transcription.openai_api_key`. After migration, the new struct reads `transcription.providers.openai.api_key`. Serde sees the old key as unknown (preserved in `config_raw`) but never reads it. Keys appear empty.

**Why it happens:** Serde doesn't auto-migrate renamed/moved fields across struct shapes.

**How to avoid:** In `AppState::load()` (or `config/persistence.rs`), after loading raw JSON, check if `transcription.openai_api_key` exists and migrate it to `transcription.providers.openai.api_key` before deserializing. Remove old keys from raw JSON after promotion.

**Warning signs:** User opens settings post-migration, sees empty API key fields despite having set them before.

### Pitfall 3: Hotkey Capture Widget Intercepts Tab/Escape Navigation

**What goes wrong:** User tabs through the settings form; the hotkey capture widget, when focused, intercepts `Tab` as a hotkey combo ("Tab" key), prevents form navigation.

**Why it happens:** `preventDefault()` on all keydown events inside the widget.

**How to avoid:** Treat `Tab` and `Escape` as focus management keys, not combo keys. On `Escape`, blur the widget and cancel any partial capture. On `Tab`, let the event propagate normally.

**Warning signs:** User reports being unable to tab past the hotkey field.

### Pitfall 4: Test Connection Command Blocks Tokio on Slow Network

**What goes wrong:** `test_connection` fires an HTTP request. If the network is slow or hangs, it blocks the command thread.

**Why it happens:** Synchronous HTTP call on an async Tauri command.

**How to avoid:** Use `reqwest` async (already a project dependency). Add a short timeout (5 seconds) to the request.

**Warning signs:** Settings UI freezes when "Test connection" is clicked with no network.

### Pitfall 5: Silence Duration Input — Float Precision Mismatch

**What goes wrong:** User types "1.5", saved as `1500` ms, reloaded as `1500 / 1000 = 1.5` — fine. But `1.55` → `1550` → `1.55` — also fine. Edge case: `1.505` → `1505` → `1.505` still fine. The pitfall is display formatting: `1500 / 1000` in JavaScript produces `1.5` but `1000 / 1000` produces `1` (not "1.0"). The UI should always show one decimal: `(vad_silence_ms / 1000).toFixed(1)`.

**How to avoid:** Use `.toFixed(1)` for display rendering. Use `parseFloat` + clamp + round-to-nearest-10ms for input saving.

### Pitfall 6: "Set as active" vs Active Provider Display Confusion

**What goes wrong:** The Transcription section has two concepts: "which tab is open" (UI state) and "which provider is active" (config). A user opens the Groq tab but doesn't click "Set as active" — the active provider remains OpenAI. If the tab open state is confused with active provider, saving the tab change as the active provider is wrong.

**Why it happens:** Conflating UI navigation state with config state.

**How to avoid:** Keep `activeTab: ref<'openai' | 'groq'>` as local UI state (not saved). Keep `config.transcription.provider` as the active provider (saved only on "Set as active" button click). Show a visual badge/border only on the tab whose provider matches `config.transcription.provider`.

---

## Code Examples

Verified patterns from existing codebase:

### Saving a Nested Config Section (Correct Pattern)
```typescript
// Source: src/windows/settings/App.vue (existing)
async function saveAudioDevice(): Promise<void> {
  if (!config.value) return;
  const saved = await saveConfig({
    audio: {
      ...config.value.audio,          // spread existing fields first
      device: nextDevice,             // then override changed field
    },
  });
}
```

### Listening to Backend Events (Established Pattern)
```typescript
// Source: src/composables/useConfig.ts (existing)
unlistenHotkeyWarning = await listen<HotkeyWarningPayload>(
  "hotkey-warning",
  (event) => {
    hotkeyWarning.value = event.payload;
  },
);
```

### Rust Command Returning Typed Struct to Frontend
```rust
// Source: src-tauri/src/commands/config.rs (existing)
#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}
```

### Avoiding MutexGuard Across Await (Critical Pattern)
```rust
// Source: src-tauri/src/transcription/service.rs (existing)
// Clone config BEFORE first .await — never hold MutexGuard across await
let config = {
    let state = app.state::<AppState>();
    let guard = state.config.lock().unwrap();
    guard.transcription.clone()     // clone here, guard drops at end of block
};
// Now use config freely across awaits
provider_impl.transcribe(audio, &config).await
```

### Serde Default for Nested Struct
```rust
// Source: src-tauri/src/config/mod.rs (existing pattern to extend)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioConfig {
    #[serde(default)]
    pub device: Option<String>,
    #[serde(default = "default_vad_threshold")]
    pub vad_threshold: f32,
    #[serde(default = "default_vad_silence_ms")]
    pub vad_silence_ms: u32,
}
```

### Conditional Field Display (Vue)
```html
<!-- Speed selector only when Keystrokes selected — pattern for conditional settings UI -->
<div v-if="config.injection.mode === 'keystroke'">
  <!-- speed selector -->
</div>
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Flat `TranscriptionConfig` fields | Nested `providers` map | Phase 8 Plan 1 | All consumers must update |
| OpenRouter provider support | Dropped entirely | Pre-Phase-8 | Remove enum variant, all match arms |
| Prototype `App.vue` debug view | Full section-based settings window | Phase 8 | Window width ~480px, full feature coverage |
| Text input for hotkey | Press-to-capture keyboard widget | Phase 8 | Better UX, auto-saves on key release |

**Deprecated/outdated:**
- `openrouter_api_key`, `openrouter_model` config fields: remove entirely
- `"openrouter"` variant in `TranscriptionProvider`: remove from both Rust enum and TypeScript union
- Prototype hotkey "Apply" button pattern: replace with press-to-capture auto-save

---

## Open Questions

1. **Sentinel value vs bool field for Auto-stop on silence toggle**
   - What we know: `vad_silence_ms = 0` could mean "disabled" (backend would never auto-stop); no `vad_enabled` field exists in `AudioConfig`
   - What's unclear: Does the backend audio capture code have any guard against `vad_silence_ms = 0` causing unexpected behavior?
   - Recommendation: Planner should examine `src-tauri/src/audio/` VAD logic before committing to sentinel-0 approach. If 0 is already handled as "no VAD", use it. Otherwise add `vad_enabled: bool` to `AudioConfig`.

2. **Test connection implementation — what HTTP response signals "key valid"**
   - What we know: OpenAI and Groq return 401 for invalid key, 400 for malformed request with valid key
   - What's unclear: Whether sending an empty audio body triggers 400 vs some other status
   - Recommendation: Implement as "if HTTP response is not 401/network-error, treat as connected". The exact status code does not matter for the user-facing result — only "key rejected" vs "key accepted" matters.

3. **Launch at login — Windows registry/startup folder implementation**
   - What we know: `launch_at_login: bool` is in `AppConfig`; no implementation exists yet
   - What's unclear: Whether to use Windows registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` or startup folder shortcut
   - Recommendation: Use the `tauri-plugin-autostart` plugin (already a common Tauri v2 community plugin) or implement via the `windows` crate registry write. Planner should verify if `tauri-plugin-autostart` is already in `Cargo.toml` before adding it.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (built-in) |
| Config file | `src-tauri/Cargo.toml` — no separate test config |
| Quick run command | `cd src-tauri && cargo test` |
| Full suite command | `cd src-tauri && cargo test && npx vue-tsc --noEmit` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SETT-01 | Settings window renders four sections | manual | `cargo tauri dev` visual check | ❌ Wave 0 |
| SETT-02 | General section controls persist | unit (Rust config round-trip) | `cd src-tauri && cargo test config` | ✅ (partial — config serde tests exist) |
| SETT-03 | Audio VAD controls map correctly | unit | `cd src-tauri && cargo test audio` | ❌ Wave 0 |
| SETT-04 | Transcription providers load models, test connection works | unit | `cd src-tauri && cargo test transcription` | ✅ (partial) |
| SETT-05 | Local section renders as stub | manual | visual check | ❌ Wave 0 |
| SETT-06 | Injection section mode/speed/fallback persist | unit | `cd src-tauri && cargo test config` | ✅ (injection_config_serde_round_trip exists) |
| SETT-07 | Config round-trip after migration | unit | `cd src-tauri && cargo test config` | ❌ Wave 0 (new struct shape) |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test`
- **Per wave merge:** `cd src-tauri && cargo test && npx vue-tsc --noEmit && npm run lint`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/config/mod.rs` — migration round-trip tests: old flat JSON → new nested struct, new nested JSON → struct, unknown fields preserved
- [ ] `src-tauri/src/commands/config.rs` — `get_provider_models` unit test: assert correct model lists per provider
- [ ] TypeScript typecheck must pass after `types/index.ts` changes: `npx vue-tsc --noEmit`

---

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection: `src-tauri/src/config/mod.rs` — current `TranscriptionConfig` flat shape
- Direct codebase inspection: `src/types/index.ts` — current TypeScript types including `openrouter` fields to remove
- Direct codebase inspection: `src/composables/useConfig.ts` — `saveConfig(partial)` pattern, event listeners
- Direct codebase inspection: `src/windows/settings/App.vue` — current prototype layout and all reusable patterns
- Direct codebase inspection: `src-tauri/src/transcription/service.rs` — flat field access sites needing migration
- Direct codebase inspection: `src-tauri/src/transcription/openai.rs` + `groq.rs` — `config.openai_api_key`, `config.groq_api_key`, `config.openai_model`, `config.groq_model` references
- Direct codebase inspection: `src-tauri/src/lib.rs` — command registration; `get_provider_models` and `test_connection` must be registered here
- CLAUDE.md project guidelines — no Pinia, no heavy UI libs, hardcoded model lists, `dark:` Tailwind variants throughout

### Secondary (MEDIUM confidence)
- CONTEXT.md decisions — locked by user in `/gsd:discuss-phase` session 2026-03-22
- STATE.md accumulated decisions — Pre-Phase-08 decision logged: nested providers structure adopted

### Tertiary (LOW confidence)
- `tauri-plugin-autostart` availability for launch-at-login: not verified against current Cargo.lock; requires planner validation

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries are already in use in the project
- Architecture: HIGH — patterns are established; migration shape is clearly defined
- Pitfalls: HIGH — derived from direct codebase analysis of affected files
- Launch-at-login implementation: LOW — existence of autostart plugin not verified

**Research date:** 2026-03-22
**Valid until:** 2026-04-22 (stable stack; Tauri and Vue versions are not changing this phase)
