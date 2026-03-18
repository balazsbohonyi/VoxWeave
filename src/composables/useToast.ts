// useToast — lightweight toast notification composable with optional action button.
import { ref } from "vue";

export interface ToastAction {
  label: string;
  onClick: () => void;
}

export interface Toast {
  id: number;
  message: string;
  type: "success" | "error" | "info";
  action?: ToastAction;
}

export interface ShowToastOptions {
  message: string;
  type?: Toast["type"];
  action?: ToastAction;
}

let nextId = 0;

export function useToast() {
  const toasts = ref<Toast[]>([]);

  function showToast(messageOrOptions: string | ShowToastOptions, type: Toast["type"] = "info") {
    const id = nextId++;
    if (typeof messageOrOptions === "string") {
      toasts.value.push({ id, message: messageOrOptions, type });
    } else {
      toasts.value.push({
        id,
        message: messageOrOptions.message,
        type: messageOrOptions.type ?? "info",
        action: messageOrOptions.action,
      });
    }
  }

  function dismissToast(id: number) {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }

  return { toasts, showToast, dismissToast };
}
