<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useConfig } from "../../composables/useConfig";
import WarningCard from "./components/WarningCard.vue";

const {
  config,
  loading,
  error,
  hotkeyWarning,
  audioWarning,
  audioInputDevices,
  audioDevicesLoadError,
  clearHotkeyWarning,
  clearAudioWarning,
  loadConfig,
  startAudioDevicePolling,
  stopAudioDevicePolling,
  saveConfig,
} = useConfig();

const hotkeyDraft = ref("");
const hotkeyApplyError = ref<string | null>(null);
const hotkeyApplySuccess = ref<string | null>(null);
const isApplyingHotkey = ref(false);
const selectedAudioDevice = ref<string>("");
const isSavingAudioDevice = ref(false);
const previouslyUnavailableAudioDevice = ref<string | null>(null);
const isAutoSyncingAudioDevice = ref(false);

const hotkeyDirty = computed(() => {
  if (!config.value) {
    return false;
  }
  return hotkeyDraft.value.trim() !== config.value.hotkey;
});

async function applyHotkey(): Promise<void> {
  if (!config.value) {
    return;
  }

  const draftValue = hotkeyDraft.value.trim();
  if (!draftValue) {
    hotkeyApplySuccess.value = null;
    hotkeyApplyError.value = "Hotkey cannot be empty.";
    return;
  }

  if (!hotkeyDirty.value) {
    hotkeyApplySuccess.value = "Hotkey is already up to date.";
    hotkeyApplyError.value = null;
    return;
  }

  isApplyingHotkey.value = true;
  hotkeyApplySuccess.value = null;
  hotkeyApplyError.value = null;

  try {
    const saved = await saveConfig({ hotkey: draftValue });
    if (!saved) {
      hotkeyApplyError.value = "Hotkey update failed because config is not loaded.";
      return;
    }
    hotkeyDraft.value = saved.hotkey;
    hotkeyApplySuccess.value = `Hotkey saved as ${saved.hotkey}.`;
  } catch (e) {
    hotkeyApplyError.value = e instanceof Error ? e.message : String(e);
  } finally {
    isApplyingHotkey.value = false;
  }
}

async function saveAudioDevice(): Promise<void> {
  if (!config.value) {
    return;
  }

  isSavingAudioDevice.value = true;
  try {
    const nextDevice = selectedAudioDevice.value.trim();
    const saved = await saveConfig({
      audio: {
        ...config.value.audio,
        device: nextDevice.length > 0 ? nextDevice : null,
      },
    });
    if (saved) {
      selectedAudioDevice.value = saved.audio.device ?? "";
      previouslyUnavailableAudioDevice.value = null;
      clearAudioWarning();
    }
  } finally {
    isSavingAudioDevice.value = false;
  }
}

async function syncAudioDeviceSelection(nextDevice: string | null): Promise<void> {
  if (!config.value) {
    return;
  }
  if ((config.value.audio.device ?? null) === nextDevice) {
    return;
  }

  isAutoSyncingAudioDevice.value = true;
  isSavingAudioDevice.value = true;
  try {
    await saveConfig({
      audio: {
        ...config.value.audio,
        device: nextDevice,
      },
    });
  } finally {
    isSavingAudioDevice.value = false;
    isAutoSyncingAudioDevice.value = false;
  }
}

onMounted(() => {
  loadConfig().then(() => {
    if (config.value) {
      hotkeyDraft.value = config.value.hotkey;
      selectedAudioDevice.value = config.value.audio.device ?? "";
    }
    startAudioDevicePolling();
  });
});

onUnmounted(() => {
  stopAudioDevicePolling();
});

watch(audioInputDevices, async (devices) => {
  if (!config.value || isAutoSyncingAudioDevice.value) {
    return;
  }

  if (selectedAudioDevice.value.length > 0 && !devices.includes(selectedAudioDevice.value)) {
    previouslyUnavailableAudioDevice.value = selectedAudioDevice.value;
    selectedAudioDevice.value = "";
    await syncAudioDeviceSelection(null);
    return;
  }

  if (
    previouslyUnavailableAudioDevice.value &&
    selectedAudioDevice.value.length === 0 &&
    devices.includes(previouslyUnavailableAudioDevice.value)
  ) {
    const restoredDevice = previouslyUnavailableAudioDevice.value;
    selectedAudioDevice.value = restoredDevice;
    previouslyUnavailableAudioDevice.value = null;
    clearAudioWarning();
    await syncAudioDeviceSelection(restoredDevice);
  }
});
</script>

<template>
  <div class="flex h-screen flex-col bg-white text-gray-900 dark:bg-gray-900 dark:text-gray-100">
    <main class="relative flex flex-1 items-center justify-center">
      <div class="absolute right-6 top-6 z-20 space-y-3">
        <WarningCard
          v-if="hotkeyWarning"
          label="Hotkey Warning"
          :title="`${hotkeyWarning.hotkey} unavailable`"
          :message="hotkeyWarning.message"
          @dismiss="clearHotkeyWarning"
        />
        <WarningCard
          v-if="audioWarning"
          label="Audio Warning"
          title="Microphone fallback"
          :message="audioWarning.message"
          @dismiss="clearAudioWarning"
        />
      </div>

      <div v-if="loading && !config" class="text-center text-gray-400 dark:text-gray-500">
        <p class="text-sm">Loading...</p>
      </div>

      <div v-else-if="!config && error" class="text-center text-red-500">
        <p class="text-sm">Failed to load config: {{ error }}</p>
      </div>

      <div v-else-if="config" class="w-full max-w-sm space-y-4 px-6 text-sm">
        <p class="font-medium text-gray-700 dark:text-gray-300">Active configuration</p>
        <dl class="space-y-1 text-gray-500 dark:text-gray-400">
          <div class="space-y-2 rounded-lg border border-gray-200 bg-gray-50 p-3 dark:border-gray-700 dark:bg-gray-800/40">
            <div class="flex items-center justify-between">
              <dt class="font-medium text-gray-700 dark:text-gray-300">Hotkey</dt>
              <dd class="font-mono text-[11px] text-gray-500 dark:text-gray-400">
                Active: {{ config.hotkey }}
              </dd>
            </div>
            <div class="flex items-start gap-2">
              <input
                v-model="hotkeyDraft"
                type="text"
                spellcheck="false"
                class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 font-mono text-xs text-gray-900 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-900 dark:text-gray-100 dark:focus:border-blue-400 dark:focus:ring-blue-400"
                placeholder="Ctrl+Shift+Space"
                :disabled="isApplyingHotkey"
              >
              <button
                class="rounded-md bg-blue-600 px-3 py-2 text-xs font-semibold text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-gray-400 dark:disabled:bg-gray-600"
                :disabled="isApplyingHotkey || !hotkeyDirty"
                @click="applyHotkey"
              >
                {{ isApplyingHotkey ? "Saving..." : "Apply" }}
              </button>
            </div>
            <p v-if="hotkeyApplySuccess" class="text-xs text-emerald-700 dark:text-emerald-300">
              {{ hotkeyApplySuccess }}
            </p>
            <p v-else-if="hotkeyApplyError" class="text-xs text-red-600 dark:text-red-300">
              {{ hotkeyApplyError }}
            </p>
            <p v-else-if="error" class="text-xs text-red-600 dark:text-red-300">
              {{ error }}
            </p>
          </div>
          <div class="flex justify-between">
            <dt>Provider</dt>
            <dd class="font-mono text-gray-800 dark:text-gray-200">{{ config.transcription.provider }}</dd>
          </div>
          <div class="space-y-2 rounded-lg border border-gray-200 bg-gray-50 p-3 dark:border-gray-700 dark:bg-gray-800/40">
            <dt class="font-medium text-gray-700 dark:text-gray-300">Microphone</dt>
            <div class="flex items-start gap-2">
              <select
                v-model="selectedAudioDevice"
                class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-xs text-gray-900 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-900 dark:text-gray-100 dark:focus:border-blue-400 dark:focus:ring-blue-400"
                :disabled="isSavingAudioDevice"
              >
                <option value="">System default</option>
                <option v-for="device in audioInputDevices" :key="device" :value="device">
                  {{ device }}
                </option>
              </select>
              <button
                class="rounded-md bg-blue-600 px-3 py-2 text-xs font-semibold text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-gray-400 dark:disabled:bg-gray-600"
                :disabled="isSavingAudioDevice"
                @click="saveAudioDevice"
              >
                {{ isSavingAudioDevice ? "Saving..." : "Save" }}
              </button>
            </div>
            <p v-if="audioDevicesLoadError" class="text-xs text-red-600 dark:text-red-300">
              Failed to refresh microphones: {{ audioDevicesLoadError }}
            </p>
            <p
              v-else-if="audioInputDevices.length === 0"
              class="text-xs text-amber-700 dark:text-amber-300"
            >
              No microphone devices detected. Connect a microphone to continue.
            </p>
          </div>
          <div class="flex justify-between">
            <dt>Injection mode</dt>
            <dd class="font-mono text-gray-800 dark:text-gray-200">{{ config.injection.mode }}</dd>
          </div>
          <div class="flex justify-between">
            <dt>First launch</dt>
            <dd class="font-mono text-gray-800 dark:text-gray-200">{{ config.first_launch }}</dd>
          </div>
        </dl>
        <p class="pt-2 text-xs text-gray-400 dark:text-gray-500">
          Full settings UI coming in Phase 8.
        </p>
      </div>

      <div v-else class="text-center text-gray-400 dark:text-gray-500">
        <p class="text-sm">No configuration loaded.</p>
      </div>
    </main>

    <footer class="border-t border-gray-200 px-6 py-3 dark:border-gray-700">
      <p class="text-xs text-gray-400 dark:text-gray-500">VoxFlow v0.1.0</p>
    </footer>
  </div>
</template>
