<script setup lang="ts">
defineProps<{
  title: string;
}>();
</script>

<template>
  <div>
    <h2 class="text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
      {{ title }}
    </h2>
    <hr class="mt-1 mb-4 border-gray-200 dark:border-gray-700">
  </div>
</template>
