---
phase: quick-4
plan: 1
type: execute
wave: 1
depends_on: []
files_modified:
  - src/windows/indicator/App.vue
  - src/styles.css
autonomous: true
requirements: []
must_haves:
  truths:
    - "Idle: 15 small pill-shaped bars at minimum height, no animation"
    - "Recording: bars animate to varying heights with organic wave motion driven by audio-level events or sine simulation"
    - "Processing: all bars gently pulse at low uniform amplitude"
    - "Bars are mirrored (grow up and down from center axis)"
    - "Bars have fully rounded ends"
    - "Smooth lerp transitions — no instant jumps between states"
    - "Returning to idle animates bars down, not a snap"
  artifacts:
    - path: src/windows/indicator/App.vue
      provides: "Inline canvas-based waveform replacing grid-of-squares"
    - path: src/styles.css
      provides: "Updated .indicator-waveform rules; old segment CSS removed"
  key_links:
    - from: "audio-level event listener"
      to: "bar height targets"
      via: "per-bar lerp in rAF loop"
---

<objective>
Replace the current grid-of-squares equalizer in the floating indicator with smooth, mirrored frequency bars on a canvas.

Purpose: Match Wispr Flow style — organic, natural waveform that feels alive during recording.
Output: Reworked waveform section inside App.vue (canvas) + updated CSS.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/windows/indicator/App.vue
@src/styles.css
</context>

<tasks>

<task type="auto">
  <name>Task 1: Replace waveform with canvas-based mirrored bars</name>
  <files>src/windows/indicator/App.vue, src/styles.css</files>
  <action>
In App.vue:

1. Remove the `bars` computed, `WAVE_SEGMENTS`, `WAVE_BAR_COUNT` constants, and the v-for grid template inside `.indicator-waveform`.

2. Add a `<canvas ref="waveCanvas" class="indicator-waveform-canvas" />` in the template where the grid was (keep it inside `.indicator-waveform` div).

3. Add a rAF animation loop (canvas-based). Key constants:
   - BAR_COUNT = 15
   - BAR_WIDTH = 3 (px)
   - BAR_GAP = 2 (px)
   - MIN_HEIGHT = 2 (px, idle pill)
   - MAX_HEIGHT = 18 (px, half-height; bar grows ±MAX_HEIGHT from center)
   - LERP_SPEED = 0.18

4. Per-bar state: `barHeights = new Float32Array(BAR_COUNT).fill(MIN_HEIGHT)`

5. Target height computation per frame:
   - **idle**: target = MIN_HEIGHT for all bars
   - **recording** (when `isRecording` and `animatedLevel.value > 0`):
     Map `animatedLevel.value` (0–1) to bar heights using the existing center-envelope approach: `centerProfile = 1 - |t - 0.5|` where `t = (i+1)/BAR_COUNT`; `target = MIN_HEIGHT + animatedLevel * (centerProfile * 0.75 + 0.25) * MAX_HEIGHT`
   - **recording** (when `animatedLevel.value === 0` — no fresh audio yet):
     Simulate using layered sine waves per bar. Use two counters advancing each frame: `t1 += 0.04`, `t2 += 0.027`. Per bar: `sim = |sin(t1 * 1.3 + i * 0.55) * 0.5 + sin(t2 * 2.1 + i * 0.38) * 0.3|`; `target = MIN_HEIGHT + sim * MAX_HEIGHT * 0.5`
   - **processing**: uniform breathing — `breath = (sin(t1 * 0.9) + 1) / 2`; `target = MIN_HEIGHT + breath * MAX_HEIGHT * 0.25`

6. Lerp: `barHeights[i] += (target - barHeights[i]) * LERP_SPEED`

7. Canvas draw: for each bar, draw two filled rounded rects — one above center, one below center (mirror). Use `canvas.getContext('2d')`. Center Y = canvas.height / 2. Bar color: `rgba(255,255,255,0.9)`. For rounded ends use `ctx.roundRect(x, y, BAR_WIDTH, h, BAR_WIDTH / 2)` then `ctx.fill()` (no stroke). Draw upper half: `roundRect(xOffset + i*(BAR_WIDTH+BAR_GAP), centerY - barHeights[i], BAR_WIDTH, barHeights[i], ...)` and lower half: `roundRect(xOffset + i*(BAR_WIDTH+BAR_GAP), centerY, BAR_WIDTH, barHeights[i], ...)`. Where `xOffset = (canvas.width - BAR_COUNT*(BAR_WIDTH+BAR_GAP) + BAR_GAP) / 2`.

8. Set canvas size in `onMounted` from the container's clientWidth/clientHeight (or hardcode 76×22 to match current CSS constraints). Use `canvas.width` / `canvas.height` (physical pixels, no DPR scaling needed for this small canvas).

9. Start/stop rAF: `onMounted` starts the loop; `onBeforeUnmount` cancels it. The existing `rafId` variable from the old Waveform.vue can be reused.

10. The `state` ref already drives `isRecording` and can drive a new `isProcessing = computed(() => state.value === 'processing')`. Use both to branch target logic.

In styles.css:
- Remove `.indicator-waveform-column`, `.indicator-waveform-segment`, `.indicator-waveform-segment-active` blocks entirely.
- Keep `.indicator-waveform` but change `align-items` to `center`, remove `--wave-segment-size`/`--wave-segment-gap` CSS vars, remove `align-items: flex-end`.
- Add `.indicator-waveform-canvas { display: block; width: 100%; height: 100%; }`.

Note: `Waveform.vue` component is unused (was never imported in current App.vue). Leave it in place — do not delete.
  </action>
  <verify>
    <automated>npx vue-tsc --noEmit && npm run lint</automated>
  </verify>
  <done>
    - TypeScript and lint both pass.
    - Canvas element renders inside indicator pill.
    - Old segment CSS classes are gone from styles.css.
    - No references to WAVE_SEGMENTS or indicator-waveform-segment remain in App.vue.
  </done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <what-built>Canvas waveform with mirrored bars, lerp animation, and state-branched simulation</what-built>
  <how-to-verify>
    1. Run `cargo tauri dev`.
    2. Press the hotkey to start recording — observe bars animate with organic wave motion, mirrored above/below center, rounded ends.
    3. Speak loudly — bars should grow; silence — bars drop toward minimum pills.
    4. Press hotkey again to stop — observe bars smoothly return to small pill height during processing, then go fully idle.
    5. Idle state: all bars are tiny pills, no motion.
  </how-to-verify>
  <resume-signal>Type "approved" or describe visual issues</resume-signal>
</task>

</tasks>

<verification>
- `npx vue-tsc --noEmit` passes (no type errors)
- `npm run lint` passes
- No old segment CSS classes referenced anywhere
</verification>

<success_criteria>
Floating indicator shows 15 mirrored, rounded frequency bars. Recording state drives organic animation. Idle state is static pills. Processing state breathes gently. No instant height snaps.
</success_criteria>

<output>
After completion, create `.planning/quick/4-waveforms-refactoring/4-SUMMARY.md`
</output>
