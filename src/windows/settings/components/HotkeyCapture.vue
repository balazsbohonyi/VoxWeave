<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  modelValue: string;
  warning: string | null;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "save", combo: string): void;
}>();

const MODIFIERS = new Set(["Control", "Shift", "Alt", "Meta"]);
const KEY_MAP: Record<string, string> = {
  " ": "Space",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Control: "Ctrl",
  Meta: "Super",
};

const heldKeys = ref<Set<string>>(new Set());
const isCapturing = ref(false);
const currentPreview = ref("");

function normalizeKey(key: string): string {
  return KEY_MAP[key] ?? key;
}

function buildCombo(): string {
  const mods = ["Ctrl", "Shift", "Alt", "Super"].filter((m) =>
    heldKeys.value.has(m),
  );
  const nonMods = [...heldKeys.value].filter(
    (k) => !["Ctrl", "Shift", "Alt", "Super"].includes(k),
  );
  return [...mods, ...nonMods].join("+");
}

function onKeyDown(e: KeyboardEvent): void {
  if (e.key === "Escape") {
    isCapturing.value = false;
    heldKeys.value.clear();
    currentPreview.value = "";
    (e.target as HTMLElement).blur();
    return;
  }
  if (e.key === "Tab") {
    // Let Tab propagate for form navigation — do not capture
    return;
  }
  e.preventDefault();
  isCapturing.value = true;
  heldKeys.value.add(normalizeKey(e.key));
  const onlyModifiers =
    heldKeys.value.size === 1 && MODIFIERS.has(e.key);
  currentPreview.value = buildCombo() + (onlyModifiers ? "+..." : "");
}

function onKeyUp(e: KeyboardEvent): void {
  const normalized = normalizeKey(e.key);
  if (!MODIFIERS.has(e.key)) {
    const combo = buildCombo();
    if (combo) {
      emit("save", combo);
    }
    heldKeys.value.clear();
    isCapturing.value = false;
    currentPreview.value = "";
  } else {
    heldKeys.value.delete(normalized);
    currentPreview.value = buildCombo() + "+...";
  }
}

function onFocus(): void {
  isCapturing.value = false;
  heldKeys.value.clear();
  currentPreview.value = "";
}

function onBlur(): void {
  isCapturing.value = false;
  heldKeys.value.clear();
  currentPreview.value = "";
}

// Suppress clicks only — we use keyboard events on the div
function onMouseDown(e: MouseEvent): void {
  // Focus the element so keyboard events are captured
  (e.currentTarget as HTMLElement).focus();
}
</script>

<template>
  <div>
    <div
      tabindex="0"
      class="cursor-text select-none rounded border border-gray-300 px-3 py-2 font-mono text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-800 dark:focus:border-blue-400 dark:focus:ring-blue-400"
      :class="isCapturing ? 'border-blue-400 ring-1 ring-blue-400' : ''"
      @keydown="onKeyDown"
      @keyup="onKeyUp"
      @focus="onFocus"
      @blur="onBlur"
      @mousedown="onMouseDown"
    >
      <span v-if="isCapturing && currentPreview" class="text-blue-600 dark:text-blue-400">
        {{ currentPreview }}
      </span>
      <span v-else-if="isCapturing" class="text-gray-400 italic">
        Press keys...
      </span>
      <span v-else class="text-gray-900 dark:text-white">
        {{ modelValue || "Click to set hotkey" }}
      </span>
    </div>
    <p v-if="warning" class="mt-1 text-xs text-amber-600 dark:text-amber-400">
      {{ warning }}
    </p>
  </div>
</template>
