# kairn — Agent Steering Document

## CRITICAL: No External Tools

kairn must NEVER depend on external CLI tools for core functionality. Use pure Rust crates.

Forbidden: `rg`, `grep`, `git` CLI, `find`, `pbcopy`, etc.
Use instead: `ignore` (file walking), `regex` (search), `git2` (git), `syntect` (highlighting).

---

## CRITICAL: No Silent Errors

Every failure MUST either show in status bar + message ring (user-facing) or log at
WARN/ERROR (background). Never `let _ = fallible_op();` without logging.

---

## GRITS: Feature Implementation Cycle

Every feature/fix follows this cycle iteratively:

1. **G**reen — start from passing build + tests
2. **R**egression — confirm existing tests still pass
3. **I**mplementation with new tests — code AND tests together
4. **T**ests **S**ucceed — all tests (old + new) pass before done

---

## CRITICAL: Pre-Commit Hook

The pre-commit hook enforces ALL of the following. Code MUST pass before commit:

1. `cargo fmt --all -- --check` (rustfmt with project's rustfmt.toml)
2. `cargo clippy --workspace -- -D warnings` (zero warnings)
3. **240 CODE lines per file maximum** (blank/comment lines don't count)
4. `cargo test --workspace --no-fail-fast` (all tests pass)

**NEVER use `--no-verify` without explicit user permission.**

---

## CRITICAL: Build Gate — All Tests Must Pass

**`make install-local` WILL NOT proceed unless ALL tests pass.**

The Makefile `test` target enforces:
- All tests must **pass** (non-zero exit from `cargo test` = build failure)
- **Zero ignored/skipped tests allowed** — any `ignored` count in test output = build failure

A skipped test is a broken build. If a test cannot run, fix it or delete it.
Never mark tests `#[ignore]` to work around failures.

---

## Test Reliability — CRITICAL

- **Every test MUST be deterministic.** A flaky test is a SEVERE RISK.
- **Every test MUST run independently** — no shared state, no ordering dependencies,
  safe to run in parallel with all other tests.
- **ALL workspace tests must pass** at every step, not just the component being worked on.
- **Use `content_contains()` for buffer content assertions** — `contains()` includes
  the status bar where the clock widget can produce false positives (e.g., "11:29"
  matching `contains("11")`).
- **"Likely" is not acceptable.** Every claim about behavior MUST be supported with data
  (test output, grep results, code inspection). Do not speculate.

---

## 240 Code Lines Rule — SOP

**"Code lines" = total lines minus blank lines and comment-only lines.**
Comments (`//`, `//!`, `///`, `/*`, `*/`, `*`) do NOT count. The pre-commit hook enforces this.

**CRITICAL: The response to exceeding 240 lines is ALWAYS conceptual splitting.
NEVER reduce code or documentation quality to fit the limit.**

When a file exceeds 240 code lines after formatting:

**DO:**
- Split the file CONCEPTUALLY into two or more files
- Each new file gets a clear, nameable single responsibility
- Use `mod.rs` to re-export if needed for API stability

**DO NOT:**
- Collapse code (shorter names, cramming onto one line)
- Remove comments or documentation
- Use macros just to reduce line count
- Fight the formatter

**The test:** If you can't name the new file with a clear noun/verb phrase
describing its single responsibility, you're splitting wrong.

**Examples:**
- `editor/execute.rs` → `editor/dispatch_movement.rs` + `editor/dispatch_editing.rs`
- `desktop/mod.rs` → `desktop/tabs.rs` + `desktop/focus.rs`
- `status_items.rs` → `status_items/key_label.rs` + `status_items/clock.rs`

---

## Architecture: TXV Framework

kairn is built on TXV — a Turbo Vision-inspired TUI framework split into crates:

```
txv-core        Pure logic. Zero I/O. Defines View trait, Group dispatch, geometry.
txv-render      Terminal I/O (crossterm). Implements Backend trait.
txv-widgets     Concrete Views (TextArea, StatusBar, PtyTerminal, etc.)
kairn           Application: SlottedDesktop, Editor, Tree, Handler.
```

### Key Design: High Cohesion via Composition + Delegation Macros

The framework solves the **agentic code generation problem**: AI agents produce
code with low cohesion (god objects) and high coupling (direct dependencies).
TXV forces high cohesion through:

1. **Composition over inheritance** — ViewState/GroupState/WindowState/DialogState
   are composed into views, not inherited.

2. **Delegation macros** eliminate boilerplate while keeping each view focused:
   - `delegate_view_state!(field)` — implements View trait methods via field
   - `delegate_view_state!(field, override { title, needs_redraw })` — skip specific methods
   - `delegate_group_state!(field)` — Group dispatch delegation
   - `delegate_window_state!(field)` — Window (border + title) delegation
   - `delegate_dialog_state!(field)` — Dialog (modal + buttons) delegation

3. **Three-phase event dispatch** (Group):
   - Phase 1: Preprocess — views with `preprocess: true` see events first (StatusBar)
   - Phase 2: Focused child handles the event
   - Phase 3: Postprocess — views with `postprocess: true` see events last

4. **EventQueue (putEvent model)** — views emit commands via `queue.put_command(id, data)`,
   never call each other directly. This decouples all views.

5. **exec_view for modals** — nested event loop where keys go to modal only,
   but Tick/Resize/Command still reach the full tree.

### DRY Principle in Practice

- **Search before creating**: Check existing widgets/traits before writing new ones
- **One interface = one concept**: View, Group, Window, Dialog are composable
- **Reuse delegation macros**: Every view uses `delegate_view_state!` — never
  hand-implement bounds/select/unselect/dirty
- **Command IDs are the API**: Views communicate through commands, not method calls

### File Organization

```
src/
  buffer/           PieceTable text buffer + undo
  editor/           Vi editor (keymap, commands, motions, visual, search, ex)
  layout_group/     SlottedDesktop (layout, chrome, tabs, dropdown)
  lsp/              LSP client, registry, diagnostics, handler, response, completion
  mcp/              MCP server (snapshot, tools, commands, listener)
  scripting/        Tcl engine (ScriptEngine + bridge_* modules)
  session/          Session persistence (save/restore workspace)
  structured/       Structured document models (JSON, JSONL, JSONC)
  views/            Concrete views:
    editor/           Editor view (draw, handle, completion, diff, build)
    csv_view/         CSV/TSV table view
    struct_view/      Structured (JSON tree) view
    todo_tree/        Todo panel
    git_changes/      Git changes panel
    tree.rs           File tree
    terminal.rs       PTY terminal
    results.rs        Grep/build results
    messages.rs       Message ring viewer
    help.rs / welcome.rs
  handler.rs        Command handler (wires commands to actions)
  handler_*.rs      Split handlers (open, exec, build, git, confirm, context, drain, evict)
  commands.rs       Command ID constants + ViewContext struct
  config.rs         Configuration loading (Tcl-based)
  app_state.rs      Application state struct
  app_palette.rs    Color palette definitions
  config_colors.rs  Color config parsing
  config_keys.rs    Key binding config parsing
  init.rs           --init-home / --init-wp CLI commands
  status.rs         Status bar setup
  status_items.rs   Context-aware status bar items
  build.rs          Async build/test execution
  build_detect.rs   Auto-detect build system
  build_parse.rs    Parse build output for errors
  grep.rs           Async project-wide search
  csv_parse.rs      CSV/TSV parsing
  diff.rs           File diff computation
  highlight.rs      Syntax highlighting (syntect)
  glyphs.rs         Glyph style (ascii/utf/nerd)
  git_watcher.rs    Background git status polling
  git_ops.rs        Git operations (stage, unstage, commit)
  git_status.rs     Git status model
  clipboard.rs      System clipboard access
  completer.rs      M-x command/path completion
  kiro_registry.rs  Track active Kiro sessions
  message_ring.rs   Message history ring buffer
  suspend.rs        Ctrl-Z suspend/resume
  eviction.rs       Tab eviction policy
  main.rs           Entry point
txv-core/src/       Framework core (View, Group, Surface, Event, Program)
txv-render/src/     Terminal backend (crossterm, TermBuf, diff flush)
txv-widgets/src/    Reusable widgets (TextArea, StatusBar items, PtyTerminal)
rusticle/src/       Tcl interpreter (pure Rust)
tests/              Integration/scenario tests (one concern per file)
```

### Coding Conventions

- `rustfmt.toml`: `max_width = 120`, `single_line_if_else_max_width = 0`
- No `unwrap()`/`expect()`/`panic!()` in runtime code
- Tests use `TestHarness` from `tests/helpers.rs` (mock backend, inject keys)
- Each test file covers ONE feature/scenario
- Commit messages: imperative mood, body explains WHY

---


---

## Panic Safety — Three Layers of Defense

### Layer 1: Prevention (compile-time)
- NO `unwrap()`, `expect()`, `panic!()` in runtime code (clippy enforces)
- All kairn/txv/rusticle code MUST be safe — propagate errors via Result

### Layer 2: Component Isolation (runtime)
- Every external call (PTY spawn, file I/O, syntect, git, shell commands)
  MUST be wrapped in error handling that:
  1. Logs the error
  2. Attempts graceful recovery (fallback view, empty result, etc.)
  3. Continues operation — never crashes the app

### Layer 3: Global Catch-All (last resort)
- `std::panic::set_hook` in main.rs
- On panic: restore terminal (leave alternate screen, show cursor, disable raw mode)
- Save workspace state if possible
- Print panic info to stderr
- Exit with non-zero code

## Reference Documents

- `CONVENTIONS.md` — Patterns and rules (error handling, confirmations, drain pattern)
- `doc/f4-design/v-013-txv-architecture.md` — Definitive TXV design
- `doc/f4-design/STATUS.md` — Feature status table + development cycle SOP
- `hooks/pre-commit` — The pre-commit hook source
- `doc/example-init.tcl` — Reference config (update when adding settings/keys/colors)
