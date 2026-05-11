# Task: MCP Server

## Objective
Run a background MCP server inside the kairn process, accessible to kiro via
Unix socket. Kiro connects through a stdio↔socket proxy (`kairn --mcp-connect`).

## Context
- Design doc: `doc/f4-design/v-014-session-mcp-todo.md` (Feature 4)
- Reference: `duir-tui/src/app/app_kiron_mcp.rs` (listener + agent file)
- Reference: `duir-tui/src/main.rs` `run_mcp_bridge()` (proxy)
- Reference: `duir-core/src/mcp_server/` (JSON-RPC protocol)

## Requirements

1. **CLI flag** — add `--mcp-connect` to kairn's clap args:
   - When set, run `bridge::run_mcp_bridge()` and exit
   - Reads `KAIRN_MCP_SOCKET` env var for socket path
   - Bridges stdin↔socket (two threads, io::copy)

2. **Socket listener** (`src/mcp/listener.rs`):
   - Bind Unix socket at `$XDG_RUNTIME_DIR/kairn-{hash}.sock`
   - Hash = first 8 hex chars of SHA256 of canonical project root
   - Accept connections in a thread, spawn per-connection handler
   - Clean up socket file on kairn exit

3. **Instance lock**:
   - On startup, try connecting to the socket path
   - If connection succeeds → another kairn is running → exit with error
   - If connection fails → safe to start → bind socket

4. **MCP server** (`src/mcp/server.rs`):
   - Implements JSON-RPC 2.0 (same protocol as duir-core's McpServer)
   - Methods: `initialize`, `tools/list`, `tools/call`
   - Reads from shared snapshot (Arc<Mutex<McpSnapshot>>)

5. **Snapshot** (`src/mcp/snapshot.rs`):
   - Updated on each Tick from main thread
   - Contains: open tabs list, terminal buffers, editor content, selections, todo tree

6. **Tools** (`src/mcp/tools.rs`):
   | Tool | Description |
   |------|-------------|
   | `list_tabs` | All open tabs with type/title/path |
   | `list_terminals` | Terminal tabs: name, status, type (shell/kiro) |
   | `get_file_content` | Editor buffer content by path |
   | `get_selection` | Current selection text + file + range |
   | `get_terminal_content` | Terminal scrollback + visible (by tab name) |
   | `get_todo_tree` | Todo subtree listing |
   | `search_files` | Grep project files, return matches |
   | `open_file` | Open file in editor (sends CM_OPEN_FILE) |
   | `close_file` | Close editor tab by path |

7. **Agent file** (`src/mcp/agent_file.rs`):
   - On startup, write `.kiro/agents/kairn.json`:
     ```json
     {
       "name": "kairn",
       "mcpServers": {
         "kairn": {
           "command": "/path/to/kairn",
           "args": ["--mcp-connect"],
           "env": { "KAIRN_MCP_SOCKET": "<socket_path>" }
         }
       },
       "includeMcpJson": true,
       "tools": ["*"],
       "allowedTools": ["@kairn"]
     }
     ```

8. **Logging** (`src/mcp/log.rs`):
   - Append-only log at `$XDG_RUNTIME_DIR/kairn-mcp.log`
   - Timestamp + PID + component + message

## Constraints
- 240 code lines per file max (split tools into tools_read.rs / tools_write.rs if needed)
- No unwrap/expect/panic in runtime code
- Socket cleanup on normal exit AND on panic (via panic hook)
- Timeouts on all socket I/O (300s read, 30s write)

## Files to Create/Modify
- CREATE: `src/mcp/mod.rs`
- CREATE: `src/mcp/bridge.rs`
- CREATE: `src/mcp/listener.rs`
- CREATE: `src/mcp/server.rs`
- CREATE: `src/mcp/tools.rs`
- CREATE: `src/mcp/snapshot.rs`
- CREATE: `src/mcp/agent_file.rs`
- CREATE: `src/mcp/log.rs`
- MODIFY: `src/main.rs` (--mcp-connect flag, start listener, instance lock)
- MODIFY: `Cargo.toml` (add sha2 or similar for hash)
