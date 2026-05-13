# Task: Full Tab & Editor Visibility via MCP

## Objective
Expose complete UI state through the MCP server so that Kiro can see
everything a human user sees: tab contents, focus, selections, cursor
positions, modification state, and terminal buffers.

## Current Gaps
The existing MCP tools (`list_tabs`, `list_terminals`, `get_terminal_content`)
return partial information. After a restart, terminal lookup by name fails,
and there is no way to query:
- Which tab is focused/active
- Cursor position or selections in editors
- Whether a buffer is modified (unsaved) or saved
- Whether a tab is structural (tree view) or text (editor)
- Todo panel contents
- Full terminal content by the names reported by `list_terminals`

## Requirements

### 1. `list_tabs` enhancements
Return additional fields for each tab:
- `focused: bool` — is this the active/visible tab
- `modified: bool` — has unsaved changes
- `cursor: { line, col }` — current cursor position (editors only)
- `tab_kind: "editor" | "tree" | "shell" | "kiro"` — structural vs text
- `order: usize` — position in tab bar

### 2. `get_selection` improvements
- Work by tab name or path
- Return multiple selections if editor has multi-cursor
- Include selection mode (char/line/block)
- Return empty list (not error) when no selection active

### 3. `get_todo_tree` 
- Return full todo tree contents (task descriptions, checked state, nesting)
- Should work even when Todo panel is not focused

### 4. `get_terminal_content` fix
- Terminal name resolution must match names returned by `list_terminals`
- Support lookup by index as fallback (e.g. terminal 0, terminal 1)
- Return scrollback + visible content

### 5. Snapshot updates
Update `McpSnapshot` to capture:
- Focused tab index/name
- Per-editor: cursor pos, selection ranges, modified flag
- Todo tree state
- Terminal name→index mapping for reliable lookup

## Constraints
- Keep per-tool response under 64KB (truncate terminal scrollback if needed)
- No new dependencies
- 240 lines per file max

## Files to Modify
- `src/mcp/snapshot.rs` — extend snapshot struct
- `src/mcp/tools.rs` (or `tools_read.rs`) — update tool implementations
- `src/mcp/server.rs` — register any new tools
- Wherever the snapshot is populated from app state (tick handler)
