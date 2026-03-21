<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMounted } from "vue";
import { useToast } from "../../composables/useToast";
import type { ShowToastOptions } from "../../composables/useToast";

interface TranscriptionErrorPayload {
  code: "invalid_key" | "rate_limit" | "network" | "server" | "cancelled" | "too_short";
  message: string;
  provider?: string;
  fallback_provider?: string;
  retryable: boolean;
}

interface InjectionErrorPayload {
  code: "cancelled" | "all_methods_failed" | "elevation_required";
  message: string;
  typed_chars: number | null;
  total_chars: number | null;
}

interface PlainToastPayload {
  type: "info" | "warning" | "error";
  message: string;
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

function isInjectionPayload(payload: unknown): payload is InjectionErrorPayload {
  if (typeof payload !== "object" || payload === null) return false;
  const p = payload as Record<string, unknown>;
  // Injection payloads have typed_chars/total_chars fields (even if null),
  // or have injection-specific codes that transcription never uses.
  if (p.code === "all_methods_failed" || p.code === "elevation_required") return true;
  // Injection cancelled has typed_chars field present (transcription cancelled does not)
  if (p.code === "cancelled" && "typed_chars" in p) return true;
  return false;
}

function isPlainToast(payload: unknown): payload is PlainToastPayload {
  if (typeof payload !== "object" || payload === null) return false;
  const p = payload as Record<string, unknown>;
  return typeof p.type === "string" && typeof p.message === "string" && !("code" in p);
}

onMounted(() => {
  document.documentElement.style.overflow = "hidden";
  document.body.style.margin = "0";
  document.body.style.overflow = "hidden";
  document.body.style.background = "transparent";

  // Rust delivers toast payloads via eval() instead of Tauri events because
  // WebView2 may not deliver events to hidden windows before they are shown.
  (window as unknown as Record<string, unknown>).__voxflowShowToast = (
    payload: TranscriptionErrorPayload | InjectionErrorPayload | PlainToastPayload,
  ) => {
    // Handle plain {type, message} toasts (e.g. "Copied to clipboard — paste manually")
    if (isPlainToast(payload)) {
      showToast({ message: payload.message, type: payload.type });
      return;
    }

    // Handle injection error payloads
    if (isInjectionPayload(payload)) {
      if (payload.code === "cancelled") {
        // message is already formatted as "Cancelled — N of M chars typed" by Rust
        showToast({ message: payload.message, type: "info" });
      } else if (payload.code === "all_methods_failed") {
        showToast({ message: payload.message, type: "error" });
      } else if (payload.code === "elevation_required") {
        showToast({ message: payload.message, type: "warning" });
      }
      return;
    }

    // Handle transcription error payloads
    const transcriptionPayload = payload as TranscriptionErrorPayload;
    if (transcriptionPayload.code === "cancelled") return;

    if (transcriptionPayload.code === "invalid_key") {
      const provider = transcriptionPayload.provider;
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

    if (transcriptionPayload.fallback_provider) {
      const fallbackProvider = transcriptionPayload.fallback_provider;
      showTranscriptionErrorToast({
        message: transcriptionPayload.message,
        type: "error",
        action: {
          label: `Try with ${fallbackProvider}?`,
          onClick: () => {
            void invoke("retry_transcription_with_fallback", { provider: fallbackProvider });
          },
        },
      });
    } else if (transcriptionPayload.retryable) {
      showTranscriptionErrorToast({
        message: transcriptionPayload.message,
        type: "error",
        action: {
          label: "Retry",
          onClick: () => {
            void invoke("retry_transcription");
          },
        },
      });
    } else {
      showTranscriptionErrorToast({ message: transcriptionPayload.message, type: "error" });
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
