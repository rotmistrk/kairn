//! Help topic: MCP tool permissions reference.

pub(crate) fn help_mcp() -> String {
    "\
─── MCP Server ─────────────────────────────────
  kairn exposes state to Kiro AI via JSON-RPC over Unix socket.
  Socket path: $KAIRN_MCP_SOCKET

─── Tool Categories ──────────────────────────────
  Tabs         list, close, get content
  Files        open, create, save
  Editor       read state, edit buffer, insert, set cursor,
               undo/redo
  Terminal     read content, send input
  Build        run build/test, get errors, search (grep)
  Diff         revert hunk under cursor
  Split        create/close/focus/open/linked scroll
  Todo         add, toggle, remove, move, promote/demote, notes
  Git          stage, unstage, commit
  LSP          start/restart/stop, hover/definition/references/
               rename/code-action/format
  Scripting    eval Tcl
  Workspace    list/add/remove roots
  Messages     read message log

─── Permissions ────────────────────────────────
  All tools are available to any connected client.
  Access is controlled by socket file permissions.

See also:
  → :help tcl         Tcl scripting (eval tool)
  → :help commands    All commands
"
    .to_string()
}
