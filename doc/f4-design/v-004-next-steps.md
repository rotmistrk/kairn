# v-004 — Next Steps

## Process

Five steps before writing application code:

1. **Brainstorm Q&A** — explore design questions, weigh alternatives
2. **Align and iron out** — resolve contradictions, update docs
3. **Form design and instrumentation** — finalize specs, test criteria, dev tooling
4. **Describe implementation approach** — phase ordering, agent strategy
5. **Form plans/ directory** — epic/story/task hierarchy for execution

## Build order

Five crates, five deliverables. rusticle and txv have zero mutual
dependency and can be developed in parallel.

```
Deliverable 1          Deliverable 2
┌──────────┐          ┌──────────────┐
│ rusticle │          │ txv          │
│ (interp) │          │ (rendering)  │
└────┬─────┘          └──────┬───────┘
     │                       │
     │                ┌──────┴───────┐
     │                │ txv-widgets  │
     │                │ (components) │
     │                └──────┬───────┘
     │                       │
     └───────┬───────────────┘
             │
      ┌──────┴───────┐     Deliverable 3
      │ rusticle-tk  │
      │ (TUI framework)│
      └──────┬───────┘
             │
      ┌──────┴───────┐     Deliverable 4
      │    kairn     │
      │   (IDE)      │
      └──────┬───────┘
             │
      ┌──────┴───────┐     Deliverable 5
      │ rusticlish   │
      │  (shell)     │
      └──────────────┘
```

### Phase 0a: rusticle (Deliverable 1)

See v-008-tcl-spec.md. No TUI dependency.

1. `error.rs` — TclError, ErrorCode, Location
2. `value.rs` — TclValue with dual representation
3. `parser.rs` — command parsing, `%{}` `%[]` literals, accessor syntax
4. `interpreter.rs` — eval loop, scope chain, command dispatch
5. `builtins.rs` — set, proc, if, foreach, while, expr, list, string, dict
6. `context.rs` — context blocks, typed declarations
7. `types.rs` — type declarations, checking, inference
8. `manifest.rs` — command manifest loading
9. `validate.rs` — load-time validation pass

**Ships as**: crate on crates.io + `rusticle` REPL binary.

**Validation**: can load config files, define procs, run control flow,
validate types, catch errors at load time. REPL works interactively.

### Phase 0b: txv (part of Deliverable 2)

See v-006-txv-spec.md. Can run in parallel with Phase 0a.

1. `cell.rs` — Cell, Color, Attrs, Style, Span
2. `text.rs` — display_width, truncate, wrap, byte↔col
3. `surface.rs` — Surface with clipping, wide char handling
4. `screen.rs` — Screen with dual-buffer diff flush
5. `layout.rs` — Rect::split with constraints
6. `border.rs` — pretty + copy-friendly box drawing
7. `termbuf.rs` — VTE-driven virtual terminal (port from master)

**Validation**: demo program that draws panels, handles resize, renders
styled text and an embedded terminal — no rendering artifacts.

### Phase 1a: txv-widgets (completes Deliverable 2)

See v-007-txv-widgets-spec.md. Depends on txv.

1. `widget.rs` — Widget trait, EventResult
2. `event_loop.rs` — EventLoop with timers and pollers
3. `scroll_view.rs` — ScrollView helper
4. `status_bar.rs` — StatusBar
5. `input_line.rs` — InputLine with history
6. `tab_bar.rs` — TabBar
7. `list_view.rs` — ListView
8. `tree_view.rs` — TreeView
9. Additional: Dialog, Notification, Overlay, FuzzySelect, ProgressBar,
   CheckList, RadioList, Table, Menu, Splitter, FileSelect

**Ships as**: txv + txv-widgets crates on crates.io.

**Validation**: demo app with tree, list, input, tabs, status bar.

### Phase 1b: rusticle-tk (Deliverable 3)

See v-009-rusticle-tk-spec.md. Depends on rusticle + txv-widgets.

1. `widget_mgr.rs` — widget ID registry
2. `tk_bridge.rs` — register widget commands in rusticle
3. `layout_mgr.rs` — window/frame/layout commands
4. `event_mgr.rs` — bind, after, on-* event wiring
5. `main.rs` — CLI, script loading, REPL
6. Additional widget commands as needed

**Ships as**: `rusticle-tk` binary + example scripts.

**Validation**: yazi-style file manager script works. Dialog one-liners
work from shell. Log viewer example works.

### Phase 2+: kairn (Deliverable 4)

Depends on all four crates.

1. Piece table buffer
2. Command enum + vim/emacs/classic keymaps
3. Ex-command engine (port)
4. Panel system (editor, tree, control, bottom)
5. Rusticle bridge (kairn commands in interpreter)
6. Config loading from `.kairnrc.tcl`
7. LSP client
8. Tree-sitter integration
9. Build/test runners
10. Kiro integration

**Ships as**: `kairn` binary.

### Phase 3: rusticlish (Deliverable 5)

After kairn is usable. A shell built on rusticle with structured data pipes.

1. Readline (InputLine + history file)
2. PATH lookup + command execution
3. Tab completion framework (files, commands, git)
4. Process pipes (value pipes vs process pipes)
5. Rich output (inline tables, colors via rusticle-tk)
6. Job control (bg/fg, Ctrl-Z, process groups)
7. Prompt customization, globbing, aliases, startup files

**Ships as**: `rusticlish` binary. Default shell for kairn's terminal panel.

## Immediate next action

Start **Phase 0a (rusticle)** — the interpreter has zero external
dependencies and is the foundation for configuration, scripting, and
rusticle-tk. Can be developed and tested purely with unit tests.

In parallel (or immediately after): **Phase 0b (txv)**.

All design is captured in v-001 through v-009.
