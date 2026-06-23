# Code Structure SOP

## ABSOLUTE RULES (NO EXCEPTIONS)

### Git Commits
- **`--no-verify` is FORBIDDEN.** Every commit MUST pass the pre-commit hook.
- If the hook fails, FIX the code. Never bypass.
- This applies to ALL agents, ALL contexts, ALL circumstances.
- Rationale: the hook IS the quality gate. Bypassing it means shipping broken code.

### Method Length: max 40 code lines
- Split MUST be logical — each extracted function has a clear single purpose
- Split ratio: at least 70/30 (no 95/5 splits with trivial 2-line helpers)
- Better: split into 3+ balanced pieces when possible
- Names must describe WHAT the function does, not WHERE it came from
- NO EXCEPTIONS to the 40-line limit. If a struct init exceeds it, decompose the struct.

### File Length: max 240 code lines
- Same rules: split by responsibility, not arbitrary line count
- Each file has a single clear topic
- NO EXCEPTIONS.

### Pre-commit Hook Checks (ALL must pass)
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. File length ≤ 240 code lines
4. `cargo test --workspace --no-fail-fast` (zero failures, zero ignored)
5. No hardcoded colors outside palette files
6. No bare `pub` fields (use `pub(crate)`)
7. `mcp-lint-cli` (method-length, nesting, no-deep-path, etc.)

### File Length: max 240 code lines
- Same rules as methods: split by responsibility, not arbitrary line count
- Each file has a single clear topic (one struct + its impl, or one logical group of functions)
- Sub-structs go in separate files

### Struct Decomposition
AppState (and similar large state holders) MUST be decomposed into sub-structs:
- Each sub-struct groups fields that are **used together** (same handlers, same lifecycle)
- Sub-struct MUST be defined in its own file
- Chained field access (`a.b.c`) is FORBIDDEN — use accessor methods: `a.b().c()`
- This means each sub-struct exposes `pub(crate) fn field(&self)` and `pub(crate) fn field_mut(&mut self)` accessors
- The parent struct exposes accessors to the sub-struct itself

### Why
- Agents generate code that touches state. Without encapsulation, they produce fragile spaghetti.
- Accessors make refactoring possible (change internals without touching callsites).
- Sub-structs with clear boundaries let agents reason about smaller scopes.
- File-per-struct means agents can read just what they need, not 500-line god objects.

## AppState Sub-structs

| Struct | File | Fields | Purpose |
|--------|------|--------|---------|
| `Workspace` | `workspace_state.rs` | broker, buffers, root_dir, roots, settings | Core file/project state |
| `LspSubsystem` | `lsp_subsystem.rs` | lsp, lsp_pending, lsp_state, lsp_languages | LSP lifecycle |
| `BuildState` | `build_state.rs` | errors, error_idx, pending | Build/test results |
| `ScriptState` | `script_state.rs` | script, pending_hooks, plugins, command_list, completer_roots | Scripting + completions |
| `EditorShared` | `editor_shared.rs` | shared_register, clipboard, command_history, search_history, linked_scroll | Cross-editor state |
| `UiChrome` | `ui_chrome.rs` | waker, tty_file, last_window_title, pty_last_output, tab_titles_dirty, show_messages_on_start, key_bindings, theme_state | Terminal/window chrome |
| `PendingOps` | `pending_ops.rs` | grep_pending, pending_tab, confirm_context, todo_note_path | Transient async ops |
| *(AppState top-level)* | `app_state.rs` | workspace, lsp, build, scripting, editor, ui, pending, cursor_pos, messages, kiro_registry, mcp, diff_base | Composition root |
