---
phase: 04-floating-indicator
plan: "04-02"
type: execute
wave: 2
depends_on:
  - 04-01-indicator-window-runtime-PLAN.md
files_modified:
  - index.html
  - src/windows/indicator/main.ts
  - src/windows/indicator/App.vue
  - src/windows/indicator/components/Waveform.vue
  - src/windows/indicator/components/StateBadge.vue
  - src/styles.css
  - src/types/index.ts
autonomous: true
requirements:
  - FLOT-03
  - FLOT-04
must_haves:
  truths:
    - "Indicator frontend has an isolated entrypoint and mount target."
    - "Waveform renders 10 bars with smoothed updates at recording cadence."
    - "Recording/processing/injecting states are visually distinct with brief transitions."
  artifacts:
    - path: "src/windows/indicator/main.ts"
      provides: "Dedicated indicator app bootstrap"
    - path: "src/windows/indicator/App.vue"
      provides: "State event subscription and visual state orchestration"
    - path: "src/windows/indicator/components/Waveform.vue"
      provides: "10-bar waveform renderer with smoothing"
    - path: "src/windows/indicator/components/StateBadge.vue"
      provides: "Processing/injecting state cues"
    - path: "src/types/index.ts"
      provides: "Frontend indicator event payload typing"
  key_links:
    - from: "src/windows/indicator/App.vue"
      to: "src/windows/indicator/components/Waveform.vue"
      via: "audio-level payload mapped to waveform props/state"
      pattern: "(audio-level|waveform|levels?)"
    - from: "src/windows/indicator/App.vue"
      to: "src/windows/indicator/components/StateBadge.vue"
      via: "indicator-state event drives state badge rendering"
      pattern: "(indicator-state|recording|processing|injecting)"
---

<objective>
Implement indicator frontend UI with live waveform and distinct recording/processing/injecting visuals consuming backend indicator events.

Purpose: Deliver FLOT-03/FLOT-04 visible behavior on top of runtime foundation.
Output: Indicator Vue entrypoint + waveform + state visuals with event mapping.
</objective>

<execution_context>
@C:/Users/Balazs/.codex/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@index.html
@src/windows/indicator/main.ts
@src/windows/indicator/App.vue
@src/windows/indicator/components/Waveform.vue
@src/windows/indicator/components/StateBadge.vue
@src/styles.css
@src/types/index.ts
</context>

<tasks>
<task type="auto">
  <name>Task 1: Add indicator frontend entrypoint and boot wiring</name>
  <files>index.html, src/windows/indicator/main.ts, src/windows/indicator/App.vue</files>
  <action>Create indicator app bootstrap and mount target for the `indicator` window route, ensuring settings and indicator windows can load separate roots cleanly in Tauri runtime. Keep startup minimal and avoid introducing global store dependencies.</action>
  <verify>
    <automated>cmd /c npx vue-tsc --noEmit</automated>
  </verify>
  <done>Indicator window can render its own Vue app independently from settings.</done>
</task>

<task type="auto">
  <name>Task 2: Build waveform component with smoothing and baseline pulse</name>
  <files>src/windows/indicator/components/Waveform.vue, src/windows/indicator/App.vue, src/types/index.ts</files>
  <action>Implement a reusable 10-bar waveform component that maps normalized audio-level events into smoothed animated bar heights at animation-frame cadence, retaining low baseline motion during silence and freezing/fading during processing transition.</action>
  <verify>
    <automated>cd src-tauri && cargo test indicator::tests::audio_level_throttle_target_fps -- --exact</automated>
  </verify>
  <done>FLOT-03 waveform behavior is available and testable end-to-end.</done>
</task>

<task type="auto">
  <name>Task 3: Implement state visuals and transition choreography</name>
  <files>src/windows/indicator/App.vue, src/windows/indicator/components/StateBadge.vue, src/styles.css</files>
  <action>Render recording/processing/injecting states with clear iconography and labels, apply quick crossfade transitions (120-180ms), and bind state changes to backend `indicator-state` events. Keep visuals lightweight and readable at ~260x48 size.</action>
  <verify>
    <automated>cd src-tauri && cargo test indicator::tests::state_event_sequence -- --exact</automated>
  </verify>
  <done>FLOT-04 distinct visual-state contract is implemented with deterministic event mapping.</done>
</task>
</tasks>

<verification>
<criteria>
- Waveform remains responsive and visually stable with 10 bars during recording.
- Processing and injecting states are clearly distinct from recording state.
- State transitions are smooth and brief rather than abrupt.
</criteria>

<commands>
- `npx vue-tsc --noEmit`
- `cd src-tauri && cargo test indicator -- --nocapture`
- Manual: `cargo tauri dev`, run record -> processing -> injecting simulation and validate visual transitions
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
