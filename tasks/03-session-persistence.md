# Task: Session Persistence

## Objective
Save workspace state on quit, restore on launch. Includes open editor tabs,
cursor positions, layout, panel sizes, unfolded directories, and kiro session map.

## Context
- Design doc: `doc/f4-design/v-014-session-mcp-todo.md` (Feature 3)
- State file: `.kairn.state` (JSON, gitignored)
- Existing: `Ctrl-X S` saves session, `Ctrl-Shift-O` loads — wire these to new system

## Requirements

1. **State schema** (`src/session/schema.rs`):
   ```rust
   struct SessionState {
       version: u32,
       layout: String,           // "wide", "tall_right", "tall_bottom"
       left_width: u16,
       right_width: u16,
       active_tabs: HashMap<String, usize>,  // "left"→0, "center"→2, "right"→1
       editor_tabs: Vec<EditorTabState>,
       kiro_sessions: Vec<KiroSessionState>,
       unfolded_dirs: Vec<String>,
   }
   struct EditorTabState { path: String, line: u32, col: u32 }
   struct KiroSessionState { name: String, session_id: String }
   ```

2. **Save** (`src/session/mod.rs`):
   - Collect state from LayoutGroup (layout, widths)
   - Collect open editor tabs with positions from center TabGroup
   - Collect kiro tab names + session IDs from right TabGroup
   - Collect unfolded dirs from FileTreeView
   - Serialize to `.kairn.state`
   - Triggered on: quit, `Ctrl-X S`

3. **Restore** (`src/session/mod.rs`):
   - On launch, if `.kairn.state` exists, parse it
   - Set layout and panel widths
   - Open editor tabs at saved positions
   - Spawn kiro tabs with `kiro-cli chat --resume-id <session_id> --agent kairn`
   - Expand saved directories in file tree
   - Shell tabs: always spawn fresh (not persisted)

4. **Auto-save on quit**:
   - In quit handler (CM_QUIT), save state before exit
   - Don't save if state file doesn't exist and no tabs are open (first run)

5. **Unfolded dirs**:
   - FileTreeView needs to expose `get_unfolded() -> Vec<String>`
   - FileTreeView needs `set_unfolded(dirs: &[String])` for restore

## Constraints
- 240 code lines per file max
- No unwrap/expect/panic in runtime code
- Graceful handling of missing/corrupt state file (just skip restore)
- Session IDs: captured from `kiro-cli chat --list-sessions` after first kiro quit

## Files to Create/Modify
- CREATE: `src/session/mod.rs`
- CREATE: `src/session/schema.rs`
- MODIFY: `src/handler.rs` (save on quit)
- MODIFY: `src/main.rs` (restore on launch)
- MODIFY: `src/views/tree.rs` (expose unfolded dirs)
- MODIFY: `src/build_desktop.rs` (accept restore state)
