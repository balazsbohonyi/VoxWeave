<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { useConfig } from "../../../composables/useConfig";
import ToggleSwitch from "../../../components/ToggleSwitch.vue";
import HotkeyCapture from "./HotkeyCapture.vue";

const { config, runtimeInfo, hotkeyWarning, saveConfig } = useConfig();

async function onHotkeySave(combo: string): Promise<void> {
  await saveConfig({ hotkey: combo });
  // Backend emits hotkey-warning if conflicting — HotkeyCapture shows it via :warning prop
}

async function onOpenWizard(): Promise<void> {
  await invoke<void>("open_wizard_window");
}

async function onPushToTalkChange(enabled: boolean): Promise<void> {
  await saveConfig({ push_to_talk: enabled });
}

async function onLaunchAtLoginChange(enabled: boolean): Promise<void> {
  await invoke<void>("set_launch_at_login", { enabled });
  await saveConfig({ launch_at_login: enabled });
}
</script>

<template>
  <section v-if="config">
    <!-- Hotkey -->
    <div class="mb-4">
      <label class="mb-1.5 block text-sm font-medium text-gray-700 dark:text-gray-300">
        Hotkey
      </label>
      <HotkeyCapture
        :model-value="config.hotkey"
        :warning="hotkeyWarning?.message ?? null"
        @save="onHotkeySave"
      />
      <p class="mt-1 text-xs text-gray-400 dark:text-gray-500">
        Click the field, then press your desired key combination.
      </p>
    </div>

    <!-- Push to talk -->
    <div class="mb-4">
      <div class="flex items-center justify-between">
        <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
          Push to talk
        </p>
        <ToggleSwitch
          :model-value="config.push_to_talk"
          label="Push to talk"
          described-by="push-to-talk-help"
          @update:model-value="onPushToTalkChange"
        />
      </div>
      <p id="push-to-talk-help" class="text-xs text-gray-400 dark:text-gray-500">
        Hold the hotkey to record; release it to start processing
      </p>
    </div>

    <!-- Launch at login -->
    <div class="mb-4">
      <div class="flex items-center justify-between">
        <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
          Launch on Windows startup
        </p>
        <ToggleSwitch
          :model-value="runtimeInfo?.is_portable ? false : config.launch_at_login"
          label="Launch on Windows startup"
          :disabled="runtimeInfo?.is_portable ?? false"
          described-by="portable-autostart-help"
          @update:model-value="onLaunchAtLoginChange"
        />
      </div>
      <p id="portable-autostart-help" class="text-xs text-gray-400 dark:text-gray-500">
        {{ runtimeInfo?.is_portable ? "Unavailable in portable mode. Disable installed autostart before moving a copy." : "Start VoxWeave automatically when you log in" }}
      </p>
    </div>

    <!-- Setup Wizard re-open -->
    <div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
      <button
        type="button"
        class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
        @click="onOpenWizard"
      >
        Setup Wizard...
      </button>
      <p class="mt-0.5 text-xs text-gray-400 dark:text-gray-500">
        Re-run the first-launch setup guide
      </p>
    </div>

  </section>
</template>
