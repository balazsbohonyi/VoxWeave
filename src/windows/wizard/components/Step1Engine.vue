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
        class="rounded-lg border-2 p-4 cursor-pointer transition-colors"
        :class="
          modelValue === 'cloud'
            ? 'border-blue-600 bg-blue-50 dark:bg-blue-900/20'
            : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-500'
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
        <div class="font-medium text-gray-900 dark:text-white mb-1">Cloud</div>
        <div class="text-sm text-gray-500 dark:text-gray-400">
          Use OpenAI or Groq — bring your own API key
        </div>
      </label>

      <!-- Local card -->
      <label
        class="rounded-lg border-2 p-4 cursor-pointer transition-colors"
        :class="
          modelValue === 'local'
            ? 'border-blue-600 bg-blue-50 dark:bg-blue-900/20'
            : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-500'
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
        <div class="font-medium text-gray-900 dark:text-white mb-1">Local</div>
        <div class="text-sm text-gray-500 dark:text-gray-400">
          Runs offline on your machine — coming in a future update
        </div>
      </label>
    </div>
  </div>
</template>
