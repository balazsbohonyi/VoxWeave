<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useConfig } from "../../../composables/useConfig";
import LanguageSelect from "./LanguageSelect.vue";
import type { TranscriptionProvider } from "../../../types/index";

const { config, saveConfig } = useConfig();

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
           LOCAL MODE — model stub cards
           ================================================================ -->
      <template v-else>
        <div class="space-y-3">
          <p class="text-sm text-gray-500 dark:text-gray-400">
            Download a local Whisper model to transcribe without an internet connection.
          </p>
          <div
            v-for="model in LOCAL_MODELS"
            :key="model.id"
            class="flex items-center justify-between p-3 border border-gray-200 dark:border-gray-700 rounded-lg"
          >
            <div>
              <span class="font-medium text-sm text-gray-900 dark:text-gray-100">{{ model.label }}</span>
              <span class="ml-2 text-xs text-gray-400">{{ model.size }}</span>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{{ model.quality }}</p>
            </div>
            <button
              disabled
              class="px-3 py-1.5 text-xs bg-blue-500 text-white rounded opacity-40 cursor-not-allowed"
              title="Local transcription coming in a future update"
            >
              Download
            </button>
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
