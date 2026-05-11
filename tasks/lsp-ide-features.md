# Task: LSP Wiring + IDE Features (IntelliJ-class)

## Overview

Complete the LSP integration by adding the runtime glue (server registry,
lifecycle management) and expand to full IDE functionality: build, run,
test, refactoring, workspace configuration.

## Part 1: LSP Server Registry + Lifecycle

### 1.1 Default Server Mappings

Create `src/lsp/registry.rs`:

```rust
pub struct LspRegistry {
    servers: HashMap<String, ServerConfig>,  // language_id → config
    active: HashMap<String, LspClient>,      // language_id → running client
}

pub struct ServerConfig {
    pub command: String,
    pub args: Vec<String>,
}
```

Hardcoded defaults (user can override via init.tcl):

| Language | Command | Args |
|----------|---------|------|
| rust | rust-analyzer | |
| go | gopls | |
| typescript | typescript-language-server | --stdio |
| javascript | typescript-language-server | --stdio |
| c | clangd | |
| cpp | clangd | |
| java | jdtls | |
| python | pyright-langserver | --stdio |

### 1.2 Lifecycle Management

- `registry.get_or_start(language_id, root_dir) -> Option<&mut LspClient>`
- Lazy start: server spawned on first file open for that language
- If spawn fails: log error, return None (graceful degradation)
- On editor close (last file of that language): optionally shut down server
- On quit: send `shutdown` + `exit` to all active servers

### 1.3 Wire into Handler

In `src/handler.rs`:
- On CM_OPEN_FILE: call `registry.get_or_start(lang)`, send `didOpen`
- On CM_LSP_GOTO_DEF: send `textDocument/definition`, handle response (open file at location)
- On CM_LSP_FIND_REFS: send `textDocument/references`, show in Find tab
- On CM_LSP_HOVER: send `textDocument/hover`, show in status bar or overlay
- On Tick: poll all active clients for notifications (diagnostics, etc.)

### 1.4 Tcl API for Configuration

Register rusticle commands in `src/config.rs` or a new `src/lsp/config_commands.rs`:

```tcl
# Override or add language server
lsp-server rust rust-analyzer
lsp-server typescript typescript-language-server --stdio
lsp-server java jdtls -data /tmp/jdtls-workspace

# Disable LSP for a language
lsp-disable python

# Check status
lsp-status  ;# shows running servers
```

## Part 2: Build / Run / Test Integration

### 2.1 Build Command

- `M-x build` or configurable key
- Runs configured build command (default: `cargo build` for Rust, `go build` for Go, etc.)
- Output in a `Compile:` tab (singleton, replaces previous)
- Parse errors: extract file:line:col from output (regex per language)
- Navigate errors: `M-x next-error` / `M-x prev-error` jumps to location

### 2.2 Run Command

- `M-x run` — runs configured run command in a Shell tab
- Default: `cargo run` for Rust, `go run .` for Go, etc.
- Configurable via tcl: `set run-command "cargo run --release"`

### 2.3 Test Command

- `M-x test` — runs test suite
- `M-x test-file` — runs tests for current file
- `M-x test-at-cursor` — runs test under cursor (if detectable)
- Output in Compile tab with error navigation
- Default: `cargo test` for Rust, `go test ./...` for Go

### 2.4 Tcl Configuration

```tcl
# Build/run/test commands (per workspace)
set build-command "cargo build"
set run-command "cargo run"
set test-command "cargo test"
set test-file-command "cargo test --lib {file}"
```

## Part 3: Refactoring

### 3.1 Rename Symbol

- `M-x rename` (when in editor, not tool tab) or LSP-aware rename
- Send `textDocument/rename` request
- Apply workspace edit (may touch multiple files)
- Show summary: "Renamed X in N files"

### 3.2 Code Actions

- `M-x code-action` — request available actions at cursor
- Show in dropdown/menu
- Apply selected action (workspace edit)

## Part 4: Workspace Configuration

### 4.1 Workspace Tcl File

- Location: `.kairn/workspace.tcl` in project root
- Auto-loaded after `~/.config/kairn/init.tcl`
- Contains project-specific settings (build commands, LSP overrides, etc.)

### 4.2 Save Workspace Config

- `M-x save-workspace` — writes current settings to `.kairn/workspace.tcl`
- Preserves existing user commands in the file (parse, update known vars, keep rest)
- Format:
```tcl
# kairn workspace configuration
set build-command "cargo build"
set run-command "cargo run"
set test-command "cargo test"
lsp-server rust rust-analyzer

# User commands below this line are preserved
proc my-helper {} { ... }
```

### 4.3 M-x Configuration Commands

```
M-x set build-command <cmd>
M-x set run-command <cmd>
M-x set test-command <cmd>
M-x lsp-server <lang> <cmd> [args...]
M-x save-workspace
```

## Part 5: Completion Enhancement

### 5.1 Trigger

- In insert mode, after typing a word char or `.` or `::`
- Debounce: 300ms after last keystroke
- Send `textDocument/completion`

### 5.2 Popup

- Overlay widget showing completion items
- Up/Down to navigate, Enter/Tab to accept, Esc to dismiss
- Show item kind (function, variable, type, etc.) with icons/colors
- Filter as user types more characters

## Implementation Order (iterative, each step commits)

1. Create `src/lsp/registry.rs` with defaults + get_or_start
2. Wire registry into handler (didOpen on file open, poll on Tick)
3. Handle CM_LSP_GOTO_DEF response (open file at location)
4. Handle CM_LSP_FIND_REFS response (show in Find tab)
5. Handle CM_LSP_HOVER response (show in status/overlay)
6. Add completion trigger in insert mode (debounced)
7. Add Tcl commands: `lsp-server`, `lsp-disable`, `lsp-status`
8. Add build command (`M-x build`) with Compile tab + error parsing
9. Add run command (`M-x run`)
10. Add test commands (`M-x test`, `test-file`, `test-at-cursor`)
11. Add error navigation (`M-x next-error`, `M-x prev-error`)
12. Add rename symbol (`M-x rename` via LSP)
13. Add code actions (`M-x code-action`)
14. Add workspace.tcl loading
15. Add `M-x save-workspace`
16. Update help text with all new commands

## Part 6: Messages Window Integration

The Messages view already exists (`src/views/messages.rs`, F6 to show).
Wire it into all discovery/lifecycle events:

- LSP server started: `[info] rust-analyzer started for /path/to/project`
- LSP server failed to start: `[error] clangd not found — install with: brew install llvm`
- LSP server crashed: `[error] rust-analyzer exited unexpectedly`
- Build started: `[info] Running: cargo build`
- Build succeeded: `[info] Build succeeded (0 errors, 2 warnings)`
- Build failed: `[error] Build failed (3 errors)`
- Tool discovery: `[info] Found rust-analyzer at /usr/local/bin/rust-analyzer`
- Tool not found: `[warn] gopls not found — Go LSP disabled`

Use `CM_STATUS_MESSAGE` to push messages, or add a `CM_LOG_MESSAGE` command
that the handler routes to the Messages view.

### Discovery on Startup

On first file open for a language, before spawning:
1. Check if the server command exists (`which <cmd>`)
2. If found: log info + start
3. If not found: log warning with install hint, disable LSP for that language

## Constraints

- Pre-commit hook MUST pass at every step
- 240 code line max per file
- No unwrap/expect/panic in runtime code
- Non-blocking: never block main thread on LSP/build I/O
- Graceful: missing server/tool = feature disabled, no crash
- Tests must be deterministic (mock LSP server for unit tests)
- Each step: regression test → implement → test → add tests → commit

## Dependencies

Already added by previous LSP work:
- `serde` + `serde_json` for JSON-RPC

May need:
- `regex` (already in deps) for error parsing

## Key Files

- `src/lsp/registry.rs` — NEW: server registry + lifecycle
- `src/lsp/client.rs` — existing: spawn + JSON-RPC
- `src/lsp/protocol.rs` — existing: message builders
- `src/handler.rs` — wire LSP commands
- `src/config.rs` — add Tcl commands for LSP config
- `src/commands.rs` — add CM_BUILD, CM_RUN, CM_TEST, etc.
- `src/views/editor/mod.rs` — completion trigger, diagnostics display
