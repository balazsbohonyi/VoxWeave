---
phase: 03
slug: audio-capture
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-15
updated: 2026-03-15
---

# Phase 03 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness + Vue/TS type checks |
| **Config file** | none - Wave 1 introduces audio module and focused tests |
| **Quick run command** | `cd src-tauri && cargo test audio -- --nocapture` |
| **Full suite command** | `cd src-tauri && cargo test && cd .. && npx vue-tsc --noEmit` |
| **Estimated runtime** | ~45 seconds |

---

## Sampling Rate

- **After every task commit:** run focused audio/hotkey tests touched by the task, then `cd src-tauri && cargo test audio -- --nocapture`
- **After every plan wave:** run `cd src-tauri && cargo test`
- **Before `$gsd-verify-work`:** Rust full suite and TypeScript typecheck must be green
- **Max feedback latency:** 45 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | AUDI-01, AUDI-02 | unit | `cd src-tauri && cargo test hotkey::tests::recording_transition_starts_audio_session -- --exact` | yes after Wave 1 | pending |
| 03-01-02 | 01 | 1 | AUDI-01, AUDI-05, AUDI-06 | unit | `cd src-tauri && cargo test audio::tests::missing_selected_device_falls_back -- --exact` and `cd src-tauri && cargo test audio::tests::no_device_returns_error -- --exact` | yes after Wave 1 | pending |
| 03-01-03 | 01 | 1 | AUDI-01, AUDI-02 | unit | `cd src-tauri && cargo test audio::tests::captures_mono_16khz_contract -- --exact` | yes after Wave 1 | pending |
| 03-02-01 | 02 | 2 | AUDI-03 | unit | `cd src-tauri && cargo test audio::tests::selects_encoder_from_provider -- --exact` | yes after Wave 2 | pending |
| 03-02-02 | 02 | 2 | AUDI-03 | unit | `cd src-tauri && cargo test audio::tests::encode_retry_once_then_fail -- --exact` | yes after Wave 2 | pending |
| 03-03-01 | 03 | 2 | AUDI-04 | unit | `cd src-tauri && cargo test commands::audio::tests::lists_input_devices -- --exact` | yes after Wave 2 | pending |
| 03-03-02 | 03 | 2 | AUDI-04, AUDI-05 | manual | `cargo tauri dev` then verify dropdown persistence and fallback warning flow | yes after Wave 2 | pending |
| 03-03-03 | 03 | 2 | AUDI-04 | static | `npx vue-tsc --noEmit` | yes | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

Existing infrastructure is sufficient once Wave 1 creates the `audio` module and initial test seams. No separate bootstrap phase is required.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Recording starts perceptibly fast from hotkey | AUDI-02 | 200ms latency target must be validated in real desktop runtime | Run `cargo tauri dev`, focus another app, press hotkey, verify indicator/state enters recording immediately |
| Selected device disconnect falls back to default with warning | AUDI-05 | Requires real OS device plug/unplug behavior | Start with explicit mic selected, disconnect it, verify fallback and user-visible warning |
| No microphone available shows error and no stuck recording | AUDI-06 | Requires machine-level device removal scenario | Disable/remove all input devices, trigger hotkey, verify error and state remains/reverts to `Idle` |
| Settings dropdown lists devices and persists selected value | AUDI-04 | Requires UI + persistence + restart path | Select a mic in settings, restart app, confirm selection is restored |

---

## Validation Sign-Off

- [x] All tasks have automated verify or explicit manual verification
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all missing references
- [x] No watch-mode flags
- [x] Feedback latency < 45s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** planned 2026-03-15
