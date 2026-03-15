---
phase: 02
slug: hotkey
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-15
updated: 2026-03-15
---

# Phase 02 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness |
| **Config file** | none - Wave 1 creates the hotkey module and focused tests |
| **Quick run command** | `cargo test hotkey -- --nocapture` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run the narrowest relevant hotkey target, then `cargo test hotkey -- --nocapture` once the task's focused tests exist
- **After every plan wave:** Run `cargo test`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | HOTK-01 | unit | `cargo test hotkey::tests::default_hotkey_press_starts_recording -- --exact` | yes after Wave 1 | pending |
| 02-01-02 | 01 | 1 | HOTK-01, HOTK-02 | unit | `cargo test hotkey::tests::toggle_respects_state_machine -- --exact` | yes after Wave 1 | pending |
| 02-01-03 | 01 | 1 | HOTK-01, HOTK-02 | unit | `cargo test hotkey` | yes after Wave 1 | pending |
| 02-02-01 | 02 | 2 | HOTK-03 | unit | `cargo test hotkey::tests::apply_hotkey_change_persists_canonical_value -- --exact` | yes after Wave 2 | pending |
| 02-02-01 | 02 | 2 | HOTK-03 | unit | `cargo test hotkey::tests::unchanged_canonical_save_short_circuit -- --exact` | yes after Wave 2 | pending |
| 02-02-02 | 02 | 2 | HOTK-04 | unit | `cargo test hotkey::tests::conflicting_hotkey_keeps_last_working_binding -- --exact` | yes after Wave 2 | pending |
| 02-02-02 | 02 | 2 | HOTK-04 | unit | `cargo test hotkey::tests::startup_conflict_leaves_app_inactive -- --exact` | yes after Wave 2 | pending |
| 02-02-03 | 02 | 2 | HOTK-03, HOTK-04 | unit + manual | `cargo test hotkey && cargo test` | yes after Wave 2 | pending |
| 02-03-01 | 03 | 3 | HOTK-03 | manual | Open Settings, edit hotkey field, click Apply, confirm button shows `Saving...` then success text | yes in Wave 3 | pending |
| 02-03-02 | 03 | 3 | HOTK-03, HOTK-04 | manual | Try saving a conflicting hotkey and confirm inline error text is shown while prior active hotkey remains displayed | yes in Wave 3 | pending |
| 02-03-03 | 03 | 3 | HOTK-03 | manual | Save a valid new hotkey, restart app, verify Settings shows the canonical saved value and hotkey still triggers globally | yes in Wave 3 | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

Existing infrastructure covers the phase once Wave 1 adds the `src-tauri/src/hotkey/` module and its focused test targets. No separate pre-phase bootstrap plan is required.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Global hotkey works from another focused application | HOTK-01 | Requires Windows desktop shell and another foreground app | Launch VoxFlow on Windows, focus a different app, press `Ctrl+Shift+Space`, confirm VoxFlow transitions to recording without stealing focus |
| Second press advances to processing placeholder and recovers to idle | HOTK-02 | Requires runtime state observation across real Tauri events | With recording active, press the hotkey again and confirm the app reaches `Transcribing` / processing briefly, then returns to `Idle` without getting stuck |
| Custom hotkey save takes effect immediately and survives restart | HOTK-03 | Requires real runtime re-registration and app restart | Change the hotkey in Settings, save, trigger it from another app, restart VoxFlow, and confirm the same canonical hotkey still works |
| Conflict warning focuses Settings and shows toast-like feedback | HOTK-04 | Requires real registration conflict plus desktop focus behavior | Occupy the same hotkey in another app if possible, try to save it in VoxFlow, confirm the old binding stays active, Settings is foregrounded, and the warning appears immediately |

### Gap-closure plan reference

- Plan: `02-03-hotkey-settings-input-PLAN.md`
- Scope: minimal Settings hotkey edit/apply UX using existing `save_config` backend flow.
- Manual verification command set: `cargo tauri dev` then execute the Wave 3 manual checklist above on Windows.

---

## Validation Sign-Off

- [x] All tasks have automated verify or explicit manual verification
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all missing references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** planned 2026-03-15
