// useRecording — reactive recording state and audio level.
// Placeholder: full implementation in Phase 3 (Recording).
import { ref } from "vue";
import type { RecordingState } from "../types/index";

export function useRecording() {
  const recordingState = ref<RecordingState>("idle");
  const audioLevel = ref(0); // 0.0 – 1.0 RMS

  // TODO (Phase 3): listen to "state-changed" and "audio-level" events,
  // expose startRecording() / stopRecording() invoking Rust commands.

  return { recordingState, audioLevel };
}
