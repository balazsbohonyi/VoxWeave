<script setup lang="ts">
import { computed } from "vue";
import type { IndicatorVisualState, InjectionMode } from "../../../types";

const props = defineProps<{
  state: IndicatorVisualState;
  injectionMode: InjectionMode;
}>();

const icon = computed(() => {
  if (props.state === "recording") return "REC";
  if (props.state === "processing") return "...";
  if (props.state === "injecting") return "INJ";
  if (props.state === "success") return "OK";
  return "IDLE";
});

const isSuccess = computed(() => props.state === "success");

const methodHint = computed(() => {
  if (props.state !== "injecting") return "";
  if (props.injectionMode === "flash_paste") return "FlashPaste";
  if (props.injectionMode === "keystroke") return "Keystroke";
  return "Clipboard";
});
</script>

<template>
  <div class="indicator-badge" :class="{ 'indicator-badge--success': isSuccess }">
    <span class="indicator-badge-icon">{{ icon }}</span>
    <span v-if="methodHint" class="indicator-badge-hint">{{ methodHint }}</span>
  </div>
</template>

<style scoped>
.indicator-badge--success {
  background-color: #22c55e;
  border-radius: 4px;
  padding: 0 4px;
}
</style>
