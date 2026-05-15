# MCP: Kiro Orchestration

<!-- TODO: mark done → todo tree [2][11] -->

## Overview

Allow a parent Kiro session to spawn sub-Kiro agents in separate tabs and
send them instructions via MCP. Enables multi-agent workflows where one
Kiro delegates tasks to specialized agents.

## MCP Tools

### `spawn_kiro`

Spawn a new Kiro session tab.

```json
{
  "name": "spawn_kiro",
  "params": {
    "agent": "reviewer",       // optional: --agent=name
    "title": "Review:handler"  // optional: custom tab title
  }
}
```

Returns: `{ "tab_name": "Kiro:2", "tab_index": 3 }`

### `send_to_kiro`

Send text (instructions) to a Kiro tab. Writes to the PTY as if typed.

```json
{
  "name": "send_to_kiro",
  "params": {
    "tab_name": "Kiro:2",
    "text": "Review src/handler.rs for error handling gaps"
  }
}
```

Only works for Kiro tabs (not shell tabs — safety constraint).

Returns: `{ "ok": true }`

### `get_kiro_status`

Check if a Kiro tab is busy or idle.

```json
{
  "name": "get_kiro_status",
  "params": { "tab_name": "Kiro:2" }
}
```

Returns: `{ "status": "idle" | "busy", "last_activity_ms": 3200 }`

Detection: based on PTY output inactivity (no output for N seconds = idle).

## Implementation

1. `spawn_kiro`: dispatch command to open a kiro tab (reuse existing kiro spawn logic), return tab info
2. `send_to_kiro`: find tab by name, verify it's a Kiro tab, write text + newline to PTY
3. `get_kiro_status`: check last-output timestamp for the tab's PTY

### Safety

- `send_to_kiro` ONLY works for tabs whose title starts with "Kiro:" — refuse for Shell tabs
- Rate limit: max 1 send per second per tab (prevent flooding)
- Text is sent as-is (no escaping) — the parent Kiro is responsible for content

### Tab Identification

Need to track tab type (kiro vs shell) in the MCP snapshot. Already have
`tab_kind` concept from mcp-full-tab-visibility task.

## Tcl API

```tcl
# Spawn and send from scripts
set tab [kiro spawn --agent=reviewer]
kiro send $tab "Review the error handling in src/handler.rs"
kiro status $tab  ;# returns "idle" or "busy"
```

## Constraints

- No sending to shell tabs via MCP (only kiro)
- 240 code line max per file
- Non-blocking: spawn is async, send is fire-and-forget
- Must integrate with PTY activity badges (new tab starts as "busy")
