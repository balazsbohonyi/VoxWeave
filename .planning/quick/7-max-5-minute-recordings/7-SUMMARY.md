---
phase: quick-7
plan: 01
subsystem: audio, indicator
tags: [recording-limits, auto-stop, ux, toast]
dependency_graph:
  requires: []
  provides: [5-minute recording hard cap wired end-to-end]
  affects: [src-tauri/src/audio/mod.rs, src/windows/indicator/App.vue]
tech_stack:
  added: []
  patterns: [event-driven stop pipeline (mirrors vad-silence-stop pattern)]
key_files:
  modified:
    - src-tauri/src/audio/mod.rs
    - src/windows/indicator/App.vue
decisions:
  - Reused trigger_stop_recording invoke pattern from vad-silence-stop for recording-limit-stop handler
  - show_plain_toast invoke with toastType/message args for near-limit warning (existing Tauri command)
metrics:
  duration: 2m
  completed: 2026-04-03
  tasks_completed: 2
  files_modified: 2
---

# Phase quick-7 Plan 01: Max 5-Minute Recordings Summary

**One-liner:** Wired 5-minute recording hard cap end-to-end — Rust emits `recording-limit-stop` at cap, indicator triggers full stop pipeline; near-limit event now shows warning toast at 4.5 minutes.

## What Was Built

The 5-minute recording limit was partially implemented in Rust (buffer truncation + stop_flag) but neither the near-limit warning nor the hard-cap stop were connected to the frontend. This plan wired both events:

1. **Rust (`audio/mod.rs`):** Added `app.emit("recording-limit-stop", ())` immediately after `stop_flag.store(true)` in the hard-cap branch of `accumulate_pcm_chunk`. The stop_flag exits the capture thread; the new emit triggers the frontend to invoke the full stop/transcribe/inject pipeline.

2. **Frontend (`indicator/App.vue`):** Added two event listeners in `onMounted`:
   - `recording-near-limit` → `invoke("show_plain_toast", { toastType: "warning", message: "Recording will stop in 30 seconds (5-minute limit reached)." })`
   - `recording-limit-stop` → `invoke("trigger_stop_recording")` (same as `vad-silence-stop` pattern)
   Both unlisten refs are cleaned up in `onBeforeUnmount`.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Emit recording-limit-stop from Rust hard cap | f66f97c |
| 2 | Wire recording-near-limit and recording-limit-stop in indicator | 6748e5c |

## Verification

- `cargo check` — no errors
- `npx vue-tsc --noEmit` — no errors

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

- `src-tauri/src/audio/mod.rs` modified: confirmed
- `src/windows/indicator/App.vue` modified: confirmed
- Commit f66f97c exists: confirmed
- Commit 6748e5c exists: confirmed
