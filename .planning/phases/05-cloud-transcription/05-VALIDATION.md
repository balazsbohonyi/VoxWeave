---
phase: 5
slug: cloud-transcription
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust unit + integration tests) |
| **Config file** | `src-tauri/Cargo.toml` |
| **Quick run command** | `cd src-tauri && cargo test transcription` |
| **Full suite command** | `cd src-tauri && cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test transcription`
- **After every plan wave:** Run `cd src-tauri && cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 5-01-01 | 01 | 1 | CLOD-01 | unit | `cd src-tauri && cargo test transcription::provider` | ❌ W0 | ⬜ pending |
| 5-01-02 | 01 | 1 | CLOD-03 | unit | `cd src-tauri && cargo test transcription::openai` | ❌ W0 | ⬜ pending |
| 5-01-03 | 01 | 1 | CLOD-04 | unit | `cd src-tauri && cargo test transcription::groq` | ❌ W0 | ⬜ pending |
| 5-01-04 | 01 | 1 | CLOD-05 | unit | `cd src-tauri && cargo test transcription::openrouter` | ❌ W0 | ⬜ pending |
| 5-02-01 | 02 | 1 | CLOD-07 | unit | `cd src-tauri && cargo test transcription::retry` | ❌ W0 | ⬜ pending |
| 5-02-02 | 02 | 1 | CLOD-08 | unit | `cd src-tauri && cargo test transcription::network_error` | ❌ W0 | ⬜ pending |
| 5-02-03 | 02 | 1 | CLOD-09 | unit | `cd src-tauri && cargo test transcription::fallback` | ❌ W0 | ⬜ pending |
| 5-02-04 | 02 | 1 | CLOD-06 | unit | `cd src-tauri && cargo test transcription::invalid_key` | ❌ W0 | ⬜ pending |
| 5-03-01 | 03 | 2 | CLOD-02 | unit | `cd src-tauri && cargo test config::transcription` | ❌ W0 | ⬜ pending |
| 5-03-02 | 03 | 2 | CLOD-10 | unit | `cd src-tauri && cargo test config::language_hint` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/transcription/mod.rs` — TranscriptionProvider trait definition stubs
- [ ] `src-tauri/src/transcription/openai.rs` — OpenAI provider stub tests
- [ ] `src-tauri/src/transcription/groq.rs` — Groq provider stub tests
- [ ] `src-tauri/src/transcription/openrouter.rs` — OpenRouter provider stub tests
- [ ] `src-tauri/src/transcription/retry.rs` — Retry/backoff logic stub tests
- [ ] Update `src-tauri/Cargo.toml` — add `reqwest` with `multipart`+`json` features and `base64 = "0.22"`

*Wave 0 creates test file stubs before implementation begins.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Invalid API key opens settings on correct tab | CLOD-06 | Requires live UI + Tauri window navigation | Set invalid key, trigger transcription, verify settings opens on Transcription tab with provider highlighted |
| Fallback toast appears and works | CLOD-09 | Requires network-level failure simulation | Disable primary provider's connectivity, trigger transcription, verify toast offers fallback provider |
| Language hint persists across restarts | CLOD-02 | Requires app restart | Set language hint, restart app, verify value persists |
| Retry button re-triggers transcription | CLOD-08 | Requires UI interaction + event flow | Simulate network failure, click retry in toast, verify retry attempt happens |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
