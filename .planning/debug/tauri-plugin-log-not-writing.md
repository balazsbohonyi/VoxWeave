---
status: resolved
trigger: "Investigate why tauri-plugin-log is not writing to %APPDATA%\\VoxFlow\\logs\\app.log and why Rust log entries don't appear in the devtools console"
created: 2026-03-20T00:00:00Z
updated: 2026-03-20T00:00:00Z
---

## Current Focus

hypothesis: n/a — both root causes confirmed
test: n/a
expecting: n/a
next_action: present findings to user

## Symptoms

expected: "%APPDATA%\\VoxFlow\\logs\\app.log written; Rust logs visible in devtools console"
actual: "No logs/ folder under %APPDATA%\\VoxFlow; Rust logs absent from devtools console"
errors: none (silent failures)
reproduction: run cargo tauri dev, observe log locations and devtools
started: since tauri-plugin-log was wired in

## Eliminated

- hypothesis: "tauri-plugin-log is not registered at all"
  evidence: "lib.rs line 24-33 shows LogBuilder::new().targets([...]).build() wired into .plugin()"
  timestamp: 2026-03-20

- hypothesis: "log:default permission is absent"
  evidence: "capabilities/default.json line 15 includes log:default"
  timestamp: 2026-03-20

- hypothesis: "file write fails due to permissions"
  evidence: "logs/ dir exists and VoxFlow.log is present — plugin is writing successfully to LocalAppData"
  timestamp: 2026-03-20

## Evidence

- timestamp: 2026-03-20
  checked: "tauri-plugin-log-2.8.0/src/lib.rs lines 627-641"
  found: "LogDir resolves via app_handle.path().app_log_dir() which on Windows maps to {FOLDERID_LocalAppData}/{bundleIdentifier}/logs — NOT %APPDATA%"
  implication: "logs are being written to C:\\Users\\Balazs\\AppData\\Local\\com.balazs.bohonyi.voxflow\\logs\\VoxFlow.log — confirmed by filesystem check"

- timestamp: 2026-03-20
  checked: "tauri.conf.json line 5 — identifier: com.balazs.bohonyi.voxflow"
  found: "identifier is set, so app_log_dir() resolves to LocalAppData\\com.balazs.bohonyi.voxflow\\logs"
  implication: "the path is correct for tauri — the user's expectation (%APPDATA%\\VoxFlow\\logs) was wrong about the location"

- timestamp: 2026-03-20
  checked: "filesystem: $LOCALAPPDATA/com.balazs.bohonyi.voxflow/logs/"
  found: "directory exists, contains VoxFlow.log"
  implication: "LogDir target IS working — log file is being written, just at a different path than expected"

- timestamp: 2026-03-20
  checked: "tauri-plugin-log-2.8.0/src/lib.rs lines 642-654"
  found: "Webview target emits app_handle.emit('log://log', payload) — a global event"
  implication: "the frontend must call attachConsole() from @tauri-apps/plugin-log to subscribe to log://log events and forward them to console"

- timestamp: 2026-03-20
  checked: "capabilities/default.json"
  found: "windows: [settings, indicator] — toast window is NOT listed"
  implication: "toast window has no log:default permission, but this is not the cause of the Webview target failure"

- timestamp: 2026-03-20
  checked: "src/ frontend — checked for attachConsole usage"
  found: "attachConsole() from @tauri-apps/plugin-log is never called in the frontend"
  implication: "this is root cause 2 — without attachConsole(), no window subscribes to log://log events, so nothing routes logs to devtools console"

## Resolution

root_cause:
  issue_1_logdir_path: |
    NOT actually broken. The file IS being written. The path is:
      C:\Users\Balazs\AppData\Local\com.balazs.bohonyi.voxflow\logs\VoxFlow.log
    tauri-plugin-log's LogDir on Windows uses FOLDERID_LocalAppData (AppData\Local),
    not FOLDERID_RoamingAppData (AppData\Roaming). The user expected %APPDATA%\VoxFlow\logs
    but the actual path is %LOCALAPPDATA%\com.balazs.bohonyi.voxflow\logs\VoxFlow.log.
    Source: tauri-plugin-log-2.8.0/src/lib.rs:628 — app_handle.path().app_log_dir()

  issue_2_webview_target: |
    The Webview target emits a "log://log" Tauri event (lib.rs:652).
    The frontend must call attachConsole() from @tauri-apps/plugin-log to subscribe
    to that event and forward messages to window.console.
    No frontend file calls attachConsole(). Without this subscriber, the event fires
    into the void and nothing appears in devtools.
    Source: tauri-plugin-log-2.8.0/src/lib.rs:642-654

fix: |
  Issue 1 (path confusion): No fix needed. Log file is working at its correct location.
  If you want logs under %APPDATA%\VoxFlow\logs specifically, replace TargetKind::LogDir
  with TargetKind::Folder { path: dirs_next::data_dir()... } or note the actual path.

  Issue 2 (devtools): In the frontend entry point (e.g. src/main.ts or each window's
  main.ts), import and call attachConsole() from @tauri-apps/plugin-log before app mount.

verification: "filesystem confirmed VoxFlow.log exists at LocalAppData path; attachConsole absence confirmed by grep across all frontend source"
files_changed: []
