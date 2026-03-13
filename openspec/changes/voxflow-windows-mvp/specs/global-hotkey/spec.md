## ADDED Requirements

### Requirement: Configurable global hotkey registration
The system SHALL register a global hotkey that triggers even when VoxFlow is not the focused application. The default hotkey SHALL be `Ctrl+Shift+Space`. The user SHALL be able to change the hotkey in settings. The hotkey SHALL be re-registered on app startup using the saved configuration.

#### Scenario: Default hotkey triggers from any application
- **WHEN** VoxFlow is running and a third-party application has focus
- **THEN** pressing `Ctrl+Shift+Space` SHALL invoke the recording toggle handler

#### Scenario: Custom hotkey is saved and re-registered on restart
- **WHEN** the user sets a new hotkey in settings and restarts the app
- **THEN** the custom hotkey SHALL be registered (not the default) and work globally

#### Scenario: Hotkey conflict detected
- **WHEN** the user attempts to register a hotkey that is already claimed by another application or the OS
- **THEN** VoxFlow SHALL display a warning message indicating the conflict and SHALL NOT override the conflicting registration

### Requirement: Toggle-mode recording
The hotkey SHALL operate in toggle mode. The first press SHALL start recording. The second press SHALL stop recording and trigger transcription. There SHALL be no hold-to-record behavior.

#### Scenario: First press starts recording
- **WHEN** the app is idle and the hotkey is pressed
- **THEN** recording SHALL begin within 200ms and the floating indicator SHALL appear

#### Scenario: Second press stops recording and triggers transcription
- **WHEN** the app is recording and the hotkey is pressed again
- **THEN** recording SHALL stop, the floating indicator SHALL enter processing state, and transcription SHALL be initiated

#### Scenario: Hotkey press during transcription or injection is ignored
- **WHEN** the app is already processing (transcribing) or injecting text
- **THEN** pressing the hotkey SHALL cancel the current injection (if injecting) or be ignored (if transcribing)
