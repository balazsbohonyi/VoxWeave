<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useConfig } from "../../../composables/useConfig";
import LanguageSelect from "./LanguageSelect.vue";
import type { TranscriptionProvider } from "../../../types/index";

const { config, saveConfig, loadConfig } = useConfig();

// ---------------------------------------------------------------------------
// Component-level constant — not reactive
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
// Local model download state
// ---------------------------------------------------------------------------

type ModelDownloadState = "idle" | "downloading" | "downloaded";

const modelStates = ref<Record<string, ModelDownloadState>>({
  tiny: "idle", base: "idle", small: "idle", medium: "idle",
});
const downloadPercent = ref<Record<string, number>>({
  tiny: 0, base: 0, small: 0, medium: 0,
});
const activeDownloadId = ref<string | null>(null);

// Event unlisten functions — cleaned up in onUnmounted
const unlistenFns: UnlistenFn[] = [];

// ---------------------------------------------------------------------------
// UI state
// ---------------------------------------------------------------------------

// Which cloud provider tab is currently *viewed* — not the active provider.
const activeTab = ref<"openai" | "groq">("openai");

// Model lists fetched from Rust constants.
const providerModels = ref<{ openai: string[]; groq: string[] }>({ openai: [], groq: [] });

// Eye-icon reveal toggle per provider.
const showApiKey = ref<Record<"openai" | "groq", boolean>>({ openai: false, groq: false });

// Test connection per provider: idle | pending | success | error.
type TestState = "idle" | "pending" | "success" | "error";
const testConnectionState = ref<Record<"openai" | "groq", TestState>>({
  openai: "idle",
  groq: "idle",
});
const testConnectionMessage = ref<Record<"openai" | "groq", string>>({
  openai: "",
  groq: "",
});

// ---------------------------------------------------------------------------
// Local draft refs — never v-model directly on config
// ---------------------------------------------------------------------------

const openaiKeyDraft = ref("");
const groqKeyDraft = ref("");
const openaiModelDraft = ref("");
const groqModelDraft = ref("");

watch(
  config,
  (cfg) => {
    if (!cfg) return;
    openaiKeyDraft.value = cfg.transcription.providers.openai.api_key;
    groqKeyDraft.value = cfg.transcription.providers.groq.api_key;
    openaiModelDraft.value = cfg.transcription.providers.openai.model;
    groqModelDraft.value = cfg.transcription.providers.groq.model;
  },
  { immediate: true },
);

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

onMounted(async () => {
  try {
    providerModels.value = await invoke<{ openai: string[]; groq: string[] }>("get_provider_models");
  } catch {
    // Fall back to empty lists; Rust side will return defaults.
  }

  // Hydrate downloaded model states from Rust
  try {
    const downloaded = await invoke<string[]>("get_downloaded_models");
    for (const id of downloaded) {
      if (id in modelStates.value) {
        modelStates.value[id] = "downloaded";
      }
    }
  } catch {
    // Ignore — model cards will show idle state
  }

  // Set up download event listeners
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
      }
      activeDownloadId.value = null;
      // Reload config to pick up the absolute model_path the Rust backend saved
      void loadConfig();
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

// ---------------------------------------------------------------------------
// Derived helpers
// ---------------------------------------------------------------------------

function isLocalMode(): boolean {
  return config.value?.transcription.provider === "local";
}

function isActiveProvider(tab: "openai" | "groq"): boolean {
  return config.value?.transcription.provider === tab;
}

// ---------------------------------------------------------------------------
// Cloud/Local toggle
// ---------------------------------------------------------------------------

async function selectCloud() {
  if (!config.value) return;
  // Only switch if currently in local mode — keep existing openai/groq selection.
  if (config.value.transcription.provider === "local") {
    await saveConfig({
      transcription: {
        ...config.value.transcription,
        provider: "openai" as TranscriptionProvider,
      },
    });
  }
}

async function selectLocal() {
  if (!config.value) return;
  await saveConfig({
    transcription: {
      ...config.value.transcription,
      provider: "local" as TranscriptionProvider,
    },
  });
}

// ---------------------------------------------------------------------------
// API key save on blur
// ---------------------------------------------------------------------------

function clearTestState(tab: "openai" | "groq") {
  testConnectionState.value[tab] = "idle";
  testConnectionMessage.value[tab] = "";
}

async function saveApiKey(tab: "openai" | "groq") {
  if (!config.value) return;
  clearTestState(tab);
  const newKey = tab === "openai" ? openaiKeyDraft.value : groqKeyDraft.value;
  await saveConfig({
    transcription: {
      ...config.value.transcription,
      providers: {
        ...config.value.transcription.providers,
        [tab]: {
          ...config.value.transcription.providers[tab],
          api_key: newKey,
        },
      },
    },
  });
}

// ---------------------------------------------------------------------------
// Model dropdown save on change
// ---------------------------------------------------------------------------

async function saveModel(tab: "openai" | "groq") {
  if (!config.value) return;
  const newModel = tab === "openai" ? openaiModelDraft.value : groqModelDraft.value;
  await saveConfig({
    transcription: {
      ...config.value.transcription,
      providers: {
        ...config.value.transcription.providers,
        [tab]: {
          ...config.value.transcription.providers[tab],
          model: newModel,
        },
      },
    },
  });
}

// ---------------------------------------------------------------------------
// Test connection
// ---------------------------------------------------------------------------

async function testConnection(tab: "openai" | "groq") {
  testConnectionState.value[tab] = "pending";
  testConnectionMessage.value[tab] = "";
  try {
    await invoke<void>("test_connection", { provider: tab });
    testConnectionState.value[tab] = "success";
    testConnectionMessage.value[tab] = "Connected";
  } catch (e) {
    testConnectionState.value[tab] = "error";
    testConnectionMessage.value[tab] = String(e);
  }
}

// ---------------------------------------------------------------------------
// Set as active
// ---------------------------------------------------------------------------

async function setAsActive(tab: "openai" | "groq") {
  if (!config.value) return;
  await saveConfig({
    transcription: {
      ...config.value.transcription,
      provider: tab as TranscriptionProvider,
    },
  });
}

// ---------------------------------------------------------------------------
// Local model download actions
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

async function deleteModel(modelId: string) {
  activeDownloadId.value = null;
  // Clear state immediately so the Active badge never flickers on a card being deleted.
  modelStates.value[modelId] = "idle";
  downloadPercent.value[modelId] = 0;
  try {
    await invoke("delete_model", { modelId });
    // Reload config — Rust already cleared model_path if this was the active model
    await loadConfig();
    // Auto-activate the single remaining downloaded model (if any, and not already active)
    const remaining = LOCAL_MODELS.map(m => m.id)
      .filter(id => id !== modelId && modelStates.value[id] === "downloaded");
    if (remaining.length === 1 && !isActiveLocalModel(remaining[0])) {
      await setActiveModel(remaining[0]);
    }
  } catch (e) {
    console.error(`Failed to delete model ${modelId}:`, e);
  }
}

async function setActiveModel(modelId: string) {
  if (!config.value) return;
  const absolutePath = await invoke<string>("get_model_path", { modelId });
  await saveConfig({
    transcription: {
      ...config.value.transcription,
      providers: {
        ...config.value.transcription.providers,
        local: { model_path: absolutePath },
      },
    },
  });
}

function isActiveLocalModel(modelId: string): boolean {
  const path = config.value?.transcription.providers.local.model_path;
  if (!path) return false;
  return path.includes(modelId);
}

// ---------------------------------------------------------------------------
// Language save
// ---------------------------------------------------------------------------

async function saveLanguage(lang: string) {
  if (!config.value) return;
  await saveConfig({
    transcription: {
      ...config.value.transcription,
      language: lang,
    },
  });
}
</script>

<template>
  <section>
    <template v-if="config">
      <!-- Cloud / Local toggle -->
      <div class="flex rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden w-full mb-5">
        <button
          class="flex-1 flex items-center justify-center gap-2 py-2 text-sm font-medium transition-colors"
          :class="!isLocalMode()
            ? 'bg-blue-600 text-white'
            : 'bg-white dark:bg-gray-800 text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-750'"
          @click="selectCloud"
        >
          <!-- Cloud icon -->
          <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
          </svg>
          Cloud
        </button>
        <button
          class="flex-1 flex items-center justify-center gap-2 py-2 text-sm font-medium transition-colors"
          :class="isLocalMode()
            ? 'bg-blue-600 text-white'
            : 'bg-white dark:bg-gray-800 text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-750'"
          @click="selectLocal"
        >
          <!-- Computer/local icon -->
          <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
          Local
          <svg v-if="isLocalMode()" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
            <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
          </svg>
        </button>
      </div>

      <!-- ================================================================
           CLOUD MODE — provider tabs
           ================================================================ -->
      <template v-if="!isLocalMode()">
        <!-- Tab bar -->
        <div class="flex gap-0 border-b border-gray-200 dark:border-gray-700 mb-4">
          <button
            v-for="tab in (['openai', 'groq'] as const)"
            :key="tab"
            class="px-4 py-2 text-sm transition-colors"
            :class="activeTab === tab
              ? 'border-b-2 border-blue-500 text-blue-600 dark:text-blue-400 -mb-px'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'"
            @click="activeTab = tab"
          >
            {{ tab === 'openai' ? 'OpenAI' : 'Groq' }}
            <svg v-if="isActiveProvider(tab)" class="inline h-3.5 w-3.5 ml-1 text-blue-500 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
              <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
            </svg>
          </button>
        </div>

        <!-- Per-tab content (OpenAI) -->
        <div v-if="activeTab === 'openai'" class="space-y-4">
          <!-- API Key -->
          <div>
            <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">API Key</label>
            <div class="relative">
              <input
                :type="showApiKey.openai ? 'text' : 'password'"
                :value="openaiKeyDraft"
                autocomplete="off"
                class="w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 pl-3 pr-8 py-1.5 text-sm text-gray-900 dark:text-gray-100 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:focus:border-blue-400"
                placeholder="sk-..."
                @input="openaiKeyDraft = ($event.target as HTMLInputElement).value; clearTestState('openai')"
                @blur="saveApiKey('openai')"
              />
              <button
                type="button"
                class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                :title="showApiKey.openai ? 'Hide API key' : 'Show API key'"
                @mousedown.prevent
                @click="showApiKey.openai = !showApiKey.openai"
              >
                <svg v-if="!showApiKey.openai" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
                <svg v-else class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                </svg>
              </button>
            </div>
          </div>

          <!-- Model dropdown -->
          <div>
            <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Model</label>
            <select
              :value="openaiModelDraft"
              class="w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 px-3 py-1.5 text-sm text-gray-900 dark:text-gray-100 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:focus:border-blue-400"
              @change="openaiModelDraft = ($event.target as HTMLSelectElement).value; saveModel('openai')"
            >
              <option
                v-for="model in providerModels.openai"
                :key="model"
                :value="model"
              >
                {{ model }}
              </option>
            </select>
          </div>

          <!-- Test connection + Set as active -->
          <div>
            <div class="flex items-center justify-between">
              <button
                type="button"
                :disabled="testConnectionState.openai === 'pending'"
                class="flex items-center gap-2 px-3 py-1.5 text-sm rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-750 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                @click="testConnection('openai')"
              >
                <svg v-if="testConnectionState.openai === 'pending'" class="h-3.5 w-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8z" />
                </svg>
                <svg v-else-if="testConnectionState.openai === 'success'" class="h-3.5 w-3.5 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                </svg>
                <svg v-else-if="testConnectionState.openai === 'error'" class="h-3.5 w-3.5 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
                Test connection
              </button>
              <button
                type="button"
                :disabled="isActiveProvider('openai')"
                class="px-3 py-1.5 text-sm rounded-md bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
                @click="setAsActive('openai')"
              >
                Set as active
              </button>
            </div>
            <p v-if="testConnectionState.openai !== 'idle'" class="mt-1.5 text-xs"
               :class="testConnectionState.openai === 'success' ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'">
              {{ testConnectionMessage.openai }}
            </p>
          </div>
        </div>

        <!-- Per-tab content (Groq) -->
        <div v-if="activeTab === 'groq'" class="space-y-4">
          <!-- API Key -->
          <div>
            <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">API Key</label>
            <div class="relative">
              <input
                :type="showApiKey.groq ? 'text' : 'password'"
                :value="groqKeyDraft"
                autocomplete="off"
                class="w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 pl-3 pr-8 py-1.5 text-sm text-gray-900 dark:text-gray-100 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:focus:border-blue-400"
                placeholder="gsk_..."
                @input="groqKeyDraft = ($event.target as HTMLInputElement).value; clearTestState('groq')"
                @blur="saveApiKey('groq')"
              />
              <button
                type="button"
                class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                :title="showApiKey.groq ? 'Hide API key' : 'Show API key'"
                @mousedown.prevent
                @click="showApiKey.groq = !showApiKey.groq"
              >
                <svg v-if="!showApiKey.groq" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
                <svg v-else class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                </svg>
              </button>
            </div>
          </div>

          <!-- Model dropdown -->
          <div>
            <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Model</label>
            <select
              :value="groqModelDraft"
              class="w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 px-3 py-1.5 text-sm text-gray-900 dark:text-gray-100 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:focus:border-blue-400"
              @change="groqModelDraft = ($event.target as HTMLSelectElement).value; saveModel('groq')"
            >
              <option
                v-for="model in providerModels.groq"
                :key="model"
                :value="model"
              >
                {{ model }}
              </option>
            </select>
          </div>

          <!-- Test connection + Set as active -->
          <div>
            <div class="flex items-center justify-between">
              <button
                type="button"
                :disabled="testConnectionState.groq === 'pending'"
                class="flex items-center gap-2 px-3 py-1.5 text-sm rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-750 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                @click="testConnection('groq')"
              >
                <svg v-if="testConnectionState.groq === 'pending'" class="h-3.5 w-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8z" />
                </svg>
                <svg v-else-if="testConnectionState.groq === 'success'" class="h-3.5 w-3.5 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                </svg>
                <svg v-else-if="testConnectionState.groq === 'error'" class="h-3.5 w-3.5 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
                Test connection
              </button>
              <button
                type="button"
                :disabled="isActiveProvider('groq')"
                class="px-3 py-1.5 text-sm rounded-md bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
                @click="setAsActive('groq')"
              >
                Set as active
              </button>
            </div>
            <p v-if="testConnectionState.groq !== 'idle'" class="mt-1.5 text-xs"
               :class="testConnectionState.groq === 'success' ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'">
              {{ testConnectionMessage.groq }}
            </p>
          </div>
        </div>

        <!-- Language hint — global field, rendered once below tabs -->
        <div class="mt-5 pt-4 border-t border-gray-100 dark:border-gray-800">
          <div class="flex items-center justify-between">
            <div>
              <label class="text-sm font-medium text-gray-700 dark:text-gray-300">Language</label>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">Leave blank for auto-detect.</p>
            </div>
            <LanguageSelect
              :model-value="config.transcription.language"
              @change="saveLanguage"
            />
          </div>
        </div>
      </template>

      <!-- ================================================================
           LOCAL MODE — model cards with download/delete/active
           ================================================================ -->
      <template v-else>
        <div class="space-y-3">
          <p class="text-sm text-gray-500 dark:text-gray-400">
            Download a local Whisper model to transcribe without an internet connection.
          </p>
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
                    class="text-xs text-blue-500 hover:text-blue-700 dark:hover:text-blue-400 transition-colors"
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
              <div class="flex items-center gap-2">
                <!-- Trash icon delete button — no text label, no border -->
                <button
                  type="button"
                  class="p-1.5 text-blue-500 hover:text-blue-700 dark:hover:text-blue-400 transition-colors"
                  title="Delete model"
                  @click="deleteModel(model.id)"
                >
                  <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
                <button
                  v-if="!isActiveLocalModel(model.id)"
                  type="button"
                  class="px-3 py-1.5 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                  @click="setActiveModel(model.id)"
                >
                  Set Active
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Language hint also available in local mode -->
        <div class="mt-5 pt-4 border-t border-gray-100 dark:border-gray-800">
          <div class="flex items-center justify-between">
            <div>
              <label class="text-sm font-medium text-gray-700 dark:text-gray-300">Language</label>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">Leave blank for auto-detect.</p>
            </div>
            <LanguageSelect
              :model-value="config.transcription.language"
              @change="saveLanguage"
            />
          </div>
        </div>
      </template>
    </template>
  </section>
</template>
