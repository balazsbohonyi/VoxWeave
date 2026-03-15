<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import type {
  AppConfig,
  AudioLevelPayload,
  IndicatorStatePayload,
  IndicatorVisualState,
  InjectionMode,
  RecordingState,
} from "../../types";

const state = ref<IndicatorVisualState>("hidden");
const level = ref(0);
const phase = ref(0);
const WAVE_BAR_COUNT = 15;
const noise = ref<number[]>(Array.from({ length: WAVE_BAR_COUNT }, () => Math.random()));
const injectionMode = ref<InjectionMode>("flash_paste");
const win = getCurrentWindow();

let unlistenState: UnlistenFn | null = null;
let unlistenHidden: UnlistenFn | null = null;
let unlistenAudioLevel: UnlistenFn | null = null;
let rafId = 0;

const isRecording = computed(() => state.value === "recording");
const isProcessing = computed(() => state.value === "processing");
const isInjecting = computed(() => state.value === "injecting");
const WAVE_SEGMENTS = 7;
const RECORDING_BASELINE = 0.08;
const RECORDING_STALE_MS = 260;
const RECORDING_SMOOTHING = 0.42;
const lastAudioLevelAt = ref(0);

const statusLabel = computed(() => {
  if (isRecording.value) return "Recording";
  if (isProcessing.value) return "Transcribing";
  if (isInjecting.value) return "Injecting";
  return "Ready";
});

const methodHint = computed(() => {
  if (!isInjecting.value) return "";
  if (injectionMode.value === "flash_paste") return "FlashPaste";
  if (injectionMode.value === "keystroke") return "Keystroke";
  return "Clipboard";
});

const animatedLevel = computed(() => {
  if (isRecording.value) {
    const now = performance.now();
    const hasFreshLevel = now - lastAudioLevelAt.value <= RECORDING_STALE_MS;
    if (!hasFreshLevel) {
      return RECORDING_BASELINE;
    }
    return RECORDING_BASELINE + level.value * 0.92;
  }
  if (isProcessing.value) {
    return 0.18 + (Math.sin(phase.value * 0.12) + 1) * 0.05;
  }
  if (isInjecting.value) {
    return 0.1;
  }
  return 0.06;
});

const bars = computed(() => {
  const clamped = Math.max(0, Math.min(1, animatedLevel.value));
  return Array.from({ length: WAVE_BAR_COUNT }, (_, index) => {
    const t = (index + 1) / WAVE_BAR_COUNT;
    const centerProfile = 1 - Math.abs(t - 0.5);
    const wobble = 0.04 * Math.sin((phase.value + index * 7) * 0.14);
    const jitter = (noise.value[index] - 0.5) * 0.58;
    const spike = noise.value[index] > 0.86 ? 0.22 : 0;
    const base = 0.1;
    const value = Math.max(
      0.08,
      (base + clamped * (0.45 + centerProfile * 0.55 + wobble + jitter + spike)) * 1.4,
    );
    const activeSegments = Math.max(1, Math.min(WAVE_SEGMENTS, Math.round(value * WAVE_SEGMENTS)));
    return activeSegments;
  });
});

async function loadInjectionMode(): Promise<void> {
  try {
    const config = await invoke<AppConfig>("get_config");
    injectionMode.value = config.injection.mode;
  } catch {
    injectionMode.value = "flash_paste";
  }
}

function tick(): void {
  phase.value += 1;
  noise.value = noise.value.map((prev) => prev * 0.62 + Math.random() * 0.38);
  rafId = requestAnimationFrame(tick);
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
  if (!dragCompleted) {
    return;
  }
  const pos = await win.outerPosition();
  await invoke("persist_indicator_position", { x: Math.round(pos.x), y: Math.round(pos.y) });
}

async function onRecordButtonClick(): Promise<void> {
  await invoke("toggle_recording_from_indicator");
}

onMounted(async () => {
  document.documentElement.style.overflow = "hidden";
  document.body.style.margin = "0";
  document.body.style.overflow = "hidden";
  document.body.style.background = "transparent";

  await loadInjectionMode();
  rafId = requestAnimationFrame(tick);

  unlistenState = await listen<IndicatorStatePayload>("indicator-state", (event) => {
    state.value = event.payload.state;
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
  cancelAnimationFrame(rafId);
  if (unlistenState) unlistenState();
  if (unlistenHidden) unlistenHidden();
  if (unlistenAudioLevel) unlistenAudioLevel();
});
</script>

<template>
  <main class="indicator-root" @pointerdown="onPointerDown">
    <section class="indicator-pill" :data-state="state">
      <div class="indicator-left">
        <button class="indicator-record-button" type="button" @click.stop="onRecordButtonClick">
          <span class="indicator-record-dot" :class="{ 'indicator-record-dot-active': isRecording }" />
        </button>
        <span class="indicator-label">{{ statusLabel }}</span>
        <span v-if="methodHint" class="indicator-method">{{ methodHint }}</span>
      </div>
      <div class="indicator-waveform">
        <div
          v-for="(activeSegments, index) in bars"
          :key="index"
          class="indicator-waveform-column"
        >
          <span
            v-for="segment in WAVE_SEGMENTS"
            :key="segment"
            class="indicator-waveform-segment"
            :class="{ 'indicator-waveform-segment-active': segment <= activeSegments }"
          />
        </div>
      </div>
    </section>
  </main>
</template>
