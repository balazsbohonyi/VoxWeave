<script setup lang="ts">
defineProps<{
  modelValue: "cloud" | "local";
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: "cloud" | "local"): void;
}>();
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">
      Choose your transcription engine
    </h2>
    <p class="text-sm text-gray-500 dark:text-gray-400 mb-6">
      You can change this later in Settings.
    </p>

    <div class="grid grid-cols-2 gap-4">
      <!-- Cloud card -->
      <label
        class="rounded-lg p-4 cursor-pointer transition-colors"
        :class="
          modelValue === 'cloud'
            ? 'border-[3px] border-blue-600 bg-blue-50 dark:bg-blue-900/20'
            : 'border-2 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-500'
        "
        @click="emit('update:modelValue', 'cloud')"
      >
        <input
          type="radio"
          name="engine"
          value="cloud"
          class="sr-only"
          :checked="modelValue === 'cloud'"
          @change="emit('update:modelValue', 'cloud')"
        />
        <div class="flex items-center gap-2 font-medium text-gray-900 dark:text-white mb-1">
          <svg class="h-4 w-4 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
          </svg>
          Cloud
        </div>
        <div class="text-sm text-gray-500 dark:text-gray-400">
          Bring your own API key — no subscription needed
        </div>
      </label>

      <!-- Local card -->
      <label
        class="rounded-lg p-4 cursor-pointer transition-colors"
        :class="
          modelValue === 'local'
            ? 'border-[3px] border-blue-600 bg-blue-50 dark:bg-blue-900/20'
            : 'border-2 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-500'
        "
        @click="emit('update:modelValue', 'local')"
      >
        <input
          type="radio"
          name="engine"
          value="local"
          class="sr-only"
          :checked="modelValue === 'local'"
          @change="emit('update:modelValue', 'local')"
        />
        <div class="flex items-center gap-2 font-medium text-gray-900 dark:text-white mb-1">
          <svg class="h-4 w-4 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
          Local
        </div>
        <div class="text-sm text-gray-500 dark:text-gray-400">
          Runs offline on your machine — coming in a future update
        </div>
      </label>
    </div>
  </div>
</template>
