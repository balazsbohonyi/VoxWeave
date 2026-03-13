## ADDED Requirements

### Requirement: Microphone audio capture
The system SHALL capture audio from the user-selected microphone (or system default) at 16kHz mono when recording is active. Recording SHALL start within 200ms of the hotkey press. The system SHALL request microphone permission on first use.

#### Scenario: Recording starts with low latency
- **WHEN** the recording hotkey is pressed and a microphone is available
- **THEN** audio capture SHALL begin within 200ms

#### Scenario: No microphone available
- **WHEN** the recording hotkey is pressed and no microphone device is found
- **THEN** recording SHALL NOT start and an error notification SHALL be displayed to the user

#### Scenario: Selected device disconnected during recording
- **WHEN** the user's selected microphone is disconnected mid-recording
- **THEN** recording SHALL stop, a notification SHALL be shown, and the system SHALL fall back to the system default device for future recordings

### Requirement: Audio encoding for transcription providers
The system SHALL encode captured audio as Opus (16kHz mono) when sending to cloud providers, and as WAV/PCM when passing to whisper.cpp for local transcription. Encoding SHALL complete within 100ms for up to 30 seconds of audio.

#### Scenario: Cloud transcription encoding
- **WHEN** recording stops and the active transcription engine is a cloud provider
- **THEN** the raw PCM buffer SHALL be encoded to Opus format before transmission

#### Scenario: Local transcription encoding
- **WHEN** recording stops and the active transcription engine is whisper.cpp
- **THEN** the raw PCM buffer SHALL be provided as WAV/PCM float32 samples to whisper.cpp (no Opus encoding)

### Requirement: Real-time amplitude data emission
The system SHALL compute RMS amplitude per ~33ms audio chunk and emit `audio-level` events to the frontend at approximately 30fps while recording is active.

#### Scenario: Waveform data flows to frontend
- **WHEN** recording is active
- **THEN** the frontend SHALL receive `audio-level` events at ~30fps with normalized amplitude values (0.0–1.0)

#### Scenario: No events emitted when not recording
- **WHEN** recording is not active
- **THEN** no `audio-level` events SHALL be emitted

### Requirement: Audio input device selection
The system SHALL enumerate all available audio input devices and allow the user to select one in settings. The selection SHALL persist across restarts. The default SHALL be the system default device.

#### Scenario: Device list populated in settings
- **WHEN** the settings Audio section is opened
- **THEN** all available audio input devices SHALL be listed in a dropdown

#### Scenario: Selected device used for recording
- **WHEN** the user selects a specific microphone device in settings
- **THEN** subsequent recordings SHALL use that device
