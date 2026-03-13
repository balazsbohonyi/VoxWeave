## ADDED Requirements

### Requirement: On-demand model download
Local transcription models SHALL NOT be bundled with the installer. The user SHALL download models on-demand from within the settings UI. Available model variants SHALL be: tiny (~75MB), base (~150MB), small (~500MB), medium (~1.5GB). Each variant SHALL display its approximate size and a quality/speed description. A progress bar and cancel option SHALL be shown during download.

#### Scenario: Model download with progress
- **WHEN** the user clicks "Download" for a model variant
- **THEN** a progress bar SHALL appear showing download progress, and a cancel button SHALL be available

#### Scenario: Download cancellation
- **WHEN** the user clicks cancel during a model download
- **THEN** the download SHALL stop and any partial file SHALL be cleaned up

#### Scenario: Downloaded model available for transcription
- **WHEN** a model has been successfully downloaded
- **THEN** it SHALL appear as "Downloaded" in the settings UI and SHALL be selectable as the active local model

### Requirement: Model storage and management
Downloaded models SHALL be stored in `%APPDATA%/VoxFlow/models/`. A "Delete model" option SHALL be available per downloaded model to free disk space. The settings SHALL display the current disk usage per model.

#### Scenario: Model deletion
- **WHEN** the user clicks "Delete" for a downloaded model
- **THEN** the model file SHALL be removed from disk and the model SHALL revert to "not downloaded" status

### Requirement: Local transcription on background thread
Transcription using whisper.cpp SHALL run on a dedicated `std::thread` (not the async runtime) to avoid blocking the UI. Audio SHALL be passed as WAV/PCM float32 samples. Transcription SHALL complete within 3 seconds for recordings up to 30 seconds using the base model.

#### Scenario: UI remains responsive during local transcription
- **WHEN** local transcription is in progress
- **THEN** the settings window, floating indicator, and system tray SHALL remain interactive

#### Scenario: Local transcription result
- **WHEN** whisper.cpp completes transcription
- **THEN** the transcribed text SHALL be passed to the injection pipeline

### Requirement: Missing or corrupt model error
If the active local model file is missing or fails integrity checks, the system SHALL show an error notification with a prompt to re-download the model.

#### Scenario: Missing model file error
- **WHEN** recording stops and the selected local model file does not exist
- **THEN** transcription SHALL fail with an error toast: "Model file missing — re-download in Settings?"

#### Scenario: Corrupt model file error
- **WHEN** whisper.cpp fails to load the model file
- **THEN** an error toast SHALL appear prompting the user to re-download the model
