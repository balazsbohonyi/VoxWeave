<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";

const props = defineProps<{
  level: number;
  active: boolean;
}>();

const BAR_COUNT = 10;
const bars = ref<number[]>(Array.from({ length: BAR_COUNT }, () => 0.08));
const smoothLevel = ref(0);
let rafId = 0;

const normalizedTarget = computed(() => {
  const clamped = Math.min(1, Math.max(0, props.level));
  return props.active ? clamped : 0.02;
});

watch(normalizedTarget, (value) => {
  smoothLevel.value = value;
});

function tick(): void {
  const target = normalizedTarget.value;
  const next = bars.value.map((_, index) => {
    const offset = (index - (BAR_COUNT - 1) / 2) / BAR_COUNT;
    const weight = 1 - Math.min(0.85, Math.abs(offset) * 1.8);
    const baseline = props.active ? 0.08 : 0.04;
    const desired = baseline + target * weight * 0.92;
    const current = bars.value[index];
    return current * 0.72 + desired * 0.28;
  });
  bars.value = next;
  rafId = requestAnimationFrame(tick);
}

onMounted(() => {
  rafId = requestAnimationFrame(tick);
});

onBeforeUnmount(() => {
  cancelAnimationFrame(rafId);
});
</script>

<template>
  <div class="indicator-waveform" aria-hidden="true">
    <div
      v-for="(height, index) in bars"
      :key="index"
      class="indicator-waveform-bar"
      :style="{ transform: `scaleY(${height})` }"
    />
  </div>
</template>

