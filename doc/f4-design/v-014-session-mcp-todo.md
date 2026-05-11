# v-014: Session Persistence, MCP Server, Todo Tree, Backscroll

## Overview

Add four interconnected features to kairn:

1. **Todo tree** — third tab in the left panel (Files / Git / Todo)
2. **Session persistence** — save/restore open tabs, positions, kiro session map
3. **Backscroll buffer** — configurable scrollback for PTY terminals
4. **MCP server** — background server giving kiro access to kairn state

## Architecture

### Dependency: `duir-core`

kairn references `duir-core` as a git dependency (separate repo, same author):

```toml
# Cargo.toml
duir-core = { git = "ssh://git@github.com/rotmistrk/duir.git", default-features = false }
```

**Reused from duir-core:**
- `TodoFile`, `TodoItem`, `Completion`, `NodeId` — data model
- `tree_ops` — tree navigation, get/insert/remove/move
- `mcp_server::McpServer` — JSON-RPC protocol, request dispatch
- `FileStorage` — load/save `.todo.json` format
- `crypto` — encrypted nodes support

**NOT reused:**
- `KironMeta` / kiron session management (too complex for kairn users)
- `config` (kairn has its own)
- `s3_storage`, `docx_*`, `pdf_*` (not needed)

### Dependency: MCP bridge code

The stdio↔socket bridge and socket listener are small (~100 lines each).
Copy them into kairn (adapted naming) rather than depending on `duir-tui`:

- `src/mcp/bridge.rs` — `kairn --mcp-connect` mode (stdio↔socket proxy)
- `src/mcp/listener.rs` — Unix socket listener thread
- `src/mcp/agent_file.rs` — writes `.kiro/agents/kairn.json`
- `src/mcp/log.rs` — append-only MCP diagnostics log

---

## Feature 1: Backscroll Buffer

**Goal:** PTY terminals keep a configurable scrollback (default: 2000 lines).
PgUp/PgDn scroll through it.

**Design:**
- `TermBuf` gets a `scrollback: VecDeque<Vec<TCell>>` ring buffer
- When a line scrolls off the top of the visible grid, it's pushed to scrollback
- `scrollback_limit` configurable (default 2000, from `.kairnrc`)
- PgUp/PgDn in terminal view scroll through scrollback (existing mechanism extended)
- MCP tool `get_terminal_content` reads from scrollback + visible grid

**Files:**
- `txv-render/src/termbuf/scrollback.rs` — ring buffer logic
- `txv-render/src/termbuf/mod.rs` — integrate scrollback into scroll_up
- `txv-widgets/src/pty_terminal.rs` — PgUp/PgDn uses scrollback

---

## Feature 2: Todo Tree (Left Panel Tab)

**Goal:** Third tab in left panel showing hierarchical tasks from `.kairn.todo`.

**Data model:** `duir-core::TodoFile` (JSON format, same as duir).

**View:** `TodoTreeView` — wraps `TreeView<TodoTreeData>` (same pattern as
`FileTreeView` and `GitChangesView`).

**Operations (keyboard, same as duir):**
- `Space` — toggle completed
- `e` — edit title (inline rename)
- `n` — new sibling
- `b` — new child
- `d` — delete (requires `y` to confirm)
- `!` — toggle important
- `J`/`K` — swap down/up
- `H`/`L` — promote/demote (change depth)
- `S` — sort children
- `c` — clone subtree
- `Tab` — focus note (edit in main panel)
- `/` — filter/search
- `:` — command mode

**Auto-collected TODO/FIXME subtree:**
A virtual (read-only) subtree at the top named "Code TODOs" that collects
`TODO` and `FIXME` comments from the file tree. Refreshed periodically
(same tick-based polling as git panel). Selecting an entry opens the file
at that line.

**File:** `.kairn.todo` in project root. Auto-created empty on first use.
Version-controllable.

**Files:**
- `src/views/todo_tree/mod.rs` — `TodoTreeView`
- `src/views/todo_tree/data.rs` — `TodoTreeData` implementing `TreeData`
- `src/views/todo_tree/handle.rs` — keyboard handling, mutations

---

## Feature 3: Session Persistence

**Goal:** On quit, save workspace state. On launch, restore it.

**State file:** `.kairn.state` (JSON). Gitignored.

**Schema:**
```json
{
  "version": 1,
  "layout": "wide",
  "left_width": 30,
  "right_width": 60,
  "active_tabs": { "left": 0, "center": 2, "right": 1 },
  "editor_tabs": [
    { "path": "src/main.rs", "line": 42, "col": 0, "mode": "normal" }
  ],
  "kiro_sessions": [
    { "name": "Kiro:0", "session_id": "f2946a26-..." },
    { "name": "Kiro:1", "session_id": "abc123-..." }
  ],
  "unfolded_dirs": [
    "src",
    "src/views",
    "src/mcp"
  ]
}
```

**Restore behavior:**
- Editor tabs: reopen files at saved positions
- Shell tabs: spawn fresh shells (no session persistence for shells)
- Kiro tabs: spawn `kiro-cli chat --resume-id <session_id> --agent kairn`
- Layout/widths: restore panel sizes

**Kiro session lifecycle:**
- New kiro tab → generate display name (`Kiro:0`), kiro-cli creates session UUID
- Rename (`Ctrl-R`) → updates display name in state
- Close tab → remove from `kiro_sessions` (session stays in kiro DB for manual resume)
- Save state → write current session IDs to `.kairn.state`

**Capturing session ID:** After spawning kiro-cli with `--resume`, we need the
session ID. Options:
1. Use `--resume-id` with a pre-generated UUID (kiro creates if not found)
2. Parse kiro's output for session ID
3. Use `kiro-cli chat --list-sessions` after close

Option 1 is cleanest but requires kiro to accept unknown UUIDs gracefully.
Option 2 is fragile. **Chosen: spawn with `--resume` (no ID) for new sessions;
after first save, capture ID from `kiro-cli chat --list-sessions` and store it.**

For restore: `kiro-cli chat --resume-id <saved_id>`.

**Files:**
- `src/session/mod.rs` — save/load state
- `src/session/schema.rs` — state data structures
- Integration in `main.rs` (load on start) and quit handler (save)

---

## Feature 4: MCP Server

**Goal:** Background server in kairn process, accessible to kiro via Unix socket.

**Architecture (same as duir):**

```
kairn process                          kiro-cli (PTY)
┌─────────────────┐                   ┌──────────────┐
│ MCP Listener    │◄──Unix socket──►  │ kairn proxy  │ ◄──stdio──► kiro agent
│ (thread)        │                   │ (--mcp-connect)│
│                 │                   └──────────────┘
│ Shared state:   │
│  - open files   │
│  - selections   │
│  - terminal buf │
│  - todo tree    │
│  - search results│
└─────────────────┘
```

**Socket:** `$XDG_RUNTIME_DIR/kairn-{hash}.sock` where hash = first 8 chars of
SHA256 of canonical project root path.

**Instance lock:** Try connecting to socket on startup. If succeeds → another
kairn is running in this dir → refuse to start with error message.

**Agent file:** On startup, kairn writes `.kiro/agents/kairn.json`:
```json
{
  "name": "kairn",
  "mcpServers": {
    "kairn": {
      "command": "/path/to/kairn",
      "args": ["--mcp-connect"],
      "env": { "KAIRN_MCP_SOCKET": "/run/user/1000/kairn-abcd1234.sock" }
    }
  },
  "includeMcpJson": true,
  "tools": ["*"],
  "allowedTools": ["@kairn"]
}
```

**MCP Tools exposed:**

| Tool | Description |
|------|-------------|
| `list_tabs` | List all open tabs (editors, shells, kiros) with metadata |
| `list_terminals` | List terminal tabs with status (running/exited, shell/kiro) |
| `get_file_content` | Read content of an open editor buffer |
| `get_selection` | Current selection in focused editor (if any) |
| `get_terminal_content` | Shell/kiro buffer content (visible + scrollback) |
| `get_todo_tree` | Todo tree subtree listing |
| `search_files` | Grep/search across project files |
| `open_file` | Open a file in the editor |
| `close_file` | Close an editor tab |

**Shared state access:** The MCP listener thread needs read access to kairn state.
Use `Arc<Mutex<McpSnapshot>>` updated on each Tick (or on-demand via channel).

**Files:**
- `src/mcp/mod.rs` — module root
- `src/mcp/bridge.rs` — `--mcp-connect` stdio↔socket proxy
- `src/mcp/listener.rs` — socket listener thread
- `src/mcp/server.rs` — request dispatch, tool implementations
- `src/mcp/tools.rs` — tool definitions (JSON schema)
- `src/mcp/agent_file.rs` — write `.kiro/agents/kairn.json`
- `src/mcp/log.rs` — diagnostics logging
- `src/mcp/snapshot.rs` — shared state snapshot structure

---

## Implementation Order

1. **Backscroll buffer** — prerequisite for MCP `get_terminal_content`
2. **Todo tree view** — standalone UI feature, uses `duir-core`
3. **Session persistence** — save/restore (initially without kiro session map)
4. **MCP server** — bridge, listener, tools, agent file
5. **Kiro session support** — session map in state, resume-id on restore

---

## File Layout

```
src/
  mcp/
    mod.rs
    bridge.rs          # --mcp-connect proxy
    listener.rs        # Unix socket listener
    server.rs          # JSON-RPC dispatch
    tools.rs           # Tool definitions + handlers
    agent_file.rs      # Write .kiro/agents/kairn.json
    log.rs             # MCP diagnostics log
    snapshot.rs        # Shared state for MCP reads
  session/
    mod.rs             # save/load orchestration
    schema.rs          # State JSON schema
  views/
    todo_tree/
      mod.rs           # TodoTreeView
      data.rs          # TodoTreeData (TreeData impl)
      handle.rs        # Key handling, mutations
txv-render/src/
  termbuf/
    scrollback.rs      # Ring buffer for scrollback
```

---

## Configuration (`.kairnrc`)

```json
{
  "scrollback_lines": 2000,
  "kiro_command": "kiro-cli",
  "kiro_args": ["chat", "--resume"],
  "kiro_agent": "kairn",
  "kiro_trust_all_tools": false
}
```

---

## References

- `duir-core` crate: `github.com/rotmistrk/duir` → `crates/duir-core/`
  - `mcp_server/mod.rs` — McpServer, JSON-RPC protocol
  - `mcp_server/tools.rs` — tool dispatch pattern
  - `model.rs` — TodoFile, TodoItem
  - `tree_ops.rs` — tree navigation
  - `crypto.rs` — encrypted nodes
- `duir-tui` (reference only, code adapted):
  - `app/app_kiron_mcp.rs` — socket listener, agent file generation
  - `main.rs` — `run_mcp_bridge()` stdio↔socket proxy
  - `mcp_log.rs` — logging utilities
- Kiro docs:
  - Session: `kiro-cli chat --resume-id <ID>`
  - MCP: `.kiro/settings/mcp.json` or agent `mcpServers` field
  - Agent: `includeMcpJson: true` picks up workspace mcp.json
  - No `--include-mcp-json` CLI flag exists
