---
phase: 7
slug: pipeline-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + vitest (Vue/TS) |
| **Config file** | src-tauri/Cargo.toml / vite.config.ts |
| **Quick run command** | `cd src-tauri && cargo test` |
| **Full suite command** | `cd src-tauri && cargo test && npx vue-tsc --noEmit && npm run lint` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test`
- **After every plan wave:** Run `cd src-tauri && cargo test && npx vue-tsc --noEmit && npm run lint`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 7-01-01 | 01 | 1 | NOTF-01 | unit | `cd src-tauri && cargo test success_toast` | ❌ W0 | ⬜ pending |
| 7-01-02 | 01 | 1 | NOTF-01 | unit | `cd src-tauri && cargo test injection_result_label` | ❌ W0 | ⬜ pending |
| 7-02-01 | 02 | 1 | NOTF-02 | unit | `cd src-tauri && cargo test error_toast` | ❌ W0 | ⬜ pending |
| 7-02-02 | 02 | 1 | NOTF-03 | unit | `cd src-tauri && cargo test cancel_toast` | ❌ W0 | ⬜ pending |
| 7-03-01 | 03 | 2 | NOTF-04 | manual | — | N/A | ⬜ pending |
| 7-03-02 | 03 | 2 | NOTF-04 | manual | — | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/hotkey/tests.rs` — stubs for NOTF-01, NOTF-02, NOTF-03 toast dispatch
- [ ] `src-tauri/src/injection/tests.rs` — stubs for injection result label mapping

*Existing test infrastructure (cargo test) covers the framework requirement.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Success toast auto-dismisses after 10s | NOTF-04 | Timer behavior requires live UI | Start recording, inject text, observe toast disappears after 10s |
| Error toast does NOT auto-dismiss | NOTF-04 | Negative timer behavior requires live UI | Trigger error (bad API key), confirm toast stays indefinitely |
| Cancel toast shows char count | NOTF-03 | Requires actual keystroke injection mid-flight cancel | Start recording, begin keystroke injection, cancel, confirm "Cancelled — N of M chars typed" |
| Indicator stays visible during 1s success flash | NOTF-01 | Indicator timing/visibility is visual only | Start recording, inject text, confirm indicator shows green flash then transitions to idle |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
