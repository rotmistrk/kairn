# Task: MCP IDE Bridge — Full Editor Integration

<!-- TODO: mark done → todo tree [2][2] -->

## Objective
Extend kairn's MCP server into a full IDE bridge so Kiro can see, navigate,
and manipulate editor state — preventing conflicts, leveraging LSP, and
providing build/search integration.

## Tier 1: Read-Only Visibility (fix/extend existing)

Ref: `tasks/mcp-full-tab-visibility.md`

### Tools to fix/extend:
| Tool | Change |
|------|--------|
| `list_tabs` | Add: `focused`, `modified`, `cursor {line,col}`, `tab_kind`, `order` |
| `list_terminals` | Fix name resolution so `get_terminal_content` works |
| `get_terminal_content` | Support lookup by index as fallback |
| `get_todo_tree` | New: return full todo tree (titles, checked state, nesting) |

### Implementation:
- Expand `McpSnapshot` with focus index, per-editor cursor/modified state
- Populate snapshot from tick handler (read from desktop state)
- Update tool responses to include new fields
- Terminal: store name→index mapping in snapshot

### Files:
- `src/mcp/snapshot.rs` — extend struct
- `src/mcp/tools.rs` — update responses
- Tick handler (wherever snapshot is populated)

### Effort: ~2-3 hours

---

## Tier 2: Tab/File Management

### New tools:
| Tool | Description | Command |
|------|-------------|---------|
| `open_file` | Open existing file in editor | `CM_OPEN_FILE` |
| `create_file` | Create file on disk + open in editor | write + `CM_OPEN_FILE` |
| `close_tab` | Close editor tab by path | `CM_CLOSE` on target |
| `switch_mode` | Toggle diff / structural edit | `CM_TOGGLE_DIFF` / `CM_STRUCT_EDIT` |

### Implementation:
- MCP tools dispatch commands via the event queue
- Need a command sender channel: MCP thread → main event loop
- Commands are processed on next tick (async from MCP caller's perspective)
- Response: confirm action taken or error

### Files:
- `src/mcp/tools.rs` (or `tools_write.rs`) — new tool handlers
- `src/mcp/mod.rs` — command sender channel setup
- Main loop — poll MCP command channel alongside terminal events

### Effort: ~2 hours

---

## Tier 3: Editor Manipulation

### New tools:
| Tool | Description |
|------|-------------|
| `edit_buffer` | Apply text edits to open buffer (line range replace) |
| `insert_text` | Insert text at cursor or position |
| `set_cursor` | Move cursor to line:col in specified tab |
| `get_buffer` | Get full buffer content for open file |
| `save_file` | Save the buffer to disk |

### Implementation:
- `edit_buffer` uses the same `apply_edit()` path as LSP workspace edits
- Edits are applied on the main thread via command channel
- Must handle: file not open (error), file modified (warn), undo integration
- Edits should be undoable (single undo group per MCP edit)

### Key design decision:
Edits go through the editor's undo system — user can always Ctrl-Z to revert
Kiro's changes. This is safer than writing to disk directly.

### Files:
- `src/mcp/tools.rs` — edit tool handlers
- `src/views/editor/mod.rs` — expose `apply_edit()` via command
- `src/handler.rs` — handle MCP edit commands

### Effort: ~3 hours

---

## Tier 4: LSP Passthrough

### New tools:
| Tool | Description |
|------|-------------|
| `goto_definition` | Returns location(s) for symbol at file:line:col |
| `find_references` | Returns all reference locations for symbol |
| `rename_symbol` | Workspace-wide rename (applies via editor) |
| `get_diagnostics` | Current diagnostics for file (or all files) |
| `get_completions` | Completion items at position |
| `hover` | Type/doc info at position |

### Implementation:
- Kairn already runs LSP clients per language
- MCP tool sends request → waits for LSP response → returns to caller
- Challenge: LSP is async, MCP handler is sync (blocking on socket)
- Solution: oneshot channel — MCP handler sends request + oneshot sender,
  main thread forwards to LSP, LSP response sent back via oneshot
- Timeout: 10s for LSP responses (some servers are slow)

### Rename flow:
1. MCP `rename_symbol` → LSP `textDocument/rename` → workspace edit
2. Apply workspace edit to all open buffers via `edit_buffer` path
3. Write changes to files not currently open
4. Return list of modified files

### Files:
- `src/mcp/tools_lsp.rs` — new file for LSP tools
- `src/mcp/mod.rs` — LSP request channel
- `src/lsp/mod.rs` — expose request forwarding
- `src/handler.rs` — route MCP→LSP requests

### Effort: ~4-5 hours

---

## Tier 5: Build & Search Integration

### New tools:
| Tool | Description |
|------|-------------|
| `run_build` | Trigger project build, return stdout/stderr + parsed errors |
| `run_command` | Run arbitrary shell command, return output |
| `get_build_errors` | Get last build's parsed error list |
| `search_project` | Grep project files (like `:grep`), return matches |
| `search_replace` | Search and replace across files (preview + apply) |

### Implementation:
- Build: spawn process, capture output, parse errors (rustc/gcc/tsc format)
- Search: reuse existing grep infrastructure (`ignore` crate walker)
- `run_command`: spawn in a shell tab or headless, return output
- `search_replace`: preview returns diffs, apply commits changes

### Safety:
- `run_command` should be limited to project directory
- `search_replace` apply should go through editor buffers (undoable)
- Build output capped at 64KB in MCP response

### Files:
- `src/mcp/tools_build.rs` — build/command tools
- `src/mcp/tools_search.rs` — search tools
- Reuse: `src/views/editor/build.rs` (if exists), grep logic

### Effort: ~2 hours

---

## Architecture Notes

### Command Channel (Tier 2+)
```
MCP thread                    Main thread
    │                              │
    ├─── McpCommand ──────────────►│ (mpsc channel)
    │    { id, action, reply_tx }  │
    │                              ├── dispatch action
    │◄── Result ───────────────────┤ (oneshot reply)
    │                              │
```

### Snapshot (Tier 1)
Updated every tick from main thread. Read-only clone sent to MCP handlers.
No locking on hot path — snapshot is `Arc<McpSnapshot>` swapped atomically.

### File Split
To stay under 240 lines per file:
- `src/mcp/tools_read.rs` — read-only tools (Tier 1)
- `src/mcp/tools_write.rs` — file/editor manipulation (Tier 2-3)
- `src/mcp/tools_lsp.rs` — LSP passthrough (Tier 4)
- `src/mcp/tools_build.rs` — build/search (Tier 5)

## Constraints
- 240 lines per file max
- No unwrap/expect/panic in runtime code
- All MCP responses under 64KB
- LSP timeout: 10s
- Socket I/O timeout: 300s read, 30s write
- Edits must be undoable
- No new dependencies (use existing: serde_json, sha2, ignore)

## Priority Order
1. Tier 1 (visibility) — immediate value, low risk
2. Tier 2 (tab management) — enables basic workflow
3. Tier 3 (editor manipulation) — core value prop
4. Tier 4 (LSP passthrough) — high value, moderate complexity
5. Tier 5 (build/search) — nice to have, easy once channel exists
