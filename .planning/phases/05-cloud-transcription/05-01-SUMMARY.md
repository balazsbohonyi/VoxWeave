---
phase: 05-cloud-transcription
plan: "01"
subsystem: transcription
tags: [rust, http, reqwest, multipart, base64, async-trait, openai, groq, openrouter]
dependency_graph:
  requires: []
  provides:
    - TranscriptionProviderTrait (async trait for all cloud providers)
    - TranscriptionError (InvalidKey, RateLimit, Network, Server, Cancelled)
    - OpenAiProvider (multipart HTTP client)
    - GroqProvider (multipart HTTP client)
    - OpenRouterProvider (chat completions + base64 audio)
    - TranscriptionConfig.fallback_order field
  affects:
    - src-tauri/src/config/mod.rs (fallback_order added)
    - src-tauri/src/lib.rs (transcription module declared)
    - src-tauri/Cargo.toml (3 new deps)
tech_stack:
  added:
    - reqwest 0.12 (multipart + json features)
    - base64 0.22
    - async-trait 0.1
    - tokio 1 (rt-multi-thread + macros)
  patterns:
    - async-trait for object-safe async provider trait
    - Testable helper functions (form_field_names, build_body) extracted for unit testing without HTTP mocks
    - status_to_error pure function for classification logic testing
key_files:
  created:
    - src-tauri/src/transcription/provider.rs
    - src-tauri/src/transcription/mod.rs
    - src-tauri/src/transcription/openai.rs
    - src-tauri/src/transcription/groq.rs
    - src-tauri/src/transcription/openrouter.rs
  modified:
    - src-tauri/src/config/mod.rs
    - src-tauri/src/lib.rs
    - src-tauri/Cargo.toml
decisions:
  - "async-trait crate used for object-safe async trait (Rust lacks native async trait object support)"
  - "Testable helper functions extracted (form_field_names, build_body, status_to_error) to avoid HTTP mocking in unit tests"
  - "format='ogg' in OpenRouter payload (identifier, not MIME type audio/ogg)"
  - "Empty language string never sent — condition guard in each provider"
  - "OpenRouter model-empty guard returns Network error immediately before HTTP call"
metrics:
  duration_seconds: 1289
  completed_date: "2026-03-17"
  tasks_completed: 3
  files_created: 5
  files_modified: 3
requirements_satisfied:
  - CLOD-01
  - CLOD-02
  - CLOD-03
  - CLOD-04
  - CLOD-05
  - CLOD-10
---

# Phase 5 Plan 1: TranscriptionProvider Trait Contracts and Cloud Provider HTTP Clients Summary

**One-liner:** Async TranscriptionProviderTrait with OpenAI/Groq multipart and OpenRouter base64 chat completions clients, backed by 15 unit tests.

## What Was Built

Three cloud transcription provider HTTP clients plus the shared trait/error contracts they implement:

- **provider.rs** — `TranscriptionProviderTrait` (async trait), `TranscriptionError` enum (5 variants), `status_to_error` pure helper, `classify_and_extract` async helper
- **openai.rs** — `OpenAiProvider` posting multipart to `api.openai.com/v1/audio/transcriptions`
- **groq.rs** — `GroqProvider` posting multipart to `api.groq.com/openai/v1/audio/transcriptions`
- **openrouter.rs** — `OpenRouterProvider` posting JSON with base64 audio to `openrouter.ai/api/v1/chat/completions`

Config extended with `fallback_order: Vec<TranscriptionProvider>` defaulting to `[Openai, Groq, Openrouter]` — backward compatible via serde default attribute.

## Task Commits

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Trait contracts, config extension, Cargo additions | c1068cc |
| 2 | OpenAI and Groq multipart providers | 82962e1 |
| 3 | OpenRouter chat completions provider | 2c5cdf2 |

## Test Results

```
cargo test transcription
running 15 tests — all passed

Provider tests (5): invalid key 401/403, rate limit 429, server 503, fallback_order default
OpenAI tests (3): language omit/include, endpoint constant
Groq tests (3): language omit/include, endpoint constant
OpenRouter tests (4): empty model guard, format=ogg (not MIME), language append, language omit
```

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written, with one minor note:

**Note: tokio added to Cargo.toml**
- The plan did not list tokio as a dependency but reqwest 0.12 with the `json` feature requires a tokio runtime at test time
- Added `tokio = { version = "1", features = ["rt-multi-thread", "macros"] }` as a dependency
- No behavioral change; this is infrastructure required for reqwest async operation

### Pre-existing Clippy Errors

The plan's verification step `cargo clippy -- -D warnings` fails due to 20 pre-existing errors in unrelated modules (hotkey/mod.rs, platform/mod.rs, state.rs, lib.rs). These existed before this plan and are outside scope. Zero new clippy errors were introduced by the transcription module.

## Self-Check: PASSED

All 5 files created, all 3 commits verified in git log.
