# Kiro Session Integration Design

## Goal

Named, resumable kiro sessions scoped per-problem, with automatic session
persistence across kairn restarts.

## Commands

```
M-x kiro --agent=mycoolagent --session=problem-one   # new or resume named session
M-x kiro --resume                                     # resume most recent in project root
M-x kiro --agent=kairn                                # anonymous session (current behavior)
```

`--resume` and `--session` are mutually exclusive.

## Directory Layout

Each named session gets a dedicated directory so kiro's `--resume` flag
(which resumes the most recent conversation in cwd) always targets the
correct session:

```
.kairn/kiro-sessions/problem-one/
├── .kiro/
│   ├── agents/
│   │   └── mycoolagent.json    ← copy of ~/.kiro/agents/mycoolagent.json + kairn MCP
│   └── steering/               ← symlink → ../../../.kiro/steering
```

Global `~/.kiro/` (steering, settings, etc.) is always visible to kiro
regardless of cwd.

## Workflow

### First launch: `M-x kiro --agent=mycoolagent --session=problem-one`

1. Validate `~/.kiro/agents/mycoolagent.json` exists → error if not
2. Create `.kairn/kiro-sessions/problem-one/` if it doesn't exist
3. Create `.kairn/kiro-sessions/problem-one/.kiro/agents/mycoolagent.json`:
   - Copy from `~/.kiro/agents/mycoolagent.json`
   - Inject kairn MCP server config (same as current `kairn.json`)
4. Symlink `.kairn/kiro-sessions/problem-one/.kiro/steering` → `../../../.kiro/steering`
5. Spawn: `kiro-cli chat --agent=mycoolagent`
   - cwd = `.kairn/kiro-sessions/problem-one/`
   - env: `KAIRN_MCP_SOCKET`
6. Tab name: `kiro:problem-one`
7. Register in `KiroTabRegistry`: `{name: "kiro:problem-one", agent: "mycoolagent", session_name: "problem-one"}`

### Re-open existing: `M-x kiro --agent=mycoolagent --session=problem-one` (dir exists)

1. Directory exists → this is a resume
2. Spawn: `kiro-cli chat --agent=mycoolagent --resume`
   - cwd = `.kairn/kiro-sessions/problem-one/`
3. Tab name: `kiro:problem-one`

### Kairn exit

- Save open kiro tabs to `.kairn/session.json`:
  ```json
  {
    "kiro_sessions": [
      {"name": "kiro:problem-one", "agent": "mycoolagent", "session_name": "problem-one"}
    ]
  }
  ```

### Kairn restart

- For each entry in `kiro_sessions` state:
  - Spawn `kiro-cli chat --agent=<agent> --resume` with cwd = `.kairn/kiro-sessions/<session_name>/`
  - Tab name from state

### Tab closed by user

- Remove from `KiroTabRegistry` (won't be saved to state)
- Session dir remains on disk (can be reopened later)

### `M-x kiro --resume` (no session name)

- Spawn `kiro-cli chat --resume` in project root (current behavior)
- No session dir, no named persistence

## Project Root Access

From session dir `.kairn/kiro-sessions/problem-one/`, the project root is `../../`.
In practice this doesn't matter because:

- Agent config uses `"allowedTools": ["@kairn"]` → all file/shell operations
  go through kairn's MCP server which operates from the real project root
- Kiro's native tools are not used when `@kairn` is the only allowed tool source

## Agent Config Injection

The kairn MCP block injected into the session's agent config:

```json
{
  "mcpServers": {
    "kairn": {
      "command": "/path/to/kairn",
      "args": ["--mcp-connect"],
      "env": {
        "KAIRN_MCP_SOCKET": "${KAIRN_MCP_SOCKET}"
      }
    }
  }
}
```

## State Schema Change

`KiroSessionState` gains `agent` and `session_name` fields:

```rust
pub struct KiroSessionState {
    pub name: String,              // tab title: "kiro:problem-one"
    pub agent: Option<String>,     // "mycoolagent"
    pub session_name: Option<String>, // "problem-one" (None for anonymous)
    pub session_id: Option<String>,   // deprecated, kept for compat
}
```
