<script setup lang="ts">
// Settings window — root component.
// Phase 8 will populate tabs: General, Audio, Transcription, Injection.
// This shell establishes the entrypoint structure that later phases extend.
</script>

<template>
  <div class="flex h-screen flex-col bg-white text-gray-900 dark:bg-gray-900 dark:text-gray-100">
    <!-- Header -->
    <header class="flex items-center border-b border-gray-200 px-6 py-4 dark:border-gray-700">
      <h1 class="text-xl font-semibold tracking-tight">VoxFlow Settings</h1>
    </header>

    <!-- Content area — Phase 8 adds sidebar nav + tab panels here -->
    <main class="flex flex-1 items-center justify-center">
      <div class="text-center text-gray-400 dark:text-gray-500">
        <p class="text-sm">Settings coming in Phase 8.</p>
      </div>
    </main>

    <!-- Footer -->
    <footer class="border-t border-gray-200 px-6 py-3 dark:border-gray-700">
      <p class="text-xs text-gray-400 dark:text-gray-500">VoxFlow v0.1.0</p>
    </footer>
  </div>
</template>
