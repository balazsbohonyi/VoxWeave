<script setup lang="ts">
const props = defineProps<{
  currentStep: 1 | 2 | 3;
}>();

const emit = defineEmits<{
  (e: "go-to", step: 1 | 2 | 3): void;
}>();
</script>

<template>
  <div class="w-full flex items-center mb-8">

    <!-- Step 1 -->
    <div class="flex flex-col items-center flex-shrink-0">
      <button
        type="button"
        class="w-5 h-5 rounded-full transition-colors focus:outline-none cursor-pointer hover:ring-2 hover:ring-blue-300 dark:hover:ring-blue-700"
        :class="props.currentStep >= 1 ? 'bg-blue-600' : 'border-2 border-gray-400 dark:border-gray-600 bg-transparent'"
        @click="emit('go-to', 1)"
      />
      <span class="text-xs text-gray-500 dark:text-gray-400 mt-1 whitespace-nowrap">Engine</span>
    </div>

    <!-- Line 1→2 -->
    <div
      class="flex-1 h-0.5 mb-4 mx-2"
      :class="props.currentStep >= 2 ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
    />

    <!-- Step 2 -->
    <div class="flex flex-col items-center flex-shrink-0">
      <button
        type="button"
        class="w-5 h-5 rounded-full transition-colors focus:outline-none cursor-pointer hover:ring-2 hover:ring-blue-300 dark:hover:ring-blue-700"
        :class="props.currentStep >= 2 ? 'bg-blue-600' : 'border-2 border-gray-400 dark:border-gray-600 bg-transparent'"
        @click="emit('go-to', 2)"
      />
      <span class="text-xs text-gray-500 dark:text-gray-400 mt-1 whitespace-nowrap">API key</span>
    </div>

    <!-- Line 2→3 -->
    <div
      class="flex-1 h-0.5 mb-4 mx-2"
      :class="props.currentStep >= 3 ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
    />

    <!-- Step 3 -->
    <div class="flex flex-col items-center flex-shrink-0">
      <button
        type="button"
        class="w-5 h-5 rounded-full transition-colors focus:outline-none cursor-pointer hover:ring-2 hover:ring-blue-300 dark:hover:ring-blue-700"
        :class="props.currentStep >= 3 ? 'bg-blue-600' : 'border-2 border-gray-400 dark:border-gray-600 bg-transparent'"
        @click="emit('go-to', 3)"
      />
      <span class="text-xs text-gray-500 dark:text-gray-400 mt-1 whitespace-nowrap">Hotkey</span>
    </div>

  </div>
</template>
