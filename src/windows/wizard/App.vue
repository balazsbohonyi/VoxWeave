<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window"; // used in onMounted for first-launch show
import { useConfig } from "../../composables/useConfig";
import type { TranscriptionProvider } from "../../types/index";
import WizardStepper from "./components/WizardStepper.vue";
import Step1Engine from "./components/Step1Engine.vue";
import Step2Cloud from "./components/Step2Cloud.vue";
import Step2Local from "./components/Step2Local.vue";
import Step3Hotkey from "./components/Step3Hotkey.vue";

const { config, loadConfig, saveConfig } = useConfig();

// ---------------------------------------------------------------------------
// Navigation state
// ---------------------------------------------------------------------------

const currentStep = ref<1 | 2 | 3>(1);
const engineChoice = ref<"cloud" | "local">("cloud");
const activeCloudTab = ref<"openai" | "groq">("openai");
const openaiKey = ref("");
const groqKey = ref("");
const showSuccessBanner = ref(false);

// Step 2 Local: canProceed gate (model downloaded OR Skip clicked)
const localCanProceed = ref(false);

onMounted(async () => {
  await loadConfig();
  if (config.value) {
    openaiKey.value = config.value.transcription.providers.openai.api_key ?? "";
    groqKey.value = config.value.transcription.providers.groq.api_key ?? "";
    // Pre-fill engine choice from current config (for re-open from Settings)
    engineChoice.value = config.value.transcription.provider === "local" ? "local" : "cloud";
    activeCloudTab.value = config.value.transcription.provider === "groq" ? "groq" : "openai";
  }
  // Show the window only on first launch — prevents the blank-window flash that
  // occurs when Rust calls show() before the webview has painted.
  // On re-open from Settings, open_wizard_window (Rust) already calls show().
  if (config.value?.first_launch) {
    await getCurrentWindow().show();
    await getCurrentWindow().setFocus();
  }

});

// ---------------------------------------------------------------------------
// Navigation functions
// ---------------------------------------------------------------------------

function back() {
  if (currentStep.value > 1) {
    currentStep.value = (currentStep.value - 1) as 1 | 2 | 3;
  }
}

function goToStep(step: 1 | 2 | 3) {
  currentStep.value = step;
}

function advanceFromStep1() {
  currentStep.value = 2;
}

async function advanceFromStep2() {
  if (engineChoice.value === "cloud" && config.value) {
    await saveConfig({
      transcription: {
        ...config.value.transcription,
        provider: activeCloudTab.value as TranscriptionProvider,
      },
    });
  }
  currentStep.value = 3;
}

function skipStep2() {
  currentStep.value = 3;
}

async function finish(): Promise<void> {
  if (!config.value) return;

  // Build the full updated config inline — bypass the composable's ensureListeners
  // so a missing event-listener capability never silently swallows the save.
  const updatedConfig = {
    ...config.value,
    first_launch: false,
    transcription: {
      ...config.value.transcription,
      provider: (engineChoice.value === "local" ? "local" : activeCloudTab.value) as TranscriptionProvider,
      language: config.value.transcription.language || "en",
      providers: {
        ...config.value.transcription.providers,
        openai: { ...config.value.transcription.providers.openai, api_key: openaiKey.value },
        groq: { ...config.value.transcription.providers.groq, api_key: groqKey.value },
      },
    },
  };

  try {
    // Direct invoke — does not go through ensureListeners()
    await invoke("save_config", { config: updatedConfig });
  } catch (e) {
    console.error("Wizard: save_config failed:", e);
    return;
  }

  const chosenProvider = engineChoice.value === "local" ? "local" : activeCloudTab.value;
  const noModelDownloaded =
    chosenProvider === "local" &&
    !updatedConfig.transcription.providers.local.model_path;

  showSuccessBanner.value = true;

  setTimeout(async () => {
    await invoke("open_settings_window");
    // Use a dedicated Rust command — does not require frontend window permissions
    await invoke("hide_wizard_window");
    // Reset state after hiding so re-open from Settings starts fresh
    showSuccessBanner.value = false;
    currentStep.value = 1;
    localCanProceed.value = false;
    // Nudge toast if user chose Local but skipped downloading a model
    if (noModelDownloaded) {
      await invoke("show_plain_toast", {
        toastType: "warning",
        message: "Download a local model from Settings to start transcribing.",
      });
    }
  }, 1200);
}

// ---------------------------------------------------------------------------
// Next button dispatch
// ---------------------------------------------------------------------------

async function onNext() {
  if (currentStep.value === 1) {
    advanceFromStep1();
  } else if (currentStep.value === 2) {
    await advanceFromStep2();
  }
}
</script>

<template>
  <div class="h-screen bg-white dark:bg-gray-800 flex flex-col px-8 py-6">

    <!-- Stepper header -->
    <WizardStepper :current-step="currentStep" @go-to="goToStep" />

    <!-- Step content -->
    <div class="flex-1 min-h-0 overflow-y-auto">
      <!-- Success banner (replaces step content after Finish) -->
      <div
        v-if="showSuccessBanner"
        class="mt-8 rounded-lg bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-700 p-6 text-center"
      >
        <p class="text-base font-medium text-green-700 dark:text-green-400">
          VoxFlow is ready! Opening Settings…
        </p>
      </div>

      <template v-else>
        <Step1Engine
          v-if="currentStep === 1"
          v-model="engineChoice"
        />

        <Step2Cloud
          v-else-if="currentStep === 2 && engineChoice === 'cloud'"
          v-model:active-tab="activeCloudTab"
          v-model:openai-key="openaiKey"
          v-model:groq-key="groqKey"
        />

        <Step2Local
          v-else-if="currentStep === 2 && engineChoice === 'local'"
          @can-proceed="(val) => { localCanProceed = val; }"
        />

        <Step3Hotkey
          v-else-if="currentStep === 3"
        />
      </template>
    </div>

    <!-- Bottom button bar — hidden while success banner is shown -->
    <div v-if="!showSuccessBanner" class="pt-4 flex items-center">
      <!-- Left group: Back + Skip for now (close together) -->
      <div class="flex items-center gap-4">
        <button
          v-if="currentStep > 1"
          type="button"
          class="text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          @click="back"
        >
          Back
        </button>
        <button
          v-if="currentStep === 2 && engineChoice === 'cloud'"
          type="button"
          class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
          @click="skipStep2"
        >
          Skip for now
        </button>
      </div>

      <!-- Right: Next / Finish (fixed width so they don't shift) -->
      <button
        v-if="currentStep < 3"
        type="button"
        :disabled="currentStep === 2 && engineChoice === 'local' && !localCanProceed"
        class="ml-auto w-24 rounded-lg bg-blue-600 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
        @click="onNext"
      >
        Next
      </button>
      <button
        v-else
        type="button"
        class="ml-auto w-24 rounded-lg bg-blue-600 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
        @click="finish"
      >
        Finish
      </button>
    </div>

  </div>
</template>

<style>
/* Subtle scrollbars — matches Settings window */
::-webkit-scrollbar { width: 5px; height: 5px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: rgba(156,163,175,0.45); border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: rgba(156,163,175,0.75); }
::-webkit-scrollbar-button { display: none; }
</style>
