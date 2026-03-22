# Deferred Items — Phase 08 Settings UI

## Pre-existing Test Failures (out of scope)

Discovered during 08-01 execution. These failures existed before phase 08 work began and are in files not modified by any 08-01 task.

| Test | File | Status |
|------|------|--------|
| injection::service::tests::focus_restore_order | src-tauri/src/injection/service.rs | Pre-existing |
| injection::service::tests::elevation_check_triggers_only_when_target_il_greater | src-tauri/src/injection/service.rs | Pre-existing |
| injection::service::tests::fallback_flashpaste_to_clipboard_when_flashpaste_fails | src-tauri/src/injection/service.rs | Pre-existing |
| injection::service::tests::fallback_keystroke_chain | src-tauri/src/injection/service.rs | Pre-existing |
| injection::service::tests::elevation_copy_to_clipboard_returns_copied_to_clipboard_variant | src-tauri/src/injection/service.rs | Pre-existing |

These should be investigated and fixed in a dedicated plan, not within the Settings UI phase.
