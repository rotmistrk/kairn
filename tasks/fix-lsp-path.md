# URGENT FIX: LSP commands (gd, gr, K) don't work

## Root Cause

`current_file_info()` in `src/lsp/send.rs` calls `state.broker.last_opened()`
which returns `None` because session restore (`src/session/mod.rs` line 84)
opens files via `desktop.insert_tab()` without calling `broker.open()`.

## Fix (choose ONE approach)

### Approach A: Send file path from EditorView (PREFERRED)

Change EditorView to include `self.path` in the command data:

In `src/views/editor/handle.rs`, change:
```rust
// FROM:
let pos = (self.editor.cursor_line as u32, self.editor.cursor_col as u32);
queue.put_command(CM_LSP_GOTO_DEF, Some(Box::new(pos)));

// TO:
let data = (self.path.clone(), self.editor.cursor_line as u32, self.editor.cursor_col as u32);
queue.put_command(CM_LSP_GOTO_DEF, Some(Box::new(data)));
```

Do this for ALL three: CM_LSP_GOTO_DEF, CM_LSP_HOVER, CM_LSP_FIND_REFS.

Then in `src/lsp/send.rs`, change `send_goto_def`, `send_hover`, `send_find_refs`:
```rust
// FROM:
let Some(&(line, col)) = boxed.downcast_ref::<(u32, u32)>()
let (uri, lang) = current_file_info(state);

// TO:
let Some((file_path, &line, &col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>()
let uri = protocol::path_to_uri(file_path);
let lang = protocol::language_id(file_path).to_string();
```

NOTE: `send_find_refs` currently sends `(u32, u32, String)` — update it to `(PathBuf, u32, u32, String)`.

### Approach B: Fix session restore to use broker

In `src/session/mod.rs` line 84, after `desktop.insert_tab(...)`, also call:
```rust
state.broker.open(&path_str, SlotId::Center, tab_index);
```

This is simpler but doesn't fix the case where files are opened via other paths.

## Verification

After fix:
1. Open kairn, open a .rs file from tree
2. Press `gd` on a function call
3. Check .kairn.log — should see "LSP: textDocument/definition at file:///..."
4. Should navigate to the definition

## Test

Add a scenario test that verifies the path is included in LSP command data.
