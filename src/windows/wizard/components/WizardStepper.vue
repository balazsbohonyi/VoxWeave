<script setup lang="ts">
defineProps<{
  currentStep: 1 | 2 | 3;
}>();
</script>

<template>
  <div class="flex items-center justify-center gap-0 mb-8">
    <!-- Step 1 dot -->
    <div class="flex flex-col items-center">
      <div
        class="w-4 h-4 rounded-full"
        :class="currentStep >= 1 ? 'bg-blue-600' : 'border-2 border-gray-400 dark:border-gray-600 bg-transparent'"
      />
      <span class="text-xs text-gray-500 dark:text-gray-400 mt-1">Engine</span>
    </div>

    <!-- Line 1→2 -->
    <div
      class="h-0.5 w-16 mb-4"
      :class="currentStep >= 2 ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
    />

    <!-- Step 2 dot -->
    <div class="flex flex-col items-center">
      <div
        class="w-4 h-4 rounded-full"
        :class="currentStep >= 2 ? 'bg-blue-600' : 'border-2 border-gray-400 dark:border-gray-600 bg-transparent'"
      />
      <span class="text-xs text-gray-500 dark:text-gray-400 mt-1">Key</span>
    </div>

    <!-- Line 2→3 -->
    <div
      class="h-0.5 w-16 mb-4"
      :class="currentStep >= 3 ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
    />

    <!-- Step 3 dot -->
    <div class="flex flex-col items-center">
      <div
        class="w-4 h-4 rounded-full"
        :class="currentStep >= 3 ? 'bg-blue-600' : 'border-2 border-gray-400 dark:border-gray-600 bg-transparent'"
      />
      <span class="text-xs text-gray-500 dark:text-gray-400 mt-1">Hotkey</span>
    </div>
  </div>
</template>
