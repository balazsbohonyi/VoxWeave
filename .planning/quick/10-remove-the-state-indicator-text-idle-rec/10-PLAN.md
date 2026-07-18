---
phase: quick-10
plan: 10
type: execute
wave: 1
depends_on: []
files_modified:
  - src/windows/indicator/App.vue
  - src/windows/indicator/components/StateBadge.vue
  - src/styles.css
  - src-tauri/tauri.conf.json
  - src-tauri/src/indicator/window.rs
autonomous: true
requirements: [FLOT-01, FLOT-03, FLOT-04]
must_haves:
  truths:
    - "The floating indicator shows no IDLE, REC, INJ, processing, or injection-method text."
    - "The indicator is a compact 150x38 fully pill-shaped dark control with a solid 2px white border."
    - "The 20-bar waveform remains wholly visible inside the narrower indicator in every visual state."
    - "Indicator placement and toast positioning continue to use the rendered indicator dimensions."
  artifacts:
    - path: "src/windows/indicator/App.vue"
      provides: "Text-free indicator composition with record dot and waveform"
    - path: "src/styles.css"
      provides: "Pill geometry, white border, and waveform-safe horizontal layout"
    - path: "src-tauri/tauri.conf.json"
      provides: "150x38 indicator window constraints"
    - path: "src-tauri/src/indicator/window.rs"
      provides: "150x38 placement constants matching the Tauri window"
  key_links:
    - from: "src-tauri/tauri.conf.json"
      to: "src-tauri/src/indicator/window.rs"
      via: "matching 150x38 indicator dimensions"
      pattern: "INDICATOR_WIDTH: i32 = 150|\"width\": 150"
    - from: "src/windows/indicator/App.vue"
      to: "src/styles.css"
      via: "indicator-pill and indicator-waveform class layout"
      pattern: "indicator-waveform"
---

<objective>
Make the recording indicator quieter and more compact while retaining the live waveform.

Purpose: Remove redundant state copy, make the overlay visually pill-shaped, and prevent the waveform from being squeezed out by the narrower window.
Output: A text-free 150x38 indicator with matched frontend/backend geometry.
</objective>

<execution_context>
@C:/Users/Balazs/.codex/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@AGENTS.md
@.planning/PROJECT.md
@.planning/STATE.md

<interfaces>
From src/windows/indicator/App.vue:
```typescript
const state = ref<IndicatorVisualState>("hidden");
const isRecording = computed(() => state.value === "recording");
const isProcessing = computed(() => state.value === "processing");
```

From src-tauri/src/indicator/window.rs:
```rust
pub const INDICATOR_WIDTH: i32 = 180;
pub const INDICATOR_HEIGHT: i32 = 38;
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Remove indicator state-copy rendering and unused badge plumbing</name>
  <files>src/windows/indicator/App.vue, src/windows/indicator/components/StateBadge.vue</files>
  <action>
Remove the `StateBadge` import and template node from the indicator. Delete `StateBadge.vue`; it exists only to render `REC`, `...`, `INJ`, `IDLE`, and the injection-method hint, all of which must disappear.

Then remove now-unused `InjectionMode`, `AppConfig`, `injectionMode`, and `loadInjectionMode()` code from `App.vue`, including its on-mounted call. Keep the state event listeners, `isRecording`, `isProcessing`, record dot, canvas sizing, animation loop, drag behavior, and waveform rendering intact: state must still drive visual behavior without displaying literal state text.
  </action>
  <verify>
    <automated>pnpm build</automated>
  </verify>
  <done>`StateBadge.vue` is removed; the indicator template contains only the record-dot control and waveform; Vue typechecking/build passes with no unused imports or state-text output.</done>
</task>

<task type="auto">
  <name>Task 2: Match compact pill styling and window-placement dimensions</name>
  <files>src/styles.css, src-tauri/tauri.conf.json, src-tauri/src/indicator/window.rs</files>
  <action>
Set the indicator window's `width`, `minWidth`, and `maxWidth` in `src-tauri/tauri.conf.json` to `150` while retaining its 38px fixed height. Change `INDICATOR_WIDTH` in `window.rs` to `150` so saved-position clamping, bottom-right fallback, and toast placement use the same rendered dimensions.

In `src/styles.css`, restyle `.indicator-pill` as a true pill: use `border-radius: 50%`, a solid `2px white` border, and remove state-specific colored border overrides so every state retains that requested border. Replace all-sides compact padding with explicit horizontal padding (14px) and no vertical padding; retain `box-sizing: border-box` and `overflow: hidden`. Remove obsolete label/badge rules. Collapse `.indicator-left` to only reserve the record dot (no 56px minimum). Keep the waveform flexing with its 98px canvas footprint; adjust its max width or flex sizing only as needed so all 20 bars remain visible within the 150px outer width and added horizontal padding.

Do not change toast dimensions or unrelated toast styling.
  </action>
  <verify>
    <automated>pnpm build; cd src-tauri; cargo test indicator:: --lib</automated>
  </verify>
  <done>The Tauri configuration and Rust placement constant both use 150x38; the pill has a 2px white 50% radius border and horizontal padding; no waveform bars are clipped by the compact layout; targeted Rust indicator tests and frontend build pass.</done>
</task>

</tasks>

<verification>
- `pnpm build` passes after component deletion and CSS/layout changes.
- `cd src-tauri; cargo test indicator:: --lib` passes with the synchronized indicator width.
- In `cargo tauri dev`, trigger recording, processing, and injecting: no state/method text appears, the dot and waveform remain visible, and the pill is 150x38 with a white 2px outline.
</verification>

<success_criteria>
The floating indicator is a text-free 150x38 pill with a solid 2px white border and 50% radius. Its waveform remains visible and unclipped, and Rust positioning logic agrees with the configured window dimensions.
</success_criteria>

<output>
After completion, create `.planning/quick/10-remove-the-state-indicator-text-idle-rec/10-SUMMARY.md`
</output>
