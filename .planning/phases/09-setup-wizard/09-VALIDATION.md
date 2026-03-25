---
phase: 9
slug: setup-wizard
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-24
---

# Phase 9 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) + TypeScript typecheck (`vue-tsc`) |
| **Config file** | `src-tauri/` (run Rust tests from there) |
| **Quick run command** | `npx vue-tsc --noEmit` |
| **Full suite command** | `cd src-tauri && cargo test && npx vue-tsc --noEmit` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx vue-tsc --noEmit`
- **After every plan wave:** Run `cd src-tauri && cargo test && npx vue-tsc --noEmit`
- **Before `/gsd:verify-work`:** Full build `cargo tauri build` + manual smoke through all 3 steps
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 9-01-01 | 01 | 0 | WIZR-01 | build | `cd src-tauri && cargo build` | ❌ W0 | ⬜ pending |
| 9-01-02 | 01 | 0 | WIZR-01 | typecheck | `npx vue-tsc --noEmit` | ❌ W0 | ⬜ pending |
| 9-01-03 | 01 | 1 | WIZR-01 | manual | — | N/A | ⬜ pending |
| 9-02-01 | 02 | 1 | WIZR-02 | typecheck | `npx vue-tsc --noEmit` | ✅ | ⬜ pending |
| 9-02-02 | 02 | 1 | WIZR-03 | typecheck | `npx vue-tsc --noEmit` | ✅ | ⬜ pending |
| 9-02-03 | 02 | 1 | WIZR-04 | typecheck | `npx vue-tsc --noEmit` | ✅ | ⬜ pending |
| 9-03-01 | 03 | 2 | WIZR-05 | typecheck | `npx vue-tsc --noEmit` | ✅ | ⬜ pending |
| 9-03-02 | 03 | 2 | WIZR-06 | typecheck | `npx vue-tsc --noEmit` | ✅ | ⬜ pending |
| 9-03-03 | 03 | 2 | WIZR-05 | manual | — | N/A | ⬜ pending |
| 9-03-04 | 03 | 2 | WIZR-06 | manual | — | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `wizard.html` — Vite MPA entry point (none yet)
- [ ] `src/windows/wizard/main.ts` — wizard entry point (none yet)
- [ ] `src/windows/wizard/App.vue` — step router shell (none yet)
- [ ] `src-tauri/src/commands/wizard.rs` — `open_wizard_window` command stub (none yet)

*Note: No new test framework needed. All automated validation is `vue-tsc` + `cargo test`.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| First-launch flag triggers wizard window on startup | WIZR-01 | Window lifecycle, no JS test framework | Delete config.json, launch app, confirm wizard opens instead of tray-only |
| Step 1 engine choice saves to config | WIZR-02 | UI interaction + config persistence | Select Cloud, finish wizard, confirm `transcription.provider` in config.json |
| Step 2 API key + test connection (Cloud) | WIZR-03 | Network call + inline validation | Enter valid/invalid key, confirm green/red feedback; confirm saved to `providers.openai.api_key` |
| Step 3 hotkey confirm/change | WIZR-04 | Global hotkey registration | Change hotkey, finish, confirm new hotkey works in app |
| Finish sets `first_launch=false`, success message shown | WIZR-05 | Config mutation + UI transition | Click Finish, confirm success banner appears, wizard closes, Settings opens |
| Re-open wizard from Settings | WIZR-06 | Window management | Open Settings → General → click "Setup Wizard...", confirm wizard opens |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
