<script setup lang="ts">
import { ref, onMounted } from "vue";
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


onMounted(async () => {
  await loadConfig();
  if (config.value) {
    openaiKey.value = config.value.transcription.providers.openai.api_key ?? "";
    groqKey.value = config.value.transcription.providers.groq.api_key ?? "";
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

// Stub — implemented in Plan 03
async function finish() {
  /* Plan 03 */
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
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg w-[480px] p-8">

      <!-- Stepper header -->
      <WizardStepper :current-step="currentStep" />

      <!-- Step content -->
      <div class="min-h-[200px]">
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
        />

        <Step3Hotkey
          v-else-if="currentStep === 3"
        />
      </div>

      <!-- Bottom button bar -->
      <div class="mt-8 flex items-center">
        <!-- Back (Steps 2 and 3) -->
        <button
          v-if="currentStep > 1"
          type="button"
          class="text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          @click="back"
        >
          Back
        </button>

        <!-- Skip for now (Step 2 only) -->
        <button
          v-if="currentStep === 2"
          type="button"
          class="text-sm text-blue-600 dark:text-blue-400 hover:underline mx-auto"
          @click="skipStep2"
        >
          Skip for now
        </button>

        <!-- Next / Finish (right-aligned) -->
        <button
          v-if="currentStep < 3"
          type="button"
          class="ml-auto rounded-lg bg-blue-600 px-5 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
          @click="onNext"
        >
          Next
        </button>
        <button
          v-else
          type="button"
          class="ml-auto rounded-lg bg-blue-600 px-5 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
          @click="finish"
        >
          Finish
        </button>
      </div>

    </div>
  </div>
</template>
