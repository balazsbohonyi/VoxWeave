<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import Waveform from "./components/Waveform.vue";
import type {
  AudioLevelPayload,
  IndicatorStatePayload,
  IndicatorVisualState,
  RecordingState,
} from "../../types";

const state = ref<IndicatorVisualState>("hidden");
const level = ref(0);
const levelUpdatedAt = ref(0);
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

  unlistenMoved = await win.onMoved(() => {
    queuePersistPosition();
  });

  unlistenState = await listen<IndicatorStatePayload>("indicator-state", (event) => {
    state.value = event.payload.state;
    if (event.payload.state !== "recording") {
      level.value = 0;
    }
  });

  unlistenHidden = await listen("indicator-hidden", () => {
    level.value = 0;
    state.value = "hidden";
  });

  unlistenAudioLevel = await listen<AudioLevelPayload>("audio-level", (event) => {
    level.value = event.payload.rms;
    levelUpdatedAt.value = performance.now();
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
    @contextmenu.prevent
  >
    <section class="indicator-pill" :data-state="state">
      <div class="indicator-left">
        <span class="indicator-record-button">
          <span class="indicator-record-dot" :class="{ 'indicator-record-dot-active': isRecording }" />
        </span>
      </div>
      <div class="indicator-waveform">
        <Waveform :state="state" :level="level" :level-updated-at="levelUpdatedAt" />
      </div>
    </section>
  </main>
</template>
