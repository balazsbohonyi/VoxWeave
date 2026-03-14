# Pitfalls Research: VoxFlow

## Critical Pitfalls

### P1: Tauri v2 Transparent Window Issues on Windows

**Problem:** Tauri's transparent window support on Windows uses WebView2, which has known issues with transparency rendering. The floating indicator needs `transparent: true` + `decorations: false` + custom shapes.

**Warning signs:**
- White/black background flash on window show/hide
- Click-through (`ignore_cursor_events`) not working consistently
- Window appearing in taskbar despite `skip_taskbar: true`

**Prevention:**
- Test transparent window early (Phase 1 scaffolding, not Phase 6)
- Use `webview_transparent: true` in addition to `transparent: true` in window config
- Consider using a solid dark semi-transparent background color instead of full transparency if rendering is unreliable
- Test on both Windows 10 and Windows 11 — behavior differs

**Phase impact:** Scaffolding / Floating Indicator phase

---

### P2: cpal Audio Thread Blocking

**Problem:** cpal's audio callback runs on a dedicated OS thread. Any blocking operation in the callback (mutex locks, allocation, channel sends that block) causes audio glitches, crackling, or dropped samples.

**Warning signs:**
- Audio pops or clicks in recorded audio
- RMS amplitude values arriving in bursts instead of smoothly
- CPU spikes during recording

**Prevention:**
- Use a lock-free ring buffer or `crossbeam::channel` (bounded, non-blocking) in the audio callback
- Never allocate memory (`Vec::push`, `String::new`) inside the callback
- Pre-allocate the PCM buffer before recording starts
- Compute RMS on the audio thread but emit events from a separate thread reading from the buffer
- Keep the callback as simple as possible: copy samples → buffer → compute RMS → done

**Phase impact:** Audio Capture phase

---

### P3: SendInput Fails Silently Against Elevated Processes

**Problem:** `SendInput` returns 0 (no events inserted) when the target window belongs to a process running at a higher integrity level than VoxFlow. There's no error — it just silently does nothing. Users see "nothing happened" with no feedback.

**Warning signs:**
- Text injection works in Notepad but not in Task Manager or admin Command Prompt
- `SendInput` return value is 0 but no error is raised
- Users report "it works sometimes"

**Prevention:**
- Check process integrity level BEFORE attempting injection (not after failure)
- Use `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` → `OpenProcessToken` → `GetTokenInformation(TokenIntegrityLevel)`
- Compare target SID against `SECURITY_MANDATORY_HIGH_RID`
- Show elevation dialog immediately — don't attempt injection then fail
- Keep clipboard fallback as last resort

**Phase impact:** Text Injection phase

---

### P4: FlashPaste Clipboard Race Condition

**Problem:** The 500ms delay between pasting and restoring the clipboard may not be enough for slow apps. Some Electron apps and heavyweight editors process paste asynchronously — by the time they read the clipboard, the original content may already be restored.

**Warning signs:**
- Paste works in Notepad but not in Slack/Discord (Electron)
- Random blank pastes or partial text
- Works when debugger is attached (timing changes)

**Prevention:**
- 500ms is a reasonable default but should be configurable (hidden setting or future feature)
- Document the limitation for users
- For known-slow apps, consider waiting for clipboard content change notification instead of fixed delay (Windows `AddClipboardFormatListener`)
- Keystroke injection doesn't have this problem — suggest it as alternative for problem apps

**Phase impact:** Text Injection phase

---

### P5: Terminal Window Class Detection is Fragile

**Problem:** Detecting terminal emulators by window class name is heuristic-based. New terminals (Alacritty, WezTerm, Hyper, etc.) won't be detected. Custom terminal emulators have unpredictable class names.

**Known window classes:**
- `ConsoleWindowClass` — cmd.exe, PowerShell (conhost)
- `CASCADIA_HOSTING_WINDOW_CLASS` — Windows Terminal
- `mintty` — Git Bash, Cygwin
- `VirtualConsoleClass` — ConEmu
- `EVERYTHING_TASKBAR_NOTIFICATION` — not a terminal (false positive risk)

**Warning signs:**
- Ctrl+V pastes raw `^V` in a terminal
- Some terminals work, others don't
- User reports that Git Bash via VS Code terminal doesn't work (different class)

**Prevention:**
- Start with the known list (4 classes above)
- Log the window class name when injection happens (helps debugging)
- Plan for a user-configurable "additional terminal classes" setting (v1.5 or v2)
- Consider also checking the executable name (`WindowsTerminal.exe`, `powershell.exe`, `cmd.exe`) as a secondary heuristic

**Phase impact:** Text Injection phase

---

### P6: whisper-rs Build Complexity

**Problem:** `whisper-rs` wraps whisper.cpp, which requires CMake, a C/C++ compiler (MSVC), and potentially CUDA/cuBLAS for GPU acceleration. This makes the build environment setup much more complex and CI pipelines harder.

**Warning signs:**
- Build fails on fresh machine without MSVC
- CI pipeline takes 10+ minutes
- Users can't build from source without installing Visual Studio
- Linking errors on different MSVC versions

**Prevention:**
- Feature-gate whisper-rs behind `[features] local = ["whisper-rs"]` in Cargo.toml
- Build and test cloud-only first — add local transcription as a separate phase
- Document MSVC + CMake requirements in README
- Consider pre-built binaries or bundling a pre-compiled whisper.cpp DLL instead of building from source
- Set up CI with MSVC toolchain explicitly

**Phase impact:** Local Transcription phase (deliberately late in build order)

---

### P7: Opus Encoding Build Issues

**Problem:** The `opus` crate links against the libopus C library, which can have build issues on Windows (missing pkg-config, wrong library paths, MSVC vs MinGW mismatches).

**Warning signs:**
- `opus-sys` build script fails with "libopus not found"
- Works in MinGW but fails in MSVC (or vice versa)
- CI builds fail but local builds work

**Prevention:**
- Consider `audiopus_sys` with the `static` feature flag — bundles and compiles libopus from source, avoiding system library dependency
- Alternative: skip Opus entirely and send WAV to cloud providers. WAV files are larger (~960KB for 30s vs ~100KB Opus) but all providers accept WAV. The upload time difference on broadband is ~1 second — acceptable for v1
- If going Opus route: test the build in CI early, don't leave it for the end

**Phase impact:** Audio Capture / Encoding phase

---

### P8: Multi-Window Focus Management

**Problem:** When the floating indicator appears/disappears, or when the settings window gains focus, the foreground window (where text should be injected) may change. Tauri window focus events can be unpredictable.

**Warning signs:**
- Text injected into VoxFlow's own window instead of the target app
- Floating indicator steals focus momentarily, causing the wrong window to be targeted
- `GetForegroundWindow` returns VoxFlow's HWND after indicator show

**Prevention:**
- Save the target window handle (`HWND`) at recording START, not at injection time
- Set `focus: false` on the floating indicator window creation
- Use `ignore_cursor_events: true` on the indicator
- Before injection: compare current foreground to saved target — if different, call `SetForegroundWindow` to restore
- Add a small delay (50-100ms) after `SetForegroundWindow` before injecting

**Phase impact:** Floating Indicator + Text Injection phases

---

## Medium Pitfalls

### P9: Tauri Event Flooding from Audio Levels

**Problem:** Emitting `audio-level` events at 30fps creates a lot of IPC traffic. If the frontend can't keep up, events queue and the waveform lags.

**Prevention:**
- Throttle events to 30fps on the Rust side (don't emit faster than every 33ms)
- Use `f32` payloads (single float), not complex objects
- Consider batching 2-3 samples per event if performance is tight

### P10: Config File Corruption

**Problem:** If the app crashes mid-write, the config JSON can be truncated/invalid.

**Prevention:**
- Write to a temp file first, then atomically rename
- Use `serde_json` with pretty printing for human readability
- Handle deserialization errors gracefully — fall back to defaults
- The config struct should implement `Default` with sensible values

### P11: Global Hotkey Conflicts

**Problem:** Other apps (Discord, OBS, Steam, etc.) may already claim the default `Ctrl+Shift+Space` hotkey. Registration will silently fail or conflict.

**Prevention:**
- Catch registration failures from `tauri-plugin-global-shortcut` and surface them to the user
- Suggest alternative hotkeys in the error message
- Don't crash or silently break — the app should still work, just without the hotkey

### P12: Cross-Platform Abstraction Over-Engineering

**Problem:** Abstracting everything behind traits from day one can lead to over-engineering — defining traits for things that don't actually vary across platforms, or creating abstractions before understanding the macOS requirements.

**Prevention:**
- Only abstract things that are KNOWN to differ: audio capture backend, text injection, clipboard, elevation checks, system integration
- Keep the abstraction at the module boundary level, not deep inside
- Don't write macOS implementations yet — just define the trait and the Windows impl
- The transcription providers, config system, and pipeline orchestration are platform-agnostic — don't wrap them in traits
