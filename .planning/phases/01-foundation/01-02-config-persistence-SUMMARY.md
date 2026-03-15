---
phase: "01-foundation"
plan: "01-02"
subsystem: "config"
tags: [config, persistence, serde, rust, typescript]
dependency_graph:
  requires: ["01-01-scaffold-platform"]
  provides: ["config-schema", "config-persistence", "config-ipc"]
  affects: ["all future phases that read/write config"]
tech_stack:
  added:
    - "dirs-next = 2 (cross-platform config dir resolution)"
    - "tempfile = 3 (dev-dependency for test infrastructure)"
  patterns:
    - "Raw JSON round-trip for unknown-field preservation (merge_into)"
    - "Serde defaults (#[serde(default)]) for forward-compatible schema evolution"
    - "AppState::load() pattern: disk load at startup, fallback to defaults on error"
key_files:
  created:
    - "src-tauri/src/config/persistence.rs (load/save/merge_into + 9 tests)"
    - "src-tauri/src/commands/config.rs (get_config, save_config Tauri commands)"
  modified:
    - "src-tauri/src/config/mod.rs (full v1 AppConfig with nested sections)"
    - "src-tauri/src/state.rs (added config_raw Arc<Mutex<Value>>, AppState::load())"
    - "src-tauri/src/lib.rs (AppState::load(), updated invoke_handler path)"
    - "src-tauri/src/commands/mod.rs (pub mod config, direct module path for macros)"
    - "src-tauri/Cargo.toml (added dirs-next, tempfile dev-dep)"
    - "src/types/index.ts (nested TS interfaces mirroring Rust structs)"
    - "src/composables/useConfig.ts (real loadConfig/saveConfig via invoke)"
decisions:
  - "Nested config sections (AudioConfig, TranscriptionConfig, etc.) rather than flat struct — enables serde defaults at section level, cleaner IPC serialization"
  - "Raw JSON preserved in AppState.config_raw — merge_into on save keeps unknown keys from future app versions"
  - "commands/mod.rs uses pub mod config (not re-exports) — tauri generate_handler! macro requires direct module path"
  - "AppState::load() at startup replaces AppState::new() — config lives on disk from day one, no TODO left"
  - "Malformed config renamed to .corrupt.{timestamp}, never silently overwritten"
metrics:
  duration_minutes: 6
  tasks_completed: 5
  tasks_total: 5
  files_created: 2
  files_modified: 7
  tests_added: 9
  completed_date: "2026-03-14"
requirements_satisfied:
  - CONF-01
  - CONF-02
  - CONF-03
---

# Phase 1 Plan 02: Config Persistence Summary

**One-liner:** Full v1 AppConfig with nested sections persisted to `%APPDATA%/VoxFlow/config.json` via raw-JSON round-trip preserving unknown future fields.

## What Was Built

### Rust

**`config/mod.rs`** — Expanded from a flat struct to nested sections:
- `AudioConfig`: device, vad_threshold (0.01), vad_silence_ms (1500ms)
- `TranscriptionConfig`: provider, API keys, model selections, language hint, local_model_path
- `InjectionConfig`: mode
- `IndicatorConfig`: show, position_x/y (nullable for "centered" default)
- Root: hotkey, launch_at_login, first_launch flag

All fields carry `#[serde(default)]` for forward-compatible schema evolution.

**`config/persistence.rs`** — Load/save with unknown-field preservation:
- `config_path()` resolves to `%APPDATA%/VoxFlow/config.json` via `dirs-next`
- `load()`: no file → defaults; malformed → rename to `.corrupt.{ts}`, return defaults; partial → serde fills missing fields
- `save(config, raw)`: serializes typed config → merges INTO raw via `merge_into()` → writes pretty JSON
- `merge_into()`: recursive deep merge; dst keys not in src are preserved

**`state.rs`** — `AppState` gains `config_raw: Arc<Mutex<Value>>`. `AppState::load()` reads disk at startup; errors fall back to defaults with `log::error`.

**`commands/config.rs`** — Thin handlers: `get_config` returns clone of typed config; `save_config` calls `persistence::save()` then updates in-memory config.

### Frontend

**`src/types/index.ts`** — Nested TypeScript interfaces (`AudioConfig`, `TranscriptionConfig`, `InjectionConfig`, `IndicatorConfig`, `AppConfig`) exactly mirroring Rust structs.

**`src/composables/useConfig.ts`** — `loadConfig()` invokes `get_config`; `saveConfig(updates)` merges with current config and invokes `save_config`.

## Tests

9 tests in `config::persistence::tests`:

| Test | Covers |
|------|--------|
| `test_default_config_values` | All defaults have correct values |
| `test_no_file_returns_defaults` | Missing file path |
| `test_partial_json_fills_defaults` | Partial JSON + serde defaults |
| `test_unknown_fields_preserved_on_round_trip` | Core unknown-field contract |
| `test_merge_into_preserves_unknown_keys` | Top-level unknown key survival |
| `test_merge_into_nested_objects` | Nested object unknown key survival |
| `test_malformed_json_parse_produces_default_value` | Corrupt file detection |
| `test_save_and_load_round_trip` | Typed save + unknown field preservation |
| `test_config_dir_returns_path` | Path ends with "VoxFlow" |

All 9 pass. `cargo test` clean. `npx vue-tsc --noEmit` clean.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Tauri generate_handler! macro requires direct module path**
- **Found during:** Task 01-02-04
- **Issue:** `pub use config::{get_config, save_config}` re-exports are not visible to `generate_handler!` macro's symbol resolution
- **Fix:** Changed `commands/mod.rs` to `pub mod config`; updated `lib.rs` invoke_handler to `commands::config::get_config`
- **Files modified:** `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`
- **Commit:** 249ea06

## Self-Check: PASSED

Files exist:
- [x] src-tauri/src/config/persistence.rs
- [x] src-tauri/src/commands/config.rs
- [x] src-tauri/src/config/mod.rs (modified)
- [x] src-tauri/src/state.rs (modified)
- [x] src/types/index.ts (modified)
- [x] src/composables/useConfig.ts (modified)

Commits exist:
- [x] c172bc4 — feat(01-02): define full v1 AppConfig schema with nested sections
- [x] 9c5274e — feat(01-02): implement config persistence with unknown-field preservation
- [x] 96242e5 — feat(01-02): update AppState to hold raw JSON and load config from disk on startup
- [x] 249ea06 — feat(01-02): wire config commands and composable to real disk persistence
