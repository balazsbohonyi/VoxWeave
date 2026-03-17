---
status: diagnosed
trigger: "OpenAI returns 400 Invalid file on transcription"
created: 2026-03-18T00:00:00Z
updated: 2026-03-18T00:00:00Z
---

## Symptoms

expected: Audio transcribed successfully via OpenAI API
actual: 400 "Invalid file" response from OpenAI; error toast appears in indicator with raw error
reproduction: Press hotkey, speak, stop — error appears after processing state

## Root Cause

`src-tauri/src/transcription/openai.rs` sends the multipart file field with:
- filename: `"audio.ogg"`
- MIME type: `"audio/ogg"`

OpenAI Whisper's accepted extensions are: mp3, mp4, mpeg, mpga, m4a, wav, webm. `.ogg` is NOT on the list. OpenAI validates by extension before inspecting bytes and immediately returns 400.

`src-tauri/src/audio/encode.rs` `format_for_provider` routes all cloud providers including OpenAI to `EncodedFormat::Opus`, which produces an Ogg container with `mime_type: "audio/ogg"`.

## Fix

**1. `src-tauri/src/audio/encode.rs` — route OpenAI to WAV:**
```rust
pub fn format_for_provider(provider: &TranscriptionProvider) -> EncodedFormat {
    match provider {
        TranscriptionProvider::Local | TranscriptionProvider::Openai => EncodedFormat::Wav,
        TranscriptionProvider::Groq
        | TranscriptionProvider::Openrouter => EncodedFormat::Opus,
    }
}
```

**2. `src-tauri/src/transcription/openai.rs` — update filename and MIME:**
```rust
let file_part = reqwest::multipart::Part::bytes(audio.bytes.clone())
    .file_name("audio.wav")
    .mime_str("audio/wav")
```

verification: not yet applied
files_changed: []
