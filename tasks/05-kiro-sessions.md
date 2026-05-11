# Task: Kiro Session Support

## Objective
Kiro tabs persist their session IDs so conversations survive kairn restart.
Display names are user-assignable. Session map stored in `.kairn.state`.

## Context
- Design doc: `doc/f4-design/v-014-session-mcp-todo.md` (Feature 5)
- Depends on: Task 03 (session persistence), Task 04 (MCP server)
- kiro-cli: `--resume-id <UUID>` resumes a specific session
- kiro-cli: `--list-sessions` shows sessions with IDs (JSON with --format json)

## Requirements

1. **Kiro tab metadata**:
   - Each kiro PtyTerminal gets associated metadata:
     - `display_name: String` (e.g., "Kiro:0", "research", "impl")
     - `session_id: Option<String>` (None until first captured)
   - Stored in a `KiroTabRegistry` in AppState

2. **Session ID capture**:
   - After a kiro tab exits or on save-state, run:
     `kiro-cli chat --list-sessions --format json`
   - Parse output, match most recent session to the tab
   - Store session_id in registry

3. **Spawn with resume**:
   - New kiro tab (no saved session): `kiro-cli chat --agent kairn`
   - Restored kiro tab: `kiro-cli chat --resume-id <id> --agent kairn`
   - Pass `KAIRN_MCP_SOCKET` env var to all kiro processes

4. **Rename** (`Ctrl-R` on kiro tab):
   - Changes display_name in registry
   - Updates tab title in TabGroup
   - Persisted in `.kairn.state`

5. **Close kiro tab**:
   - Remove from registry
   - Do NOT delete the kiro session server-side (user can resume manually)
   - Clean up from `.kairn.state` on next save

6. **State persistence** (extends Task 03):
   - `kiro_sessions` array in `.kairn.state`:
     ```json
     [{"name": "Kiro:0", "session_id": "f2946a26-..."}]
     ```
   - On restore: spawn kiro tabs in order with --resume-id

## Constraints
- 240 code lines per file max
- No unwrap/expect/panic in runtime code
- Graceful handling: if --resume-id fails (session deleted), start fresh session
- Session capture is best-effort (don't block UI)

## Files to Create/Modify
- CREATE: `src/kiro_registry.rs` — KiroTabRegistry
- MODIFY: `src/handler.rs` (new kiro tab, close kiro tab)
- MODIFY: `src/session/mod.rs` (save/restore kiro sessions)
- MODIFY: `src/views/terminal.rs` (spawn with --resume-id + env)
- MODIFY: `src/build_desktop.rs` (restore kiro tabs from state)
