// useConfig — reactive wrapper around the persisted AppConfig.
// Calls Rust commands get_config / save_config via Tauri IPC.
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "../types/index";

export function useConfig() {
  const config = ref<AppConfig | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  /** Load config from Rust / disk. */
  async function loadConfig(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      config.value = await invoke<AppConfig>("get_config");
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  /** Persist a partial or full config update. Merges with current value. */
  async function saveConfig(updates: Partial<AppConfig>): Promise<void> {
    if (!config.value) {
      error.value = "Config not loaded — call loadConfig() first";
      return;
    }
    loading.value = true;
    error.value = null;
    try {
      const merged: AppConfig = { ...config.value, ...updates };
      await invoke<void>("save_config", { config: merged });
      config.value = merged;
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  return { config, loading, error, loadConfig, saveConfig };
}
