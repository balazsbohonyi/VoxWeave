<script setup lang="ts">
import { onMounted } from "vue";
import { useConfig } from "../../../composables/useConfig";
import HotkeyCapture from "../../settings/components/HotkeyCapture.vue";

const { config, loadConfig, saveConfig, hotkeyWarning } = useConfig();

onMounted(async () => {
  await loadConfig();
});

async function onHotkeySave(combo: string) {
  await saveConfig({ hotkey: combo });
}
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">
      Confirm your hotkey
    </h2>
    <p class="text-sm text-gray-500 dark:text-gray-400 mb-6">
      Click the field and press a new combination. Avoid pressing your current hotkey — it will trigger recording instead.
    </p>

    <HotkeyCapture
      :model-value="config?.hotkey ?? 'Ctrl+Shift+Space'"
      :warning="hotkeyWarning?.message ?? null"
      @save="onHotkeySave"
    />
  </div>
</template>
