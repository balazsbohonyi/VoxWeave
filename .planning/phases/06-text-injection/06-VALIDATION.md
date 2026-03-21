---
phase: 6
slug: text-injection
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-21
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust unit tests) |
| **Config file** | src-tauri/Cargo.toml |
| **Quick run command** | `cd src-tauri && cargo test injection` |
| **Full suite command** | `cd src-tauri && cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test injection`
- **After every plan wave:** Run `cd src-tauri && cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 6-01-01 | 01 | 1 | INJC-01 | unit | `cd src-tauri && cargo test injection::flashpaste` | ❌ W0 | ⬜ pending |
| 6-01-02 | 01 | 1 | INJC-02 | unit | `cd src-tauri && cargo test injection::keystroke` | ❌ W0 | ⬜ pending |
| 6-01-03 | 01 | 1 | INJC-03 | unit | `cd src-tauri && cargo test injection::clipboard` | ❌ W0 | ⬜ pending |
| 6-02-01 | 02 | 1 | INJC-04 | unit | `cd src-tauri && cargo test injection::elevation` | ❌ W0 | ⬜ pending |
| 6-02-02 | 02 | 1 | INJC-05 | unit | `cd src-tauri && cargo test injection::escape_cancel` | ❌ W0 | ⬜ pending |
| 6-02-03 | 02 | 1 | INJC-06 | unit | `cd src-tauri && cargo test injection::fallback` | ❌ W0 | ⬜ pending |
| 6-03-01 | 03 | 2 | INJC-07 | unit | `cd src-tauri && cargo test injection::unicode` | ❌ W0 | ⬜ pending |
| 6-03-02 | 03 | 2 | INJC-08 | manual | N/A | N/A | ⬜ pending |
| 6-03-03 | 03 | 2 | INJC-09 | unit | `cd src-tauri && cargo test injection::config` | ❌ W0 | ⬜ pending |
| 6-04-01 | 04 | 2 | INJC-10 | unit | `cd src-tauri && cargo test injection::focus_restore` | ❌ W0 | ⬜ pending |
| 6-04-02 | 04 | 2 | INJC-11 | manual | N/A | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/injection/tests.rs` — test stubs for INJC-01 through INJC-11
- [ ] Module entry `mod tests;` added to `src-tauri/src/injection/mod.rs`

*Existing cargo test infrastructure covers all phase requirements — no new framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Terminal detection triggers correct paste shortcut | INJC-08 | Requires live terminal window (Windows Terminal, cmd) | Launch app, record speech, verify Ctrl+Shift+V used in terminal |
| Elevation dialog appears for elevated process | INJC-11 | Requires elevated target process | Launch notepad as admin, record speech, verify dialog appears |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
