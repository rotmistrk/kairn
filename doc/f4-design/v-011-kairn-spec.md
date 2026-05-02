# v-011 — kairn Spec: TUI IDE

## Overview

kairn is a TUI IDE built on rusticle (scripting), txv (rendering), and
txv-widgets (interactive components). It adds: a piece table editor with
three keyboard layouts, a multi-panel workspace, LSP integration,
tree-sitter highlighting, git operations, Kiro AI integration, and
build/test runners.

## Sub-specs

This spec is the top-level overview. Detailed specs for each subsystem:

| Sub-spec | Scope | Can be built independently? |
|----------|-------|---------------------------|
| [v-011.01](v-011.01-piece-table.md) | Piece table buffer, line index, undo | Yes — pure data structure |
| [v-011.02](v-011.02-editor-commands.md) | Command enum, keymap trait, vim/emacs/classic | Yes — depends on 011.01 types only |
| [v-011.03](v-011.03-panel-system.md) | Panel composition, App structure, layout | Yes — depends on txv-widgets |
| [v-011.04](v-011.04-config-and-bridge.md) | Rusticle config, bridge commands, manifest | Yes — depends on rusticle |
| [v-011.05](v-011.05-lsp.md) | LSP client, server lifecycle, document sync | Yes — depends on 011.01 for sync |
| [v-011.06](v-011.06-ports.md) | Port instructions for each existing module | Reference doc, not buildable |

## Dependencies between sub-specs

```
011.01 (piece table)  ──→ 011.02 (commands/keymaps)
                      ──→ 011.05 (LSP document sync)

011.03 (panels)       ──→ uses txv-widgets directly
                      ──→ 011.02 (editor panel needs commands)

011.04 (config)       ──→ independent (rusticle only)

011.06 (ports)        ──→ reference for all phases
```

## Build phases

### Phase A: Foundation (can run 2 agents in parallel)

**Agent 1**: Piece table + editor commands (011.01 + 011.02)
- `src/buffer/piece_table.rs` — PieceTable implementation
- `src/buffer/line_index.rs` — line number ↔ byte offset mapping
- `src/buffer/undo.rs` — undo/redo via piece table snapshots
- `src/editor/command.rs` — Command enum
- `src/editor/keymap.rs` — Keymap trait
- `src/editor/keymap_vim.rs` — vim keymap (port from feature/mini-ide)
- `src/editor/ex.rs` — ex-command parser (port)
- `src/editor/save.rs` — atomic file save (port)
- `src/editor/mod.rs` — Editor struct (buffer + cursor + mode)

**Agent 2**: Config + ports (011.04 + 011.06)
- `src/config/mod.rs` — Tcl-based config loading
- `src/rusticle_bridge.rs` — register kairn commands in rusticle
- Port: git/, nav/, search/, content_search/, session/

### Phase B: Panel system (depends on Phase A)

Single agent: 011.03
- `src/app.rs` — App struct, event loop integration
- `src/panel/editor_panel.rs` — editor triptych
- `src/panel/tree_panel.rs` — file/git/package/symbol tree
- `src/panel/control_panel.rs` — outline, blame, diagnostics
- `src/panel/bottom_panel.rs` — tabbed bottom area
- `src/panel/terminal_panel.rs` — PTY terminal
- `src/main.rs` — CLI, startup, terminal setup

### Phase C: Intelligence (can run 2 agents in parallel)

**Agent 1**: LSP (011.05)
- `src/lsp/mod.rs` — LSP client
- `src/lsp/protocol.rs` — JSON-RPC messages
- `src/lsp/capabilities.rs` — completion, diagnostics, definition

**Agent 2**: Remaining keymaps + tree-sitter
- `src/editor/keymap_emacs.rs`
- `src/editor/keymap_classic.rs`
- `src/highlight/mod.rs` — tree-sitter + syntect

### Phase D: Integration

Single agent: Kiro integration, build/test runners, polish
- Kiro panel (PTY + diff detection)
- Build runners (cargo, maven, go, npm)
- Error parsing and navigation
- Test runner with results tree
- Autosave
- Session persistence updates
- Embedded default configs

## Module map

```
src/
├── main.rs                 — CLI, terminal setup, panic handler
├── app.rs                  — App struct, event loop, focus management
├── rusticle_bridge.rs      — registers kairn commands in rusticle
│
├── buffer/
│   ├── mod.rs              — Buffer trait, re-exports
│   ├── piece_table.rs      — PieceTable implementation
│   ├── line_index.rs       — line ↔ byte offset mapping
│   └── undo.rs             — undo/redo history
│
├── editor/
│   ├── mod.rs              — Editor struct (buffer + cursor + mode)
│   ├── command.rs          — Command enum (all operations)
│   ├── keymap.rs           — Keymap trait
│   ├── keymap_vim.rs       — vim modal keymap
│   ├── keymap_emacs.rs     — emacs chord keymap
│   ├── keymap_classic.rs   — classic menu-driven keymap
│   ├── ex.rs               — ex-command parser and execution
│   └── save.rs             — atomic file save
│
├── panel/
│   ├── mod.rs              — panel types, focus enum
│   ├── editor_panel.rs     — triptych: tree + editor + control
│   ├── tree_panel.rs       — file/git/package/symbol modes
│   ├── control_panel.rs    — outline, blame, diagnostics
│   ├── bottom_panel.rs     — tabbed: terminal, errors, search, tests, kiro
│   └── terminal_panel.rs   — PTY terminal using txv::TermBuf
│
├── lsp/
│   ├── mod.rs              — client, server registry
│   ├── protocol.rs         — JSON-RPC types, message framing
│   └── capabilities.rs     — completion, diagnostics, definition, references
│
├── config/
│   └── mod.rs              — load .kairnrc.tcl, embedded defaults, --init-config
│
├── highlight/
│   └── mod.rs              — tree-sitter + syntect fallback
│
├── git/
│   └── mod.rs              — gix operations (port from master)
│
├── nav/
│   ├── mod.rs              — ImportIndex, LanguageNav trait (port)
│   ├── java.rs, go.rs, rust_nav.rs, ts.rs
│
├── search/
│   └── mod.rs              — fuzzy file search via nucleo (port)
│
├── content_search/
│   └── mod.rs              — workspace grep (port)
│
└── session/
    └── mod.rs              — session persistence (port + extend)
```

## Dependencies (Cargo.toml)

```toml
[dependencies]
# Workspace crates
txv = { path = "../txv" }
txv-widgets = { path = "../txv-widgets" }
rusticle = { path = "../rusticle" }

# Terminal
crossterm = "0.28"

# Terminal emulation (for PTY panels)
vte = "0.13"
portable-pty = "0.8"

# Git
gix = { version = "0.68", default-features = false, features = ["basic", "extras"] }

# Fuzzy search
nucleo = "0.5"

# Syntax highlighting (fallback)
syntect = "5"

# File traversal
ignore = "0.4"

# Diff
similar = "2"

# Async (LSP, file watching)
tokio = { version = "1", features = ["full"] }

# Serialization (sessions)
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# CLI
clap = { version = "4", features = ["derive"] }

# Errors
anyhow = "1"
thiserror = "2"

# Regex
regex = "1"

# Unix (FIFO, signals)
nix = { version = "0.29", features = ["fs"] }
libc = "0.2"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Verification strategy

Each phase has its own verification:

- **Phase A**: `cargo test` on buffer/ and editor/ modules. Can edit a
  file in memory, undo/redo, vim keybindings produce correct commands.
- **Phase B**: App launches, panels render, focus switching works, file
  tree navigates, editor displays file content.
- **Phase C**: LSP connects to rust-analyzer, completion popup works,
  diagnostics appear. Emacs/classic keymaps functional.
- **Phase D**: Full integration — edit, build, test, navigate errors,
  send to Kiro, apply diffs.

## What NOT to build (deferred)

- Mouse support
- Multiple cursors
- Dot-repeat (vim `.`)
- C/C++ language support
- Plugin distribution system
- Rusticlish shell integration
