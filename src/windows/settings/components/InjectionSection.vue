<script setup lang="ts">
import { useConfig } from "../../../composables/useConfig";
import type { InjectionMode, KeystrokeSpeed } from "../../../types/index";

const { config, saveConfig } = useConfig();

function onModeChange(mode: InjectionMode): void {
  if (!config.value) return;
  void saveConfig({ injection: { ...config.value.injection, mode } });
}

function onSpeedChange(speed: KeystrokeSpeed): void {
  if (!config.value) return;
  void saveConfig({ injection: { ...config.value.injection, keystroke_speed: speed } });
}

function onFallbackChange(e: Event): void {
  if (!config.value) return;
  const checked = (e.target as HTMLInputElement).checked;
  void saveConfig({ injection: { ...config.value.injection, auto_fallback: checked } });
}
</script>

<template>
  <section v-if="config">
    <!-- Injection method + Typing speed side by side -->
    <div class="mb-4 flex gap-8">
      <!-- Left: method -->
      <div class="flex-1">
        <label class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
          Injection method
        </label>
        <div class="space-y-2">
          <label class="flex items-center gap-2 py-1 cursor-pointer">
            <input
              type="radio"
              name="injection-mode"
              value="flash_paste"
              :checked="config.injection.mode === 'flash_paste'"
              class="accent-blue-500"
              @change="onModeChange('flash_paste')"
            >
            <span class="text-sm text-gray-800 dark:text-gray-200">FlashPaste (recommended)</span>
          </label>
          <label class="flex items-center gap-2 py-1 cursor-pointer">
            <input
              type="radio"
              name="injection-mode"
              value="keystroke"
              :checked="config.injection.mode === 'keystroke'"
              class="accent-blue-500"
              @change="onModeChange('keystroke')"
            >
            <span class="text-sm text-gray-800 dark:text-gray-200">Simulated keystrokes</span>
          </label>
          <label class="flex items-center gap-2 py-1 cursor-pointer">
            <input
              type="radio"
              name="injection-mode"
              value="clipboard"
              :checked="config.injection.mode === 'clipboard'"
              class="accent-blue-500"
              @change="onModeChange('clipboard')"
            >
            <span class="text-sm text-gray-800 dark:text-gray-200">Clipboard only</span>
          </label>
        </div>
      </div>

      <!-- Right: typing speed (only when keystroke mode selected) -->
      <div v-if="config.injection.mode === 'keystroke'" class="flex-1">
        <label class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
          Typing speed
        </label>
        <div class="space-y-2">
          <label class="flex items-center gap-2 py-1 cursor-pointer">
            <input
              type="radio"
              name="keystroke-speed"
              value="slow"
              :checked="config.injection.keystroke_speed === 'slow'"
              class="accent-blue-500"
              @change="onSpeedChange('slow')"
            >
            <span class="text-sm text-gray-800 dark:text-gray-200">Slow (10ms/char)</span>
          </label>
          <label class="flex items-center gap-2 py-1 cursor-pointer">
            <input
              type="radio"
              name="keystroke-speed"
              value="normal"
              :checked="config.injection.keystroke_speed === 'normal'"
              class="accent-blue-500"
              @change="onSpeedChange('normal')"
            >
            <span class="text-sm text-gray-800 dark:text-gray-200">Normal (5ms/char)</span>
          </label>
          <label class="flex items-center gap-2 py-1 cursor-pointer">
            <input
              type="radio"
              name="keystroke-speed"
              value="fast"
              :checked="config.injection.keystroke_speed === 'fast'"
              class="accent-blue-500"
              @change="onSpeedChange('fast')"
            >
            <span class="text-sm text-gray-800 dark:text-gray-200">Fast (2ms/char)</span>
          </label>
        </div>
      </div>
    </div>

    <!-- Auto-fallback checkbox -->
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
          Auto-fallback
        </p>
        <p class="text-xs text-gray-400 dark:text-gray-500">
          Automatically fall back to next method if injection fails
        </p>
      </div>
      <input
        type="checkbox"
        :checked="config.injection.auto_fallback"
        class="h-4 w-4 rounded accent-blue-500"
        @change="onFallbackChange"
      >
    </div>
  </section>
</template>
