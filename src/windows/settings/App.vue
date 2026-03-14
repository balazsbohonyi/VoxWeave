<script setup lang="ts">
// Settings window — root component.
// Loads config from Rust on mount to prove the persistence layer is wired.
// Phase 8 will populate tabs: General, Audio, Transcription, Injection.
import { onMounted } from "vue";
import { useConfig } from "../../composables/useConfig";

const { config, loading, error, loadConfig } = useConfig();

onMounted(() => {
  loadConfig();
});
</script>

<template>
  <div class="flex h-screen flex-col bg-white text-gray-900 dark:bg-gray-900 dark:text-gray-100">
    <!-- Header -->
    <header class="flex items-center border-b border-gray-200 px-6 py-4 dark:border-gray-700">
      <h1 class="text-xl font-semibold tracking-tight">VoxFlow Settings</h1>
    </header>

    <!-- Content area — Phase 8 adds sidebar nav + tab panels here -->
    <main class="flex flex-1 items-center justify-center">
      <div v-if="loading" class="text-center text-gray-400 dark:text-gray-500">
        <p class="text-sm">Loading…</p>
      </div>

      <div v-else-if="error" class="text-center text-red-500">
        <p class="text-sm">Failed to load config: {{ error }}</p>
      </div>

      <div v-else-if="config" class="w-full max-w-sm space-y-3 px-6 text-sm">
        <p class="font-medium text-gray-700 dark:text-gray-300">Active configuration</p>
        <dl class="space-y-1 text-gray-500 dark:text-gray-400">
          <div class="flex justify-between">
            <dt>Hotkey</dt>
            <dd class="font-mono text-gray-800 dark:text-gray-200">{{ config.hotkey }}</dd>
          </div>
          <div class="flex justify-between">
            <dt>Provider</dt>
            <dd class="font-mono text-gray-800 dark:text-gray-200">{{ config.transcription.provider }}</dd>
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

    <!-- Footer -->
    <footer class="border-t border-gray-200 px-6 py-3 dark:border-gray-700">
      <p class="text-xs text-gray-400 dark:text-gray-500">VoxFlow v0.1.0</p>
    </footer>
  </div>
</template>
