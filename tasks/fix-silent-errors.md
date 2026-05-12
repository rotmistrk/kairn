# URGENT: Fix All Silent Error Handling

## Rule
NO error may be silently swallowed. Every failure MUST either:
- (a) Show in status bar + message ring (user-facing actions)
- (b) Log at WARN/ERROR level (internal/background operations)

Use `CM_STATUS_MESSAGE` with `Message::new(MsgLevel::Error, origin, text)` for (a).
Use `log::error!` or `log::warn!` for (b).

## CRITICAL — Fix Immediately

### 1. File save on close (src/views/editor/mod.rs:211)
```rust
// BROKEN: discards write failure, closes tab anyway
let _ = crate::editor::save::save_file(&self.path, &content);
self.editor.buffer.mark_saved();
```
FIX: Check result. If Err, show error message, do NOT mark saved, do NOT close tab.

### 2. LSP rename file write (src/lsp/workspace_edit.rs:79)
```rust
std::fs::write(path, &result).is_ok()
```
FIX: On failure, log::error! with path and error, return false, show message to user.

### 3. LSP resource ops (src/lsp/resource_ops.rs:38,44,73)
```rust
let _ = std::fs::create_dir_all(parent);
std::fs::rename(old, new).is_ok()
```
FIX: Log the actual io::Error. Return meaningful error string.

### 4. Todo tree save (src/views/todo_tree/model.rs:62)
```rust
fs::write(path, content).is_ok()
```
FIX: On failure, log::error! and show message "Failed to save TODO file: {error}".

### 5. MCP agent file (src/mcp/agent_file.rs:11,34)
```rust
let _ = std::fs::create_dir_all(&agents_dir);
let _ = std::fs::write(agents_dir.join("kairn.json"), json);
```
FIX: Log errors. Show message on write failure.

### 6. Clipboard paste (src/clipboard.rs:20)
```rust
let output = std::process::Command::new("pbpaste").output().ok()?;
```
FIX: On failure, return Err(String) not None. Caller shows "Clipboard unavailable: {reason}".

### 7. Suspend shell (src/suspend.rs:18)
```rust
let _ = std::process::Command::new(&shell).env("KAIRN_SUSPENDED", "1").status();
```
FIX: On failure, show "Failed to spawn shell: {error}".

## HIGH — Fix Next

### 8. LSP didOpen file read (src/lsp/send.rs:28)
```rust
let text = std::fs::read_to_string(path).unwrap_or_default();
```
FIX: log::warn! on failure. Send empty content is acceptable but log why.

### 9. LSP client send failures (src/lsp/client.rs:51,58)
```rust
let _ = self.write_tx.send(data);
```
FIX: log::error!("LSP send failed — server connection lost"). Set a dead flag.

### 10. LSP message parsing (src/lsp/client.rs:104,112,119,120)
```rust
.ok()?
```
FIX: Replace with match, log::warn! on each failure with context.

### 11. Build command spawn (src/build.rs:58)
```rust
let output = Command::new("sh").arg("-c").arg(cmd).output().ok()?;
```
FIX: Capture io::Error, include in "Build failed: {error}" message.

### 12. Todo tree operations (src/views/todo_tree/model.rs:28-44)
```rust
tree_ops::remove_item(file, path).ok()
```
FIX: log::warn! on failure for each operation.

### 13. File watcher registration (src/git_watcher.rs:35-40)
```rust
let _ = watcher.watch(...)
```
FIX: log::warn! with path that failed to watch.

### 14. LSP message serialization (src/lsp/messages.rs:47)
```rust
serde_json::to_string(body).unwrap_or_default()
```
FIX: log::error! — this means a bug in our code.

### 15. Message ring mutex (src/handler.rs:97)
```rust
if let Ok(mut ring) = state.messages.lock() { ... }
```
FIX: log::error!("Message ring mutex poisoned") on Err.

### 16. MCP snapshot mutex (src/handler.rs:142)
FIX: Same — log poisoned mutex.

## Verification

After fixing:
1. `cargo clippy --workspace -- -D warnings` must pass
2. `cargo test --workspace` must pass
3. Manually test: try to save a read-only file → error must appear in status bar
4. Manually test: clipboard paste with no clipboard tool → error in status bar
5. Check .kairn.log after normal usage — no unexpected errors

## Pattern to Use

```rust
// For user-facing actions:
match some_operation() {
    Ok(result) => { /* proceed */ }
    Err(e) => {
        let msg = Message::new(MsgLevel::Error, "origin", format!("What failed: {e}"));
        queue.put_command(CM_STATUS_MESSAGE, Some(Box::new(msg)));
        return; // or don't close tab, etc.
    }
}

// For background operations:
if let Err(e) = some_operation() {
    log::error!("context: {e}");
}
```
