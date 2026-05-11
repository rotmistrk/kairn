# Task: Todo Tree View (Left Panel Tab)

## Objective
Add a third tab "Todo" to the left panel showing hierarchical tasks from `.kairn.todo`.
Uses `duir-core` as a git dependency for the data model and tree operations.

## Context
- Design doc: `doc/f4-design/v-014-session-mcp-todo.md` (Feature 2)
- duir-core repo: `github.com/rotmistrk/duir` → `crates/duir-core/`
- Left panel tabs: Files (index 0), Git (index 1), Todo (index 2)
- Pattern: same as `GitChangesView` — wraps `TreeView<TodoTreeData>`

## Requirements

1. **Add duir-core dependency**:
   ```toml
   duir-core = { git = "ssh://git@github.com/rotmistrk/duir.git", default-features = false }
   ```
   Only needs: `model`, `tree_ops`, `file_storage`, `crypto`

2. **TodoTreeData** (`src/views/todo_tree/data.rs`):
   - Implements `TreeData` trait
   - Wraps `duir_core::TodoFile`
   - Loads from `.kairn.todo` (creates empty if absent)
   - Saves on every mutation
   - Virtual "Code TODOs" subtree at top (collected from file tree grep)

3. **TodoTreeView** (`src/views/todo_tree/mod.rs`):
   - Wraps `TreeView<TodoTreeData>`
   - Non-closeable tab (like Git)
   - Title: "Todo"

4. **Key bindings** (same as duir, `src/views/todo_tree/handle.rs`):
   - `Space` — toggle completed
   - `e` — edit title (inline)
   - `n` — new sibling
   - `b` — new child
   - `d` — delete (requires `y` confirm)
   - `!` — toggle important
   - `J`/`K` — swap down/up
   - `H`/`L` — promote/demote
   - `S` — sort children
   - `c` — clone subtree
   - `Tab` — focus note in main panel
   - `/` — filter/search

5. **Code TODOs collection**:
   - Grep for `TODO` and `FIXME` in project files (respecting .gitignore)
   - Present as read-only virtual subtree at top of todo tree
   - Each entry shows file:line and the comment text
   - Selecting opens file at that line
   - Refreshed periodically (every ~60 ticks, like git panel)

6. **Integration**:
   - Add to left panel in `build_desktop.rs` as third tab
   - F6 cycles through Files → Git → Todo

## Constraints
- 240 code lines per file max (split into mod/data/handle)
- No unwrap/expect/panic in runtime code
- duir-core used as library only — no kiron features

## Files to Create/Modify
- MODIFY: `Cargo.toml` (add duir-core dependency)
- CREATE: `src/views/todo_tree/mod.rs`
- CREATE: `src/views/todo_tree/data.rs`
- CREATE: `src/views/todo_tree/handle.rs`
- CREATE: `src/views/todo_tree/code_todos.rs` (grep collector)
- MODIFY: `src/build_desktop.rs` (add Todo tab)
- MODIFY: `src/views/mod.rs` (declare module)
