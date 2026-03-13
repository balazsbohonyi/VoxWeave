## ADDED Requirements

### Requirement: Floating recording indicator window
The system SHALL display a small floating window (~200×48px, pill-shaped) when recording starts. The window SHALL be always-on-top, click-through (does not receive mouse events), and SHALL NOT steal focus from the active application. The window SHALL disappear when recording stops (after the transcription completes and text is injected or an error is shown).

#### Scenario: Indicator appears on recording start
- **WHEN** the recording hotkey is pressed and recording begins
- **THEN** the floating indicator SHALL appear within 200ms on screen

#### Scenario: Indicator does not steal focus
- **WHEN** the floating indicator is visible
- **THEN** the previously active application SHALL retain focus and keyboard input SHALL go to that application

#### Scenario: Indicator is click-through
- **WHEN** the user clicks in the area occupied by the floating indicator
- **THEN** the click SHALL pass through to the application behind it

#### Scenario: Indicator disappears after injection
- **WHEN** text has been successfully injected (or clipboard copy completed)
- **THEN** the floating indicator SHALL close

### Requirement: Real-time audio waveform visualization
The floating indicator SHALL display a real-time waveform visualization (5–10 amplitude bars) reflecting microphone input levels during recording. The visualization SHALL update at ≥ 24fps without visible jank.

#### Scenario: Waveform reflects audio input
- **WHEN** the user speaks while recording
- **THEN** the waveform bars SHALL animate in response to voice amplitude

#### Scenario: Waveform is silent when no audio detected
- **WHEN** recording is active but no audio is detected
- **THEN** the waveform bars SHALL display at or near minimum height

### Requirement: Indicator state transitions
The floating indicator SHALL show distinct visual states: recording (red pulsing dot + waveform), processing (spinner or indeterminate animation replacing waveform), and injecting (brief "pasting"/"typing" cue).

#### Scenario: Processing state after recording stops
- **WHEN** recording stops and transcription is in progress
- **THEN** the waveform SHALL be replaced by a spinner or indeterminate animation

#### Scenario: Injection state during text injection
- **WHEN** text injection (FlashPaste or keystroke) is in progress
- **THEN** the indicator SHALL show a brief "pasting" or "typing" visual cue

### Requirement: Draggable position with persistence
The floating indicator SHALL be draggable to any screen position. The last position SHALL be saved and restored on the next recording session.

#### Scenario: Position persists across sessions
- **WHEN** the user drags the indicator to a new position and then starts a new recording session
- **THEN** the indicator SHALL appear at the previously saved position
