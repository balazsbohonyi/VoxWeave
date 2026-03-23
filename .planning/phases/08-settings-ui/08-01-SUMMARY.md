---
phase: 08-settings-ui
plan: 01
subsystem: config
tags: [rust, serde, typescript, migration, transcription]

# Dependency graph
requires:
  - phase: 07-pipeline-integration
    provides: Working transcription pipeline using flat TranscriptionConfig fields
provides:
  - CloudProviderConfig and TranscriptionProviders structs in config/mod.rs
  - migrate_transcription_fields() for zero-downtime old-config migration
  - New nested providers shape in TypeScript types/index.ts
affects: [08-02, 08-03, 08-04, 08-05]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Struct-level migration at load time (not disk rewrite) for backward compat", "TDD: RED commit (compile-fail tests) before GREEN (implementation)"]

key-files:
  created: []
  modified:
    - src-tauri/src/config/mod.rs
    - src-tauri/src/config/persistence.rs
    - src-tauri/src/transcription/openai.rs
    - src-tauri/src/transcription/groq.rs
    - src-tauri/src/transcription/service.rs
    - src/types/index.ts

key-decisions:
  - "migrate_transcription_fields runs at load time on the raw JSON Value — no disk rewrite needed, old flat keys coexist safely"
  - "Migration skips if providers key already present — idempotent and safe to call on every load"
  - "CloudProviderConfig.model defaults to empty string; per-provider defaults set via default_openai_provider/default_groq_provider fns in TranscriptionProviders"

patterns-established:
  - "Old flat fields in config are promoted to nested structs in persistence.rs load() before serde deserialization"
  - "TranscriptionConfig field access: config.providers.openai.api_key / config.providers.groq.model"

requirements-completed: [SETT-07]

# Metrics
duration: 5min
completed: 2026-03-22
---

# Phase 8 Plan 01: TranscriptionConfig Nested Providers Migration Summary

**CloudProviderConfig + TranscriptionProviders structs replace 4 flat fields; old on-disk configs migrate automatically via migrate_transcription_fields() at load time**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-22T19:32:12Z
- **Completed:** 2026-03-22T19:37:30Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- Replaced openai_api_key, groq_api_key, openai_model, groq_model flat fields with providers: TranscriptionProviders struct
- Added migrate_transcription_fields() in persistence.rs called at load time to promote old flat JSON keys to new nested shape — no API key loss on upgrade
- Updated openai.rs, groq.rs, service.rs to read config.providers.openai/groq.*
- Updated TypeScript TranscriptionConfig to match: added CloudProviderConfig/TranscriptionProviders interfaces, removed openrouter fields, added fallback_order

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Failing tests for nested providers** - `f32901d` (test)
2. **Task 1 GREEN: Migrate TranscriptionConfig struct + migration fn** - `182e0b0` (feat)
3. **Task 2: Document pre-existing injection test failures** - `a90ec6e` (chore)
4. **Task 3: Update TypeScript types** - `46f3dfb` (feat)

_Note: Tasks 1 and 2 were executed together since all Rust changes were required to compile (service.rs tests also referenced old flat fields)._

## Files Created/Modified
- `src-tauri/src/config/mod.rs` — Added CloudProviderConfig, TranscriptionProviders structs; replaced 4 flat fields with providers field; new tests
- `src-tauri/src/config/persistence.rs` — Added migrate_transcription_fields(); call in load() path; 3 new migration tests; updated old tests
- `src-tauri/src/transcription/openai.rs` — config.providers.openai.model and .api_key
- `src-tauri/src/transcription/groq.rs` — config.providers.groq.model and .api_key
- `src-tauri/src/transcription/service.rs` — find_fallback_provider uses providers.*; test helpers updated
- `src/types/index.ts` — TranscriptionProvider drops openrouter; CloudProviderConfig + TranscriptionProviders added; fallback_order added

## Decisions Made
- Migration is idempotent: skips if `providers` key already present in raw JSON
- Migration only sets `providers` if at least one old flat key exists — avoids injecting empty object on fresh configs
- Pre-existing injection test failures (5 tests) logged to deferred-items.md and left untouched (out of scope)

## Deviations from Plan

None — plan executed exactly as written. The service.rs test updates (using new struct fields) were anticipated by the plan's instructions to update test helpers.

## Issues Encountered
- 5 pre-existing injection::service tests fail (unrelated to this plan). Logged to `deferred-items.md`. These predate phase 08 work.
- `npm run lint` script does not exist in package.json — pre-existing gap. TypeScript check passes instead.

## Next Phase Readiness
- All downstream code (openai.rs, groq.rs, service.rs) targets the new struct shape
- TypeScript types mirror Rust structs exactly — UI components can safely read config.transcription.providers.openai.api_key
- Migration is transparent to users with old configs — API keys preserved on first launch after update
- Ready for 08-02 (Settings UI scaffold and tab layout)

---
*Phase: 08-settings-ui*
*Completed: 2026-03-22*
