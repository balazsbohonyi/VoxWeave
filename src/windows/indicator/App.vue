<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { LogicalPosition } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import type { AppConfig, AudioLevelPayload, IndicatorStatePayload, IndicatorVisualState, InjectionMode } from "../../types";
import StateBadge from "./components/StateBadge.vue";
import Waveform from "./components/Waveform.vue";

const HOLD_TO_DRAG_MS = 180;
const state = ref<IndicatorVisualState>("hidden");
const level = ref(0);
const injectionMode = ref<InjectionMode>("flash_paste");
const dragging = ref(false);
const dragArmed = ref(false);
const dragStartCursor = ref({ x: 0, y: 0 });
const dragStartWindow = ref({ x: 0, y: 0 });
const win = getCurrentWindow();

let dragArmTimeout: ReturnType<typeof setTimeout> | null = null;
let unlistenState: UnlistenFn | null = null;
let unlistenHidden: UnlistenFn | null = null;
let unlistenAudioLevel: UnlistenFn | null = null;

const visible = computed(() => state.value !== "hidden");
const showWaveform = computed(() => state.value === "recording" || state.value === "processing");

async function loadInjectionMode(): Promise<void> {
  try {
    const config = await invoke<AppConfig>("get_config");
    injectionMode.value = config.injection.mode;
  } catch {
    injectionMode.value = "flash_paste";
  }
}

async function endDrag(commit: boolean): Promise<void> {
  if (!dragging.value) {
    dragArmed.value = false;
    return;
  }
  dragging.value = false;
  dragArmed.value = false;
  try {
    const pos = await win.outerPosition();
    if (commit) {
      await invoke("persist_indicator_position", { x: Math.round(pos.x), y: Math.round(pos.y) });
    }
  } finally {
    await invoke("end_indicator_drag");
  }
}

function clearDragTimer(): void {
  if (!dragArmTimeout) {
    return;
  }
  clearTimeout(dragArmTimeout);
  dragArmTimeout = null;
}

function onPointerDown(event: PointerEvent): void {
  if (!visible.value) {
    return;
  }
  event.preventDefault();
  dragArmed.value = true;
  dragStartCursor.value = { x: event.screenX, y: event.screenY };
  dragArmTimeout = setTimeout(async () => {
    dragArmTimeout = null;
    if (!dragArmed.value) {
      return;
    }
    const position = await win.outerPosition();
    dragStartWindow.value = { x: position.x, y: position.y };
    await invoke("begin_indicator_drag");
    dragging.value = true;
  }, HOLD_TO_DRAG_MS);
}

async function onPointerMove(event: PointerEvent): Promise<void> {
  if (!dragging.value) {
    return;
  }
  const dx = event.screenX - dragStartCursor.value.x;
  const dy = event.screenY - dragStartCursor.value.y;
  await win.setPosition(
    new LogicalPosition(dragStartWindow.value.x + dx, dragStartWindow.value.y + dy),
  );
}

async function onPointerUp(): Promise<void> {
  clearDragTimer();
  if (!dragging.value) {
    dragArmed.value = false;
    return;
  }
  await endDrag(true);
}

onMounted(async () => {
  await loadInjectionMode();

  unlistenState = await listen<IndicatorStatePayload>("indicator-state", (event) => {
    state.value = event.payload.state;
  });

  unlistenHidden = await listen("indicator-hidden", () => {
    level.value = 0;
    state.value = "hidden";
  });

  unlistenAudioLevel = await listen<AudioLevelPayload>("audio-level", (event) => {
    level.value = Math.min(1, Math.max(0, event.payload.rms));
  });

  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
  window.addEventListener("blur", () => {
    void endDrag(false);
  });
});

onBeforeUnmount(async () => {
  clearDragTimer();
  window.removeEventListener("pointermove", onPointerMove);
  window.removeEventListener("pointerup", onPointerUp);
  await endDrag(false);
  if (unlistenState) {
    unlistenState();
  }
  if (unlistenHidden) {
    unlistenHidden();
  }
  if (unlistenAudioLevel) {
    unlistenAudioLevel();
  }
});
</script>

<template>
  <main
    class="indicator-root"
    :class="{ 'indicator-root-hidden': !visible }"
    @pointerdown="onPointerDown"
  >
    <transition name="indicator-fade" mode="out-in">
      <section v-if="visible" :key="state" class="indicator-pill" :data-state="state">
        <StateBadge :state="state" :injection-mode="injectionMode" />
        <Waveform v-if="showWaveform" :level="level" :active="state === 'recording'" />
      </section>
    </transition>
  </main>
</template>

