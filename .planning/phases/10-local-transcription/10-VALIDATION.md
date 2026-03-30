---
phase: 10
slug: local-transcription
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust unit tests, existing infrastructure) |
| **Config file** | none — inline `#[cfg(test)]` modules |
| **Quick run command** | `cd src-tauri && cargo test --features local-transcription` |
| **Full suite command** | `cd src-tauri && cargo test --features local-transcription && npx vue-tsc --noEmit` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test --features local-transcription`
- **After every plan wave:** Run `cd src-tauri && cargo test --features local-transcription && npx vue-tsc --noEmit && npm run lint`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 1 | LOCL-01 | unit | `cd src-tauri && cargo test --features local-transcription transcription::local` | ❌ W0 | ⬜ pending |
| 10-01-02 | 01 | 1 | LOCL-06 | unit | `cd src-tauri && cargo test --features local-transcription audio` | ❌ W0 | ⬜ pending |
| 10-01-03 | 01 | 1 | LOCL-05 | unit | `cd src-tauri && cargo test --features local-transcription` | ❌ W0 | ⬜ pending |
| 10-01-04 | 01 | 1 | LOCL-07 | unit | `cd src-tauri && cargo test --features local-transcription transcription::local` | ❌ W0 | ⬜ pending |
| 10-02-01 | 02 | 1 | LOCL-02 | manual | N/A | N/A | ⬜ pending |
| 10-02-02 | 02 | 1 | LOCL-04 | unit | `cd src-tauri && cargo test --features local-transcription commands` | ❌ W0 | ⬜ pending |
| 10-02-03 | 02 | 1 | LOCL-03 | unit | `npx vue-tsc --noEmit` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/transcription/local.rs` — stubs for LOCL-01, LOCL-05, LOCL-06, LOCL-07
- [ ] `src-tauri/src/transcription/download.rs` or `src-tauri/src/commands/download.rs` — stubs for LOCL-04
- [ ] Feature flag: `[dependencies] whisper-rs = { version = "0.11", optional = true }` and `[features] local-transcription = ["dep:whisper-rs"]` in Cargo.toml

*Existing infrastructure (cargo test, vue-tsc) covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Download progress bar and cancel | LOCL-02 | Requires real HF HTTP download or mock server | 1. Open Settings > Transcription 2. Click Download on any model 3. Verify progress bar appears 4. Click Cancel, verify partial file deleted |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
