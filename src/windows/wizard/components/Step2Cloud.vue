<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  activeTab: "openai" | "groq";
  openaiKey: string;
  groqKey: string;
}>();

const emit = defineEmits<{
  (e: "update:activeTab", value: "openai" | "groq"): void;
  (e: "update:openaiKey", value: string): void;
  (e: "update:groqKey", value: string): void;
}>();

type TestResult = "idle" | "testing" | "ok" | "invalid_key" | "network_error" | "timeout";

const testResult = ref<TestResult>("idle");
const showKey = ref(false);

// Draft refs — do NOT bind directly to props
const openaiDraft = ref(props.openaiKey);
const groqDraft = ref(props.groqKey);

// Sync drafts when parent props change (e.g., after loadConfig)
watch(
  () => props.openaiKey,
  (val) => { openaiDraft.value = val; },
);
watch(
  () => props.groqKey,
  (val) => { groqDraft.value = val; },
);

// Reset test result and key visibility when switching tabs
watch(
  () => props.activeTab,
  () => {
    testResult.value = "idle";
    showKey.value = false;
  },
);

function onBlurKey() {
  if (props.activeTab === "openai") {
    emit("update:openaiKey", openaiDraft.value);
  } else {
    emit("update:groqKey", groqDraft.value);
  }
  testResult.value = "idle";
}

async function testConnection() {
  testResult.value = "testing";
  // Pass the current draft key directly — the wizard hasn't saved to config yet.
  const currentKey = props.activeTab === "openai" ? openaiDraft.value : groqDraft.value;
  try {
    await invoke<void>("test_connection", { provider: props.activeTab, apiKey: currentKey });
    testResult.value = "ok";
  } catch (e) {
    const msg = String(e);
    if (msg.includes("Invalid API key") || msg.includes("401")) {
      testResult.value = "invalid_key";
    } else if (msg.includes("timed out") || msg.includes("timeout") || msg.includes("Timeout")) {
      testResult.value = "timeout";
    } else {
      testResult.value = "network_error";
    }
  }
}
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-1">
      Add your API key
    </h2>
    <p class="text-sm text-gray-500 dark:text-gray-400">
      Your key is stored locally and never leaves your machine.
    </p>
    <p class="text-xs text-gray-400 dark:text-gray-500 mt-1 mb-6">
      You only need to add a key for one provider — OpenAI or Groq.
    </p>

    <!-- Provider tab bar -->
    <div class="flex border-b border-gray-200 dark:border-gray-700 mb-6">
      <button
        v-for="tab in (['openai', 'groq'] as const)"
        :key="tab"
        class="px-4 py-2 text-sm font-medium transition-colors"
        :class="
          activeTab === tab
            ? 'border-b-2 border-blue-600 text-blue-600 dark:text-blue-400'
            : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'
        "
        @click="emit('update:activeTab', tab)"
      >
        {{ tab === 'openai' ? 'OpenAI' : 'Groq' }}
      </button>
    </div>

    <!-- API key field -->
    <div class="mb-6">
      <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
        {{ activeTab === 'openai' ? 'OpenAI' : 'Groq' }} API Key
      </label>
      <!-- Input wrapper: relative so the eye button can be absolutely positioned inside -->
      <div class="relative">
        <input
          v-if="activeTab === 'openai'"
          v-model="openaiDraft"
          :type="showKey ? 'text' : 'password'"
          placeholder="sk-..."
          class="w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 pl-3 pr-10 py-2 text-sm text-gray-900 dark:text-white placeholder-gray-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 [&::-ms-reveal]:hidden [&::-webkit-credentials-auto-fill-button]:hidden"
          @blur="onBlurKey"
        />
        <input
          v-else
          v-model="groqDraft"
          :type="showKey ? 'text' : 'password'"
          placeholder="gsk_..."
          class="w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 pl-3 pr-10 py-2 text-sm text-gray-900 dark:text-white placeholder-gray-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 [&::-ms-reveal]:hidden [&::-webkit-credentials-auto-fill-button]:hidden"
          @blur="onBlurKey"
        />
        <!-- Eye toggle: absolutely positioned inside the input -->
        <button
          type="button"
          class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 focus:outline-none"
          :title="showKey ? 'Hide key' : 'Show key'"
          tabindex="-1"
          @click="showKey = !showKey"
        >
          <!-- Eye open -->
          <svg v-if="!showKey" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
          </svg>
          <!-- Eye closed -->
          <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.542-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.542 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Test connection: result is absolutely positioned so it never grows the layout -->
    <div class="relative inline-block">
      <button
        type="button"
        class="rounded-lg border border-gray-300 dark:border-gray-600 px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 transition-colors"
        :disabled="testResult === 'testing'"
        @click="testConnection"
      >
        <span v-if="testResult === 'testing'" class="flex items-center gap-2">
          <svg class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
          Testing...
        </span>
        <span v-else>Test connection</span>
      </button>

      <!-- Result: absolute so it overlays below the button without shifting layout -->
      <p
        v-if="testResult === 'ok'"
        class="absolute left-0 top-full mt-1 whitespace-nowrap text-sm text-green-600 dark:text-green-400"
      >
        Connected
      </p>
      <p
        v-else-if="testResult === 'invalid_key'"
        class="absolute left-0 top-full mt-1 whitespace-nowrap text-sm text-red-600 dark:text-red-400"
      >
        Invalid API key
      </p>
      <p
        v-else-if="testResult === 'network_error'"
        class="absolute left-0 top-full mt-1 whitespace-nowrap text-sm text-amber-600 dark:text-amber-400"
      >
        Network error
      </p>
      <p
        v-else-if="testResult === 'timeout'"
        class="absolute left-0 top-full mt-1 whitespace-nowrap text-sm text-amber-600 dark:text-amber-400"
      >
        Request timed out
      </p>
    </div>
  </div>
</template>
