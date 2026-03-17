---
phase: 05-cloud-transcription
verified: 2026-03-18T10:00:00Z
status: passed
score: 17/17 must-haves verified
re_verification:
  previous_status: passed
  previous_score: 15/15
  gaps_closed:
    - "OpenAI Whisper API accepts the audio file — format_for_provider routes Openai to EncodedFormat::Wav"
    - "Network error toast shows a friendly human-readable message, not a raw reqwest URL error"
    - "Invalid API key triggers an error toast with an Open Settings button — Settings window does NOT open automatically"
    - "Fallback provider toast appears when primary provider fails (invalid_key no longer short-circuits via return)"
    - "Network error toast is visually styled (card background, coloured border, readable text)"
    - "Retry button is visible and clickable in the toast"
  gaps_remaining: []
  regressions: []
---

# Phase 5: Cloud Transcription Verification Report

**Phase Goal:** Transcribed text is returned from any of the three configured cloud providers, with graceful handling of errors, rate limits, and provider fallback
**Verified:** 2026-03-18T10:00:00Z
**Status:** PASSED
**Re-verification:** Yes — after gap closure (plans 05-05 and 05-06)

---

## Goal Achievement

### Observable Truths

The five success criteria from ROADMAP.md, the three gap-closure truths from plan 05-04, plus six new truths verified from plans 05-05 and 05-06:

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Audio is successfully transcribed via OpenAI, Groq, and OpenRouter using user's API keys and chosen models | VERIFIED | `openai.rs`, `groq.rs`, `openrouter.rs` each implement `TranscriptionProviderTrait`; multipart and JSON shapes match API docs; endpoints confirmed by constant tests |
| 2 | Each provider's API key, model selection, and language hint are independently configurable and persist | VERIFIED | `TranscriptionConfig` has per-provider fields (`openai_api_key`, `groq_api_key`, `openrouter_api_key`, `openai_model`, `groq_model`, `openrouter_model`, `language`); all have `#[serde(default)]` |
| 3 | An invalid API key triggers a toast with an "Open Settings" button — Settings window does NOT open automatically | VERIFIED | `invalid_key` branch in `App.vue` lines 172-184 now calls `showTranscriptionErrorToast` with `action.onClick` invoking `open_settings_on_transcription_tab`; direct `invoke('open_settings_on_transcription_tab')` is gone from the hot path |
| 4 | Rate-limit errors (429) retry with exponential backoff up to 3 times before surfacing an error | VERIFIED | `run_with_retry_inner` implements 1s/2s/4s backoff; `transcribe_with_retry` uses `attempt < 3` guard; `test_retry_three_times_on_rate_limit` confirms 4 total calls |
| 5 | If the active provider fails after retries and another provider is configured, a toast offers to retry with the fallback provider | VERIFIED | `find_fallback_provider` skips current and empty-key providers; `transcribe_with_retry` emits `fallback_provider` field when exhausted; frontend shows "Try with X?" button calling `retry_transcription_with_fallback`; `invalid_key` branch no longer returns before the `fallback_provider` path |
| 6 | Stopping recording does not open the Settings window uninvited | VERIFIED | Both `emit_hotkey_warning` call sites (line 97 Startup, line 198 Save) pass `focus_settings=false`; confirmed in `src-tauri/src/hotkey/service.rs` |
| 7 | On transcription error, indicator stays visible and shows error toast with Retry button | VERIFIED | `Err(())` branch in async spawn calls `indicator::show_idle` (not `indicator::hide`); indicator window remains visible; `indicator-toast-action` CSS class styled in `styles.css` |
| 8 | Indicator hides after toast is dismissed or times out | VERIFIED | `handleDismissToast` invokes `hide_indicator` when `toasts.value.length === 0`; `showTranscriptionErrorToast` sets a `setTimeout(ms + 50)` that also invokes `hide_indicator` after auto-expiry |
| 9 | OpenAI Whisper API accepts the audio file (no 400 Invalid file error) | VERIFIED | `format_for_provider` line 61: `TranscriptionProvider::Local \| TranscriptionProvider::Openai => EncodedFormat::Wav`; `openai.rs` file part uses `audio.wav` and `audio/wav`; 4 unit tests in `encode.rs` assert all provider-to-format mappings |
| 10 | Network error toast shows a friendly human-readable message, not a raw reqwest URL error | VERIFIED | Both Network match arms in `service.rs` (lines 239-242 and 310-312) emit `"Network error. Check your connection and try again.".into()` instead of `message.clone()` |
| 11 | Error toasts are visually styled (card background, coloured border, readable text) | VERIFIED | `src/styles.css` lines 202-290 define `.indicator-toasts`, `.indicator-toast`, `.indicator-toast--error`, `.indicator-toast--info`, `.indicator-toast--success`, `.indicator-toast-message`, `.indicator-toast-action`, `.indicator-toast-dismiss` with dark card backgrounds and coloured borders |

**Score:** 11/11 truths verified

---

### Required Artifacts — Three-Level Verification

#### Plan 01 Artifacts

| Artifact | Exists | Substantive | Wired | Status |
|----------|--------|-------------|-------|--------|
| `src-tauri/src/transcription/provider.rs` | Yes | `TranscriptionProviderTrait` trait, `TranscriptionError` enum (5 variants), `status_to_error`, `classify_and_extract`; 5 unit tests | Imported by `openai.rs`, `groq.rs`, `openrouter.rs`, `service.rs` | VERIFIED |
| `src-tauri/src/transcription/openai.rs` | Yes | `OpenAiProvider` struct, `TranscriptionProviderTrait` impl, multipart form with `file`, `model`, `response_format`, conditional `language`; file_name="audio.wav", mime_str="audio/wav" (updated plan 05-05); 3 unit tests | Imported by `service.rs` via `make_provider` | VERIFIED |
| `src-tauri/src/transcription/groq.rs` | Yes | Identical shape to OpenAI, Groq endpoint and model field; 3 unit tests | Imported by `service.rs` via `make_provider` | VERIFIED |
| `src-tauri/src/transcription/openrouter.rs` | Yes | `build_body` with base64 encoding, `format:"ogg"`, language instruction appending, empty-model guard, chat completions JSON parsing; 4 unit tests | Imported by `service.rs` via `make_provider` | VERIFIED |
| `src-tauri/src/config/mod.rs` — `fallback_order` field | Yes | `fallback_order: Vec<TranscriptionProvider>` with `#[serde(default = "default_fallback_order")]`, default fn returns `[Openai, Groq, Openrouter]`; `FromStr` impl | Read in `service.rs::find_fallback_provider` and `transcribe_with_retry` | VERIFIED |

#### Plan 02 Artifacts

| Artifact | Exists | Substantive | Wired | Status |
|----------|--------|-------------|-------|--------|
| `src-tauri/src/transcription/service.rs` | Yes | `transcribe_with_retry`, `transcribe_with_provider`, `run_with_retry_inner`, `make_provider`, `find_fallback_provider`, event constants, payload types; both Network arms emit friendly string (updated plan 05-05); 6 unit tests | Re-exported from `transcription/mod.rs`; called from `hotkey/service.rs` and `commands/transcription.rs` | VERIFIED |
| `src-tauri/src/transcription/mod.rs` — `pub mod service` | Yes | All 5 submodules declared; `transcribe_with_retry` and `transcribe_with_provider` re-exported | Consumed by callers as `transcription::transcribe_with_retry` | VERIFIED |

#### Plan 03 Artifacts

| Artifact | Exists | Substantive | Wired | Status |
|----------|--------|-------------|-------|--------|
| `src-tauri/src/hotkey/service.rs` — real transcription spawn | Yes | `audio::stop_recording_and_encode` stores `last_encoded_audio`, calls `indicator::show_processing`, spawns `transcription::transcribe_with_retry`; both Ok and Err paths reset `RecordingState::Idle` | `use crate::transcription` at top; spawned via `tauri::async_runtime::spawn` | VERIFIED |
| `src-tauri/src/commands/transcription.rs` | Yes | `retry_transcription` reads `last_encoded_audio` and calls `transcription::transcribe_with_retry`; `retry_transcription_with_fallback` parses provider via `FromStr`, calls `transcription::transcribe_with_provider`; `open_settings_on_transcription_tab` calls `tray::show_settings_window` | Registered in `lib.rs` invoke_handler (lines 81–83) | VERIFIED |
| `src-tauri/src/transcription/service.rs` — `transcribe_with_provider` | Yes | Clones `TranscriptionConfig`, overrides `provider` field, calls `make_provider`, dispatches single attempt without retry, emits `TRANSCRIPTION_DONE_EVENT` or full `TranscriptionErrorPayload` with `fallback_provider: None` | Re-exported from `mod.rs`; called from `commands/transcription.rs` | VERIFIED |
| `src/windows/indicator/App.vue` — transcription-error listener | Yes | `listen<TranscriptionErrorPayload>('transcription-error', handler)` registered in `onMounted`; unlisten stored and called in `onBeforeUnmount`; `invalid_key` shows toast with "Open Settings" button (updated plan 05-06); `fallback_provider`, `retryable`, and `cancelled` paths all handled | Wired to `invoke('open_settings_on_transcription_tab')`, `invoke('retry_transcription_with_fallback')`, `invoke('retry_transcription')` | VERIFIED |

#### Plan 04 Artifacts (gap closure)

| Artifact | Exists | Substantive | Wired | Status |
|----------|--------|-------------|-------|--------|
| `src-tauri/src/hotkey/service.rs` — `focus_settings=false` at both call sites | Yes | Line 97 (Startup): `emit_hotkey_warning(app, &warning, HotkeyWarningSource::Startup, false)`. Line 198 (Save): `emit_hotkey_warning(app, &warning, HotkeyWarningSource::Save, false)`. Err(()) branch: `let _ = indicator::show_idle(&app_clone)` at line 275 | Both edits confirmed in file; `show_idle` call replaces previous `hide` call | VERIFIED |
| `src-tauri/src/commands/indicator.rs` — `hide_indicator` command | Yes | `#[tauri::command] pub fn hide_indicator(app: AppHandle) -> Result<(), String>` calls `indicator::hide(&app)` | Registered in `lib.rs` `generate_handler!` | VERIFIED |
| `src-tauri/src/lib.rs` — `hide_indicator` in invoke_handler | Yes | `commands::indicator::hide_indicator` present in `tauri::generate_handler!` macro | Consumed by frontend via `invoke("hide_indicator")` | VERIFIED |
| `src/windows/indicator/App.vue` — auto-hide after toast lifecycle | Yes | `handleDismissToast` calls `dismissToast` + `invoke("hide_indicator")` when `toasts.value.length === 0`; `showTranscriptionErrorToast` wraps `showToast` + `setTimeout(ms + 50)`; template uses `handleDismissToast` for both action-button and X-dismiss paths; all error branches use `showTranscriptionErrorToast` | `hide_indicator` invoked from two code paths (manual dismiss and timer expiry) | VERIFIED |

#### Plan 05 Artifacts (gap closure — audio format + friendly network error)

| Artifact | Exists | Substantive | Wired | Status |
|----------|--------|-------------|-------|--------|
| `src-tauri/src/audio/encode.rs` — `format_for_provider` routes Openai to Wav | Yes | Line 61: `TranscriptionProvider::Local \| TranscriptionProvider::Openai => EncodedFormat::Wav`; line 62: `TranscriptionProvider::Groq \| TranscriptionProvider::Openrouter => EncodedFormat::Opus`; 4 unit tests (`test_format_for_provider_openai_returns_wav`, `_groq_returns_opus`, `_openrouter_returns_opus`, `_local_returns_wav`) | `format_for_provider` consumed by `encode_for_provider` which is called from `audio/mod.rs` recording pipeline | VERIFIED |
| `src-tauri/src/transcription/openai.rs` — WAV multipart file part | Yes | Line 56: `.file_name("audio.wav")`; line 57: `.mime_str("audio/wav")` | File part assembled in `transcribe` and sent via `.multipart(form)` POST | VERIFIED |
| `src-tauri/src/transcription/service.rs` — friendly Network error messages | Yes | Lines 239-242: `TranscriptionError::Network { .. } => (TranscriptionErrorCode::Network, "Network error. Check your connection and try again.".into())`; lines 310-312: same in `transcribe_with_provider` | Both message strings flow to `TranscriptionErrorPayload.message` field emitted to frontend | VERIFIED |

#### Plan 06 Artifacts (gap closure — frontend toast bugs)

| Artifact | Exists | Substantive | Wired | Status |
|----------|--------|-------------|-------|--------|
| `src/windows/indicator/App.vue` — `invalid_key` shows toast, not auto-open | Yes | Lines 172-184: `if (payload.code === "invalid_key")` calls `showTranscriptionErrorToast({ message: "Invalid API key. Open Settings to fix.", type: "error", action: { label: "Open Settings", onClick: () => { void invoke("open_settings_on_transcription_tab", { provider }); } } })`; NO direct `invoke` call at the top level of the branch | `showTranscriptionErrorToast` is defined and imported in the same component; `invoke` is called only inside the action `onClick` closure | VERIFIED |
| `src/styles.css` — full toast CSS ruleset | Yes | Lines 202-290 define `.indicator-toasts` (position/layout), `.indicator-toast` (card base), `.indicator-toast--error` (red border/dark-red bg), `.indicator-toast--info`, `.indicator-toast--success`, `.indicator-toast-message`, `.indicator-toast-action` (button with hover), `.indicator-toast-dismiss` (X button with hover) | Classes referenced directly in `App.vue` template DOM nodes; no inline styles override these | VERIFIED |

---

### Key Link Verification

| From | To | Via | Status | Evidence |
|------|----|-----|--------|----------|
| `openai.rs` | `https://api.openai.com/v1/audio/transcriptions` | reqwest multipart POST with WAV | WIRED | `OPENAI_ENDPOINT` confirmed by `test_openai_endpoint_constant`; `.multipart(form)` call present; file_name="audio.wav", mime_str="audio/wav" (plan 05-05 fix) |
| `openrouter.rs` | `input_audio` base64 payload with `format:"ogg"` | `base64::engine::general_purpose::STANDARD.encode` | WIRED | `build_body` encodes bytes, sets `"format":"ogg"`; confirmed by `test_format_is_ogg_not_mime` |
| `encode.rs::format_for_provider` | `openai.rs` (WAV routing) | `EncodedFormat::Wav` returned for `Openai` variant | WIRED | `format_for_provider(Openai)` returns `Wav`; `encode_for_provider` uses `Wav` arm to produce `audio/wav` bytes; `openai.rs` consumes the `bytes` field directly |
| `service.rs` | `app.emit("transcription-error", payload)` | `tauri::Emitter` | WIRED | `use tauri::Emitter` imported; `app.emit(TRANSCRIPTION_ERROR_EVENT, ...)` called in 4 distinct code paths; Network arms emit friendly string (plan 05-05 fix) |
| `service.rs` | `fallback_order` in `TranscriptionConfig` | `config.transcription.fallback_order` iteration | WIRED | `find_fallback_provider` iterates `config.fallback_order`; called in `transcribe_with_retry` retry-exhausted branch |
| `hotkey/service.rs` | `transcription::transcribe_with_retry` | `tauri::async_runtime::spawn` | WIRED | `use crate::transcription` at top; `transcription::transcribe_with_retry(&app_clone, &encoded).await` inside spawn closure |
| `commands/transcription.rs` | `transcription::transcribe_with_provider` | direct call with target `TranscriptionProvider` | WIRED | `transcription::transcribe_with_provider(&app, &audio, target)` with parsed `target_provider` |
| `App.vue` | `transcription-error` event | `listen('transcription-error', handler)` | WIRED | `unlistenTranscriptionError = await listen<TranscriptionErrorPayload>('transcription-error', ...)` |
| `App.vue invalid_key branch` | `invoke('open_settings_on_transcription_tab')` | toast action button `onClick` (plan 05-06 fix) | WIRED | `action.onClick` calls `invoke("open_settings_on_transcription_tab", { provider })`; direct auto-invoke removed |
| `App.vue dismissToast` | `invoke("hide_indicator")` | `handleDismissToast` + `showTranscriptionErrorToast` timer | WIRED | `handleDismissToast` calls `invoke("hide_indicator")` when `toasts.value.length === 0`; timer path fires at `ms + 50` |
| `hotkey/service.rs` Err(()) branch | `indicator::show_idle` | direct call | WIRED | `let _ = indicator::show_idle(&app_clone)` at line 275; keeps indicator visible for JS event delivery |
| `styles.css` toast classes | App.vue DOM template | CSS class selectors | WIRED | `.indicator-toast`, `.indicator-toast--error`, `.indicator-toast-action`, `.indicator-toast-dismiss` all defined in `styles.css` and referenced in App.vue template |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| CLOD-01 | 05-01, 05-05 | App supports three cloud providers: OpenAI, Groq, and OpenRouter | SATISFIED | Three concrete provider structs each implementing `TranscriptionProviderTrait`; OpenAI now uses WAV (not Ogg) so API actually accepts the audio (plan 05-05) |
| CLOD-02 | 05-01 | Each provider has its own API key, model selection, and language hint | SATISFIED | `TranscriptionConfig` has `openai_api_key`, `groq_api_key`, `openrouter_api_key`, `openai_model`, `groq_model`, `openrouter_model`, `language` — all serialized to config.json |
| CLOD-03 | 05-01, 05-05 | OpenAI sends to `/v1/audio/transcriptions` with correct models and accepted audio format | SATISFIED | `OPENAI_ENDPOINT` constant; file_name="audio.wav", mime_str="audio/wav"; `format_for_provider(Openai)` returns `Wav` |
| CLOD-04 | 05-01 | Groq sends to its transcription endpoint with correct models | SATISFIED | `GROQ_ENDPOINT = "https://api.groq.com/openai/v1/audio/transcriptions"`, multipart form with `model = config.groq_model`; default "whisper-large-v3" |
| CLOD-05 | 05-01 | OpenRouter sends to chat completions endpoint with audio as input_audio | SATISFIED | `OPENROUTER_ENDPOINT = "https://openrouter.ai/api/v1/chat/completions"`, base64-encoded audio as `input_audio` with `format:"ogg"` |
| CLOD-06 | 05-02, 05-03, 05-04, 05-06 | Invalid API key errors prompt user to open settings with provider tab highlighted; hotkey warning does not open settings uninvited | SATISFIED | `invalid_key` branch shows toast with "Open Settings" action button (plan 05-06); both `emit_hotkey_warning` call sites use `focus_settings=false` (plan 05-04) |
| CLOD-07 | 05-02 | Rate limit errors (429) retry with exponential backoff, max 3 retries | SATISFIED | `run_with_retry_inner` implements 1s/2s/4s backoff; `transcribe_with_retry` uses `attempt < 3`; test confirms 4 total calls |
| CLOD-08 | 05-02, 05-03, 05-04, 05-05, 05-06 | Network errors show a notification with a retry button; toast is visible and styled in indicator | SATISFIED | `Network`/`Server` errors emit `transcription-error` with `retryable=true`; friendly message string (plan 05-05); toast CSS renders card with coloured border (plan 05-06); indicator stays visible via `show_idle` (plan 05-04) |
| CLOD-09 | 05-02, 05-03, 05-05, 05-06 | If active provider fails after retries and another is configured, offer fallback via toast action | SATISFIED | `find_fallback_provider` finds next configured provider; `fallback_provider` in error payload; frontend shows "Try with X?" button; `invalid_key` branch no longer blocks this path (plan 05-06) |
| CLOD-10 | 05-01 | User can configure a language hint per provider (or leave on auto-detect) | SATISFIED | `language: String` in `TranscriptionConfig` with `#[serde(default)]` (empty = auto); all three providers check `config.language.is_empty()` |

All 10 requirements SATISFIED. No orphaned requirements.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `transcription/service.rs` | 58 | `unimplemented!("local transcription is phase 10")` | Info | Expected deferred stub — Local transcription is Phase 10 scope; `make_provider` panics only if `Local` is dispatched, which cannot happen in Phase 5 cloud-only configuration |
| `hotkey/service.rs` | 270 | Comment: "Phase 6 will handle injection here." | Info | Correct phase boundary comment; `show_injecting` is called as visual placeholder until Phase 6 wires injection; this is documented scope |

No blocker or warning anti-patterns. The two info items are correctly scoped to future phases.

---

### Human Verification Required

The following items cannot be verified without a running app and real API keys:

#### 1. End-to-end cloud transcription with real keys (OpenAI WAV path)

**Test:** Configure a valid OpenAI API key, press hotkey twice (with actual microphone audio), observe transcription result.
**Expected:** Audio is sent as WAV; `transcription-done` event fires with recognized text; indicator transitions from processing to injecting state. No 400 "Invalid file" error.
**Why human:** Requires real API credentials, live audio capture, network round-trip, and WAV acceptance by the API.

#### 2. Settings window does NOT open on startup hotkey conflict

**Test:** Launch the app with a hotkey already registered by another application.
**Expected:** No Settings window opens automatically. A `hotkey-warning` toast may appear elsewhere but the Settings window stays closed.
**Why human:** Requires a controlled hotkey conflict scenario and live Tauri app observation.

#### 3. Invalid API key shows toast, not auto-opened Settings

**Test:** Configure an invalid OpenAI API key, trigger recording, observe UI response.
**Expected:** Error toast appears with "Invalid API key. Open Settings to fix." and an "Open Settings" button. The Settings window stays closed until the user clicks the button.
**Why human:** Requires live Tauri app, real API call returning 401, UI interaction observation.

#### 4. Error toast is visually styled in the indicator

**Test:** Disconnect the network, trigger a recording, observe indicator behavior after transcription fails.
**Expected:** Indicator stays visible in idle state. Error toast appears with dark card background, red border, readable text, "Retry" button. After toast auto-dismisses (~5s) or user clicks X, indicator hides.
**Why human:** Requires controlled network disconnection, live indicator rendering observation, and timing verification.

#### 5. Fallback "Try with X?" toast appears and works

**Test:** Configure OpenAI with invalid key and Groq with valid key; trigger recording; let OpenAI fail 3 retries.
**Expected:** Toast appears with "Try with groq?" button; clicking it triggers Groq transcription; fallback branch is not blocked by the `invalid_key` early return (plan 05-06 fix).
**Why human:** Requires live app, controlled API failures, and visual toast observation.

---

### Gaps Summary

No gaps found. All phase 5 must-haves (original 13 + 2 from plan 05-04 + 2 from plan 05-05 + 2 from plan 05-06) are implemented, substantive, and wired. All 10 requirements are satisfied.

The six UAT-diagnosed gaps are closed:
- Plan 05-04: `emit_hotkey_warning` no longer opens the Settings window uninvited; indicator stays visible on error via `show_idle`; indicator auto-hides after all toasts clear
- Plan 05-05: OpenAI audio format routed to WAV (`format_for_provider` fix); both Network error arms in `service.rs` emit a friendly user-facing string
- Plan 05-06: `invalid_key` handler now shows a toast with an "Open Settings" action button (not auto-invoke); `styles.css` contains the full toast CSS block

The two deferred stubs (`Local` provider dispatch and Phase 6 injection wiring) are correctly scoped to future phases and do not block the Phase 5 goal.

Human verification items 1–5 remain pending until the app can be run with real API keys or controlled failure conditions.

---

_Verified: 2026-03-18T10:00:00Z_
_Verifier: Claude (gsd-verifier)_
