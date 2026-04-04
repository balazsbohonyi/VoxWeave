---
phase: quick-9
plan: 9
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/injection/service.rs
  - docs/TODO.md
autonomous: true
requirements: [F001]
must_haves:
  truths:
    - "After FlashPaste injection, transcribed text is in clipboard"
    - "After Keystroke injection, transcribed text is in clipboard"
    - "Clipboard injection is unaffected (already writes the text)"
    - "CopiedToClipboard elevation path is unaffected (already writes the text)"
    - "Error paths do not write to clipboard"
  artifacts:
    - path: "src-tauri/src/injection/service.rs"
      provides: "Modified inject_text that writes text to clipboard on Ok result"
  key_links:
    - from: "inject_text"
      to: "clipboard.write_text(text)"
      via: "final step before returning Ok(InjectionResult::Ok)"
      pattern: "clipboard\\.write_text\\(text\\)"
---

<objective>
After any successful injection via FlashPaste or Keystroke, write the transcribed text to the clipboard so the user can paste it manually if the target window loses focus or injection fails silently.

Purpose: Clipboard serves as a safety net — if injection didn't land where the user expected, they can still paste the transcribed text.
Output: Modified inject_text in service.rs that writes text to clipboard on Ok result; F001 marked complete in TODO.md.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Write transcribed text to clipboard after Ok injection</name>
  <files>src-tauri/src/injection/service.rs</files>
  <action>
In `inject_text`, the function already has `clipboard: &dyn ClipboardAccess` and `text: &str` parameters.

The current flow:
1. Primary mode runs -> returns Ok, CopiedToClipboard, or falls through to fallback
2. Fallback chain runs
3. Returns final result

The change: whenever `inject_text` is about to return `Result::Ok(InjectionResult::Ok)`, first write `text` to the clipboard via `clipboard.write_text(text)`. Ignore the error from the write (best-effort, like the existing clipboard restore in flashpaste).

Specifically, there are three `return Result::Ok(...)` sites for `InjectionResult::Ok`:
- Line 257: `InjectionResult::Ok => return Result::Ok(primary_result),` — primary success
- Line 288: `InjectionResult::Ok => return Result::Ok(InjectionResult::Ok),` — fallback success

Before each of these `InjectionResult::Ok` returns, add:
```rust
let _ = clipboard.write_text(text);
```

Do NOT add this before `InjectionResult::CopiedToClipboard` returns — those paths already wrote the text to clipboard.

Do NOT add this to error returns.

For `Clipboard` mode: `clipboard_only` already writes the text, and returning Ok before this new write means the write is redundant but harmless (idempotent). That is acceptable — do not add a special case.

After the change, the complete Ok-path logic in the primary-result match block (step 4) becomes:
```rust
match &primary_result {
    InjectionResult::Ok => {
        let _ = clipboard.write_text(text);
        return Result::Ok(primary_result);
    }
    InjectionResult::CopiedToClipboard => return Result::Ok(primary_result),
    InjectionResult::Err(_) => { /* fall through to fallback */ }
}
```

And in the fallback loop:
```rust
match result {
    InjectionResult::Ok => {
        let _ = clipboard.write_text(text);
        return Result::Ok(InjectionResult::Ok);
    }
    InjectionResult::CopiedToClipboard => return Result::Ok(result),
    InjectionResult::Err(_) => continue,
}
```

Also add a test `clipboard_has_text_after_flashpaste_ok` that verifies: after a successful FlashPaste injection, the clipboard contains the injected text (not the original pre-paste content). Use the existing MockClipboard, set it to "original", run inject_text with FlashPaste mode, assert clipboard.read_text() == "injected text".

Also add a test `clipboard_has_text_after_keystroke_ok` that verifies: after a successful Keystroke injection, the clipboard contains the injected text.
  </action>
  <verify>
    <automated>cd src-tauri &amp;&amp; cargo test injection::service::tests -- --nocapture 2>&amp;1 | tail -20</automated>
  </verify>
  <done>All existing tests pass; new tests clipboard_has_text_after_flashpaste_ok and clipboard_has_text_after_keystroke_ok pass; clipboard contains transcribed text after Ok injection for FlashPaste and Keystroke modes.</done>
</task>

<task type="auto">
  <name>Task 2: Mark F001 complete in TODO.md</name>
  <files>docs/TODO.md</files>
  <action>
Change the F001 line from:
```
- [ ] F001: Add all transcribed text to the clipboard as well
```
to:
```
- [x] F001: Add all transcribed text to the clipboard as well
```
  </action>
  <verify>grep "F001" docs/TODO.md</verify>
  <done>F001 line shows `[x]` status in docs/TODO.md.</done>
</task>

</tasks>

<verification>
- cargo test passes in src-tauri (no regressions)
- New tests confirm clipboard contains transcribed text after FlashPaste and Keystroke Ok paths
- F001 marked [x] in docs/TODO.md
</verification>

<success_criteria>
After any successful FlashPaste or Keystroke injection, clipboard.read_text() returns the transcribed text. Clipboard mode behavior unchanged. All cargo tests pass.
</success_criteria>

<output>
After completion, create `.planning/quick/9-add-transcribed-text-to-clipboard/9-SUMMARY.md`
</output>
