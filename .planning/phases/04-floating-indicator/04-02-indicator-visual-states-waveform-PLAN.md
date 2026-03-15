---
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
---

<plan>
<goal>
Implement the indicator frontend UI with live waveform and distinct recording/processing/injecting visuals that consume backend indicator events.
</goal>

<must_haves>
- Indicator frontend has its own entrypoint and mount path, not the settings entry.
- Live waveform renders 10 bars and updates at >=24fps effective cadence with smoothing.
- Visual states are distinct and readable: recording (pulse+waveform), processing (spinner), injecting (method cue+label).
- State transitions use short crossfade timing aligned with context constraints.
</must_haves>

<tasks>
<task type="auto">
  <name>Task 1: Add indicator frontend entrypoint and boot wiring</name>
  <files>index.html, src/windows/indicator/main.ts, src/windows/indicator/App.vue</files>
  <action>Create indicator app bootstrap and mount target for the `indicator` window route, ensuring settings and indicator windows can load separate roots cleanly in Tauri runtime. Keep startup minimal and avoid introducing global store dependencies.</action>
  <verify>`npx vue-tsc --noEmit` passes</verify>
  <done>Indicator window can render its own Vue app independently from settings.</done>
</task>

<task type="auto">
  <name>Task 2: Build waveform component with smoothing and baseline pulse</name>
  <files>src/windows/indicator/components/Waveform.vue, src/windows/indicator/App.vue, src/types/index.ts</files>
  <action>Implement a reusable 10-bar waveform component that maps normalized audio-level events into smoothed animated bar heights at animation-frame cadence, retaining low baseline motion during silence and freezing/fading during processing transition.</action>
  <verify>`cd src-tauri && cargo test indicator::tests::audio_level_throttle_target_fps -- --exact` and `npx vue-tsc --noEmit` pass</verify>
  <done>FLOT-03 waveform behavior is available and testable end-to-end.</done>
</task>

<task type="auto">
  <name>Task 3: Implement state visuals and transition choreography</name>
  <files>src/windows/indicator/App.vue, src/windows/indicator/components/StateBadge.vue, src/styles.css</files>
  <action>Render recording/processing/injecting states with clear iconography and labels, apply quick crossfade transitions (120-180ms), and bind state changes to backend `indicator-state` events. Keep visuals lightweight and readable at ~260x48 size.</action>
  <verify>`cd src-tauri && cargo test indicator::tests::state_event_sequence -- --exact` and manual visual validation in `cargo tauri dev` pass</verify>
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
</plan>

