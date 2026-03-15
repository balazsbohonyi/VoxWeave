// useConfig - reactive wrapper around the persisted AppConfig.
// Calls Rust commands get_config / save_config via Tauri IPC.
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppConfig,
  AudioWarningPayload,
  HotkeyWarningPayload,
} from "../types/index";

export function useConfig() {
  const config = ref<AppConfig | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const hotkeyWarning = ref<HotkeyWarningPayload | null>(null);
  const audioWarning = ref<AudioWarningPayload | null>(null);
  const audioInputDevices = ref<string[]>([]);
  const audioDevicesLoadError = ref<string | null>(null);
  let unlistenHotkeyWarning: UnlistenFn | null = null;
  let unlistenAudioWarning: UnlistenFn | null = null;

  async function ensureListeners(): Promise<void> {
    if (unlistenHotkeyWarning) {
      // hotkey listener already set
    } else {
      unlistenHotkeyWarning = await listen<HotkeyWarningPayload>(
        "hotkey-warning",
        (event) => {
          hotkeyWarning.value = event.payload;
        },
      );
    }

    if (!unlistenAudioWarning) {
      unlistenAudioWarning = await listen<AudioWarningPayload>(
        "audio-warning",
        (event) => {
          audioWarning.value = event.payload;
        },
      );
    }
  }

  async function loadAudioInputDevices(): Promise<void> {
    audioDevicesLoadError.value = null;
    try {
      const devices = await invoke<string[]>("list_audio_input_devices");
      audioInputDevices.value = devices;
    } catch (e) {
      const message = String(e);
      error.value = message;
      audioDevicesLoadError.value = message;
      audioInputDevices.value = [];
    }
  }

  /** Load config from Rust / disk. */
  async function loadConfig(): Promise<void> {
    await ensureListeners();
    loading.value = true;
    error.value = null;
    try {
      config.value = await invoke<AppConfig>("get_config");
      await loadAudioInputDevices();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  /** Persist a partial or full config update. Merges with current value. */
  async function saveConfig(
    updates: Partial<AppConfig>,
  ): Promise<AppConfig | null> {
    if (!config.value) {
      error.value = "Config not loaded - call loadConfig() first";
      return null;
    }
    await ensureListeners();
    loading.value = true;
    error.value = null;
    try {
      const merged: AppConfig = { ...config.value, ...updates };
      const saved = await invoke<AppConfig>("save_config", { config: merged });
      config.value = saved;
      return saved;
    } catch (e) {
      const message = String(e);
      error.value = message;
      throw new Error(message);
    } finally {
      loading.value = false;
    }
  }

  function clearHotkeyWarning(): void {
    hotkeyWarning.value = null;
  }

  function clearAudioWarning(): void {
    audioWarning.value = null;
  }

  return {
    config,
    loading,
    error,
    hotkeyWarning,
    audioWarning,
    audioInputDevices,
    audioDevicesLoadError,
    clearHotkeyWarning,
    clearAudioWarning,
    loadConfig,
    loadAudioInputDevices,
    saveConfig,
  };
}
