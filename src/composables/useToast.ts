// useToast — lightweight toast notification composable.
// Placeholder: full implementation in Phase 5 (Injection).
import { ref } from "vue";

export interface Toast {
  id: number;
  message: string;
  type: "success" | "error" | "info";
}

let nextId = 0;

export function useToast() {
  const toasts = ref<Toast[]>([]);

  function showToast(
    message: string,
    type: Toast["type"] = "info",
    durationMs = 3000,
  ) {
    const id = nextId++;
    toasts.value.push({ id, message, type });
    setTimeout(() => {
      toasts.value = toasts.value.filter((t) => t.id !== id);
    }, durationMs);
  }

  return { toasts, showToast };
}
