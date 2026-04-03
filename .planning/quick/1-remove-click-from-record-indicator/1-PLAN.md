---
phase: quick-1
plan: 1
type: execute
wave: 1
depends_on: []
files_modified:
  - src/windows/indicator/App.vue
  - src-tauri/src/tray.rs
  - src-tauri/src/commands/indicator.rs
  - src-tauri/src/lib.rs
autonomous: true
requirements: []

must_haves:
  truths:
    - "Clicking the indicator circle does nothing"
    - "Tray menu shows Settings and Quit only (no Start/Stop Recording)"
    - "Hotkey-driven recording still works end-to-end"
  artifacts:
    - path: "src/windows/indicator/App.vue"
      provides: "Indicator circle with click handler removed"
    - path: "src-tauri/src/tray.rs"
      provides: "Tray menu without recording items"
  key_links:
    - from: "src-tauri/src/hotkey/service.rs"
      to: "tray::update_recording_menu"
      via: "function calls"
      pattern: "update_recording_menu"
---

<objective>
Remove click-to-record from the indicator circle and remove Start/Stop Recording from the tray menu.

Purpose: Hotkey is the sole trigger for recording; the circle and tray items are dead/confusing UI.
Output: Indicator circle is display-only; tray menu has Settings + Quit only; `update_recording_menu` calls remain but rebuild a menu without the recording item.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Remove click handler from indicator circle</name>
  <files>src/windows/indicator/App.vue</files>
  <action>
    In `src/windows/indicator/App.vue`:

    1. Delete the `onRecordButtonClick` function (lines 103–105).
    2. Remove `@click.stop="onRecordButtonClick"` from the `<button class="indicator-record-button">` element.
    3. The `<button>` itself can stay as the structural wrapper for the dot, but if keeping it as a `<button>` is confusing without a click handler, convert it to a `<span class="indicator-record-button">` instead — either is acceptable.
    4. Do NOT remove the `indicator-record-button` CSS class selector — `onPointerDown` in the same file uses `target?.closest(".indicator-record-button")` as a guard to skip drag initiation; removing the class would break drag behavior.
    5. Do NOT touch the `toggle_recording_from_indicator` Tauri command registration in `lib.rs` yet — remove it only if no other caller exists (check next task). Actually, after this change the command has no frontend caller; remove its registration from `lib.rs` and delete the function body from `src-tauri/src/commands/indicator.rs` as part of this task.

    Specifically in `src-tauri/src/commands/indicator.rs`: delete the `toggle_recording_from_indicator` function (the `#[tauri::command]` block at lines 51–55).

    In `src-tauri/src/lib.rs`: remove `commands::indicator::toggle_recording_from_indicator` from the `.invoke_handler(tauri::generate_handler![...])` list (line 166).
  </action>
  <verify>
    <automated>npx vue-tsc --noEmit && cd src-tauri && cargo clippy 2>&1 | grep -E "^error" | head -20</automated>
  </verify>
  <done>
    vue-tsc and cargo clippy both pass with no errors. The indicator circle no longer has any click handler wired to recording.
  </done>
</task>

<task type="auto">
  <name>Task 2: Remove Start/Stop Recording from tray menu</name>
  <files>src-tauri/src/tray.rs</files>
  <action>
    In `src-tauri/src/tray.rs`:

    1. Delete the `START_STOP_ID` constant (`const START_STOP_ID: &str = "start_stop_recording";`).

    2. In `build_tray_menu`: remove the `start_stop` `MenuItem` construction and remove `&start_stop` from `Menu::with_items`. The menu should now be: `[&open_settings, &sep1, &quit]`.

    3. In `handle_menu_event`: remove the `START_STOP_ID =>` match arm entirely.

    4. Delete the `recording_menu_label` function — it is now unused.

    5. `update_recording_menu` is still called from `hotkey/service.rs` at several points (lines 245, 274, 307, 320, 414, 571). Do NOT remove those call sites — the function is still useful as a no-op rebuild that keeps the tray valid. Simply update `build_tray_menu` so it no longer accepts or uses the `recording_state` parameter:
       - Change signature: `fn build_tray_menu<R: Runtime, M: Manager<R>>(manager: &M) -> Result<Menu<R>, Box<dyn std::error::Error>>`
       - Update `setup_tray`: remove the `recording_state` variable and the `build_tray_menu(app, recording_state)?` call → `build_tray_menu(app)?`
       - Update `update_recording_menu`: change `build_tray_menu(app, state)` → `build_tray_menu(app)` and remove the unused `state` parameter from its signature: `pub fn update_recording_menu<R: Runtime>(app: &AppHandle<R>)`.
       - Update all callers of `update_recording_menu` in `hotkey/service.rs` to drop the second argument. Each call passes a `RecordingState` value that is now unused at the tray layer.
  </action>
  <verify>
    <automated>cd src-tauri && cargo clippy 2>&1 | grep -E "^error" | head -20</automated>
  </verify>
  <done>
    cargo clippy passes with no errors. Tray menu contains Settings + separator + Quit only. Recording state no longer affects tray menu items.
  </done>
</task>

</tasks>

<verification>
Run full build check after both tasks:

```
npx vue-tsc --noEmit
cd src-tauri && cargo test
```

Both must pass with no errors.
</verification>

<success_criteria>
- Clicking indicator circle does not trigger recording
- Tray menu: "Settings", separator, "Quit VoxWeave" — no recording items
- Hotkey-triggered recording still functions (no regression in pipeline)
- `cargo clippy` and `vue-tsc --noEmit` both clean
</success_criteria>

<output>
After completion, create `.planning/quick/1-remove-click-from-record-circle-remove-s/1-SUMMARY.md`
</output>
