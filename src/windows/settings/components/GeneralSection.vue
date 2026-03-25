<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { useConfig } from "../../../composables/useConfig";
import HotkeyCapture from "./HotkeyCapture.vue";

const { config, hotkeyWarning, saveConfig } = useConfig();

async function onHotkeySave(combo: string): Promise<void> {
  await saveConfig({ hotkey: combo });
  // Backend emits hotkey-warning if conflicting — HotkeyCapture shows it via :warning prop
}

async function onOpenWizard(): Promise<void> {
  await invoke<void>("open_wizard_window");
}

async function onLaunchAtLoginChange(e: Event): Promise<void> {
  const enabled = (e.target as HTMLInputElement).checked;
  await saveConfig({ launch_at_login: enabled });
  try {
    await invoke<void>("set_launch_at_login", { enabled });
  } catch {
    // Command may not be registered yet (Plan 02 not executed); ignore gracefully
  }
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

    <!-- Launch at login -->
    <div class="mb-4 flex items-center justify-between">
      <div>
        <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
          Launch on Windows startup
        </p>
        <p class="text-xs text-gray-400 dark:text-gray-500">
          Start VoxFlow automatically when you log in
        </p>
      </div>
      <input
        type="checkbox"
        :checked="config.launch_at_login"
        class="h-4 w-4 rounded accent-blue-500"
        @change="onLaunchAtLoginChange"
      >
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
