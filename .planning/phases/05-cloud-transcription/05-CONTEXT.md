# Phase 5: Cloud Transcription - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver working transcription via OpenAI, Groq, and OpenRouter cloud providers. Audio encoded by Phase 3 is sent to the active provider's API, and the transcribed text is returned to the pipeline. This phase covers the provider implementations, API call mechanics, error handling, retry logic, and provider fallback. Settings UI for transcription configuration is Phase 8.

</domain>

<decisions>
## Implementation Decisions

### Language hint
- Single shared `language` field (existing `TranscriptionConfig.language`) applies to all providers.
- Empty/unset language → omit the parameter entirely from the API request (let provider auto-detect). Never send an empty string.

### Provider fallback (CLOD-09)
- When active provider fails after exhausting auto-retries: offer one fallback attempt via toast.
- Fixed priority order for fallback: **OpenAI → Groq → OpenRouter** (skip providers without an API key configured).
- Toast shows next available provider: "Try with Groq?" with a single action button.
- If fallback provider also fails: surface the error with no further fallback offers.
- Fallback priority is configurable — add `fallback_order: Vec<TranscriptionProvider>` to `TranscriptionConfig` with the default order above. Settings UI to change it comes in Phase 8.

### Invalid API key UX (CLOD-06)
- 401/403 from any provider → toast with "Open Settings" action button.
- Clicking the action opens the settings window focused on the Transcription tab with the offending provider highlighted.
- **Invalid key bypasses the fallback flow entirely** — it's a configuration error, not a transient failure.

### Error flow sequencing (CLOD-07, CLOD-08, CLOD-09)
- **Rate limits (429):** Silent exponential backoff, up to 3 retries. Floating indicator stays in processing state throughout. Toast only appears if all 3 retries fail.
- **Network errors:** Auto-retry with backoff (3 attempts). If all fail → show toast with manual "Retry" button.
- **Manual retry (CLOD-08):** Resets the retry counter — treated as a fresh request, not counted against the auto-retry budget.
- **Fallback trigger:** After auto-retries exhaust (for transient errors only — network, rate-limit, server errors). No intermediate manual retry step before fallback offer.
- **Error classification:**
  - 401/403 → invalid key path (settings toast, no fallback)
  - 429 → rate-limit path (silent backoff, then fallback offer)
  - Network/timeout/5xx → network-error path (backoff, then fallback offer)

### Claude's Discretion
- Exact exponential backoff intervals (e.g., 1s, 2s, 4s) within the 3-retry budget.
- Whether to emit a `transcription-error` event or fold into the existing `audio-error` event pattern.
- HTTP client choice (reqwest is already in the dependency graph).
- OpenRouter `input_audio` content structure (base64 encoding, MIME type, message format).

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src-tauri/src/config/mod.rs`: `TranscriptionConfig` already has `provider`, API keys (openai/groq/openrouter), model fields, and a shared `language` field. Needs a new `fallback_order` field.
- `src-tauri/src/config/mod.rs`: `TranscriptionProvider` enum (`Openai`, `Groq`, `Openrouter`, `Local`) — use directly.
- `src-tauri/src/audio/mod.rs`: `stop_recording_and_encode()` returns `EncodedAudio` with `.bytes` (Vec<u8>) and `.format` (Opus/WAV). Phase 5 consumes this as its input.
- `src-tauri/src/audio/encode.rs`: `format_for_provider()` already maps `Openai`/`Groq`/`Openrouter` → Opus, `Local` → WAV.
- `src-tauri/src/state.rs`: `RecordingState::Transcribing` already exists — transition into it when transcription starts.

### Established Patterns
- Thin command handlers in `src-tauri/src/commands/`; business logic in service modules.
- Rust emits frontend events via `app.emit()` — existing events: `audio-warning`, `audio-error`, `hotkey-warning`. Add `transcription-error` and `transcription-done` following the same pattern.
- Error payloads use `#[serde(rename_all = "snake_case")]` enum codes + message string.
- `AppState` is the single source of truth; transcription service should read provider config from `state.config`.

### Integration Points
- `src-tauri/src/audio/mod.rs`: `stop_recording_and_encode()` is the handoff point — transcription starts immediately after it returns `EncodedAudio`.
- `src-tauri/src/hotkey/service.rs`: The hotkey toggle stop path needs to chain into transcription after encoding.
- `src-tauri/src/transcription/` directory: Create here, following the `TranscriptionProvider` trait pattern described in CLAUDE.md.
- `src-tauri/src/state.rs`: Add cancellation flag consumption for transcription (already has `cancel_flag` for injection — check if it applies here too).

</code_context>

<specifics>
## Specific Ideas

- Fallback order must be stored in config (not hardcoded) so Phase 8 can expose a setting for it.
- The "Open Settings" action on the invalid key toast needs the frontend to support navigating to a specific provider tab — coordinate with Phase 8 settings window design.

</specifics>

<deferred>
## Deferred Ideas

- Settings UI for fallback provider order — Phase 8.
- Per-provider language hints — explicitly decided against (shared language is fine).

</deferred>

---

*Phase: 05-cloud-transcription*
*Context gathered: 2026-03-17*
