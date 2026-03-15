---
wave: 2
depends_on:
  - 03-01-capture-lifecycle-PLAN.md
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/src/audio/mod.rs
  - src-tauri/src/audio/encode.rs
  - src-tauri/src/config/mod.rs
  - src-tauri/src/hotkey/service.rs
autonomous: true
requirements:
  - AUDI-03
---

<plan>
<goal>
Add format-specific encoding at finalize time: Opus for cloud providers, WAV for local provider, with one shared retry-once failure policy.
</goal>

<must_haves>
- Encoder decision uses active provider at stop time (not start-time snapshot).
- Cloud route returns Ogg/Opus bytes; local route returns WAV PCM16 bytes.
- Encode failures retry once, then fail with explicit error event and clean state recovery.
- Encoding is backend-owned and remains isolated from settings UI logic.
</must_haves>

<tasks>
<task type="auto">
  <name>Task 1: Add encoder module and provider-driven selector</name>
  <files>src-tauri/src/audio/encode.rs, src-tauri/src/audio/mod.rs, src-tauri/src/config/mod.rs</files>
  <action>Implement `encode_for_provider(provider, pcm)` abstraction that branches to Opus (cloud) or WAV (local) from current `transcription.provider`. Keep encoding output in a transport-ready payload struct consumed by later transcription phases.</action>
  <verify>`cd src-tauri && cargo test audio::tests::selects_encoder_from_provider -- --exact` passes</verify>
  <done>Finalize path deterministically produces the right format from active provider contract.</done>
</task>

<task type="auto">
  <name>Task 2: Apply retry-once policy and integrate into stop/finalize flow</name>
  <files>src-tauri/src/audio/mod.rs, src-tauri/src/audio/encode.rs, src-tauri/src/hotkey/service.rs</files>
  <action>Wrap encoding in shared retry-once helper invoked by recording stop path, emit concise encode failure event on terminal failure, and guarantee state returns to `Idle` after failure to avoid stuck transcribing behavior.</action>
  <verify>`cd src-tauri && cargo test audio::tests::encode_retry_once_then_fail -- --exact` passes</verify>
  <done>Encoding failures are resilient and predictable without introducing cross-phase transcription side effects.</done>
</task>

<task type="auto">
  <name>Task 3: Add focused format contract tests</name>
  <files>src-tauri/src/audio/encode.rs, src-tauri/src/audio/mod.rs</files>
  <action>Add unit tests validating output signatures/metadata for Opus and WAV branches plus failure retry behavior. Keep test seams deterministic via fake encoder adapters where needed.</action>
  <verify>`cd src-tauri && cargo test audio::tests::wav_output_contract -- --exact` and `cd src-tauri && cargo test audio::tests::opus_output_contract -- --exact` pass</verify>
  <done>AUDI-03 is covered by automated format-specific tests and integrated finalize behavior.</done>
</task>
</tasks>

<verification>
<criteria>
- Local provider always yields WAV and cloud providers always yield Opus.
- Stop path reads active provider at finalize time.
- Encode failure path retries once and exits cleanly.
</criteria>

<commands>
- `cd src-tauri && cargo test audio -- --nocapture`
- `cd src-tauri && cargo test`
</commands>
</verification>

<unresolved_questions>
None.
</unresolved_questions>
</plan>
