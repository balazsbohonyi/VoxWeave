## ADDED Requirements

### Requirement: Config file storage
All user settings SHALL be persisted to a JSON config file at `%APPDATA%/VoxFlow/config.json`. The config SHALL be loaded on app startup and written whenever any setting changes. The config file SHALL be created with sensible defaults on first launch.

#### Scenario: Config created on first launch
- **WHEN** the app starts for the first time and no config file exists
- **THEN** a config file SHALL be created at `%APPDATA%/VoxFlow/config.json` with all default values after wizard completion

#### Scenario: Config loaded on startup
- **WHEN** the app starts and a config file exists
- **THEN** all settings SHALL be loaded from the config and applied before the app becomes operational

#### Scenario: Config written on setting change
- **WHEN** any setting is changed in the UI
- **THEN** the updated config SHALL be written to disk immediately without requiring a "Save" button

### Requirement: Config covers all user preferences
The config file SHALL store: active transcription engine (cloud/local), active cloud provider, per-provider API keys and model selections, language hint per provider, global hotkey, audio input device, local model size selection, injection method, injection speed, auto-fallback enabled flag, auto-start on Windows startup flag, floating indicator position (last x/y coordinates), and first-launch completed flag.

#### Scenario: All settings round-trip correctly
- **WHEN** the user changes every configurable setting and restarts the app
- **THEN** all settings SHALL be exactly as configured before the restart

### Requirement: Config migration resilience
If the config file contains unknown fields (from a future version) or is missing fields (from an older version), the app SHALL load successfully using defaults for missing fields and ignoring unknown fields.

#### Scenario: Missing fields use defaults
- **WHEN** the config file is missing a field (e.g. after a downgrade)
- **THEN** the app SHALL use the default value for that field and continue normally

#### Scenario: Unknown fields ignored
- **WHEN** the config file contains fields not recognized by the current version
- **THEN** the app SHALL load successfully without error, ignoring the unknown fields
