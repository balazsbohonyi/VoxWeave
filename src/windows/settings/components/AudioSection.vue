<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useConfig } from "../../../composables/useConfig";
import SectionDivider from "./SectionDivider.vue";
import WarningCard from "./WarningCard.vue";

const {
  config,
  audioWarning,
  clearAudioWarning,
  audioInputDevices,
  audioDevicesLoadError,
  startAudioDevicePolling,
  stopAudioDevicePolling,
  saveConfig,
} = useConfig();

// Local refs
const selectedDevice = ref<string>("");
const lastNonZeroVadMs = ref(1500);

const silenceEnabled = computed({
  get: () => config.value?.audio.vad_silence_ms !== 0,
  set: (val: boolean) => {
    if (!config.value) return;
    if (val) {
      void saveConfig({
        audio: { ...config.value.audio, vad_silence_ms: lastNonZeroVadMs.value },
      });
    } else {
      if (config.value.audio.vad_silence_ms > 0) {
        lastNonZeroVadMs.value = config.value.audio.vad_silence_ms;
      }
      void saveConfig({ audio: { ...config.value.audio, vad_silence_ms: 0 } });
    }
  },
});

const silenceSecs = computed(() =>
  ((config.value?.audio.vad_silence_ms || 1500) / 1000).toFixed(1),
);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

function onSilenceDurationInput(e: Event): void {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    const val = parseFloat((e.target as HTMLInputElement).value);
    if (isNaN(val) || !config.value) return;
    const ms = Math.round(Math.max(500, Math.min(5000, val * 1000)));
    lastNonZeroVadMs.value = ms;
    void saveConfig({ audio: { ...config.value.audio, vad_silence_ms: ms } });
  }, 300);
}

function onDeviceChange(): void {
  if (!config.value) return;
  void saveConfig({
    audio: {
      ...config.value.audio,
      device: selectedDevice.value.length > 0 ? selectedDevice.value : null,
    },
  });
}

function initSelectedDevice(): void {
  if (config.value) {
    selectedDevice.value = config.value.audio.device ?? "";
    const vadMs = config.value.audio.vad_silence_ms;
    if (vadMs > 0) {
      lastNonZeroVadMs.value = vadMs;
    }
  }
}

watch(config, initSelectedDevice, { immediate: false });

onMounted(() => {
  initSelectedDevice();
  startAudioDevicePolling();
});

onUnmounted(() => {
  stopAudioDevicePolling();
  if (debounceTimer) clearTimeout(debounceTimer);
});
</script>

<template>
  <section v-if="config">
    <SectionDivider title="Audio" />

    <!-- Audio warning -->
    <WarningCard
      v-if="audioWarning"
      label="Audio Warning"
      title="Microphone fallback"
      :message="audioWarning.message"
      @dismiss="clearAudioWarning"
    />

    <!-- Microphone dropdown -->
    <div class="mb-4">
      <label class="mb-1.5 block text-sm font-medium text-gray-700 dark:text-gray-300">
        Microphone
      </label>
      <select
        v-model="selectedDevice"
        class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100 dark:focus:border-blue-400 dark:focus:ring-blue-400"
        @change="onDeviceChange"
      >
        <option value="">System default</option>
        <option v-for="device in audioInputDevices" :key="device" :value="device">
          {{ device }}
        </option>
      </select>
      <p v-if="audioDevicesLoadError" class="mt-1 text-xs text-red-600 dark:text-red-400">
        Failed to refresh microphones: {{ audioDevicesLoadError }}
      </p>
      <p
        v-else-if="audioInputDevices.length === 0"
        class="mt-1 text-xs text-amber-600 dark:text-amber-400"
      >
        No microphone devices detected. Connect a microphone to continue.
      </p>
    </div>

    <!-- Auto-stop on silence toggle -->
    <div class="mb-3 flex items-center justify-between">
      <div>
        <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
          Auto-stop on silence
        </p>
        <p class="text-xs text-gray-400 dark:text-gray-500">
          Automatically stop recording after a period of silence
        </p>
      </div>
      <input
        type="checkbox"
        :checked="silenceEnabled"
        class="h-4 w-4 rounded accent-blue-500"
        @change="silenceEnabled = ($event.target as HTMLInputElement).checked"
      >
    </div>

    <!-- Silence duration input (indented, disabled when toggle is off) -->
    <div class="mb-4 ml-4 flex items-center gap-2">
      <label class="text-sm text-gray-600 dark:text-gray-400">
        Stop after
      </label>
      <div class="flex items-center gap-1">
        <input
          type="number"
          :value="silenceSecs"
          min="0.5"
          max="5.0"
          step="0.1"
          :disabled="!silenceEnabled"
          class="w-20 rounded border border-gray-300 bg-white px-2 py-1 text-sm text-gray-900 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:cursor-not-allowed disabled:opacity-40 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100 dark:focus:border-blue-400 dark:focus:ring-blue-400"
          @input="onSilenceDurationInput"
        >
        <span class="text-sm text-gray-500 dark:text-gray-400">s</span>
      </div>
    </div>
  </section>
</template>
