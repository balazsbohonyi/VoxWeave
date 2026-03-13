## ADDED Requirements

### Requirement: Multi-provider cloud transcription
The system SHALL support three cloud transcription providers: OpenAI, Groq, and OpenRouter. Only one provider SHALL be active at a time (the "active provider"). Each provider SHALL have its own API key, model selection, and optional language hint configuration.

#### Scenario: Active provider sends audio for transcription
- **WHEN** recording stops and a cloud provider is the active transcription engine
- **THEN** the encoded Opus audio SHALL be sent to the active provider's API endpoint with the configured API key and model

#### Scenario: Transcription result returned as plain text
- **WHEN** the cloud provider returns a successful response
- **THEN** the transcription SHALL be extracted as plain text and passed to the injection pipeline

### Requirement: OpenAI transcription
The system SHALL support OpenAI's transcription API. The endpoint SHALL be `https://api.openai.com/v1/audio/transcriptions`. Available models SHALL include at minimum: `whisper-1`, `gpt-4o-transcribe`, `gpt-4o-mini-transcribe`.

#### Scenario: OpenAI transcription request
- **WHEN** OpenAI is the active provider
- **THEN** the system SHALL POST the audio file to `https://api.openai.com/v1/audio/transcriptions` with the `Authorization: Bearer <api-key>` header and the selected model parameter

### Requirement: Groq transcription
The system SHALL support Groq's transcription API (`/openai/v1/audio/transcriptions`). The model list SHALL be hardcoded. Available models SHALL be:

| Model ID | Notes |
|---|---|
| `whisper-large-v3-turbo` | Default — best speed/accuracy/cost balance; multilingual |
| `whisper-large-v3` | Highest accuracy; use when quality is critical or translation needed |
| `distil-whisper-large-v3-en` | English-only; fastest and cheapest |

The default model SHALL be `whisper-large-v3-turbo`.

#### Scenario: Groq transcription request
- **WHEN** Groq is the active provider
- **THEN** the system SHALL POST the audio to `https://api.groq.com/openai/v1/audio/transcriptions` with the `Authorization: Bearer <api-key>` header and the selected model parameter

#### Scenario: Groq default model pre-selected
- **WHEN** the user configures Groq for the first time
- **THEN** `whisper-large-v3-turbo` SHALL be pre-selected as the model

### Requirement: OpenRouter transcription
The system SHALL support OpenRouter's API for audio-capable multimodal models. OpenRouter does NOT provide a Whisper-style transcription endpoint — all audio requests SHALL use the standard chat completions endpoint (`/api/v1/chat/completions`) with audio provided as an `input_audio` content part. The model list SHALL be hardcoded. Available models SHALL be:

| Model ID | Notes |
|---|---|
| `google/gemini-2.5-flash` | Default — best balance of speed, cost, and accuracy for transcription |
| `google/gemini-2.5-pro` | Highest quality; higher latency and cost |
| `google/gemini-2.5-flash-lite` | Lowest cost and latency |
| `openai/gpt-4o-audio-preview` | OpenAI audio model via OpenRouter; audio input + text output |
| `openai/gpt-audio` | Bidirectional audio; higher cost |
| `openai/gpt-audio-mini` | Cost-efficient bidirectional audio |

The default model SHALL be `google/gemini-2.5-flash`.

#### Scenario: OpenRouter transcription request
- **WHEN** OpenRouter is the active provider
- **THEN** the system SHALL POST to `https://openrouter.ai/api/v1/chat/completions` with the audio encoded as base64 in an `input_audio` content part and the `Authorization: Bearer <api-key>` header

#### Scenario: OpenRouter default model pre-selected
- **WHEN** the user configures OpenRouter for the first time
- **THEN** `google/gemini-2.5-flash` SHALL be pre-selected as the model

#### Scenario: OpenRouter transcription response parsed as plain text
- **WHEN** OpenRouter returns a chat completion response
- **THEN** the transcription text SHALL be extracted from `choices[0].message.content` and passed to the injection pipeline

### Requirement: Error handling and retry
The system SHALL handle cloud transcription errors gracefully. On rate limit errors, the system SHALL retry with exponential backoff (max 3 retries). On invalid API key errors, the system SHALL show a settings prompt with the relevant provider tab highlighted. On network errors, the system SHALL show a notification with a retry button.

#### Scenario: Rate limit retry
- **WHEN** the cloud API returns a rate limit error (HTTP 429)
- **THEN** the system SHALL retry up to 3 times with exponential backoff before showing an error

#### Scenario: Invalid API key error
- **WHEN** the cloud API returns an authentication error (HTTP 401/403)
- **THEN** a notification SHALL appear prompting the user to open settings with the offending provider tab pre-selected

#### Scenario: Network error with retry button
- **WHEN** a network error occurs during transcription
- **THEN** a toast notification SHALL appear with a "Retry" action button

### Requirement: Fallback to secondary provider
If the active cloud provider fails after all retries and another provider is configured with an API key, the system SHALL offer to retry with the fallback provider via a toast action button.

#### Scenario: Fallback provider offered on failure
- **WHEN** the active provider fails after retries and a secondary provider is configured
- **THEN** a toast SHALL appear: "[Provider] failed — retry with [Fallback]?" with a one-tap action button

### Requirement: Language hint configuration
The system SHALL allow a configurable language hint (BCP-47 language code) per provider, or auto-detect when set to auto. The language hint SHALL be sent with each transcription request.

#### Scenario: Language hint sent with request
- **WHEN** a language is configured for the active provider
- **THEN** the language parameter SHALL be included in the transcription API request

#### Scenario: Auto-detect language
- **WHEN** the language setting is set to auto
- **THEN** no language parameter SHALL be sent (provider uses auto-detection)
