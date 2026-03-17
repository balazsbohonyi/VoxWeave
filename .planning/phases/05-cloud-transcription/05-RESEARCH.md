# Phase 5: Cloud Transcription - Research

**Researched:** 2026-03-17
**Domain:** Rust async HTTP (reqwest), multipart form uploads, OpenAI/Groq/OpenRouter STT APIs, retry/backoff patterns, Tauri event emission
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Language hint: single shared `TranscriptionConfig.language`; empty/unset → omit param entirely (never send empty string)
- Provider fallback (CLOD-09): fixed priority OpenAI → Groq → OpenRouter; skip providers without API key; toast shows next available provider; if fallback also fails, surface error with no further fallback
- Fallback order stored as `fallback_order: Vec<TranscriptionProvider>` in `TranscriptionConfig`; Phase 8 exposes UI for it
- Invalid API key (401/403): toast with "Open Settings" action button, opens settings on Transcription tab with offending provider highlighted; bypasses fallback entirely
- Error classification: 401/403 → invalid-key path; 429 → rate-limit path (silent backoff, fallback offer); network/timeout/5xx → network-error path (backoff, fallback offer)
- Rate limits (429): silent exponential backoff up to 3 retries; toast only if all 3 fail
- Network errors: auto-retry with backoff 3 attempts; toast with manual "Retry" button if all fail
- Manual retry (CLOD-08): resets counter, treated as fresh request, not counted against auto-retry budget
- Fallback trigger: after auto-retries exhaust (transient errors only); no intermediate manual retry before fallback offer

### Claude's Discretion
- Exact exponential backoff intervals (e.g., 1s, 2s, 4s) within 3-retry budget
- Whether to emit `transcription-error` or fold into existing `audio-error` event pattern
- HTTP client choice (reqwest is already in the dependency graph)
- OpenRouter `input_audio` content structure (base64 encoding, MIME type, message format)

### Deferred Ideas (OUT OF SCOPE)
- Settings UI for fallback provider order — Phase 8
- Per-provider language hints — explicitly decided against (shared language is fine)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CLOD-01 | App supports OpenAI, Groq, OpenRouter cloud providers | TranscriptionProvider trait pattern; reqwest multipart + JSON HTTP |
| CLOD-02 | Each provider has API key, model selection, language hint in settings | Config struct already has all fields; add `fallback_order` field |
| CLOD-03 | OpenAI sends to `/v1/audio/transcriptions`; models: whisper-1, gpt-4o-transcribe, gpt-4o-mini-transcribe | Verified: multipart/form-data with `file`, `model`, `language` fields |
| CLOD-04 | Groq sends to its transcription endpoint; models: whisper-large-v3-turbo (default), whisper-large-v3, distil-whisper-large-v3-en | Verified: `POST https://api.groq.com/openai/v1/audio/transcriptions` same multipart shape as OpenAI |
| CLOD-05 | OpenRouter chat completions with input_audio; specific model list | Verified: `/api/v1/chat/completions` with base64 `input_audio` content block; ogg/opus supported |
| CLOD-06 | Invalid API key → open settings with offending provider tab highlighted | Event payload must carry provider identity; frontend reads it to navigate tab |
| CLOD-07 | Rate limit 429 → exponential backoff, max 3 retries | Tokio sleep inside retry loop; silent (no UI feedback during retries) |
| CLOD-08 | Network errors → notification with retry button | `transcription-error` event with `retry_token` or provider context; frontend button re-invokes command |
| CLOD-09 | After retries fail and another provider configured → fallback toast | `fallback_order` config consumed at runtime; event carries next provider name |
| CLOD-10 | User can configure language hint per session (shared field) | Already in config; omit field when empty |
</phase_requirements>

---

## Summary

Phase 5 implements the cloud transcription pipeline in Rust. `EncodedAudio` bytes produced by Phase 3 are sent via HTTP to one of three providers; the transcribed text string is returned to the caller (hotkey service). All network I/O runs in a `tauri::async_runtime::spawn` task so the Tokio runtime is never blocked.

OpenAI and Groq share an identical multipart/form-data shape (`file`, `model`, optional `language`) to their respective `/v1/audio/transcriptions` endpoints. OpenRouter takes a different path: JSON chat completions with the audio embedded as base64 `input_audio` content alongside a text instruction. The Opus bytes produced by Phase 3 are valid for all three providers (ogg container with Opus codec accepted by all).

Error handling follows a strict classification: 401/403 → settings toast (no retry, no fallback); 429/5xx/network → exponential backoff up to 3 silent retries, then a toast offering either manual retry or provider fallback. The `reqwest` crate already present in the project handles all HTTP; `tokio::time::sleep` handles backoff delays.

**Primary recommendation:** Add `reqwest` with `multipart` + `json` features to Cargo.toml. Implement a `TranscriptionService` in `src-tauri/src/transcription/` with a `TranscriptionProvider` trait and three concrete impls. Wire it into `hotkey/service.rs` to replace `complete_transcription_placeholder`.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.12.x | Async HTTP client | Already a transitive dep via tauri; confirmed in crates.io; multipart + json features needed |
| tokio | 1.x | Async runtime | Already provided by Tauri; use `tauri::async_runtime::spawn` not `tokio::spawn` |
| base64 (standard) | built-in via std | Encode audio bytes for OpenRouter | `use std::io::Write; base64` or use the `base64` crate 0.22 |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| base64 | 0.22 | Base64 encode for OpenRouter payload | Only needed for OpenRouter provider impl |
| serde_json | 1.x (already present) | Build OpenRouter JSON body, parse all responses | All three providers return JSON |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| reqwest | hyper directly | reqwest is the right level of abstraction; hyper is lower level |
| reqwest | ureq (blocking) | ureq is sync-only; incompatible with Tokio async task pattern |
| base64 crate | manual encoding | base64 crate is standard; no reason to hand-roll |

**Installation (additions to Cargo.toml):**
```toml
reqwest = { version = "0.12", features = ["multipart", "json"] }
base64 = "0.22"
```

> Note: `reqwest` may already be a transitive dependency. Verify with `cargo tree | grep reqwest` before adding. If present at 0.11, upgrade to 0.12 for async multipart support.

---

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/
├── transcription/
│   ├── mod.rs           # TranscriptionService, retry loop, fallback logic, event emission
│   ├── provider.rs      # TranscriptionProvider trait + enum dispatch
│   ├── openai.rs        # OpenAI multipart impl
│   ├── groq.rs          # Groq multipart impl (near-identical to openai.rs)
│   └── openrouter.rs    # OpenRouter JSON chat completions impl
```

### Pattern 1: TranscriptionProvider Trait

**What:** A trait with a single async method that each provider implements. The service dispatches to the right impl based on `TranscriptionConfig.provider`.

**When to use:** Any time a new provider needs to be added without touching core retry/fallback logic.

```rust
// Source: CLAUDE.md architecture principle; pattern established in platform/mod.rs
#[async_trait::async_trait]
pub trait TranscriptionProvider: Send + Sync {
    async fn transcribe(
        &self,
        audio: &EncodedAudio,
        config: &TranscriptionConfig,
    ) -> Result<String, TranscriptionError>;
}

#[derive(Debug)]
pub enum TranscriptionError {
    InvalidKey { provider: String },
    RateLimit,
    Network { message: String },
    Server { status: u16, message: String },
    Cancelled,
}
```

> Note: `async_trait` macro may be needed unless using RPITIT (Rust 1.75+). Check Rust edition in Cargo.toml — edition 2021 is current; RPITIT is stable in 1.75+. Prefer `async_trait` crate for broad compatibility.

### Pattern 2: Service with Retry Loop

**What:** A service function wraps a single provider call in a retry loop. On exhaustion it emits an event and optionally offers fallback.

**When to use:** Called from the hotkey toggle path after encoding.

```rust
// Source: pattern inferred from CONTEXT.md decisions + reqwest docs
pub async fn transcribe_with_retry<R: Runtime>(
    app: &AppHandle<R>,
    audio: &EncodedAudio,
) -> Result<String, ()> {
    let config = app.state::<AppState>().config.lock().unwrap().transcription.clone();
    let provider = make_provider(&config);

    let mut attempt = 0u32;
    loop {
        // Check cancel flag before each attempt
        if *app.state::<AppState>().cancel_flag.lock().unwrap() {
            return Err(());
        }

        match provider.transcribe(audio, &config).await {
            Ok(text) => return Ok(text),
            Err(TranscriptionError::InvalidKey { provider }) => {
                emit_transcription_error(app, TranscriptionErrorPayload {
                    code: TranscriptionErrorCode::InvalidKey,
                    message: "Invalid API key. Check your settings.".into(),
                    provider: Some(provider),
                    retryable: false,
                });
                return Err(());
            }
            Err(TranscriptionError::RateLimit) | Err(TranscriptionError::Network { .. })
            | Err(TranscriptionError::Server { .. }) if attempt < 3 => {
                let delay_ms = 1000u64 * (1 << attempt); // 1s, 2s, 4s
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                attempt += 1;
            }
            Err(err) => {
                // All retries exhausted — emit error, offer fallback if available
                emit_transcription_error(app, /* ... */);
                return Err(());
            }
        }
    }
}
```

### Pattern 3: Tauri Async Spawn from Hotkey Service

**What:** The hotkey toggle stop path spawns the transcription task onto the Tauri runtime.

**When to use:** Replacing `complete_transcription_placeholder` in `hotkey/service.rs`.

```rust
// Source: tauri::async_runtime docs; confirmed pattern from community
let app_clone = app.clone();
let encoded = audio::stop_recording_and_encode(app)?;
tauri::async_runtime::spawn(async move {
    match transcription::service::transcribe_with_retry(&app_clone, &encoded).await {
        Ok(text) => {
            // Phase 6: inject text
            indicator::show_injecting(&app_clone);
            // ... injection happens here in Phase 6
        }
        Err(()) => {
            indicator::hide(&app_clone);
        }
    }
    let state = app_clone.state::<AppState>();
    *state.recording_state.lock().unwrap() = RecordingState::Idle;
    tray::update_recording_menu(&app_clone, RecordingState::Idle);
});
```

### Pattern 4: OpenAI / Groq Multipart Request

**What:** Both providers accept identical multipart/form-data structure.

```rust
// Source: OpenAI API reference, Groq speech-to-text docs
async fn transcribe(&self, audio: &EncodedAudio, config: &TranscriptionConfig) -> Result<String, TranscriptionError> {
    let filename = match audio.format {
        EncodedFormat::Opus => "audio.ogg",
        EncodedFormat::Wav  => "audio.wav",
    };
    let mut form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(audio.bytes.clone())
                .file_name(filename)
                .mime_str(audio.mime_type)
                .map_err(|e| TranscriptionError::Network { message: e.to_string() })?,
        )
        .text("model", config.openai_model.clone())
        .text("response_format", "text"); // plain text response — no JSON parsing

    if !config.language.is_empty() {
        form = form.text("language", config.language.clone());
    }

    let response = self.client
        .post(&self.endpoint)
        .bearer_auth(&config.openai_api_key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| TranscriptionError::Network { message: e.to_string() })?;

    classify_and_extract(response).await
}
```

### Pattern 5: OpenRouter JSON Chat Completions

**What:** Audio sent as base64 `input_audio` in the user message content array alongside a transcript instruction text part.

```rust
// Source: openrouter.ai/docs/guides/overview/multimodal/audio (verified 2026-03-17)
// input_audio.data = base64-encoded audio bytes
// input_audio.format = "ogg" (Opus in Ogg container)
let b64 = base64::engine::general_purpose::STANDARD.encode(&audio.bytes);
let body = serde_json::json!({
    "model": config.openrouter_model,
    "messages": [{
        "role": "user",
        "content": [
            {
                "type": "text",
                "text": "Transcribe this audio exactly as spoken. Return only the transcription."
            },
            {
                "type": "input_audio",
                "input_audio": {
                    "data": b64,
                    "format": "ogg"
                }
            }
        ]
    }]
});
// Optionally append language instruction to the text part when config.language is set
```

### Pattern 6: Event Emission

**What:** Use a new `transcription-error` event (not folding into `audio-error`) for clear separation of domains. Follow the existing enum + payload pattern.

```rust
// Source: existing audio/mod.rs and indicator/events.rs patterns in this codebase
pub const TRANSCRIPTION_ERROR_EVENT: &str = "transcription-error";
pub const TRANSCRIPTION_DONE_EVENT: &str = "transcription-done";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionErrorCode {
    InvalidKey,
    RateLimit,
    Network,
    Server,
    Cancelled,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionErrorPayload {
    pub code: TranscriptionErrorCode,
    pub message: String,
    /// Provider string for invalid-key path so frontend can highlight the right tab
    pub provider: Option<String>,
    /// True if a fallback provider is available (frontend shows "Try with X?" action)
    pub fallback_provider: Option<String>,
    /// True if a manual retry is appropriate (frontend shows "Retry" button)
    pub retryable: bool,
}
```

### Anti-Patterns to Avoid

- **Blocking Tokio with reqwest blocking client:** Always use `reqwest::Client` (async), never `reqwest::blocking::Client` inside an async Tauri command or spawn. Blocking Tokio threads causes deadlocks.
- **Using `tokio::spawn` directly in Tauri:** Use `tauri::async_runtime::spawn` to avoid "no reactor running" panics in Tauri v2.
- **Locking `AppState` across `.await` points:** Lock, clone what you need, drop the lock, then await. Holding a `MutexGuard` across an `.await` causes a compile error (non-Send) or deadlock.
- **Setting `Content-Type: multipart/form-data` manually:** `reqwest` sets the boundary automatically when you use `.multipart(form)`. Manual header breaks the boundary.
- **Sending empty `language` string to API:** Per locked decision, omit the field entirely when empty.
- **Building `reqwest::Client` per request:** Build once (in the service constructor or lazily) and reuse. Reusing the client reuses the connection pool.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Multipart form upload | Custom boundary serializer | `reqwest::multipart` | RFC 2046 boundary edge cases; content-disposition headers; CRLF requirements |
| Base64 encoding | Manual byte-to-char mapping | `base64` crate 0.22 | Padding, alphabet variants, performance |
| HTTP retry loop with backoff | Custom sleep/retry | Tokio sleep + match arms (already designed in CONTEXT.md) | Simple enough to inline; no dep needed |
| JSON request/response | Manual string building | `serde_json::json!` macro + `serde` derive | Type safety, edge cases with special chars |
| TLS | Custom TLS | `reqwest` default-tls or rustls feature | Certificate validation, ALPN, SNI — never hand-roll |

**Key insight:** The HTTP and multipart layers have many subtle correctness requirements (TLS cert validation, multipart boundaries, chunked encoding). `reqwest` handles all of them.

---

## Common Pitfalls

### Pitfall 1: Holding AppState Lock Across await

**What goes wrong:** Code locks `state.config`, gets a guard, then calls `.await` — the compiler rejects it (MutexGuard is not Send) or it deadlocks.

**Why it happens:** Standard `std::sync::Mutex` guards are not Send; they cannot cross await points.

**How to avoid:** Clone the needed config values before the first await:
```rust
let config = state.config.lock().unwrap().transcription.clone();
// drop the guard here — config is owned data, Send + 'static
do_async_work(config).await;
```

**Warning signs:** Compile error "future cannot be sent between threads safely" mentioning MutexGuard.

### Pitfall 2: OpenRouter Model Not Supporting Audio Input

**What goes wrong:** A model is selected that doesn't accept `input_audio` content, returns 400 with "unsupported content type" or similar.

**Why it happens:** Not all OpenRouter models have audio input capability; only specific models listed in CLOD-05 support it.

**How to avoid:** Only expose the hardcoded model list from CLOD-05 in the UI (Phase 8). For Phase 5, the service uses whatever is in `config.openrouter_model` — document that an empty model string should be rejected early.

**Warning signs:** HTTP 400 response from OpenRouter with message about content type or model capabilities.

### Pitfall 3: OpenRouter Opus Format String

**What goes wrong:** Sending `"format": "audio/ogg"` (MIME type) instead of `"format": "ogg"` (format string). API returns 400.

**Why it happens:** The `input_audio.format` field takes a format identifier, not a MIME type.

**How to avoid:** Use `"ogg"` for Opus-in-Ogg. Verified from OpenRouter audio docs: supported values are "wav, mp3, aiff, aac, ogg, flac, m4a, pcm16, pcm24".

### Pitfall 4: reqwest Not in Cargo.toml as Direct Dependency

**What goes wrong:** `reqwest` is available via transitive deps but the version or features needed for multipart are not enabled, causing compile errors.

**Why it happens:** Tauri pulls in `reqwest` transitively but does not enable `multipart` or `json` features.

**How to avoid:** Add `reqwest = { version = "0.12", features = ["multipart", "json"] }` explicitly to Cargo.toml.

### Pitfall 5: response_format "text" vs "json" Parsing

**What goes wrong:** Using `response_format=json` and then trying to `.text()` the response body instead of `.json::<TranscriptionResponse>()`.

**Why it happens:** JSON response wraps the text in `{"text": "..."}` rather than returning plain text.

**How to avoid:** Use `response_format=text` for OpenAI/Groq — the response body is the raw transcript string, no JSON deserialization needed. For OpenRouter, parse `choices[0].message.content` from the standard chat completion JSON response.

### Pitfall 6: Cancel Flag Check

**What goes wrong:** A transcription in progress ignores the cancel flag; UI appears stuck.

**Why it happens:** The `cancel_flag` in AppState exists but nothing checks it during the retry loop.

**How to avoid:** Check `cancel_flag` before each retry attempt. On cancel, return `TranscriptionError::Cancelled` and emit a cancellation event. Reset the flag to false after handling.

---

## Code Examples

### HTTP Status Classification

```rust
// Source: HTTP status semantics; pattern consistent with CONTEXT.md error classification
async fn classify_and_extract<R: Runtime>(
    response: reqwest::Response,
    provider_name: &str,
) -> Result<String, TranscriptionError> {
    let status = response.status();
    match status.as_u16() {
        200 => Ok(response.text().await.map_err(|e| TranscriptionError::Network { message: e.to_string() })?),
        401 | 403 => Err(TranscriptionError::InvalidKey { provider: provider_name.to_string() }),
        429 => Err(TranscriptionError::RateLimit),
        500..=599 => {
            let msg = response.text().await.unwrap_or_default();
            Err(TranscriptionError::Server { status: status.as_u16(), message: msg })
        }
        other => {
            let msg = response.text().await.unwrap_or_default();
            Err(TranscriptionError::Network { message: format!("HTTP {other}: {msg}") })
        }
    }
}
```

### Config Addition: fallback_order

```rust
// Source: CONTEXT.md locked decision; config/mod.rs pattern
#[serde(default = "default_fallback_order")]
pub fallback_order: Vec<TranscriptionProvider>,

fn default_fallback_order() -> Vec<TranscriptionProvider> {
    vec![
        TranscriptionProvider::Openai,
        TranscriptionProvider::Groq,
        TranscriptionProvider::Openrouter,
    ]
}
```

### Backoff Intervals (Claude's Discretion — Recommendation)

Use 1s, 2s, 4s (powers of 2): `let delay_ms = 1000u64 * (1 << attempt)` where `attempt` starts at 0.

- Attempt 0 fails → sleep 1s → attempt 1
- Attempt 1 fails → sleep 2s → attempt 2
- Attempt 2 fails → sleep 4s → attempt 3
- Attempt 3 fails → exhausted, emit error/fallback offer

Total worst-case wait before user sees error: ~7 seconds (acceptable for a transcription flow).

### OpenRouter Response Extraction

```rust
// Source: OpenAI chat completions response format (OpenRouter mirrors this)
#[derive(serde::Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}
#[derive(serde::Deserialize)]
struct Choice {
    message: Message,
}
#[derive(serde::Deserialize)]
struct Message {
    content: String,
}

let chat: ChatResponse = response.json().await
    .map_err(|e| TranscriptionError::Network { message: e.to_string() })?;
let text = chat.choices.into_iter().next()
    .map(|c| c.message.content)
    .ok_or_else(|| TranscriptionError::Network { message: "empty choices".into() })?;
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `tokio::spawn` in Tauri | `tauri::async_runtime::spawn` | Tauri v2 | Direct tokio::spawn panics with "no reactor running" in Tauri v2 contexts |
| `async_trait` crate for async traits | RPITIT (stable, native async fn in traits) | Rust 1.75 (2023) | Can avoid `async_trait` dep; check if needed for dyn dispatch |
| reqwest 0.11 | reqwest 0.12 (Tauri 2 compat) | 2024 | API largely compatible; multipart Part::bytes still available |

**Deprecated/outdated:**
- `reqwest::blocking` in async context: never appropriate in this codebase
- `whisper-rs` in Phase 5: feature-gated behind `local-transcription` cargo feature; not compiled here

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | none — inline `#[test]` and `#[cfg(test)]` modules |
| Quick run command | `cd src-tauri && cargo test transcription` |
| Full suite command | `cd src-tauri && cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLOD-01 | Provider dispatch selects correct impl | unit | `cargo test transcription::tests::provider_dispatch` | Wave 0 |
| CLOD-03 | OpenAI multipart request has correct fields | unit (mock HTTP) | `cargo test transcription::openai::tests` | Wave 0 |
| CLOD-04 | Groq uses correct endpoint and model field | unit (mock HTTP) | `cargo test transcription::groq::tests` | Wave 0 |
| CLOD-05 | OpenRouter sends base64 input_audio with ogg format | unit (mock HTTP) | `cargo test transcription::openrouter::tests` | Wave 0 |
| CLOD-06 | 401/403 → InvalidKey error variant | unit | `cargo test transcription::tests::invalid_key_classified` | Wave 0 |
| CLOD-07 | 429 → retry 3 times with backoff, then error | unit (mock counter) | `cargo test transcription::tests::rate_limit_retries` | Wave 0 |
| CLOD-08 | Network error → error variant; retryable=true in payload | unit | `cargo test transcription::tests::network_error_retryable` | Wave 0 |
| CLOD-09 | Fallback order respected; skip providers without key | unit | `cargo test transcription::tests::fallback_order_skips_unconfigured` | Wave 0 |
| CLOD-10 | Empty language omitted from request | unit | `cargo test transcription::tests::empty_language_omitted` | Wave 0 |

**Note on HTTP mocking:** The standard approach in Rust for testing HTTP clients without network is to inject a mock `reqwest::Client` or use the `mockito` crate (or `wiremock` for async). Given the project uses trait injection patterns (see `EncoderBackend`), a `TranscriptionHttpClient` trait approach allows test doubles without external crates. Alternatively, `mockito = "1"` in dev-dependencies provides a real local HTTP server.

Recommendation: use the same trait-injection pattern already established (`EncoderBackend` in `audio/encode.rs`) — define an `HttpClient` trait and inject it, making tests hermetic without additional deps.

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test transcription`
- **Per wave merge:** `cd src-tauri && cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/transcription/mod.rs` — module + service + retry loop
- [ ] `src-tauri/src/transcription/provider.rs` — trait definition + error enum
- [ ] `src-tauri/src/transcription/openai.rs` — OpenAI impl + tests
- [ ] `src-tauri/src/transcription/groq.rs` — Groq impl + tests
- [ ] `src-tauri/src/transcription/openrouter.rs` — OpenRouter impl + tests
- [ ] `Cargo.toml` additions: `reqwest` (multipart + json features), `base64 = "0.22"`

---

## Open Questions

1. **`async_trait` vs native RPITIT for dyn dispatch**
   - What we know: Rust 1.75+ supports async fn in traits natively; project uses edition 2021
   - What's unclear: Whether dyn TranscriptionProvider (needed for runtime dispatch by provider enum) works with native async fn in traits without `async_trait` crate (RPITIT with dyn requires `async_trait` or manual boxing as of Rust 1.75)
   - Recommendation: Use `async_trait = "0.1"` in dependencies to guarantee dyn dispatch works cleanly; it is the established pattern

2. **reqwest version in transitive deps**
   - What we know: reqwest is not in Cargo.toml directly; Tauri 2 pulls it transitively
   - What's unclear: Exact version and whether features already include multipart
   - Recommendation: Run `cargo tree | grep reqwest` to confirm version; add explicit dep regardless to lock features

3. **OpenRouter `ogg` format string for Opus**
   - What we know: OpenRouter audio docs list "wav, mp3, aiff, aac, ogg, flac, m4a, pcm16, pcm24" as supported formats
   - What's unclear: Whether Opus-in-Ogg specifically requires `"ogg"` or some other identifier for models like Gemini
   - Recommendation: Use `"ogg"` (confirmed from docs); test against a real endpoint during phase execution; document that `audio/ogg` (MIME) is wrong

---

## Sources

### Primary (HIGH confidence)
- OpenRouter audio guide (fetched 2026-03-17) — input_audio JSON structure, base64 requirement, format field values
- Groq speech-to-text docs (fetched 2026-03-17) — endpoint URL, form fields, model names, supported formats
- reqwest::multipart docs (docs.rs) — Form, Part::bytes, file_name, mime_str API
- Existing codebase (audio/encode.rs, config/mod.rs, state.rs, hotkey/service.rs) — all integration points verified by direct read

### Secondary (MEDIUM confidence)
- OpenAI audio transcription API reference (web search verified against official URL) — multipart form fields, model names, response_format options
- tauri::async_runtime::spawn documentation and community discussions — confirmed spawn pattern for Tauri v2

### Tertiary (LOW confidence)
- reqwest transitive version in this project — inferred from Cargo.toml not having it directly; needs `cargo tree` verification

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — reqwest multipart and serde_json are well-documented; verified against official docs
- Architecture: HIGH — closely follows existing codebase patterns; integration points confirmed by code reading
- API request shapes: HIGH — OpenAI/Groq verified via official docs; OpenRouter verified via fetched docs page
- Pitfalls: HIGH — most are based on direct code reading and confirmed Rust/Tauri behavior

**Research date:** 2026-03-17
**Valid until:** 2026-06-17 (stable APIs; reqwest/Tauri APIs move slowly)
