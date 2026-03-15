<script setup lang="ts">
defineProps<{
  label: string;
  title: string;
  message: string;
}>();

defineEmits<{
  dismiss: [];
}>();
</script>

<template>
  <div
    class="relative w-80 rounded-lg border border-amber-300 bg-amber-50 p-4 text-xs text-amber-900 shadow-lg dark:border-amber-500/60 dark:bg-amber-900/20 dark:text-amber-100"
  >
    <button
      class="absolute right-4 top-4 text-[11px] font-semibold text-amber-700 hover:text-amber-900 dark:text-amber-200 dark:hover:text-amber-50"
      @click="$emit('dismiss')"
    >
      Dismiss
    </button>
    <div>
      <p class="text-[11px] uppercase tracking-wide text-amber-700 dark:text-amber-200">
        {{ label }}
      </p>
      <p class="mt-1 font-medium">
        {{ title }}
      </p>
      <p class="mt-1 text-amber-700/90 dark:text-amber-200/90">
        {{ message }}
      </p>
    </div>
  </div>
</template>
