## ADDED Requirements

### Requirement: Status toast notifications
The system SHALL display toast notifications for key events: successful injection, clipboard copy, injection cancellation, elevated target detection, provider fallback, and errors. Toasts SHALL appear near the floating indicator or at the bottom-right of the screen. Toasts SHALL auto-dismiss after 4 seconds and SHALL be manually dismissible.

#### Scenario: Success toast after FlashPaste
- **WHEN** FlashPaste injection completes successfully
- **THEN** a toast SHALL appear with message "Text pasted" and a preview of the first ~50 characters

#### Scenario: Success toast after keystroke injection
- **WHEN** keystroke injection completes successfully
- **THEN** a toast SHALL appear with message "Text typed" and a preview of the first ~50 characters

#### Scenario: Success toast after clipboard copy
- **WHEN** clipboard mode completes
- **THEN** a toast SHALL appear with message "Copied to clipboard" and a preview of the first ~50 characters

#### Scenario: Toast auto-dismisses
- **WHEN** a toast appears
- **THEN** it SHALL automatically disappear after 4 seconds if not dismissed

#### Scenario: Toast manually dismissed
- **WHEN** the user clicks a dismiss button on the toast
- **THEN** the toast SHALL immediately close

### Requirement: Error and actionable toasts
Error toasts SHALL include actionable messages. API key errors SHALL include a "Open Settings" action. Network errors SHALL include a "Retry" action. Provider fallback toasts SHALL name both providers. Injection cancellation toasts SHALL state characters injected vs. total.

#### Scenario: API key error toast
- **WHEN** a cloud provider returns an authentication error
- **THEN** a toast SHALL appear: "Invalid API key — open settings?" with an "Open Settings" action button

#### Scenario: Injection cancellation toast
- **WHEN** keystroke injection is cancelled
- **THEN** a toast SHALL appear: "Injection cancelled — X of Y characters typed"

#### Scenario: Provider fallback toast
- **WHEN** the active provider fails and a fallback provider is used
- **THEN** a toast SHALL appear: "[Provider] failed — retried with [Fallback]"

#### Scenario: FlashPaste paste cancelled toast
- **WHEN** FlashPaste is cancelled via Escape
- **THEN** a toast SHALL appear: "Paste cancelled"
