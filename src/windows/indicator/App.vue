<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import StateBadge from "./components/StateBadge.vue";
import { useToast } from "../../composables/useToast";
import type { ShowToastOptions } from "../../composables/useToast";
import type {
  AppConfig,
  AudioLevelPayload,
  IndicatorStatePayload,
  IndicatorVisualState,
  InjectionMode,
  RecordingState,
} from "../../types";

interface TranscriptionErrorPayload {
  code: "invalid_key" | "rate_limit" | "network" | "server" | "cancelled";
  message: string;
  provider?: string;
  fallback_provider?: string;
  retryable: boolean;
}

const state = ref<IndicatorVisualState>("hidden");
const level = ref(0);
const WAVE_BAR_COUNT = 15;
const injectionMode = ref<InjectionMode>("flash_paste");
const win = getCurrentWindow();
const { toasts, showToast, dismissToast } = useToast();

function handleDismissToast(id: number): void {
  dismissToast(id);
  if (toasts.value.length === 0) {
    void invoke("hide_indicator");
  }
}

function showTranscriptionErrorToast(opts: ShowToastOptions): void {
  showToast(opts);
  const ms = opts.durationMs ?? 5000;
  setTimeout(() => {
    if (toasts.value.length === 0) {
      void invoke("hide_indicator");
    }
  }, ms + 50);
}

let unlistenState: UnlistenFn | null = null;
let unlistenHidden: UnlistenFn | null = null;
let unlistenAudioLevel: UnlistenFn | null = null;
let unlistenMoved: UnlistenFn | null = null;
let unlistenTranscriptionError: UnlistenFn | null = null;
let persistTimer: ReturnType<typeof setTimeout> | null = null;

const isRecording = computed(() => state.value === "recording");
const WAVE_SEGMENTS = 7;
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

const bars = computed(() => {
  const clamped = Math.max(0, Math.min(1, animatedLevel.value));
  return Array.from({ length: WAVE_BAR_COUNT }, (_, index) => {
    const t = (index + 1) / WAVE_BAR_COUNT;
    const centerProfile = 1 - Math.abs(t - 0.5);
    const envelope = 0.35 + centerProfile * 0.65;
    const value = clamped * envelope;
    const activeSegments =
      value <= 0 ? 0 : Math.max(1, Math.min(WAVE_SEGMENTS, Math.ceil(value * WAVE_SEGMENTS)));
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

async function onRecordButtonClick(): Promise<void> {
  await invoke("toggle_recording_from_indicator");
}

onMounted(async () => {
  document.documentElement.style.overflow = "hidden";
  document.body.style.margin = "0";
  document.body.style.overflow = "hidden";
  document.body.style.background = "transparent";

  await loadInjectionMode();
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

  unlistenTranscriptionError = await listen<TranscriptionErrorPayload>(
    "transcription-error",
    (event) => {
      const payload = event.payload;

      if (payload.code === "invalid_key") {
        // Open settings focused on the transcription tab with the offending provider highlighted
        void invoke("open_settings_on_transcription_tab", { provider: payload.provider });
        return;
      }

      if (payload.code === "cancelled") {
        // No toast for cancellation — silent UX
        return;
      }

      // Build toast with action button depending on error type
      if (payload.fallback_provider) {
        const fallbackProvider = payload.fallback_provider;
        showTranscriptionErrorToast({
          message: payload.message,
          type: "error",
          action: {
            label: `Try with ${fallbackProvider}?`,
            onClick: () => {
              void invoke("retry_transcription_with_fallback", { provider: fallbackProvider });
            },
          },
        });
      } else if (payload.retryable) {
        showTranscriptionErrorToast({
          message: payload.message,
          type: "error",
          action: {
            label: "Retry",
            onClick: () => {
              void invoke("retry_transcription");
            },
          },
        });
      } else {
        showTranscriptionErrorToast({ message: payload.message, type: "error" });
      }
    },
  );

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
  if (unlistenMoved) unlistenMoved();
  if (unlistenTranscriptionError) unlistenTranscriptionError();
  if (persistTimer) clearTimeout(persistTimer);
});
</script>

<template>
  <main class="indicator-root" @pointerdown="onPointerDown">
    <section class="indicator-pill" :data-state="state">
      <div class="indicator-left">
        <button class="indicator-record-button" type="button" @click.stop="onRecordButtonClick">
          <span class="indicator-record-dot" :class="{ 'indicator-record-dot-active': isRecording }" />
        </button>
        <StateBadge :state="state" :injection-mode="injectionMode" />
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
    <!-- Transcription error toasts -->
    <div class="indicator-toasts">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="indicator-toast"
        :class="`indicator-toast--${toast.type}`"
      >
        <span class="indicator-toast-message">{{ toast.message }}</span>
        <button
          v-if="toast.action"
          class="indicator-toast-action"
          type="button"
          @click.stop="() => { toast.action!.onClick(); handleDismissToast(toast.id); }"
        >
          {{ toast.action.label }}
        </button>
        <button
          class="indicator-toast-dismiss"
          type="button"
          aria-label="Dismiss"
          @click.stop="handleDismissToast(toast.id)"
        >
          &times;
        </button>
      </div>
    </div>
  </main>
</template>
