<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useConfig } from "../../../composables/useConfig";

// ---------------------------------------------------------------------------
// Emits
// ---------------------------------------------------------------------------

const emit = defineEmits<{
  canProceed: [value: boolean];
  navigateNext: [];
}>();

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

const { config } = useConfig();

// ---------------------------------------------------------------------------
// Model definitions
// ---------------------------------------------------------------------------

interface LocalModel {
  id: string;
  label: string;
  size: string;
  quality: string;
}

const LOCAL_MODELS: LocalModel[] = [
  { id: "tiny",   label: "Tiny",   size: "~75 MB",  quality: "Fastest, lowest quality" },
  { id: "base",   label: "Base",   size: "~150 MB", quality: "Fast, basic quality" },
  { id: "small",  label: "Small",  size: "~500 MB", quality: "Balanced" },
  { id: "medium", label: "Medium", size: "~1.5 GB", quality: "High quality, slower" },
];

// ---------------------------------------------------------------------------
// Download state
// ---------------------------------------------------------------------------

type ModelDownloadState = "idle" | "downloading" | "downloaded";

const modelStates = ref<Record<string, ModelDownloadState>>({
  tiny: "idle", base: "idle", small: "idle", medium: "idle",
});
const downloadPercent = ref<Record<string, number>>({
  tiny: 0, base: 0, small: 0, medium: 0,
});
const activeDownloadId = ref<string | null>(null);
const anyDownloaded = ref(false);
const skipClicked = ref(false);

const unlistenFns: UnlistenFn[] = [];

function updateCanProceed() {
  emit("canProceed", anyDownloaded.value || skipClicked.value);
}

function isActiveLocalModel(modelId: string): boolean {
  const path = config.value?.transcription.providers.local.model_path;
  if (!path) return false;
  return path.includes(modelId);
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

async function startDownload(modelId: string) {
  try {
    await invoke("start_model_download", { modelId });
  } catch (e) {
    console.error(`Failed to start download for ${modelId}:`, e);
  }
}

async function cancelDownload() {
  try {
    await invoke("cancel_model_download");
  } catch (e) {
    console.error("Failed to cancel download:", e);
  }
}

function onSkip() {
  skipClicked.value = true;
  updateCanProceed();
  // Auto-advance to next step immediately on skip
  emit("navigateNext");
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

onMounted(async () => {
  // Hydrate downloaded models
  try {
    const downloaded = await invoke<string[]>("get_downloaded_models");
    for (const id of downloaded) {
      if (id in modelStates.value) {
        modelStates.value[id] = "downloaded";
      }
    }
    anyDownloaded.value = Object.values(modelStates.value).some((s) => s === "downloaded");
  } catch {
    // Ignore
  }

  // Emit initial canProceed state
  updateCanProceed();

  // Set up event listeners
  interface DownloadProgressPayload { model_id: string; percent: number; bytes_done: number; bytes_total: number }
  interface DownloadDonePayload { model_id: string; model_path: string }
  interface DownloadEventPayload { model_id: string; message: string }

  unlistenFns.push(
    await listen<DownloadProgressPayload>("model-download-progress", (event) => {
      const { model_id, percent } = event.payload;
      if (model_id in modelStates.value) {
        modelStates.value[model_id] = "downloading";
        downloadPercent.value[model_id] = percent;
        activeDownloadId.value = model_id;
      }
    }),
    await listen<DownloadDonePayload>("model-download-done", (event) => {
      const { model_id } = event.payload;
      if (model_id in modelStates.value) {
        modelStates.value[model_id] = "downloaded";
        downloadPercent.value[model_id] = 100;
        anyDownloaded.value = true;
        updateCanProceed();
      }
      activeDownloadId.value = null;
    }),
    await listen<DownloadEventPayload>("model-download-cancelled", (event) => {
      const { model_id } = event.payload;
      if (model_id in modelStates.value) {
        modelStates.value[model_id] = "idle";
        downloadPercent.value[model_id] = 0;
      }
      activeDownloadId.value = null;
    }),
    await listen<DownloadEventPayload>("model-download-error", (event) => {
      const { model_id, message } = event.payload;
      if (model_id in modelStates.value) {
        modelStates.value[model_id] = "idle";
        downloadPercent.value[model_id] = 0;
      }
      activeDownloadId.value = null;
      console.error(`Model download error for ${model_id}: ${message}`);
    }),
  );
});

onUnmounted(() => {
  for (const unlisten of unlistenFns) {
    unlisten();
  }
});
</script>

<template>
  <div class="py-2">
    <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
      Download a local Whisper model to transcribe without an internet connection.
    </p>

    <!-- Model cards -->
    <div class="space-y-3">
      <div
        v-for="model in LOCAL_MODELS"
        :key="model.id"
        class="p-3 rounded-lg"
        :class="isActiveLocalModel(model.id)
          ? 'border-2 border-blue-600 dark:border-blue-500'
          : 'border border-gray-200 dark:border-gray-700'"
      >
        <!-- Idle state -->
        <div v-if="modelStates[model.id] === 'idle'" class="flex items-center justify-between">
          <div>
            <span class="font-medium text-sm text-gray-900 dark:text-gray-100">{{ model.label }}</span>
            <span class="ml-2 text-xs text-gray-400">{{ model.size }}</span>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{{ model.quality }}</p>
          </div>
          <button
            type="button"
            :disabled="activeDownloadId !== null"
            class="px-3 py-1.5 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            @click="startDownload(model.id)"
          >
            Download
          </button>
        </div>

        <!-- Downloading state -->
        <div v-else-if="modelStates[model.id] === 'downloading'">
          <div class="flex items-center justify-between mb-2">
            <span class="font-medium text-sm text-gray-900 dark:text-gray-100">{{ model.label }}</span>
            <div class="flex items-center gap-3">
              <span class="text-xs text-gray-500 dark:text-gray-400">{{ Math.round(downloadPercent[model.id]) }}%</span>
              <button
                type="button"
                class="text-xs text-red-500 hover:text-red-700 dark:hover:text-red-400 transition-colors"
                @click="cancelDownload()"
              >
                Cancel
              </button>
            </div>
          </div>
          <div class="w-full bg-gray-200 dark:bg-gray-700 h-2 rounded-full overflow-hidden">
            <div
              class="bg-blue-500 h-2 rounded-full"
              :style="{ width: `${Math.round(downloadPercent[model.id])}%` }"
            />
          </div>
        </div>

        <!-- Downloaded state -->
        <div v-else class="flex items-center justify-between">
          <div>
            <div class="flex items-center gap-2">
              <span class="font-medium text-sm text-gray-900 dark:text-gray-100">{{ model.label }}</span>
              <span class="ml-1 text-xs text-gray-400">{{ model.size }}</span>
              <!-- Active badge — blue bg, white text, 4px radius -->
              <span
                v-if="isActiveLocalModel(model.id)"
                class="inline-flex items-center gap-1 px-1.5 py-0.5 text-xs font-medium bg-blue-600 text-white"
                style="border-radius: 4px;"
              >
                <svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                </svg>
                Active
              </span>
            </div>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{{ model.quality }}</p>
          </div>
          <!-- Downloaded checkmark badge -->
          <span class="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 border border-green-200 dark:border-green-800 rounded">
            <svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
              <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
            </svg>
            Downloaded
          </span>
        </div>
      </div>
    </div>

    <!-- Skip for now -->
    <div class="mt-4 text-center">
      <button
        type="button"
        class="text-sm text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
        @click="onSkip"
      >
        Skip for now
      </button>
    </div>
  </div>
</template>
