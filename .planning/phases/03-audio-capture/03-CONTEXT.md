# Phase 3: Audio Capture - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver microphone capture for toggle recording, device selection/fallback handling, and correct recording output encoding for downstream transcription paths. This phase includes capture startup latency, capture stop behavior, and encode handoff contracts. Indicator rendering, provider API calls, injection behavior, and full settings UX remain outside this phase.

</domain>

<decisions>
## Implementation Decisions

### Capture lifecycle
- Start capture immediately when state enters `Recording`; do not block state transition on pre-validation.
- For Phase 3 runtime behavior, stop is manual-only (second hotkey); no silence-based auto-stop active yet.
- Short recordings should not continue to transcription: show a warning and drop the run.
- At stop, include a small trailing audio tail to reduce final-word clipping.

### Device behavior
- Persist selected mic by device name; if not found at runtime, fall back to system default and notify.
- Device list source is raw OS input device names from capture backend.
- If selected device disconnects during recording, auto-switch to system default and continue recording with warning.
- If no microphone is available at start, show an error notification and remain in `Idle`.

### Encoding contract
- Choose encoding format from active transcription provider at stop time.
- Cloud path outputs `Ogg/Opus`.
- Local path outputs `16-bit PCM WAV`.
- On encoding failure, retry once; if retry fails, show an error and abort the run.

### Carry-forward to Phase 8 settings
- Add Audio controls for `Auto-stop on silence` (toggle) and silence-duration value.
- Keep these backend-managed only in Phase 3; expose UI controls in Phase 8.

### Claude's Discretion
- Exact short-audio minimum threshold value.
- Exact trailing-tail duration value.
- Exact notification copy and severity styling.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src-tauri/src/state.rs`: existing `RecordingState` (`Idle`, `Recording`, `Transcribing`) and managed app state wiring.
- `src-tauri/src/config/mod.rs`: `AudioConfig` already includes `device`, `vad_threshold`, and `vad_silence_ms` defaults.
- `src-tauri/src/hotkey/service.rs`: toggle state transition path and stop-trigger integration point.
- `src-tauri/src/commands/config.rs` + `src/composables/useConfig.ts`: immediate config persistence path for audio settings.
- `src/types/index.ts`: TS mirror of `AudioConfig` fields already present.

### Established Patterns
- Tauri command handlers stay thin; business logic belongs in backend service modules.
- Runtime behavior is driven via managed `AppState` plus tray/hotkey events.
- Config is persisted immediately with typed struct + raw JSON preservation.

### Integration Points
- Add capture lifecycle orchestration to backend audio module and invoke from hotkey toggle transitions.
- Extend config command/read paths only as needed for device enumeration and audio setting updates.
- Emit frontend events for capture/device/encoding failures using the existing event pattern.

</code_context>

<specifics>
## Specific Ideas

- Prioritize reliable "press hotkey -> recording starts now" feel over preflight checks.
- Preserve dictation flow on device loss by switching to default mic instead of hard failing mid-sentence.
- Keep phase scope strict: runtime behavior now, full user control surface in Phase 8.

</specifics>

<deferred>
## Deferred Ideas

- User-facing Audio UI controls for silence auto-stop toggle and silence duration (Phase 8).

</deferred>

---

*Phase: 03-audio-capture*
*Context gathered: 2026-03-15*
