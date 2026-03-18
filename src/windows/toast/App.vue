<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onBeforeUnmount, onMounted } from "vue";
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
let unlistenTranscriptionError: UnlistenFn | null = null;

async function handleDismissToast(id: number): Promise<void> {
  dismissToast(id);
  if (toasts.value.length === 0) {
    await invoke("hide_toast_window");
  }
}

async function showTranscriptionErrorToast(opts: ShowToastOptions): Promise<void> {
  const wasEmpty = toasts.value.length === 0;
  showToast(opts);
  if (wasEmpty) {
    await invoke("show_toast_window");
  }
}

onMounted(async () => {
  document.documentElement.style.overflow = "hidden";
  document.body.style.margin = "0";
  document.body.style.overflow = "hidden";
  document.body.style.background = "transparent";

  unlistenTranscriptionError = await listen<TranscriptionErrorPayload>(
    "transcription-error",
    (event) => {
      const payload = event.payload;

      if (payload.code === "cancelled") return;

      if (payload.code === "invalid_key") {
        const provider = payload.provider;
        void showTranscriptionErrorToast({
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
        void showTranscriptionErrorToast({
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
        void showTranscriptionErrorToast({
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
        void showTranscriptionErrorToast({ message: payload.message, type: "error" });
      }
    },
  );
});

onBeforeUnmount(() => {
  if (unlistenTranscriptionError) unlistenTranscriptionError();
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
        <button
          class="indicator-toast-dismiss"
          type="button"
          aria-label="Dismiss"
          @click.stop="handleDismissToast(toast.id)"
        >
          &times;
        </button>
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
