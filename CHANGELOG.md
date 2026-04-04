# Changelog

All notable changes to VoxWeave will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [1.0.0-beta.1] - [UNRELEASED]

### Added

- **Global hotkey dictation** — press `Ctrl+Shift+Space` from any application (including terminals) to start recording; press again to stop and inject the transcribed text into the active window
- **Cloud transcription** via OpenAI (whisper-1, gpt-4o-transcribe, gpt-4o-mini-transcribe) and Groq (whisper-large-v3-turbo, whisper-large-v3, distil-whisper-large-v3-enspi); each provider has its own API key and model selection
- **Local (offline) transcription** via whisper.cpp — download tiny, base, small, or medium models on demand from Settings; no internet connection required after download
- **Floating recording indicator** — a small always-on-top pill shows a live audio waveform while recording; draggable and remembers its position
- **Text injection modes** — FlashPaste (clipboard-based, default), simulated keystrokes (configurable speed), or copy-to-clipboard only; terminals automatically receive the correct paste shortcut (`Ctrl+Shift+V` / `Shift+Insert`)
- **Automatic fallback chain** — if the selected injection method fails, VoxWeave falls back to the next available method automatically
- **Elevation handling** — if the target window runs at a higher privilege level, VoxWeave offers to relaunch as Administrator or copy the text to the clipboard instead
- **First-launch setup wizard** — guides new users through choosing a transcription engine, configuring an API key or downloading a local model, and setting a hotkey before first use
- **Settings window** — configure hotkey, microphone device, transcription engine and provider, language hint (or auto-detect), injection method, keystroke speed, auto-fallback, and Windows startup launch
- **System tray presence** — VoxWeave lives in the system tray; the icon reflects recording state and the context menu provides quick access to settings
- **Toast notifications** — confirm successful injection (with a text preview), surface actionable errors (open settings, retry, use fallback provider), and report cancellations with a character count
