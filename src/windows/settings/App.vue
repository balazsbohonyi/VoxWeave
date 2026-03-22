<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useConfig } from "../../composables/useConfig";
import GeneralSection from "./components/GeneralSection.vue";
import AudioSection from "./components/AudioSection.vue";
import TranscriptionSection from "./components/TranscriptionSection.vue";
import InjectionSection from "./components/InjectionSection.vue";

const {
  config,
  loading,
  error,
  loadConfig,
  startAudioDevicePolling,
  stopAudioDevicePolling,
} = useConfig();

onMounted(() => {
  loadConfig().then(() => {
    startAudioDevicePolling();
  });
});

onUnmounted(() => {
  stopAudioDevicePolling();
});
</script>

<template>
  <div class="min-h-screen bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
    <div class="max-w-[480px] mx-auto px-6 py-6 space-y-8">
      <!-- Initial loading state (config not yet fetched) -->
      <div v-if="loading && !config" class="flex items-center justify-center py-12">
        <span class="text-gray-400 dark:text-gray-500 text-sm">Loading settings...</span>
      </div>
      <!-- Error on initial load only -->
      <div v-else-if="error && !config" class="text-red-500 text-sm p-4 bg-red-50 dark:bg-red-900/20 rounded-lg">
        {{ error }}
      </div>
      <!-- Settings sections — rendered once config is loaded; stay mounted during saves -->
      <template v-else-if="config">
        <GeneralSection />
        <AudioSection />
        <TranscriptionSection />
        <InjectionSection />
      </template>
    </div>
  </div>
</template>
