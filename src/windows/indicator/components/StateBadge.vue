<script setup lang="ts">
import { computed } from "vue";
import type { IndicatorVisualState, InjectionMode } from "../../../types";

const props = defineProps<{
  state: IndicatorVisualState;
  injectionMode: InjectionMode;
}>();

const label = computed(() => {
  if (props.state === "recording") return "Recording";
  if (props.state === "processing") return "Processing";
  if (props.state === "injecting") return "Injecting";
  return "Idle";
});

const icon = computed(() => {
  if (props.state === "recording") return "REC";
  if (props.state === "processing") return "...";
  if (props.state === "injecting") return "INJ";
  return "IDLE";
});

const methodHint = computed(() => {
  if (props.state !== "injecting") return "";
  if (props.injectionMode === "flash_paste") return "FlashPaste";
  if (props.injectionMode === "keystroke") return "Keystroke";
  return "Clipboard";
});
</script>

<template>
  <div class="indicator-badge">
    <span class="indicator-badge-icon">{{ icon }}</span>
    <span class="indicator-badge-label">{{ label }}</span>
    <span v-if="methodHint" class="indicator-badge-hint">{{ methodHint }}</span>
  </div>
</template>

