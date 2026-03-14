---
phase: 01
slug: foundation
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-14
---

# Phase 01 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness + vue-tsc |
| **Config file** | none - Wave 1 creates the initial app scaffold |
| **Quick run command** | `cargo test config` |
| **Full suite command** | `cargo test && npx vue-tsc --noEmit` |
| **Estimated runtime** | ~45 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test config` once config tests exist; before that, run the narrowest available Rust test target touched by the task.
- **After every plan wave:** Run `cargo test && npx vue-tsc --noEmit`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 45 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-02 | 01 | 1 | none - scaffold prerequisite | smoke | `npx vue-tsc --noEmit` | ? pending | ? pending |
| 01-02-02 | 02 | 2 | CONF-01, CONF-02, CONF-03 | unit | `cargo test config` | ? pending | ? pending |
| 01-03-01 | 03 | 3 | TRAY-01, TRAY-02, TRAY-03, TRAY-04 | manual + smoke | `cargo test && npx vue-tsc --noEmit` | ? pending | ? pending |

*Status: ? pending · ? green · ? red · ?? flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tray-only startup | TRAY-01 | Requires native Windows tray runtime | Launch app on Windows and confirm no normal window appears while tray icon is visible |
| Tray menu shape | TRAY-02 | Native context menu behavior | Right-click tray icon and verify `Settings`, disabled `Start/Stop Recording`, separator, `Quit` |
| Tray double-click opens settings | TRAY-03 | Desktop shell interaction | Double-click tray icon and confirm existing settings window opens/focuses |
| Close hides to tray | TRAY-04 | Native window-close lifecycle | Close settings window and confirm app keeps running in tray |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 45s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-14
