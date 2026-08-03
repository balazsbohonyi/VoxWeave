<script setup lang="ts">
import { onMounted } from "vue";
import { useConfig } from "../../../composables/useConfig";
import ToggleSwitch from "../../../components/ToggleSwitch.vue";
import HotkeyCapture from "../../settings/components/HotkeyCapture.vue";

const { config, loadConfig, saveConfig, hotkeyWarning } = useConfig();

onMounted(async () => {
  await loadConfig();
});

async function onHotkeySave(combo: string) {
  await saveConfig({ hotkey: combo });
}

async function onPushToTalkChange(enabled: boolean) {
  await saveConfig({ push_to_talk: enabled });
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

    <div class="mt-4">
      <div class="flex items-center justify-between pr-1">
        <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
          Push to talk
        </p>
        <ToggleSwitch
          :model-value="config?.push_to_talk ?? false"
          label="Push to talk"
          described-by="wizard-push-to-talk-help"
          @update:model-value="onPushToTalkChange"
        />
      </div>
      <p id="wizard-push-to-talk-help" class="text-xs text-gray-400 dark:text-gray-500">
        Hold the hotkey to record; release it to start processing
      </p>
    </div>
  </div>
</template>
