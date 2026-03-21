<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMounted } from "vue";
import { useToast } from "../../composables/useToast";
import type { ShowToastOptions } from "../../composables/useToast";

interface TranscriptionErrorPayload {
  code: "invalid_key" | "rate_limit" | "network" | "server" | "cancelled";
  message: string;
  provider?: string;
  fallback_provider?: string;
  retryable: boolean;
}

const { toasts, showToast, dismissToast } = useToast();

async function handleDismissToast(id: number): Promise<void> {
  dismissToast(id);
  if (toasts.value.length === 0) {
    await invoke("hide_toast_window");
  }
}

function showTranscriptionErrorToast(opts: ShowToastOptions): void {
  showToast(opts);
}

onMounted(() => {
  document.documentElement.style.overflow = "hidden";
  document.body.style.margin = "0";
  document.body.style.overflow = "hidden";
  document.body.style.background = "transparent";

  // Rust delivers toast payloads via eval() instead of Tauri events because
  // WebView2 may not deliver events to hidden windows before they are shown.
  (window as unknown as Record<string, unknown>).__voxflowShowToast = (
    payload: TranscriptionErrorPayload,
  ) => {
    if (payload.code === "cancelled") return;

    if (payload.code === "invalid_key") {
      const provider = payload.provider;
      showTranscriptionErrorToast({
        message: "Invalid API key. Open Settings to fix.",
        type: "error",
        action: {
          label: "Open Settings",
          onClick: () => {
            void invoke("open_settings_on_transcription_tab", { provider });
          },
        },
      });
      return;
    }

    if (payload.fallback_provider) {
      const fallbackProvider = payload.fallback_provider;
      showTranscriptionErrorToast({
        message: payload.message,
        type: "error",
        action: {
          label: `Try with ${fallbackProvider}?`,
          onClick: () => {
            void invoke("retry_transcription_with_fallback", { provider: fallbackProvider });
          },
        },
      });
    } else if (payload.retryable) {
      showTranscriptionErrorToast({
        message: payload.message,
        type: "error",
        action: {
          label: "Retry",
          onClick: () => {
            void invoke("retry_transcription");
          },
        },
      });
    } else {
      showTranscriptionErrorToast({ message: payload.message, type: "error" });
    }
  };
});
</script>

<template>
  <main class="toast-root">
    <div class="indicator-toasts">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="indicator-toast"
        :class="`indicator-toast--${toast.type}`"
      >
        <div class="indicator-toast-header">
          <span class="indicator-toast-title">VoxFlow</span>
          <button
            class="indicator-toast-dismiss"
            type="button"
            aria-label="Dismiss"
            @click.stop="handleDismissToast(toast.id)"
          >
            &times;
          </button>
        </div>
        <span class="indicator-toast-message">{{ toast.message }}</span>
        <button
          v-if="toast.action"
          class="indicator-toast-action"
          type="button"
          @click.stop="() => { toast.action!.onClick(); void handleDismissToast(toast.id); }"
        >
          {{ toast.action.label }}
        </button>
      </div>
    </div>
  </main>
</template>
