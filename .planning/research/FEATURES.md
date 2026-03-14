# Features Research: VoxFlow

## Competitive Landscape

| Product | Type | Price | Key Differentiator |
|---------|------|-------|--------------------|
| Wispr Flow | Commercial | $8-15/mo | Polished UX, AI editing, per-app context |
| Windows Voice Typing | Built-in | Free | Win+H, limited to standard inputs, no terminal support |
| macOS Dictation | Built-in | Free | System-wide but no terminal, no customization |
| Dragon NaturallySpeaking | Commercial | $200+ | Enterprise-grade accuracy, custom vocabularies |
| Talon | Open source | Free | Voice coding focus, command grammar, steep learning curve |
| Nerd Dictation | Open source | Free | Linux-only, simple Whisper wrapper, keystroke output |
| whisper.cpp CLI | Open source | Free | CLI tool, no GUI, manual pipeline |

## Table Stakes (Must Have)

These are features users expect from any voice-to-text dictation tool. Missing any of these makes the product feel broken.

| Feature | Complexity | Dependencies | Notes |
|---------|-----------|--------------|-------|
| Global hotkey trigger | Low | Tauri global-shortcut plugin | Every dictation tool has this. Must work from any app |
| Toggle mode (press to start, press to stop) | Low | Hotkey system | Standard interaction pattern. Hold-to-record is less common |
| Microphone audio capture | Medium | cpal crate | 16kHz mono is standard for speech-to-text |
| Visual recording indicator | Medium | Tauri multi-window | Users need to know the mic is hot. Red dot minimum |
| Cloud transcription (at least one provider) | Medium | reqwest, API integration | OpenAI Whisper is the baseline |
| Text injection into active window | High | Windows APIs (SendInput, clipboard) | THE core feature. Must work in most apps |
| System tray presence | Low | Tauri tray plugin | Background app must live in tray |
| Settings persistence | Low | serde_json, config file | Settings must survive restarts |
| Error feedback (no mic, bad API key, network) | Medium | Toast/notification system | Users need to know when things go wrong |

## Differentiators (Competitive Advantage)

These separate VoxFlow from built-in OS dictation and simple Whisper wrappers.

| Feature | Complexity | Dependencies | VoxFlow Advantage |
|---------|-----------|--------------|-------------------|
| Terminal support | High | Window class detection, paste shortcut switching | Neither Windows Voice Typing nor Wispr Flow work in terminals |
| Multiple cloud providers (OpenAI, Groq, OpenRouter) | Medium | Provider abstraction trait | BYOK flexibility — users choose based on cost, speed, quality |
| Local transcription (whisper.cpp) | High | whisper-rs, model management | Offline capability, zero API cost, privacy |
| FlashPaste (clipboard save/restore) | Medium | arboard, SendInput | Seamless — doesn't destroy user's clipboard |
| Keystroke injection alternative | High | SendInput, KEYEVENTF_UNICODE | Works where clipboard doesn't (some apps block paste) |
| Injection cancellation (Escape) | Medium | Atomic flag, key listener | Power user feature — stop mid-injection |
| Elevation detection + relaunch | High | Windows process integrity APIs | Graceful handling of admin apps instead of silent failure |
| Auto-fallback chain | Medium | Injection pipeline | Resilient — always delivers text somehow |
| First-launch setup wizard | Medium | Vue multi-step UI | Better onboarding than dumping users into settings |
| Configurable injection speed | Low | Timer in injection loop | Power user tuning for apps that choke on fast input |

## Anti-Features (Deliberately NOT Building in v1)

| Feature | Reason | Risk if Included |
|---------|--------|------------------|
| AI post-processing (grammar, punctuation) | Adds latency, complexity, LLM cost; not core to dictation | Scope creep, extra API calls, user confusion about edits |
| Streaming/real-time transcription display | Requires WebSocket streaming, partial results UI, much more complex pipeline | Doubles pipeline complexity for marginal UX gain |
| Per-app tone adaptation | Requires app detection, context switching, LLM integration | Massive scope expansion |
| Personal dictionary / custom vocabulary | Requires persistent word list, integration with each provider differently | Complex, provider-specific, low ROI for v1 |
| Usage analytics / telemetry | Against BYOK philosophy, privacy concern | Trust violation with target audience |
| User accounts / cloud sync | Against BYOK philosophy, requires backend infrastructure | Opposite of the product vision |
| Hold-to-record mode | Toggle mode is simpler and more standard for dictation | Adds state machine complexity |
| Multi-language auto-detection | Each provider handles this differently; manual selection + "auto" option is sufficient | Inconsistent behavior across providers |

## Feature Dependencies

```
Global Hotkey → Audio Capture → Encoding → Transcription → Text Injection
                     ↓                          ↓
              Floating Indicator          Toast Notifications
                                               ↓
                                          Settings UI
                                               ↓
                                         Setup Wizard
```

- **Config persistence** is foundational — everything reads from it
- **System tray** is foundational — the app's shell
- **Audio capture** must work before transcription can be tested
- **Text injection** is independent of transcription (can test with hardcoded strings)
- **Cloud transcription** and **local transcription** are independent of each other
- **Setup wizard** depends on settings UI components being built first
- **Floating indicator** is visually independent but needs audio-level events from capture
