# Task: MCP IDE Bridge — Full Editor Integration

<!-- TODO: mark done → todo tree [2][2] -->

## Objective

Extend kairn's MCP server into a full IDE bridge so Kiro can see, navigate,
and manipulate editor state — preventing conflicts, leveraging LSP, and
providing build/search integration.

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

## Subtasks

1. [003.01 — Tier 1: Read-Only Visibility](003.01-mcp-tier1-read-only.md)
2. [003.02 — Tier 2: Tab/File Management](003.02-mcp-tier2-tab-management.md)
3. [003.03 — Tier 3: Editor Manipulation](003.03-mcp-tier3-editor-manipulation.md)
4. [003.04 — Tier 4: LSP Passthrough](003.04-mcp-tier4-lsp-passthrough.md)
5. [003.05 — Tier 5: Build & Search Integration](003.05-mcp-tier5-build-search.md)

## Constraints

- 240 lines per file max
- No unwrap/expect/panic in runtime code
- All MCP responses under 64KB
- LSP timeout: 10s
- Socket I/O timeout: 300s read, 30s write
- Edits must be undoable
- No new dependencies (use existing: serde_json, sha2, ignore)

## Priority Order

Tiers are implemented sequentially — each builds on the previous.
Commit and push after each tier when tests pass.
