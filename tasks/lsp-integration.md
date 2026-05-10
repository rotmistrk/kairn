# Task: LSP Integration for Editor

## Overview

Add Language Server Protocol support to the kairn editor. This enables:
- Diagnostics (errors/warnings inline)
- Go to definition/declaration/implementation
- Find references/usages
- Symbol completion (context-aware)
- Hover information

## Pre-requisite Fix

**Tree cursor background**: dark blue bg only when tree is focused.
In `txv-widgets/src/tree_view.rs`, check `self.state.focused` before
applying the blue background. When unfocused, use no background (or
very subtle gray).

## Architecture

```
Editor ←→ LspClient ←→ LSP Server (rust-analyzer, etc.)
              ↑
         Background thread (reads stdout)
         Main thread sends requests via stdin
```

The LspClient:
- Spawns the language server process
- Communicates via JSON-RPC over stdin/stdout
- Runs a reader thread (like PtySession pattern)
- Queues responses/notifications for the editor to poll on Tick

## Iterative Implementation Plan

Each step MUST:
1. Run ALL existing tests first (regression check)
2. Implement the feature
3. Run ALL tests again (must pass)
4. Add tests for the new code
5. Run ALL tests (old + new must pass)
6. Pre-commit hook must pass
7. Commit

### Step 1: Tree cursor focus fix

- In `txv-widgets/src/tree_view.rs`: cursor bg = Ansi(4) only if `self.state.focused`
- When unfocused: cursor bg = default (no highlight) or subtle (Ansi 8 bg)
- Test: scenario test that checks cursor style changes on select/unselect

### Step 2: LspClient struct (spawn + basic JSON-RPC)

- Create `src/lsp/mod.rs` and `src/lsp/client.rs`
- LspClient: spawn process, stdin writer (background thread), stdout reader (background thread)
- JSON-RPC message framing: `Content-Length: N\r\n\r\n{json}`
- Methods: `send_request(method, params) -> id`, `send_notification(method, params)`
- Poll: `poll_messages() -> Vec<LspMessage>` (non-blocking, like PtySession)
- Test: unit test with mock process (echo server or similar)
- Dependencies: `serde_json` (add to Cargo.toml)

### Step 3: Initialize handshake

- Send `initialize` request with capabilities
- Handle `initialized` notification
- Send `textDocument/didOpen` when editor opens a file
- Send `textDocument/didChange` on edits
- Send `textDocument/didClose` on tab close
- Test: integration test that verifies initialize sequence (mock server)

### Step 4: Diagnostics

- Handle `textDocument/publishDiagnostics` notification
- Store diagnostics per file in the editor
- Render: underline error ranges in red, warnings in yellow
- Show diagnostic message in status bar when cursor is on a diagnostic line
- Test: mock diagnostic notification → verify editor stores and renders

### Step 5: Go to definition

- Command: `gd` in normal mode (vim-style) or M-x `goto-definition`
- Send `textDocument/definition` request at cursor position
- On response: open file at target location (or jump if same file)
- Test: mock response → verify cursor moves to target

### Step 6: Find references

- Command: `gr` in normal mode or M-x `find-references`
- Send `textDocument/references` request
- Show results in a Find-style tab in the bottom panel
- Navigate results with Enter
- Test: mock response → verify results displayed

### Step 7: Completion

- Trigger: in insert mode, after typing (debounced, ~300ms)
- Send `textDocument/completion` request
- Show completion popup (overlay widget)
- Navigate with Up/Down, select with Enter/Tab, dismiss with Esc
- Test: mock completion response → verify popup appears with items

### Step 8: Hover

- Command: `K` in normal mode (vim-style) or M-x `hover`
- Send `textDocument/hover` request
- Show result in a floating overlay or status bar
- Test: mock hover response → verify display

## LSP Server Configuration

In `~/.config/kairn/init.tcl`:
```tcl
# Language server configuration
lsp rust rust-analyzer
lsp typescript typescript-language-server --stdio
lsp go gopls
```

The `lsp` command registers a language → command mapping.
Editor auto-starts the appropriate server based on file extension.

## Key Design Decisions

1. **One server per language** (not per file) — shared across all editor tabs of same language
2. **Lazy start** — server spawned on first file open for that language
3. **Graceful degradation** — if server not found or crashes, editor works without LSP
4. **Non-blocking** — all LSP communication is async (background threads + polling on Tick)
5. **No external async runtime** — use threads + channels (same pattern as PtySession)

## File Structure

```
src/lsp/
  mod.rs          — LspRegistry (manages servers per language)
  client.rs       — LspClient (spawn, send, poll)
  messages.rs     — JSON-RPC types, LSP message structs
  protocol.rs     — Initialize, didOpen, didChange, etc.
  diagnostics.rs  — Diagnostic storage and rendering
  completion.rs   — Completion popup logic
```

## Dependencies to Add

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Constraints

- Pre-commit hook MUST pass at every step
- 240 code line max per file
- No unwrap/expect/panic in runtime code
- Non-blocking: never block the main thread on LSP I/O
- Graceful: missing/crashed server = no LSP features, no crash
- Tests must be deterministic (mock the LSP server, don't depend on rust-analyzer being installed)

## References

- LSP spec: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/
- `txv-widgets/src/pty_session.rs` — background thread + channel pattern
- `src/views/editor/mod.rs` — where to integrate LSP calls
- `.kiro/steering/steering.md` — project SOPs
