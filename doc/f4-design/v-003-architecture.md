# v-003 — Architecture

## Five-layer model

```
┌─────────────────────────────────────────────────────────┐
│                        kairn                            │
│  IDE application: editor, LSP, nav, Kiro, config        │
│  rusticle bridge connects interpreter to kairn commands  │
├──────────────────────────────────────────────────────────┤
│                     rusticle-tk                          │
│  TUI application framework: script-driven apps          │
│  dialog replacement, rapid prototyping, kairn plugins    │
├──────────────────────┬──────────────────────────────────┤
│     txv-widgets      │        rusticle                  │
│  Interactive TUI     │   Tcl subset interpreter         │
│  components: tree,   │   with lexical scoping,          │
│  list, input, tabs,  │   typed declarations,            │
│  dialog, event loop  │   load-time validation           │
├──────────────────────┘──────────────────────────────────┤
│                        txv                              │
│  Rendering primitives: cells, surfaces, screen diffing, │
│  layout engine, borders, text, TermBuf (VTE terminal)   │
├─────────────────────────────────────────────────────────┤
│                     crossterm                           │
│  Terminal I/O: raw mode, input events, escape sequences │
└─────────────────────────────────────────────────────────┘
```

### Dependency graph

```
kairn ──→ rusticle-tk ──→ txv-widgets ──→ txv ──→ crossterm
  │           │                            ↑        vte
  │           ├──→ rusticle                │        unicode-width
  │           │                            │
  ├──→ rusticle ───────────────────────────┘
  │
  ├──→ gix, nucleo, syntect, tree-sitter, ignore, regex
  ├──→ tokio (async: LSP, file watch, PTY I/O)
  ├──→ portable-pty
  ├──→ serde + serde_json (session persistence)
  ├──→ clap (CLI)
  └──→ anyhow + thiserror (errors)
```

Key constraints:
- **txv** has no async dependency (no tokio)
- **rusticle** has no TUI dependency (no txv, no crossterm)
- **txv-widgets** depends on txv but not on rusticle or kairn
- **rusticle-tk** depends on txv-widgets + rusticle (the integration layer)
- **kairn** depends on all four, plus external crates

### Crate purposes

| Crate | Purpose | Reusable? |
|-------|---------|-----------|
| txv | Terminal cell grid, differential rendering, layout, borders, TermBuf | Yes |
| txv-widgets | Interactive components (tree, list, input, dialog, event loop) | Yes |
| rusticle | Tcl subset interpreter with validation | Yes |
| rusticle-tk | TUI app framework (Tcl/Tk for terminals) | Yes — standalone product |
| kairn | IDE: editor, LSP, nav, Kiro, config, rusticle bridge | No — the application |

### Deliverables

| # | Deliverable | What ships | Value |
|---|-------------|-----------|-------|
| 1 | **rusticle** | Crate + REPL binary | Embeddable scripting language |
| 2 | **txv + txv-widgets** | Crates | TUI rendering + widget library |
| 3 | **rusticle-tk** | Binary + examples | TUI app framework, dialog replacement |
| 4 | **kairn** | Binary | TUI IDE |

## Workspace structure

```
kairn/                          (workspace root)
├── Cargo.toml                  (workspace manifest)
├── txv/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── cell.rs             — Cell, Color, Attrs, Span
│       ├── surface.rs          — Surface (bounded writable region)
│       ├── screen.rs           — Screen (dual-buffer, diff flush)
│       ├── layout.rs           — LayoutNode, Size, Rect computation
│       ├── termbuf.rs          — TermBuf (VTE → cells)
│       ├── text.rs             — wrapping, truncation, Unicode width
│       └── border.rs           — box drawing (pretty + copy-friendly)
├── txv-widgets/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── widget.rs           — Widget trait, EventResult
│       ├── event_loop.rs       — EventLoop, Timer, Poller
│       ├── tree_view.rs        — TreeView<T: TreeData>
│       ├── list_view.rs        — ListView<T: ListData>
│       ├── input_line.rs       — InputLine (text input + history)
│       ├── tab_bar.rs          — TabBar
│       ├── dialog.rs           — Dialog (modal confirmation/prompt)
│       ├── notification.rs     — flash messages
│       ├── overlay.rs          — positioned popup container
│       ├── fuzzy_select.rs     — FuzzySelect (input + filtered list)
│       ├── status_bar.rs       — StatusBar (left/right spans)
│       └── scroll_view.rs      — ScrollView (virtual content + scroll)
├── rusticle/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── interpreter.rs      — Interpreter (eval, vars, commands)
│       ├── parser.rs           — command parser, %{} %[] literals, accessor syntax
│       ├── value.rs            — TclValue (dual representation)
│       ├── builtins.rs         — set, proc, if, foreach, while, expr, etc.
│       ├── context.rs          — context blocks, typed declarations
│       ├── types.rs            — TypeDecl, type checking, type inference
│       ├── validate.rs         — load-time validation pass
│       ├── manifest.rs         — command manifest for external commands
│       └── error.rs            — TclError, return codes
├── rusticle-tk/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             — CLI, script loading, REPL
│       ├── tk_bridge.rs        — registers widget commands in rusticle
│       ├── widget_mgr.rs       — widget ID registry, lifecycle
│       ├── layout_mgr.rs       — window/layout commands
│       └── event_mgr.rs        — bind/after/on-* event wiring
└── src/                        (kairn binary)
    ├── main.rs
    ├── app.rs                  — App (implements Widget)
    ├── tcl_bridge.rs           — registers kairn commands in Tcl
    ├── buffer/
    │   ├── mod.rs              — PieceTable, Buffer trait
    │   ├── piece_table.rs
    │   └── undo.rs
    ├── editor/
    │   ├── mod.rs              — Editor (buffer + cursor + mode)
    │   ├── command.rs          — Command enum
    │   ├── keymap_vim.rs
    │   ├── keymap_emacs.rs
    │   ├── keymap_classic.rs
    │   ├── ex.rs               — ex-command parser
    │   └── save.rs             — atomic file save
    ├── panel/
    │   ├── mod.rs
    │   ├── editor_panel.rs
    │   ├── tree_panel.rs
    │   ├── control_panel.rs
    │   ├── bottom_panel.rs
    │   └── terminal_panel.rs
    ├── lsp/
    │   ├── mod.rs
    │   ├── protocol.rs
    │   └── capabilities.rs
    ├── nav/                    — import navigation (port)
    ├── search/                 — fuzzy file search (port)
    ├── content_search/         — workspace grep (port)
    ├── highlight/              — tree-sitter + syntect
    ├── git/                    — gix operations (port)
    ├── session/                — session persistence (port)
    └── config/                 — Tcl-based config loading
```

## What ports from existing code

| Module | Source | Destination | Changes |
|--------|--------|-------------|---------|
| TermBuf + vte | master src/termbuf.rs | txv/src/termbuf.rs | Render to Surface instead of ratatui |
| Git operations | master src/diff/ | kairn src/git/ | None — pure data |
| Fuzzy search (nucleo) | master src/search/ | kairn src/search/ | UI renders to Surface |
| Import navigation | feature/mini-ide src/nav/ | kairn src/nav/ | None — pure data |
| Content search | feature/mini-ide src/content_search/ | kairn src/content_search/ | None — pure data |
| Session persistence | master src/session/ | kairn src/session/ | Extend schema |
| CLI parsing | master src/cli.rs | kairn src/main.rs | None |
| Vi command logic | feature/mini-ide src/editor/vi.rs | kairn src/editor/keymap_vim.rs | Rewrite against PieceTable + Command enum |
| Ex-command parser | feature/mini-ide src/editor/ex.rs | kairn src/editor/ex.rs | Port as-is (pure parsing) |
| Atomic file save | feature/mini-ide src/editor/save.rs | kairn src/editor/save.rs | Port as-is |

Detailed specs for each crate follow in v-006 (txv), v-007 (txv-widgets),
and v-008 (tcl).
