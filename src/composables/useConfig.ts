// useConfig — reactive wrapper around the persisted AppConfig.
// Placeholder: full implementation in Phase 8 (Settings).
import { ref } from "vue";
import type { AppConfig } from "../types/index";

// Default config mirrors Rust defaults in config/mod.rs.
const defaultConfig: AppConfig = {
  hotkey: "Alt+Shift+Space",
  audio_device: null,
  transcription_provider: "openai",
  openai_api_key: "",
  groq_api_key: "",
  openrouter_api_key: "",
  openai_model: "whisper-1",
  groq_model: "whisper-large-v3",
  openrouter_model: "",
  injection_mode: "flash_paste",
  show_indicator: true,
  launch_at_login: false,
};

export function useConfig() {
  const config = ref<AppConfig>({ ...defaultConfig });
  const loading = ref(false);
  const error = ref<string | null>(null);

  // TODO (Phase 8): invoke("get_config") and invoke("save_config", { config })

  return { config, loading, error };
}
