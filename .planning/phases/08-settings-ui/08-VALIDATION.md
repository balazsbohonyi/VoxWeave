---
phase: 8
slug: settings-ui
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (built-in) + `vue-tsc` + ESLint |
| **Config file** | `src-tauri/Cargo.toml` — no separate test config |
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
| 8-01-01 | 01 | 1 | SETT-07 | unit | `cd src-tauri && cargo test config` | ❌ Wave 0 | ⬜ pending |
| 8-01-02 | 01 | 1 | SETT-07 | unit | `cd src-tauri && cargo test config` | ❌ Wave 0 | ⬜ pending |
| 8-01-03 | 01 | 1 | SETT-04 | unit | `cd src-tauri && cargo test transcription` | ✅ partial | ⬜ pending |
| 8-01-04 | 01 | 1 | SETT-04 | unit | `cd src-tauri && cargo test transcription` | ❌ Wave 0 | ⬜ pending |
| 8-02-01 | 02 | 2 | SETT-01 | manual | `cargo tauri dev` visual check | ❌ Wave 0 | ⬜ pending |
| 8-02-02 | 02 | 2 | SETT-02 | unit | `cd src-tauri && cargo test config` | ✅ partial | ⬜ pending |
| 8-02-03 | 02 | 2 | SETT-03 | unit | `cd src-tauri && cargo test audio` | ❌ Wave 0 | ⬜ pending |
| 8-03-01 | 03 | 2 | SETT-04 | unit | `cd src-tauri && cargo test transcription` | ✅ partial | ⬜ pending |
| 8-03-02 | 03 | 2 | SETT-04 | manual | `cargo tauri dev` visual check | ❌ Wave 0 | ⬜ pending |
| 8-03-03 | 03 | 2 | SETT-05 | manual | `cargo tauri dev` visual check | ❌ Wave 0 | ⬜ pending |
| 8-04-01 | 04 | 2 | SETT-06 | unit | `cd src-tauri && cargo test config` | ✅ partial | ⬜ pending |
| 8-04-02 | 04 | 2 | SETT-02 | unit | `cd src-tauri && cargo test config` | ✅ partial | ⬜ pending |
| 8-05-01 | 05 | 3 | SETT-01 | manual | `cargo tauri dev` visual + `npx vue-tsc --noEmit` | ❌ Wave 0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/config/mod.rs` — migration round-trip tests: old flat JSON → new nested struct, new nested JSON → struct, unknown fields preserved
- [ ] `src-tauri/src/commands/config.rs` — `get_provider_models` unit test: assert correct model lists per provider
- [ ] TypeScript typecheck baseline: `npx vue-tsc --noEmit` passes after `src/types/index.ts` changes

*These are required before Plan 01 execution tasks begin.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Settings window renders four sections correctly | SETT-01 | UI layout, no DOM test framework | Launch `cargo tauri dev`, open settings, verify 4 sections visible |
| Hotkey capture widget captures and previews key combos | SETT-02 | Keyboard event simulation across OS | Launch app, click hotkey field, press combo, verify preview and save |
| Local transcription section renders as stub | SETT-05 | Visual rendering, no local models available in dev | Launch app, open settings, verify Local tab shows model list with sizes |
| Test connection shows inline result | SETT-04 | Network + UI interaction | Enter valid/invalid API key, click Test, verify spinner → result |
| Auto-stop silence duration input enables/disables with toggle | SETT-03 | UI state interaction | Toggle silence detection, verify input field enables/disables |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
