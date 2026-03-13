## ADDED Requirements

### Requirement: Settings window layout
The settings window SHALL be a single-page layout with four sections: General, Audio, Transcription, and Injection. The window SHALL be accessible from the system tray context menu and by double-clicking the tray icon.

#### Scenario: Settings window opens from tray
- **WHEN** the user opens settings from the tray
- **THEN** the settings window SHALL display all four sections: General, Audio, Transcription, Injection

### Requirement: General settings
The General section SHALL contain: a hotkey input field that captures key combinations (default: `Ctrl+Shift+Space`), a "Launch on Windows startup" toggle (default: OFF), and a "Minimize to tray on close" toggle.

#### Scenario: Hotkey field captures key combination
- **WHEN** the user clicks the hotkey field and presses a key combination
- **THEN** the field SHALL display the pressed combination and register it as the new global hotkey

#### Scenario: Startup toggle enables auto-launch
- **WHEN** the user enables "Launch on Windows startup"
- **THEN** VoxFlow SHALL be registered in the Windows startup registry

### Requirement: Audio settings
The Audio section SHALL contain a microphone device dropdown listing all available audio input devices, with the system default pre-selected.

#### Scenario: Microphone device dropdown populated
- **WHEN** the Audio section is displayed
- **THEN** all available audio input devices SHALL be listed, with the current selection highlighted

### Requirement: Transcription settings — cloud sub-section
The Transcription section SHALL have an engine toggle (Cloud / Local). The Cloud sub-section SHALL show a tabbed interface with three tabs: "OpenAI", "Groq", "OpenRouter". Each tab SHALL display the provider's logo alongside the label. Each tab SHALL contain: an API key input field (masked by default with a toggle to reveal), a model selector dropdown, a "Test connection" button, and a "Set as active" button. The currently active provider's tab SHALL be visually highlighted.

#### Scenario: OpenAI tab shows correct controls and models
- **WHEN** the user opens the OpenAI tab
- **THEN** a masked API key field, model dropdown (`whisper-1`, `gpt-4o-transcribe`, `gpt-4o-mini-transcribe`), "Test connection" button, and "Set as active" button SHALL be visible

#### Scenario: Groq tab shows correct controls and models
- **WHEN** the user opens the Groq tab
- **THEN** a masked API key field, model dropdown (`whisper-large-v3-turbo` pre-selected, `whisper-large-v3`, `distil-whisper-large-v3-en`), "Test connection" button, and "Set as active" button SHALL be visible

#### Scenario: OpenRouter tab shows correct controls and models
- **WHEN** the user opens the OpenRouter tab
- **THEN** a masked API key field, model dropdown (`google/gemini-2.5-flash` pre-selected, `google/gemini-2.5-pro`, `google/gemini-2.5-flash-lite`, `openai/gpt-4o-audio-preview`, `openai/gpt-audio`, `openai/gpt-audio-mini`), "Test connection" button, and "Set as active" button SHALL be visible

#### Scenario: Active provider visually highlighted
- **WHEN** OpenAI is the active provider
- **THEN** the OpenAI tab SHALL have a distinct visual indicator (colored border or badge)

#### Scenario: Test connection validates API key
- **WHEN** the user clicks "Test connection"
- **THEN** a test request SHALL be sent to the provider; success shows a green checkmark, failure shows an inline error message

### Requirement: Transcription settings — local sub-section
The Local sub-section SHALL list the four model variants (tiny, base, small, medium) with their approximate sizes and quality/speed descriptions. Each variant SHALL have a "Download" button (if not downloaded) or a "Delete" button (if downloaded). A progress bar SHALL be shown during active downloads.

#### Scenario: Model download progress shown
- **WHEN** a model is being downloaded
- **THEN** a progress bar SHALL show download progress and a cancel button SHALL be available

#### Scenario: Downloaded model shows delete option
- **WHEN** a model has been downloaded
- **THEN** a "Delete" button SHALL appear replacing the "Download" button

### Requirement: Injection settings
The Injection section SHALL contain: a method selector with three options ("FlashPaste (recommended)", "Keystrokes", "Clipboard"), an injection speed sub-option (slow/normal/fast) shown only when "Keystrokes" is selected, and an "Auto-fallback" checkbox (default: enabled).

#### Scenario: Keystroke speed shown only for keystroke mode
- **WHEN** "Keystrokes" injection method is selected
- **THEN** the injection speed selector (slow/normal/fast) SHALL be visible

#### Scenario: Keystroke speed hidden for other modes
- **WHEN** "FlashPaste" or "Clipboard" is selected
- **THEN** the injection speed selector SHALL be hidden

### Requirement: All settings persist across restarts
All settings changes SHALL be saved immediately to the config file. Settings SHALL be loaded and applied on app startup.

#### Scenario: Setting saved immediately on change
- **WHEN** the user changes any setting
- **THEN** the change SHALL be written to the config file without requiring a "Save" button press

#### Scenario: Settings restored on restart
- **WHEN** the app starts after a restart
- **THEN** all previously configured settings SHALL be applied
