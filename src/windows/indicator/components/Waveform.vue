<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { IndicatorVisualState } from "../../../types";

const props = defineProps<{
  state: IndicatorVisualState;
  level: number;
  levelUpdatedAt: number;
}>();

const RECORDING_STALE_MS = 260;
const RECORDING_SMOOTHING = 0.75;
const RECORDING_NOISE_GATE = 0.001;
const RECORDING_GAIN = 30;

const BAR_COUNT = 18;
const BAR_WIDTH = 2;
const BAR_GAP = 3;
const MIN_HEIGHT = 1;
const MAX_HEIGHT = 12;
const LERP_SPEED = 0.04;

const waveCanvas = ref<HTMLCanvasElement | null>(null);
const smoothedLevel = ref(0);
const lastAudioLevelAt = ref(0);
const barHeights = new Float32Array(BAR_COUNT).fill(MIN_HEIGHT);
const idleJitter = Float32Array.from({ length: BAR_COUNT }, () => (Math.random() - 0.5) * 0.4);

let rafId: number | null = null;
let t1 = 0;
let t2 = 0;

const isRecording = computed(() => props.state === "recording");
const isProcessing = computed(() => props.state === "processing");

const animatedLevel = computed(() => {
  if (!isRecording.value) {
    return 0;
  }
  const hasFreshLevel = performance.now() - lastAudioLevelAt.value <= RECORDING_STALE_MS;
  if (!hasFreshLevel) {
    return 0;
  }
  const gated = smoothedLevel.value < RECORDING_NOISE_GATE ? 0 : smoothedLevel.value;
  return Math.min(1, Math.sqrt(gated * RECORDING_GAIN));
});

watch(() => props.levelUpdatedAt, () => {
  const clamped = Math.max(0, Math.min(1, props.level));
  smoothedLevel.value = smoothedLevel.value * (1 - RECORDING_SMOOTHING) + clamped * RECORDING_SMOOTHING;
  lastAudioLevelAt.value = performance.now();
});

watch(isRecording, (recording) => {
  if (!recording) {
    smoothedLevel.value = 0;
    lastAudioLevelAt.value = 0;
  }
});

function drawFrame(): void {
  const canvas = waveCanvas.value;
  if (!canvas) {
    rafId = requestAnimationFrame(drawFrame);
    return;
  }

  const ctx = canvas.getContext("2d");
  if (!ctx) {
    rafId = requestAnimationFrame(drawFrame);
    return;
  }

  t1 += 0.04;
  t2 += 0.027;

  const recording = isRecording.value;
  const processing = isProcessing.value;
  const level = animatedLevel.value;

  for (let index = 0; index < BAR_COUNT; index++) {
    let target: number;

    if (recording) {
      const simulation = Math.max(0, Math.abs(
        Math.sin(t1 * 1.3 + index * 0.55) * 0.5 + Math.sin(t2 * 2.1 + index * 0.38) * 0.3,
      ) + idleJitter[index]);
      const scale = 0.3 + Math.pow(level, 0.3) * 0.7;
      target = MIN_HEIGHT + simulation * MAX_HEIGHT * scale;
    } else if (processing) {
      const breath = (Math.sin(t1 * 0.9) + 1) / 2;
      const phase = (index / (BAR_COUNT - 1)) * Math.PI * 2;
      const sineValue = Math.max(0, (Math.sin(phase) + 1) / 2 + idleJitter[index]);
      target = MIN_HEIGHT + sineValue * MAX_HEIGHT * 0.28 * (0.4 + breath * 0.6);
    } else {
      const phase = (index / (BAR_COUNT - 1)) * Math.PI * 2;
      const sineValue = Math.max(0, (Math.sin(phase) + 1) / 2 + idleJitter[index]);
      target = MIN_HEIGHT + sineValue * MAX_HEIGHT * 0.42;
    }

    barHeights[index] += (target - barHeights[index]) * LERP_SPEED;
  }

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = (recording || processing) ? "rgba(255,255,255,0.9)" : "rgba(255,255,255,0.25)";

  const totalWidth = BAR_COUNT * (BAR_WIDTH + BAR_GAP) - BAR_GAP;
  const xOffset = (canvas.width - totalWidth) / 2;
  const centerY = canvas.height / 2;
  const radius = BAR_WIDTH / 2;

  for (let index = 0; index < BAR_COUNT; index++) {
    const height = barHeights[index];
    const x = xOffset + index * (BAR_WIDTH + BAR_GAP);

    ctx.beginPath();
    ctx.roundRect(x, centerY - height, BAR_WIDTH, height * 2, radius);
    ctx.fill();
  }

  rafId = requestAnimationFrame(drawFrame);
}

onMounted(() => {
  const canvas = waveCanvas.value;
  if (canvas) {
    canvas.width = BAR_COUNT * (BAR_WIDTH + BAR_GAP) - BAR_GAP;
    canvas.height = 26;
  }
  rafId = requestAnimationFrame(drawFrame);
});

onBeforeUnmount(() => {
  if (rafId !== null) cancelAnimationFrame(rafId);
});
</script>

<template>
  <canvas ref="waveCanvas" class="indicator-waveform-canvas" aria-hidden="true" />
</template>
