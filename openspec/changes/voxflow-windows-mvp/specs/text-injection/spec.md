## ADDED Requirements

### Requirement: FlashPaste injection (default)
The default text injection method SHALL be FlashPaste. The pipeline SHALL: (1) save the current foreground window handle, (2) check the target process integrity level, (3) read the current clipboard content and store it in memory, (4) detect whether the target window is a terminal emulator, (5) write the transcription text to the clipboard, (6) simulate the appropriate paste shortcut via `SendInput`, (7) wait 500ms, (8) restore the original clipboard content.

#### Scenario: FlashPaste in a standard application
- **WHEN** the active window is a standard application (browser, VS Code, Notepad, Word, Slack)
- **THEN** `Ctrl+V` SHALL be simulated and the transcription SHALL appear in the focused input

#### Scenario: FlashPaste in a terminal emulator
- **WHEN** the active window is a terminal emulator (cmd.exe, PowerShell, Windows Terminal, Git Bash)
- **THEN** `Ctrl+Shift+V` or `Shift+Insert` SHALL be simulated instead of `Ctrl+V`

#### Scenario: Original clipboard restored after FlashPaste
- **WHEN** FlashPaste completes
- **THEN** the user's original clipboard content SHALL be restored after 500ms

#### Scenario: Clipboard save/restore failure
- **WHEN** reading or restoring the original clipboard fails
- **THEN** the paste SHALL still complete and a warning toast SHALL be shown about the clipboard state

### Requirement: Simulated keystroke injection
The system SHALL support injecting text character-by-character via the Windows `SendInput` API with `KEYEVENTF_UNICODE` flags. Newline characters SHALL be injected as `VK_RETURN` keystrokes. Injection speed SHALL be configurable: slow (10ms/char), normal (5ms/char, default), fast (2ms/char).

#### Scenario: Character injection with Unicode support
- **WHEN** keystroke injection mode is active and transcription contains Unicode characters
- **THEN** each character SHALL be injected via `SendInput` with `KEYEVENTF_UNICODE`, preserving accented letters and symbols

#### Scenario: Newline injected as VK_RETURN
- **WHEN** the transcription contains a newline character and keystroke injection is used
- **THEN** a `VK_RETURN` keystroke SHALL be sent for each newline

#### Scenario: Injection speed respected
- **WHEN** the injection speed is set to "slow"
- **THEN** there SHALL be a 10ms delay between each character injection

### Requirement: Manual clipboard mode
When the injection method is set to "Clipboard", the system SHALL write the transcription to the clipboard and NOT auto-paste. The original clipboard content SHALL NOT be restored (the user explicitly wants the transcription on their clipboard).

#### Scenario: Transcription copied to clipboard
- **WHEN** clipboard mode is active and transcription completes
- **THEN** the transcription SHALL be written to the clipboard and a toast SHALL confirm "Copied to clipboard"

#### Scenario: No auto-paste in clipboard mode
- **WHEN** clipboard mode is active
- **THEN** no `Ctrl+V` or other paste shortcut SHALL be simulated

### Requirement: Terminal window detection
The system SHALL detect terminal emulator windows by their window class name before injection. Known terminal class names SHALL include at minimum: `ConsoleWindowClass` (cmd.exe), `CASCADIA_HOSTING_WINDOW_CLASS` (Windows Terminal), `mintty` (Git Bash), `VirtualConsoleClass` (PowerShell legacy).

#### Scenario: Terminal detected by window class
- **WHEN** the foreground window has a known terminal window class name
- **THEN** the terminal-appropriate paste shortcut SHALL be used

### Requirement: Elevation check before injection
Before FlashPaste or keystroke injection, the system SHALL check the target process integrity level. If the target runs at HIGH or SYSTEM integrity and VoxFlow runs at MEDIUM integrity, the system SHALL emit an `elevation-required` event and show a dialog with "Relaunch as Administrator" and "Copy to clipboard instead" options.

#### Scenario: Elevated target detected
- **WHEN** the foreground window belongs to a process with higher integrity than VoxFlow
- **THEN** a modal dialog SHALL appear offering relaunch or clipboard fallback

#### Scenario: Relaunch as administrator
- **WHEN** the user clicks "Relaunch as Administrator"
- **THEN** VoxFlow SHALL relaunch itself with elevated privileges via `ShellExecuteW` with the `runas` verb

#### Scenario: Clipboard fallback from elevation dialog
- **WHEN** the user clicks "Copy to clipboard instead"
- **THEN** the transcription SHALL be written to the clipboard without changing the global injection method setting

### Requirement: Injection cancellation
Pressing Escape or the recording hotkey during active keystroke injection SHALL cancel injection immediately. Already-injected characters SHALL remain. A toast SHALL show "Injection cancelled — X of Y characters typed".

#### Scenario: Escape cancels keystroke injection
- **WHEN** keystroke injection is in progress and the user presses Escape
- **THEN** injection SHALL stop immediately, remaining characters SHALL be discarded, and a cancellation toast SHALL appear

#### Scenario: Hotkey cancels keystroke injection
- **WHEN** keystroke injection is in progress and the recording hotkey is pressed
- **THEN** injection SHALL cancel with the same behavior as Escape cancellation

### Requirement: Automatic fallback chain
If the selected injection method fails, the system SHALL automatically fall back in this order: Keystrokes → FlashPaste → Clipboard. FlashPaste SHALL fall back to Clipboard. The fallback chain SHALL be enabled by default but configurable via a settings toggle.

#### Scenario: Keystroke injection falls back to FlashPaste
- **WHEN** `SendInput` returns 0 (failure) during keystroke injection
- **THEN** the system SHALL attempt FlashPaste injection and notify the user of the fallback

#### Scenario: FlashPaste falls back to clipboard
- **WHEN** FlashPaste injection fails
- **THEN** the transcription SHALL be written to clipboard and a toast SHALL confirm the fallback

### Requirement: Focus management
Before injection, the system SHALL ensure focus is restored to the previously active window if VoxFlow's own window gained focus between recording stop and injection.

#### Scenario: Focus restored before injection
- **WHEN** the settings window or any VoxFlow window has focus when injection begins
- **THEN** `SetForegroundWindow` SHALL be called to restore focus to the target window before injection
