## ADDED Requirements

### Requirement: First-launch setup wizard
On first launch (when no config file exists), the app SHALL open the setup wizard instead of minimizing to tray. The wizard SHALL guide the user through three steps: (1) engine selection, (2) provider/API key setup or model download, (3) hotkey confirmation.

#### Scenario: Wizard shown on first launch
- **WHEN** no config file exists and the app starts
- **THEN** the setup wizard SHALL open instead of the app minimizing to tray

#### Scenario: App minimizes to tray after wizard completion
- **WHEN** the user clicks "Finish" in the setup wizard
- **THEN** the wizard SHALL close, the config SHALL be saved, and a toast SHALL appear: "VoxFlow is ready — press [hotkey] to start dictating"

### Requirement: Step 1 — Engine selection
The first wizard step SHALL present the user with a choice between "Cloud" and "Local" transcription engines.

#### Scenario: Cloud engine selected
- **WHEN** the user selects "Cloud" on step 1
- **THEN** the next step SHALL show the provider/API key configuration

#### Scenario: Local engine selected
- **WHEN** the user selects "Local" on step 1
- **THEN** the next step SHALL show the model download interface

### Requirement: Step 2 — Provider or model setup
If Cloud was selected, step 2 SHALL show a tabbed provider interface (OpenAI, Groq, OpenRouter) requiring an API key for at least one provider. If Local was selected, step 2 SHALL show the model download interface with size/quality guidance.

#### Scenario: API key validation on entry
- **WHEN** the user enters an API key in the wizard
- **THEN** a test request SHALL be sent; success shows a green checkmark, failure shows an inline error with an actionable message (e.g. "Invalid key", "Insufficient quota")

#### Scenario: At least one provider required to proceed
- **WHEN** Cloud engine is selected and no API key has been validated
- **THEN** the "Next" button SHALL be disabled or show a validation error

### Requirement: Step 3 — Hotkey confirmation
The third wizard step SHALL display the default hotkey (`Ctrl+Shift+Space`) with an option to change it. The user SHALL be able to accept the default or set a custom hotkey.

#### Scenario: Default hotkey pre-configured
- **WHEN** the user reaches step 3
- **THEN** the hotkey field SHALL show `Ctrl+Shift+Space` as the default

#### Scenario: Custom hotkey set in wizard
- **WHEN** the user changes the hotkey in step 3
- **THEN** the new hotkey SHALL be saved when the wizard completes

### Requirement: Wizard re-accessibility
The setup wizard SHALL be re-openable from Settings at any time via a "Re-run setup wizard" button.

#### Scenario: Re-run wizard from settings
- **WHEN** the user clicks "Re-run setup wizard" in settings
- **THEN** the setup wizard SHALL open with current settings pre-filled
