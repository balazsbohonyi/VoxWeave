<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useConfig } from "../../composables/useConfig";
import GeneralSection from "./components/GeneralSection.vue";
import AudioSection from "./components/AudioSection.vue";
import TranscriptionSection from "./components/TranscriptionSection.vue";
import InjectionSection from "./components/InjectionSection.vue";

type Tab = "general" | "audio" | "transcription" | "injection";

const { config, loading, error, loadConfig } = useConfig();

const activeTab = ref<Tab>("general");

const tabs: { id: Tab; label: string }[] = [
  { id: "general",       label: "General" },
  { id: "audio",         label: "Audio" },
  { id: "transcription", label: "Transcription" },
  { id: "injection",     label: "Injection" },
];

onMounted(() => {
  loadConfig();
});
</script>

<template>
  <div class="flex h-screen bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 overflow-hidden">

    <!-- Sidebar -->
    <nav class="w-44 shrink-0 bg-gray-100 dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col py-4">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="w-full text-left px-4 py-2.5 text-sm font-medium transition-colors rounded-none"
        :class="activeTab === tab.id
          ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 border-r-2 border-blue-500'
          : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 hover:text-gray-900 dark:hover:text-gray-100'"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
      </button>
    </nav>

    <!-- Content area -->
    <main class="flex-1 overflow-y-auto">

      <!-- Initial loading -->
      <div v-if="loading && !config" class="flex items-center justify-center h-full">
        <span class="text-gray-400 dark:text-gray-500 text-sm">Loading settings...</span>
      </div>

      <!-- Load error -->
      <div v-else-if="error && !config" class="m-6 text-red-500 text-sm p-4 bg-red-50 dark:bg-red-900/20 rounded-lg">
        {{ error }}
      </div>

      <!-- Sections — all mounted, toggled with v-show to preserve state -->
      <template v-else-if="config">
        <div v-show="activeTab === 'general'" class="px-6 py-6">
          <GeneralSection />
        </div>
        <div v-show="activeTab === 'audio'" class="px-6 py-6">
          <AudioSection />
        </div>
        <div v-show="activeTab === 'transcription'" class="px-6 py-6">
          <TranscriptionSection />
        </div>
        <div v-show="activeTab === 'injection'" class="px-6 py-6">
          <InjectionSection />
        </div>
      </template>

    </main>
  </div>
</template>
