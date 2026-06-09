# Conventions

Patterns and rules for contributing to kairn. Read this before making changes.

## ⚠️ GRITS: Feature Implementation Cycle

Every feature or fix MUST follow this cycle iteratively:

1. **G**reen — Start from a green build (`cargo build` + `cargo test` pass)
2. **R**egression testing — Run existing tests, confirm nothing is broken
3. **I**mplementation with new tests — Write the code AND its tests together
4. **T**ests **S**ucceed — All tests (old + new) pass before considering it done

Repeat for each incremental step. Never push forward on a red build.

## No External Tools

**ABSOLUTE RULE**: kairn must NEVER depend on external command-line tools for core functionality. All features must use pure Rust crates.

Forbidden: `rg`, `grep`, `git` CLI, `find`, `pbcopy`, etc.

Use instead:
- `ignore` crate for file walking
- `regex` for search
- `gix` for git operations
- `syntect` for syntax highlighting

This rule exists because an agent used `rg` for grep, which silently failed on machines without ripgrep. It cost 2+ hours of debugging.

## No Silent Errors

NO error may be silently swallowed. Every failure MUST either:
- **(a)** Show in status bar + message ring (user-facing actions)
- **(b)** Log at WARN/ERROR level (internal/background operations)

```rust
// User-facing:
match some_operation() {
    Ok(result) => { /* proceed */ }
    Err(e) => {
        let msg = Message::error("origin", format!("What failed: {e}"));
        queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        return;
    }
}

// Background:
if let Err(e) = some_operation() {
    log::error!("context: {e}");
}
```

Never use `let _ = fallible_operation();` or `.unwrap_or_default()` on I/O without logging.

## User Confirmations (Statusbar Prompt Pattern)

When an action requires user confirmation (delete, discard, overwrite, etc.):

1. Set a modal flag on the view (e.g., `confirm_delete = true`)
2. Send a `CM_STATUS_MESSAGE` to the global status bar with the prompt text:
   ```rust
   let msg = Message::info("todo", "Delete item? [y]es [Esc]cancel".to_string());
   queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
   ```
3. On the next key event, intercept it:
   - If confirmed (`y`): perform the action
   - Any other key: cancel
4. Clear the status bar by sending an empty message:
   ```rust
   let msg = Message::info("todo", String::new());
   queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
   ```

Reference: `EditorView::request_close` + `handle_close_prompt`.

Do NOT use ad-hoc confirmation that lacks visual feedback.

## Background Tasks (Drain Pattern)

For async/background work (grep, build, LSP):

1. Spawn a thread that writes results to a shared `Arc<Mutex<Vec<T>>>` or similar
2. Call `waker.wake()` to interrupt the event loop when results are ready
3. In the main handler, **drain** results on every tick via `handler_drain.rs`
4. Push results into the target view (e.g., `ResultsView::append`)
5. Report errors via `CM_STATUS_MESSAGE`

Do NOT:
- Block the UI thread
- Use channels without a wake mechanism (views won't see results until next tick)
- Assume Tick events reach non-focused views directly

Reference: `src/grep.rs` + `src/handler_drain.rs` for the canonical pattern.

## Testing

Tests use `TestHarness` (in `tests/helpers.rs`) which mirrors the real app exactly:
- `Program` + `MockBackend` + `AppState`
- `inject_key()` / `inject_str()` to simulate input
- `run_cycles(n)` to advance the event loop
- `contains()` / `content_contains()` / `row(y)` to assert screen output

Integration tests go in `tests/` as separate files. Each test creates a `TempDir` for isolation.

```rust
#[test]
fn my_feature_works() {
    let tmp = tempfile::tempdir().unwrap();
    let mut h = TestHarness::new(tmp.path());
    h.run_cycles(1); // initial render
    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("expected text"));
}
```

Every new feature or bug fix MUST include tests. The pre-commit hook runs all tests.

## File Size Limit

Max **240 code lines** per `.rs` file. Blank lines and comments don't count. Enforced by pre-commit hook.

When a file grows too large, split by responsibility into submodules (e.g., `handle.rs`, `draw.rs`, `build.rs`).

## Pre-commit Hook

All commits must pass: `cargo fmt` + `cargo clippy -D warnings` + 240-line check + `cargo test --workspace`. The hook is at `hooks/pre-commit`. Run it before pushing.

**NEVER skip the pre-commit hook** (`--no-verify` is forbidden). If the hook fails or hangs, fix the underlying problem first. The only consequence of a commit that skipped pre-commit is a hard reset to the last validated commit — all work in such commits is permanently discarded, no exceptions.

## View Architecture

- Views implement the `View` trait from `txv_core`
- State: `ViewState` tracks bounds, focus, dirty flag
- Use `self.state.mark_dirty()` after any change that requires redraw
- Tree-based views wrap `TreeView<T>` where `T: TreeData`
- Use `delegate_view_state!` / `delegate_view!` macros for trait delegation
- Event dispatch: three-phase (preprocess → focused → postprocess)
- Communication between views: `EventQueue` with commands (`put_command`)
- Tab close protocol: `can_close()` returns `CloseResult::Ok` or `CloseResult::Denied(reason)`

## Command Dispatch

Views communicate via commands on the `EventQueue`:
```rust
queue.put_command(CM_OPEN_FILE, Some(Box::new(path_string)));
```

Commands are handled in `src/handler.rs`. Add new command IDs to `src/commands.rs`.

For views that need to respond to commands, handle `Event::Command { id, data }` in the `handle` method.

## Status Messages

Use `CM_STATUS_MESSAGE` for transient user-facing messages. The `MessageItem` in the status bar displays the most recent one. Messages also go to the message ring (F6).

```rust
use txv_core::message::Message;
// Levels: info, error, warn (via Message::new with MsgLevel)
let msg = Message::error("component", format!("Something failed: {e}"));
queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
```

## Palette / Colors

All colors go through the palette system. NEVER hardcode `Color::Ansi(N)` in views.

- Framework roles: `txv_core::palette::palette()` (base, interactive, chrome, popup, state)
- App roles: `crate::app_palette::app_palette()` (git, diff, editor, diag, tree, todo, msg)
- Config: users override via `set color.<group>.<role> <ansi-number>` in `init.tcl`
- Reference: `doc/example-init.tcl` lists all available color roles

## File Persistence

- Todo tree: `.kairn.todo` (duir-compatible format)
- Session state: `.kairn.state` (auto-save on quit, auto-restore on launch)
- Settings: `~/.config/kairn/init.tcl` (Tcl syntax, sparse — only set what you change)

## Code Style

- No `unwrap()` / `expect()` / `panic!()` in production code
- Prefer `Option`/`Result` propagation with `?`
- Use `log::error!` / `log::warn!` for non-fatal issues
- Keep imports organized: std → external crates → crate-internal
- No `#[ignore]` tests — pre-commit hook rejects them
