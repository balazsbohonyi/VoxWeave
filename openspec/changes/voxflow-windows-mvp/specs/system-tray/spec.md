## ADDED Requirements

### Requirement: System tray icon
The system SHALL display a tray icon in the Windows notification area (system tray) when the app is running. The tray icon SHALL change appearance when recording is active (e.g. red dot overlay). The app SHALL NOT show a taskbar button for the main settings window.

#### Scenario: Tray icon visible on launch
- **WHEN** the app starts
- **THEN** a tray icon SHALL appear in the system tray notification area

#### Scenario: Tray icon changes during recording
- **WHEN** recording is active
- **THEN** the tray icon SHALL display a red dot or pulsing overlay to indicate active recording

#### Scenario: Tray icon reverts after recording
- **WHEN** recording stops
- **THEN** the tray icon SHALL revert to its default appearance

### Requirement: Tray context menu
Right-clicking the tray icon SHALL show a context menu with: "Settings", "Start Recording" / "Stop Recording" (toggles based on state), a separator, and "Quit".

#### Scenario: Context menu opens on right-click
- **WHEN** the user right-clicks the tray icon
- **THEN** a context menu SHALL appear with "Settings", "Start/Stop Recording", and "Quit" entries

#### Scenario: Start Recording from tray
- **WHEN** the user selects "Start Recording" from the tray menu while idle
- **THEN** recording SHALL begin (same as pressing the hotkey)

#### Scenario: Stop Recording from tray
- **WHEN** the user selects "Stop Recording" from the tray menu while recording
- **THEN** recording SHALL stop and transcription SHALL begin

#### Scenario: Quit from tray
- **WHEN** the user selects "Quit" from the tray menu
- **THEN** the application SHALL terminate completely

### Requirement: Open settings on tray double-click
Double-clicking the tray icon SHALL open or bring the settings window to focus.

#### Scenario: Double-click opens settings
- **WHEN** the user double-clicks the tray icon
- **THEN** the settings window SHALL open or be brought to the foreground

### Requirement: Minimize to tray
Closing the main settings window (via the window close button) SHALL minimize to tray rather than quitting the application.

#### Scenario: Window close minimizes to tray
- **WHEN** the user clicks the X button on the settings window
- **THEN** the window SHALL hide and the app SHALL continue running with only the tray icon visible
