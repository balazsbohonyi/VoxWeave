<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: boolean;
    label: string;
    disabled?: boolean;
    describedBy?: string;
  }>(),
  {
    disabled: false,
    describedBy: undefined,
  },
);

const emit = defineEmits<{
  (e: "update:modelValue", value: boolean): void;
}>();
</script>

<template>
  <button
    type="button"
    role="switch"
    :aria-checked="modelValue"
    :aria-label="label"
    :aria-describedby="describedBy"
    :disabled="disabled"
    class="relative inline-flex h-4 w-7 shrink-0 items-center rounded-full border-2 border-transparent transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-40 dark:focus:ring-blue-400 dark:focus:ring-offset-gray-900"
    :class="modelValue ? 'bg-blue-600 dark:bg-blue-500' : 'bg-gray-300 dark:bg-gray-600'"
    @click="emit('update:modelValue', !modelValue)"
  >
    <span
      aria-hidden="true"
      class="pointer-events-none inline-block h-3 w-3 rounded-full bg-white shadow-sm transition-transform"
      :class="modelValue ? 'translate-x-3' : 'translate-x-0'"
    />
  </button>
</template>
