# Phase 10: Local Transcription - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Integrate whisper.cpp (via whisper-rs, behind the `local-transcription` cargo feature) so users can transcribe offline without any network call. Includes on-demand model download from Hugging Face with live progress, and replacing the Step2Local.vue placeholder with a real model picker. Cloud pipeline is untouched.

</domain>

<decisions>
## Implementation Decisions

### Download persistence
- Download continues as a Rust background task regardless of which window is open or closed
- When the user reopens Settings (or the wizard) mid-download, the model card shows the current live progress bar — no reconnection needed beyond listening to the existing event stream
- App quit = download cancelled immediately + partial file deleted (no resume across launches)
- Only one model downloads at a time; while a download is in progress, all other Download buttons are disabled

### Download UX
- During download: the Download button is replaced by a full-width progress bar, a percentage number, and a Cancel button
- Cancel: stops download immediately and deletes the partial file; model card returns to "Download" state
- Download source: Hugging Face GGML files (`https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{model}.bin`)
- Progress events emitted from Rust via the existing `emit()` pattern (same as `audio-level` events)

### Wizard Step 2 Local
- Step2Local.vue becomes a full model picker showing all 4 models (tiny/base/small/medium) with size/quality hints and a Download button per card
- Inline download works the same as Settings: progress bar + percentage + Cancel replaces the Download button
- Next button is disabled until either a model is fully downloaded OR the user clicks "Skip for now"
- After Finish with no model downloaded (skipped): a toast nudges the user to open Settings to download one
- When wizard is re-opened from Settings with a model already active: that model card shows an "Active" checkmark; other models show Download

### Missing/corrupt model error handling
- Missing model on recording attempt: abort the recording immediately, show error toast "No local model downloaded." with an "Open Settings" action button (matches existing invalid-key toast pattern)
- Corrupt model (whisper-rs fails to load): treated identically to missing — same error toast, same "Open Settings" action
- No SHA256 verification at download time — detect corruption on load, not upfront

### Claude's Discretion
- Exact Tailwind styling of model cards, progress bar color, and Active badge design
- Specific whisper-rs API usage for model loading and WAV input
- Tokio channel vs atomic flag for download cancellation signal
- How to wire the `local-transcription` feature flag in Cargo.toml without breaking the non-feature build

</decisions>

<specifics>
## Specific Ideas

- Model picker in wizard matches the mockup discussed: 4 rows, each with model name, size, quality hint, and a Download/progress/Cancel control on the right
- "Active" state in the wizard (re-open path) uses a checkmark badge — same visual language as the "Set as active" flow in Settings cloud tabs

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TranscriptionSection.vue`: already has `LOCAL_MODELS` array (tiny/base/small/medium with labels/sizes/quality strings) and stub download button cards — replace stubs with real download logic
- `TranscriptionProviderTrait`: local impl will implement this trait, same as OpenAI/Groq impls
- `useConfig()` + `saveConfig()`: use to persist `transcription.local_model` (active model path) after download
- Tauri `emit()` pattern: use for download progress events (same as `audio-level` at ~30fps for audio)

### Established Patterns
- `std::thread` for CPU-bound work (locked in CLAUDE.md): whisper-rs inference runs on `std::thread`, not Tokio
- Feature-gating: `whisper-rs` behind `local-transcription` cargo feature — build must succeed without the feature enabled
- Error toasts with action buttons: existing pattern from `invalid_key` toast — reuse for missing/corrupt model errors
- `TranscriptionError` enum in `provider.rs`: add a `ModelMissing` and/or `ModelLoadFailed` variant

### Integration Points
- `Step2Local.vue`: replace placeholder with model picker component (can share card logic with Settings)
- `TranscriptionSection.vue` local stubs: wire real `start_model_download`, `cancel_model_download` Rust commands
- New Rust commands needed: `start_model_download(model_id)`, `cancel_model_download()`, `get_downloaded_models()`
- `AppConfig` / `TranscriptionConfig`: persist active local model path in `local_model_path` (already exists as a config field per CONF-02)
- `service.rs`: add Local branch to the provider dispatch that constructs the whisper-rs provider
- `main.rs`: on quit, ensure any in-progress download task is cancelled and partial file cleaned up

</code_context>

<deferred>
## Deferred Ideas

- SHA256 checksum verification at download time — not in v1, detect corruption on load instead
- Resume-from-partial download across app restarts — requires byte-range HTTP, deferred to v2
- User-configurable download URL per model — out of scope, hardcoded Hugging Face URLs for v1

</deferred>

---

*Phase: 10-local-transcription*
*Context gathered: 2026-03-29*
