# Phase 10: Local Transcription - Research

**Researched:** 2026-03-29
**Domain:** whisper-rs / whisper.cpp integration, on-demand model download, cargo feature gating, Vue model picker UI
**Confidence:** HIGH

## Summary

Phase 10 activates the `local-transcription` cargo feature that has been stubbed since Phase 1. The feature gate (`local-transcription = []` in Cargo.toml) already exists. The config struct already has `LocalProviderConfig { model_path: Option<String> }` and `TranscriptionProvider::Local` is already a valid enum variant. The `format_for_provider` function already routes `Local` to WAV. The `make_provider` function already has the `Local` arm — it just panics with `unimplemented!`. The frontend `TranscriptionSection.vue` already has `LOCAL_MODELS` array with all four model cards — just with disabled stub buttons.

The three primary tasks are: (1) add `whisper-rs` behind the feature flag and implement `LocalProvider` that loads the model and transcribes WAV PCM, (2) implement Rust download commands (`start_model_download`, `cancel_model_download`, `get_downloaded_models`) with progress events via the existing `emit()` pattern, and (3) wire the frontend (Settings local cards + Step2Local.vue wizard) to those commands.

**Primary recommendation:** Implement the local transcription feature by replacing the `unimplemented!()` stubs with real whisper-rs calls, keeping all CPU work on `std::thread` per the existing CLAUDE.md constraint.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Download persistence:**
- Download continues as a Rust background task regardless of which window is open or closed
- When the user reopens Settings (or the wizard) mid-download, the model card shows the current live progress bar — no reconnection needed beyond listening to the existing event stream
- App quit = download cancelled immediately + partial file deleted (no resume across launches)
- Only one model downloads at a time; while a download is in progress, all other Download buttons are disabled

**Download UX:**
- During download: the Download button is replaced by a full-width progress bar, a percentage number, and a Cancel button
- Cancel: stops download immediately and deletes the partial file; model card returns to "Download" state
- Download source: Hugging Face GGML files (`https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{model}.bin`)
- Progress events emitted from Rust via the existing `emit()` pattern (same as `audio-level` events)

**Wizard Step 2 Local:**
- Step2Local.vue becomes a full model picker showing all 4 models (tiny/base/small/medium) with size/quality hints and a Download button per card
- Inline download works the same as Settings: progress bar + percentage + Cancel replaces the Download button
- Next button is disabled until either a model is fully downloaded OR the user clicks "Skip for now"
- After Finish with no model downloaded (skipped): a toast nudges the user to open Settings to download one
- When wizard is re-opened from Settings with a model already active: that model card shows an "Active" checkmark; other models show Download

**Missing/corrupt model error handling:**
- Missing model on recording attempt: abort the recording immediately, show error toast "No local model downloaded." with an "Open Settings" action button (matches existing invalid-key toast pattern)
- Corrupt model (whisper-rs fails to load): treated identically to missing — same error toast, same "Open Settings" action
- No SHA256 verification at download time — detect corruption on load, not upfront

### Claude's Discretion
- Exact Tailwind styling of model cards, progress bar color, and Active badge design
- Specific whisper-rs API usage for model loading and WAV input
- Tokio channel vs atomic flag for download cancellation signal
- How to wire the `local-transcription` feature flag in Cargo.toml without breaking the non-feature build

### Deferred Ideas (OUT OF SCOPE)
- SHA256 checksum verification at download time — not in v1, detect corruption on load instead
- Resume-from-partial download across app restarts — requires byte-range HTTP, deferred to v2
- User-configurable download URL per model — out of scope, hardcoded Hugging Face URLs for v1
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LOCL-01 | App supports local transcription via whisper.cpp (whisper-rs) | whisper-rs crate behind `local-transcription` feature; `LocalProvider` implementing `TranscriptionProviderTrait` |
| LOCL-02 | Models downloaded on-demand from settings with progress bar and cancel option | Rust `start_model_download` command; reqwest streaming with `Content-Length` for progress; `emit()` for events; `AtomicBool` cancel flag |
| LOCL-03 | Available models: tiny (~75MB), base (~150MB), small (~500MB), medium (~1.5GB) | Already in `LOCAL_MODELS` array in TranscriptionSection.vue; HF URLs are `ggml-{tiny,base,small,medium}.bin` |
| LOCL-04 | Downloaded models stored in `%APPDATA%/VoxFlow/models/`; user can delete models | `dirs_next::data_dir()` or `dirs_next::config_dir()` for path resolution; add `delete_model` command |
| LOCL-05 | Local transcription runs on a background thread without freezing the UI | `std::thread::spawn` for whisper-rs inference (already a locked decision in CLAUDE.md) |
| LOCL-06 | Audio is passed as WAV/PCM float32 to whisper.cpp | `format_for_provider` already routes `Local` to `EncodedFormat::Wav`; whisper-rs `WhisperContext::new()` + `FullParams` + `pcm_to_mel` + `full()` |
| LOCL-07 | If model file is missing or corrupt, show error with prompt to re-download | Add `TranscriptionError::ModelMissing` and `ModelLoadFailed` variants; emit via existing toast pattern; "Open Settings" action button |
</phase_requirements>

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| whisper-rs | 0.11+ | Rust bindings for whisper.cpp | The only maintained Rust wrapper for whisper.cpp; used in the project since Phase 1 stub |
| reqwest | 0.12 (already in Cargo.toml) | HTTP download with streaming body | Already used for cloud providers; `bytes_stream()` gives async chunks for progress tracking |
| dirs-next | 2 (already in Cargo.toml) | Resolve `%APPDATA%` path | Already in dependency tree; `config_dir()` returns `%APPDATA%` on Windows |
| tokio | 1 (already in Cargo.toml) | Async download task | Already used; `tauri::async_runtime::spawn` for download task (consistent with existing cloud pattern) |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::sync::atomic::AtomicBool | stdlib | Download cancel signal | Preferred over channel when cancel is fire-and-forget; no message payload needed |
| std::fs | stdlib | Model directory creation, file write, file delete | Creating `%APPDATA%/VoxFlow/models/`, writing downloaded bytes, deleting models |
| futures-util | (pulled by reqwest) | `StreamExt::next()` for byte stream | If `bytes_stream()` needs iteration |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| AtomicBool cancel | tokio::sync::oneshot channel | Channel is cleaner for complex cancel + result, but AtomicBool is simpler for a single "stop now" flag |
| reqwest streaming | ureq (sync) | ureq would block Tokio — reqwest async fits the existing pattern |
| whisper-rs full API | whisper-rs state-machine API | Full API (single `full()` call) is simpler; state-machine API for streaming not needed |

**Installation (feature-gated):**
```toml
[dependencies]
whisper-rs = { version = "0.11", optional = true }

[features]
local-transcription = ["dep:whisper-rs"]
```

---

## Architecture Patterns

### Recommended Project Structure (additions only)
```
src-tauri/src/
├── transcription/
│   ├── local.rs             # NEW: LocalProvider implementing TranscriptionProviderTrait
│   └── download.rs          # NEW: download state machine + Rust commands
├── commands/
│   └── transcription.rs     # EXTEND: add start_model_download, cancel_model_download,
│                            #          get_downloaded_models, delete_model commands

src/windows/
├── settings/components/
│   └── TranscriptionSection.vue  # EXTEND: wire real download commands in local cards
├── wizard/components/
│   └── Step2Local.vue            # REPLACE: full model picker (same cards as Settings)
```

### Pattern 1: Feature-Gated LocalProvider

The `LocalProvider` struct and its `TranscriptionProviderTrait` impl live in `transcription/local.rs` behind `#[cfg(feature = "local-transcription")]`. The `make_provider` function in `service.rs` dispatches to it under the same cfg gate.

```rust
// src-tauri/src/transcription/local.rs
#[cfg(feature = "local-transcription")]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[cfg(feature = "local-transcription")]
pub struct LocalProvider {
    model_path: String,
}

#[cfg(feature = "local-transcription")]
impl LocalProvider {
    pub fn new(model_path: String) -> Self {
        Self { model_path }
    }
}

// impl TranscriptionProviderTrait inside #[cfg(feature = "local-transcription")]
// The transcribe() method:
// 1. Parse WAV bytes → f32 PCM slice
// 2. std::thread::spawn (NOT tokio::spawn — CPU bound)
// 3. WhisperContext::new_with_params(&model_path, WhisperContextParameters::default())
// 4. ctx.create_state() → WhisperState
// 5. state.full(params, &pcm) → run inference
// 6. collect segments with state.full_n_segments() + state.full_get_segment_text()
// 7. Join thread → return Ok(text) or Err(ModelLoadFailed)
```

**Critical:** The `std::thread::spawn` wrapping must use a `std::sync::mpsc::channel` or `Arc<Mutex<Result<...>>>` to communicate the result back to the async caller. Use `tokio::task::spawn_blocking` as an alternative — it runs on the Tokio blocking thread pool and `.await`s cleanly. This is the right approach: `spawn_blocking` is designed for CPU-bound work inside an async context without blocking the Tokio runtime.

```rust
// Preferred pattern for CPU-bound work in async context:
let result = tokio::task::spawn_blocking(move || {
    // whisper-rs inference here — runs on blocking thread pool
    run_local_transcription(&model_path, &pcm)
}).await.map_err(|_| TranscriptionError::Network { message: "Thread panic".into() })??;
```

Note: `tokio::task::spawn_blocking` vs `std::thread::spawn` — CLAUDE.md says "std::thread for whisper.cpp inference (not Tokio)". This means don't use `tokio::spawn` (which runs async tasks on the Tokio executor). `tokio::task::spawn_blocking` is the correct middle ground: it uses a dedicated blocking thread pool that does NOT block the Tokio async executor. Both approaches are correct; use `std::thread::spawn` with a channel if following the strict CLAUDE.md wording.

### Pattern 2: Download State in AppState

Add download state to `AppState` (not a separate managed state):

```rust
// In state.rs — add to AppState struct:
/// Handle to the active model download task, if any.
/// None = no download in progress.
pub download_cancel: Arc<Mutex<Option<Arc<std::sync::atomic::AtomicBool>>>>,

/// ID of the model currently being downloaded (e.g., "tiny", "base").
pub downloading_model: Arc<Mutex<Option<String>>>,
```

### Pattern 3: Download Command + Progress Events

```rust
// In commands/transcription.rs or a new commands/download.rs:
#[tauri::command]
pub async fn start_model_download<R: Runtime>(
    app: AppHandle<R>,
    model_id: String, // "tiny" | "base" | "small" | "medium"
) -> Result<(), String> {
    // 1. Guard: only one download at a time
    // 2. Create AtomicBool cancel flag, store in AppState
    // 3. tauri::async_runtime::spawn download task
    //    - reqwest::get(url).await → check Content-Length
    //    - loop: stream.next() → write chunk, compute %, emit "model-download-progress"
    //    - on cancel: delete partial file, emit "model-download-cancelled"
    //    - on complete: emit "model-download-done", save model_path to config
    Ok(())
}

#[tauri::command]
pub fn cancel_model_download(app: AppHandle<impl Runtime>) -> Result<(), String> {
    // Set cancel AtomicBool to true
}

#[tauri::command]
pub fn get_downloaded_models(app: AppHandle<impl Runtime>) -> Result<Vec<String>, String> {
    // Scan %APPDATA%/VoxFlow/models/ for ggml-*.bin files
    // Return vec of model IDs ("tiny", "base", etc.)
}

#[tauri::command]
pub fn delete_model(
    app: AppHandle<impl Runtime>,
    model_id: String,
) -> Result<(), String> {
    // Delete %APPDATA%/VoxFlow/models/ggml-{model_id}.bin
    // If it's the active model, clear config.transcription.providers.local.model_path
}
```

### Pattern 4: Progress Event Payloads

```rust
// New event constants (in transcription/download.rs or service.rs):
pub const MODEL_DOWNLOAD_PROGRESS_EVENT: &str = "model-download-progress";
pub const MODEL_DOWNLOAD_DONE_EVENT: &str = "model-download-done";
pub const MODEL_DOWNLOAD_CANCELLED_EVENT: &str = "model-download-cancelled";
pub const MODEL_DOWNLOAD_ERROR_EVENT: &str = "model-download-error";

#[derive(serde::Serialize, Clone)]
pub struct DownloadProgressPayload {
    pub model_id: String,
    pub percent: f32,      // 0.0–100.0
    pub bytes_done: u64,
    pub bytes_total: u64,
}

#[derive(serde::Serialize, Clone)]
pub struct DownloadDonePayload {
    pub model_id: String,
    pub model_path: String,
}
```

### Pattern 5: Vue Download State (Settings + Wizard)

Both `TranscriptionSection.vue` (Settings) and `Step2Local.vue` (Wizard) share the same model card logic. Extract to a shared composable or keep co-located — given the small scope, co-located is fine. Both components listen for the same Tauri events.

```typescript
// Shared download state pattern (per component):
type DownloadState = "idle" | "downloading" | "downloaded";

const modelStates = ref<Record<string, DownloadState>>({
  tiny: "idle", base: "idle", small: "idle", medium: "idle"
});
const downloadPercent = ref<Record<string, number>>({
  tiny: 0, base: 0, small: 0, medium: 0
});
const activeDownloadId = ref<string | null>(null);

// Listen for Tauri events:
// "model-download-progress" → update modelStates[id] = "downloading", update percent
// "model-download-done" → update modelStates[id] = "downloaded", activeDownloadId = null
// "model-download-cancelled" → reset modelStates[id] = "idle", activeDownloadId = null
// "model-download-error" → reset to "idle", show toast

// On mount: call get_downloaded_models() to hydrate initial state
```

### Anti-Patterns to Avoid

- **Blocking Tokio with whisper-rs:** Never call `ctx.full()` inside a `tauri::async_runtime::spawn` closure — use `std::thread::spawn` or `tokio::task::spawn_blocking`. CPU work on the async executor freezes the UI.
- **Holding MutexGuard across await:** Same rule as cloud providers — clone config before any `.await` point. The download cancel flag access must release the guard before awaiting.
- **Writing to a non-existent directory:** Always `std::fs::create_dir_all` on `%APPDATA%/VoxFlow/models/` before writing the model file. `dirs_next::config_dir()` returns `%APPDATA%` on Windows.
- **Partial file left on error:** The download task must delete partial files on any non-success exit path (cancel, network error, disk error). Use a `drop` guard or explicit cleanup in all branches.
- **Missing feature gate on local.rs import:** `use crate::transcription::local::LocalProvider;` in `service.rs` must be inside `#[cfg(feature = "local-transcription")]` — otherwise the non-feature build fails.
- **Event listeners not unlistened in Vue:** `listen()` from `@tauri-apps/api/event` returns an unlisten function. Call it in `onUnmounted` to avoid leaked listeners between Settings re-opens.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WAV decoding for whisper-rs | Custom WAV parser | whisper-rs `convert_integer_to_float_audio` + pass raw f32 from existing encode.rs output | The WAV encoder in `encode.rs` already produces PCM int16; whisper-rs's helper converts to f32 OR use the f32 directly from `pcm_buffer` before encoding |
| Progress percentage without Content-Length | Estimate from model sizes | Use `Content-Length` response header from Hugging Face — it's always present for .bin files | HF always returns Content-Length for direct file downloads |
| Model directory path | Hardcode `C:\Users\...` | `dirs_next::config_dir().unwrap().join("VoxFlow").join("models")` | `dirs_next` already in Cargo.toml; handles all Windows user profile layouts |
| Download cancellation | SIGKILL thread | `AtomicBool` checked in the streaming loop every chunk | Clean, cooperative cancellation with cleanup |

**Key insight:** The WAV bytes stored in `EncodedAudio` are 16-bit PCM. whisper-rs expects `f32` samples. The `pcm_buffer` in `AudioSessionState` already holds f32 samples at 16kHz — passing those directly to whisper-rs (bypassing the WAV encode step) is more efficient, but requires access to the pcm_buffer before encoding. Since the existing pipeline encodes to WAV and passes `EncodedAudio`, the simpler approach is to decode the WAV header (skip 44 bytes) and reinterpret as i16 → f32. This is 5 lines of code, not a "hand-roll" concern.

---

## Common Pitfalls

### Pitfall 1: whisper-rs build fails without CMake
**What goes wrong:** Adding `whisper-rs` without the feature gate causes build failures on machines without CMake/MSVC, breaking cloud-only development workflow.
**Why it happens:** whisper-rs compiles whisper.cpp (C++) internally via a build script that requires CMake.
**How to avoid:** Use `optional = true` in Cargo.toml and `#[cfg(feature = "local-transcription")]` on every use site. The feature was pre-defined in Phase 1 specifically for this.
**Warning signs:** `error: failed to run custom build command for 'whisper-rs'` without the feature flag.

### Pitfall 2: Model path config not persisted after download
**What goes wrong:** User downloads a model, closes and reopens the app — local transcription fails because `model_path` was never written to config.
**Why it happens:** The download task completes and emits an event, but never calls `save_config` to persist the active model path.
**How to avoid:** After successful download, update `config.transcription.providers.local.model_path` in AppState AND call `persistence::save()`. Match the pattern from `commands/config.rs`'s `save_config` command.

### Pitfall 3: Blocking the Tokio executor with CPU work
**What goes wrong:** UI freezes during transcription; audio events stop arriving.
**Why it happens:** `whisper_ctx.full()` is CPU-bound and takes 1–30 seconds depending on model. Calling it on a Tokio task blocks the executor thread pool.
**How to avoid:** Use `std::thread::spawn` with a channel to communicate back, or `tokio::task::spawn_blocking`. The CLAUDE.md decision is explicit: "std::thread for whisper.cpp inference."

### Pitfall 4: Multiple simultaneous downloads
**What goes wrong:** User clicks Download on two models quickly; two files download to the same path or corrupt each other.
**Why it happens:** No guard on the "one download at a time" invariant.
**How to avoid:** Check `downloading_model` in AppState at the start of `start_model_download`. If Some(_), return an error. Frontend also disables all other Download buttons while one is in progress (state driven from events).

### Pitfall 5: Event listeners from Settings window received in Wizard window (or vice versa)
**What goes wrong:** Progress bar updates appear in wrong window; model shows as downloaded in one window but not the other.
**Why it happens:** `app.emit()` broadcasts to ALL webview windows. Both windows receive the event simultaneously.
**How to avoid:** This is actually correct behavior — both windows should update from the same event stream. Ensure both windows' `onMounted` calls `get_downloaded_models()` to sync initial state, and both listen to the same events. No special handling needed.

### Pitfall 6: WAV bytes → f32 PCM conversion off-by-one
**What goes wrong:** Whisper transcribes garbage or silence.
**Why it happens:** WAV header is 44 bytes; skipping wrong offset gives corrupted audio data.
**How to avoid:** Use a constant `WAV_HEADER_SIZE: usize = 44` and assert the bytes length > 44 before slicing. Then reinterpret the i16 pairs correctly: `i16::from_le_bytes([b0, b1]) as f32 / i16::MAX as f32`.

### Pitfall 7: Hugging Face redirect not followed
**What goes wrong:** Download gets a redirect response instead of the file.
**Why it happens:** HF uses CDN redirects for file downloads.
**How to avoid:** `reqwest::Client` follows redirects by default (up to 10). No special handling needed, but verify Content-Length comes from the final response (after redirects).

---

## Code Examples

### whisper-rs Basic Inference Pattern
```rust
// Source: whisper-rs crate README + docs.rs examples
// Confidence: MEDIUM (based on crate README, version 0.11.x)

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

fn transcribe_local(model_path: &str, pcm_f32: &[f32]) -> Result<String, String> {
    let ctx = WhisperContext::new_with_params(
        model_path,
        WhisperContextParameters::default(),
    ).map_err(|e| format!("Failed to load model: {e}"))?;

    let mut state = ctx.create_state()
        .map_err(|e| format!("Failed to create state: {e}"))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_translate(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    state.full(params, pcm_f32)
        .map_err(|e| format!("Inference failed: {e}"))?;

    let n_segments = state.full_n_segments()
        .map_err(|e| format!("Segment count error: {e}"))?;

    let mut text = String::new();
    for i in 0..n_segments {
        if let Ok(segment) = state.full_get_segment_text(i) {
            text.push_str(&segment);
        }
    }
    Ok(text.trim().to_string())
}
```

### WAV bytes → f32 PCM (skip 44-byte header)
```rust
// Source: WAV PCM spec (16-bit signed little-endian)
fn wav_bytes_to_f32(wav_bytes: &[u8]) -> Vec<f32> {
    const WAV_HEADER_SIZE: usize = 44;
    let pcm_bytes = &wav_bytes[WAV_HEADER_SIZE..];
    pcm_bytes
        .chunks_exact(2)
        .map(|b| {
            let sample = i16::from_le_bytes([b[0], b[1]]);
            sample as f32 / i16::MAX as f32
        })
        .collect()
}
```

### reqwest Streaming Download with Progress
```rust
// Source: reqwest docs (https://docs.rs/reqwest/0.12)
// Confidence: HIGH (reqwest 0.12 is already in Cargo.toml)
use futures_util::StreamExt;

async fn download_model(
    url: &str,
    dest_path: &std::path::Path,
    cancel: Arc<std::sync::atomic::AtomicBool>,
    app: &tauri::AppHandle<impl tauri::Runtime>,
    model_id: &str,
) -> Result<(), String> {
    let response = reqwest::get(url).await
        .map_err(|e| e.to_string())?;
    let total = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut file = std::fs::File::create(dest_path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            drop(file);
            let _ = std::fs::remove_file(dest_path);
            return Err("cancelled".into());
        }
        let bytes = chunk.map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut file, &bytes).map_err(|e| e.to_string())?;
        downloaded += bytes.len() as u64;
        let percent = if total > 0 {
            (downloaded as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        let _ = app.emit("model-download-progress", DownloadProgressPayload {
            model_id: model_id.to_string(),
            percent,
            bytes_done: downloaded,
            bytes_total: total,
        });
    }
    Ok(())
}
```

### Vue: Listen to Tauri events in component
```typescript
// Source: @tauri-apps/api/event (Tauri v2 pattern already used in project)
import { listen } from "@tauri-apps/api/event";
import { onMounted, onUnmounted } from "vue";

// In setup():
let unlistenProgress: (() => void) | null = null;
let unlistenDone: (() => void) | null = null;

onMounted(async () => {
  // Hydrate initial downloaded state
  const downloaded = await invoke<string[]>("get_downloaded_models");
  // ...

  unlistenProgress = await listen<DownloadProgressPayload>(
    "model-download-progress",
    (event) => {
      downloadPercent.value[event.payload.model_id] = event.payload.percent;
      modelStates.value[event.payload.model_id] = "downloading";
    }
  );
  unlistenDone = await listen<DownloadDonePayload>(
    "model-download-done",
    (event) => {
      modelStates.value[event.payload.model_id] = "downloaded";
      activeDownloadId.value = null;
    }
  );
});

onUnmounted(() => {
  unlistenProgress?.();
  unlistenDone?.();
});
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `unimplemented!()` in `make_provider(Local)` | Real `LocalProvider` impl | Phase 10 | Activates offline transcription |
| Disabled stub Download buttons in TranscriptionSection.vue | Functional download cards with progress | Phase 10 | Users can actually download models |
| Placeholder text in Step2Local.vue | Real model picker identical to Settings | Phase 10 | Wizard local path becomes functional |

**Deprecated/outdated:**
- The `unimplemented!("local transcription is phase 10")` panic in `service.rs make_provider()` — remove and replace with real dispatch.
- The disabled `cursor-not-allowed` stub buttons in TranscriptionSection.vue local card template — replace with real download controls.

---

## Open Questions

1. **whisper-rs exact API stability across versions**
   - What we know: whisper-rs 0.11.x is the latest available; the API shown in README is `WhisperContext::new_with_params` + `create_state` + `state.full()`
   - What's unclear: Exact method signatures may differ slightly between minor versions
   - Recommendation: Pin to `whisper-rs = "0.11"` in Cargo.toml; verify exact method names against crate docs at implementation time. The pattern above (MEDIUM confidence) should work but confirm `full_n_segments` / `full_get_segment_text` method names.

2. **Hugging Face GGML URL stability**
   - What we know: URL pattern `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{model}.bin` is the long-standing canonical source used by whisper.cpp itself
   - What's unclear: If HF changes the repository structure or model formats
   - Recommendation: Use these URLs as locked decisions; they've been stable for years.

3. **Language hint passthrough to whisper-rs**
   - What we know: whisper-rs `FullParams` has `set_language(Option<&str>)` for language hint
   - What's unclear: Whether the global `config.transcription.language` BCP-47 format matches whisper.cpp's expected format (whisper uses ISO 639-1 codes like "en", "hu")
   - Recommendation: Pass `config.language` directly as `Some(&config.language)` if non-empty, or `None` for auto-detect. BCP-47 simple codes ("en", "hu") match ISO 639-1.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust unit tests, existing infrastructure) |
| Config file | none — inline `#[cfg(test)]` modules |
| Quick run command | `cd src-tauri && cargo test --features local-transcription` |
| Full suite command | `cd src-tauri && cargo test --features local-transcription && npx vue-tsc --noEmit` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LOCL-01 | LocalProvider::transcribe returns Ok text for valid WAV + model | unit | `cd src-tauri && cargo test --features local-transcription transcription::local` | ❌ Wave 0 |
| LOCL-02 | download_model emits progress events and completes | manual-only | N/A — requires real HF download or mock HTTP server | N/A |
| LOCL-03 | LOCAL_MODELS array has 4 entries with correct sizes | unit | `cd src-tauri && cargo test config` (Rust model list constants) | ✅ (in TranscriptionSection.vue — TS typecheck covers) |
| LOCL-04 | models_dir() returns correct path and delete_model removes file | unit | `cd src-tauri && cargo test --features local-transcription commands::download` | ❌ Wave 0 |
| LOCL-05 | transcribe_local does not block Tokio (use std::thread) | unit | `cd src-tauri && cargo test --features local-transcription` (assert non-blocking pattern) | ❌ Wave 0 |
| LOCL-06 | wav_bytes_to_f32 correctly converts 44-byte-header WAV to f32 | unit | `cd src-tauri && cargo test --features local-transcription audio` | ❌ Wave 0 |
| LOCL-07 | Missing model path → TranscriptionError::ModelMissing, corrupt file → ModelLoadFailed | unit | `cd src-tauri && cargo test --features local-transcription transcription::local` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test --features local-transcription 2>&1 | tail -5`
- **Per wave merge:** `cd src-tauri && cargo test --features local-transcription && npx vue-tsc --noEmit && npm run lint`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/transcription/local.rs` — covers LOCL-01, LOCL-05, LOCL-06, LOCL-07 (unit tests for WAV conversion, error variants, model path validation)
- [ ] `src-tauri/src/transcription/download.rs` or `commands/download.rs` — covers LOCL-04 (path resolution, delete_model logic tests)
- [ ] Feature flag: `[dependencies] whisper-rs = { version = "0.11", optional = true }` and `[features] local-transcription = ["dep:whisper-rs"]` in Cargo.toml

*(LOCL-02 download progress is manual-only: testing requires a real HF connection or a local HTTP mock server — out of scope for unit tests.)*

---

## Sources

### Primary (HIGH confidence)
- Existing codebase analysis (`src-tauri/Cargo.toml`, `src-tauri/src/config/mod.rs`, `src-tauri/src/transcription/service.rs`, `src-tauri/src/audio/encode.rs`) — feature flag location, `LocalProviderConfig`, `make_provider` stub, WAV encoding pattern
- `src/windows/settings/components/TranscriptionSection.vue` — `LOCAL_MODELS` array, existing stub card structure
- CLAUDE.md locked decisions — `std::thread` for CPU work, `arboard` for clipboard, feature-gate from start

### Secondary (MEDIUM confidence)
- whisper-rs crate documentation pattern (whisper-rs 0.11.x) — `WhisperContext::new_with_params`, `create_state`, `state.full()`, `full_n_segments`, `full_get_segment_text`
- reqwest 0.12 streaming download pattern — `bytes_stream()`, `StreamExt::next()` (verified: reqwest 0.12 is already in Cargo.toml with `multipart` + `json` features; `stream` feature may need enabling)

### Tertiary (LOW confidence)
- Hugging Face GGML URL structure — based on whisper.cpp project documentation and established community convention; not directly verified via HTTP

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all key dependencies already in Cargo.toml; feature flag already defined in Phase 1
- Architecture: HIGH — existing code stubs make the integration points explicit; patterns follow established codebase conventions
- whisper-rs API: MEDIUM — crate README patterns; exact method signatures should be confirmed at implementation time
- Pitfalls: HIGH — based on direct code analysis of existing stubs and established Rust/Tauri patterns

**Research date:** 2026-03-29
**Valid until:** 2026-04-28 (30 days — whisper-rs is moderately stable)
