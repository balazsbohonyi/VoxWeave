<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import type {
  AudioLevelPayload,
  IndicatorStatePayload,
  IndicatorVisualState,
  RecordingState,
} from "../../types";

const state = ref<IndicatorVisualState>("hidden");
const level = ref(0);
const win = getCurrentWindow();

let unlistenState: UnlistenFn | null = null;
let unlistenHidden: UnlistenFn | null = null;
let unlistenAudioLevel: UnlistenFn | null = null;
let unlistenVadSilenceStop: UnlistenFn | null = null;
let unlistenNearLimit: UnlistenFn | null = null;
let unlistenLimitStop: UnlistenFn | null = null;
let unlistenMoved: UnlistenFn | null = null;
let persistTimer: ReturnType<typeof setTimeout> | null = null;

const isRecording = computed(() => state.value === "recording");
const isProcessing = computed(() => state.value === "processing");
const RECORDING_STALE_MS = 260;
const RECORDING_SMOOTHING = 0.75;
const RECORDING_NOISE_GATE = 0.001;
const RECORDING_GAIN = 30;
const lastAudioLevelAt = ref(0);

const animatedLevel = computed(() => {
  if (!isRecording.value) {
    return 0;
  }
  const now = performance.now();
  const hasFreshLevel = now - lastAudioLevelAt.value <= RECORDING_STALE_MS;
  if (!hasFreshLevel) {
    return 0;
  }
  const gated = level.value < RECORDING_NOISE_GATE ? 0 : level.value;
  return Math.min(1, Math.sqrt(gated * RECORDING_GAIN));
});

// Canvas waveform
const waveCanvas = ref<HTMLCanvasElement | null>(null);
let rafId: number | null = null;

const BAR_COUNT = 20;
const BAR_WIDTH = 3;
const BAR_GAP = 2;
const MIN_HEIGHT = 2;
const MAX_HEIGHT = 18;
const LERP_SPEED = 0.18;

const barHeights = new Float32Array(BAR_COUNT).fill(MIN_HEIGHT);
const idleJitter = Float32Array.from({ length: BAR_COUNT }, () => (Math.random() - 0.5) * 0.4);
let t1 = 0;
let t2 = 0;

function drawFrame() {
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
  const lvl = animatedLevel.value;

  for (let i = 0; i < BAR_COUNT; i++) {
    let target: number;

    if (recording) {
      const sim = Math.max(0, Math.abs(
        Math.sin(t1 * 1.3 + i * 0.55) * 0.5 + Math.sin(t2 * 2.1 + i * 0.38) * 0.3,
      ) + idleJitter[i]);
      const scale = 0.3 + Math.pow(lvl, 0.3) * 0.7;
      target = MIN_HEIGHT + sim * MAX_HEIGHT * scale;
    } else if (processing) {
      const breath = (Math.sin(t1 * 0.9) + 1) / 2;
      const phase = (i / (BAR_COUNT - 1)) * Math.PI * 2;
      const sineVal = Math.max(0, (Math.sin(phase) + 1) / 2 + idleJitter[i]);
      target = MIN_HEIGHT + sineVal * MAX_HEIGHT * 0.28 * (0.4 + breath * 0.6);
    } else {
      const phase = (i / (BAR_COUNT - 1)) * Math.PI * 2;
      const sineVal = Math.max(0, (Math.sin(phase) + 1) / 2 + idleJitter[i]);
      target = MIN_HEIGHT + sineVal * MAX_HEIGHT * 0.28;
    }

    barHeights[i] += (target - barHeights[i]) * LERP_SPEED;
  }

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = (recording || processing) ? "rgba(255,255,255,0.9)" : "rgba(255,255,255,0.25)";

  const totalWidth = BAR_COUNT * (BAR_WIDTH + BAR_GAP) - BAR_GAP;
  const xOffset = (canvas.width - totalWidth) / 2;
  const centerY = canvas.height / 2;
  const radius = BAR_WIDTH / 2;

  for (let i = 0; i < BAR_COUNT; i++) {
    const h = barHeights[i];
    const x = xOffset + i * (BAR_WIDTH + BAR_GAP);

    ctx.beginPath();
    ctx.roundRect(x, centerY - h, BAR_WIDTH, h * 2, radius);
    ctx.fill();
  }

  rafId = requestAnimationFrame(drawFrame);
}

async function onPointerDown(event: PointerEvent): Promise<void> {
  const target = event.target as HTMLElement | null;
  if (target?.closest(".indicator-record-button")) {
    return;
  }
  event.preventDefault();
  await invoke("begin_indicator_drag");
  let dragCompleted = false;
  try {
    await win.startDragging();
    dragCompleted = true;
  } finally {
    await invoke("end_indicator_drag");
  }
  if (!dragCompleted) return;
}

function queuePersistPosition(): void {
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(async () => {
    try {
      const pos = await win.outerPosition();
      const scale = await win.scaleFactor();
      const logicalX = Math.round(pos.x / scale);
      const logicalY = Math.round(pos.y / scale);
      await invoke("persist_indicator_position", { x: logicalX, y: logicalY });
    } catch {
      // Ignore persistence failures while dragging/moving.
    }
  }, 180);
}

onMounted(async () => {
  document.documentElement.style.overflow = "hidden";
  document.documentElement.style.height = "100%";
  document.body.style.margin = "0";
  document.body.style.overflow = "hidden";
  document.body.style.height = "100%";
  document.body.style.background = "transparent";
  const appEl = document.getElementById("app");
  if (appEl) {
    appEl.style.height = "100%";
    appEl.style.display = "flex";
    appEl.style.flexDirection = "column";
  }

  // Set canvas physical size — derived from bar constants so all bars always fit
  const canvas = waveCanvas.value;
  if (canvas) {
    canvas.width = BAR_COUNT * (BAR_WIDTH + BAR_GAP) - BAR_GAP;
    canvas.height = 22;
  }

  // Start animation loop
  rafId = requestAnimationFrame(drawFrame);

  unlistenMoved = await win.onMoved(() => {
    queuePersistPosition();
  });

  unlistenState = await listen<IndicatorStatePayload>("indicator-state", (event) => {
    state.value = event.payload.state;
    if (event.payload.state !== "recording") {
      level.value = 0;
      lastAudioLevelAt.value = 0;
    }
  });

  unlistenHidden = await listen("indicator-hidden", () => {
    level.value = 0;
    lastAudioLevelAt.value = 0;
    state.value = "hidden";
  });

  unlistenAudioLevel = await listen<AudioLevelPayload>("audio-level", (event) => {
    const incoming = Math.max(0, Math.min(1, event.payload.rms));
    level.value = level.value * (1 - RECORDING_SMOOTHING) + incoming * RECORDING_SMOOTHING;
    lastAudioLevelAt.value = performance.now();
  });

  unlistenVadSilenceStop = await listen("vad-silence-stop", () => {
    void invoke("trigger_stop_recording");
  });

  unlistenNearLimit = await listen("recording-near-limit", () => {
    void invoke("show_plain_toast", { toastType: "warning", message: "Recording will stop in 30 seconds.", keepIndicator: true });
  });

  unlistenLimitStop = await listen("recording-limit-stop", () => {
    void invoke("trigger_stop_recording");
    void invoke("show_plain_toast", { toastType: "info", message: "5-minute recording limit reached — transcribing and injecting.", keepIndicator: true });
  });

  const syncState = async () => {
    try {
      const snapshot = await invoke<IndicatorStatePayload>("get_indicator_state");
      const recording = await invoke<RecordingState>("get_recording_state");
      if (recording === "recording") {
        state.value = "recording";
      } else if (recording === "transcribing") {
        if (snapshot.state === "injecting") {
          state.value = "injecting";
        } else {
          state.value = "processing";
        }
      } else {
        state.value = snapshot.state;
      }
    } catch {
      // Keep current state when command is unavailable.
    }
  };
  await syncState();
});

onBeforeUnmount(() => {
  if (rafId !== null) cancelAnimationFrame(rafId);
  if (unlistenState) unlistenState();
  if (unlistenHidden) unlistenHidden();
  if (unlistenAudioLevel) unlistenAudioLevel();
  if (unlistenVadSilenceStop) unlistenVadSilenceStop();
  if (unlistenNearLimit) unlistenNearLimit();
  if (unlistenLimitStop) unlistenLimitStop();
  if (unlistenMoved) unlistenMoved();
  if (persistTimer) clearTimeout(persistTimer);
});
</script>

<template>
  <main
    class="indicator-root"
    @pointerdown="onPointerDown"
  >
    <section class="indicator-pill" :data-state="state">
      <div class="indicator-left">
        <span class="indicator-record-button">
          <span class="indicator-record-dot" :class="{ 'indicator-record-dot-active': isRecording }" />
        </span>
      </div>
      <div class="indicator-waveform">
        <canvas ref="waveCanvas" class="indicator-waveform-canvas" />
      </div>
    </section>
  </main>
</template>
