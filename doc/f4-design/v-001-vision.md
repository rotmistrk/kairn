# v-001 — Vision: kairn as a TUI IDE

## Origin

kairn started as a TUI code viewer oriented around Kiro AI — three panels
(file tree, syntax-highlighted viewer, terminal with kiro-cli/shell tabs),
built in a weekend sprint of 37 commits. The `feature/mini-ide` branch on
toolbox added a vi editor (~5K lines), content search, import-based navigation
for Java/Go/Rust/TypeScript, and a full ex-command engine — 265 tests, all
green.

The redesign draws inspiration from two earlier projects:

- **F4** — a DOS/OS/2 multi-window text editor (C++, 1993–1994) with
  configurable menus, keyboard bindings, macros, and a shareware distribution
  model. The interaction design was built by a programmer for programmers.

- **TXV** (TeXt Views) — a C++ text-mode windowing/UI library (1993–1995)
  providing a Turbo Vision-style class hierarchy: views, windows, dialogs,
  menus, buttons, status bars, help system. Supported Watcom and Borland
  compilers on DOS and OS/2.

The key insight: the *design patterns* from F4 and TXV — event-driven
architecture, hierarchical views, command dispatch, direct screen buffer
rendering — are proven and map well to Rust. The platform code is obsolete,
but the interaction model is timeless.

## What kairn becomes

A modern Rust TUI IDE with:

- **Piece table buffer** for efficient editing, undo/redo, and large file support
- **Virtual screen buffer** (TXV-style) replacing ratatui for reliable,
  garbage-free rendering with differential terminal output
- **Three keyboard layouts** — vim, emacs, and classic (F4-style) — switchable
  at runtime, all first-class
- **Per-window triptych layout**: tree panel (left) + editor (center) +
  control panel (right), with a shared bottom panel
- **Tcl scripting** for configuration, keybindings, custom commands, and
  terminal automation (expect-style)
- **LSP integration** per file (Java, Go, Rust, TypeScript, HTML, CSS, Markdown)
- **Tree-sitter** for semantic highlighting and symbol outlines
- **Kiro AI integration** via PTY — interactive and report modes, send-to-kiro
  from editor, diff-apply from responses
- **Single binary** with all default configs embedded, template-based config
  generation

## Project concept

kairn operates on a **project root** detected by `.git`, `Cargo.toml`,
`gradlew`, `Makefile`, or similar markers. But it does not require a project —
you can run it in `$HOME` and edit `.zshrc`. The project root determines:

- File tree root
- Search scope
- LSP workspace
- Terminal cwd
- Import navigation roots

## Language support

Core languages (built-in support from day one):

| Language | LSP server | Build tool |
|----------|-----------|------------|
| Java | jdtls | Maven/Gradle |
| Go | gopls | go build/test |
| Rust | rust-analyzer | cargo |
| TypeScript | tsserver | npm/yarn |
| HTML/CSS | vscode-html/css | — |
| Markdown | — | — |

C/C++ is desirable but complex (clangd, CMake/Make/Bazel) — deferred.

Special handling:
- **Makefile**: literal tab characters preserved and displayed correctly
- **Java**: package-based navigation, source root detection, classpath resolution

## Layout model

```
┌─────────────────────────────────────────────────────────┐
│ Tab bar / Window title                                  │
├──────┬──────────────────────────────┬───────────────────┤
│      │                              │                   │
│ Tree │       Editor (main)          │  Control panel    │
│      │                              │  (layout-dep.)    │
│      │                              │                   │
│      │                              │                   │
├──────┴──────────────────────────────┴───────────────────┤
│ Bottom panel (terminal / errors / search / tests / AI)  │
├─────────────────────────────────────────────────────────┤
│ Status bar                                              │
└─────────────────────────────────────────────────────────┘
```

Each **window** (or tab) owns its own triptych: tree + editor + control.
The bottom panel is shared/global. Windows vs tabs is a toggle.

### Tree panel modes

- **Files** — filesystem tree rooted at project dir
- **Packages** — language-aware grouping (Java packages, Rust modules)
- **Git** — only modified/untracked/staged files
- **Symbols** — outline of current file (from tree-sitter)

### Control panel

Position is user-configurable: right side (wide screens), under editor,
or under editor+tree. Contents:

- Symbol outline of current file
- Git blame / line annotations
- Diagnostics for current file
- AI context

### Bottom panel tabs

| Tab | Content |
|-----|---------|
| Terminal | Embedded shell (PTY) — multiple tabs |
| Kiro | PTY running kiro-cli — interactive + report modes |
| Errors | Compiler diagnostics, navigable (next/prev jumps to file:line) |
| Output | Program/test stdout+stderr |
| Tests | Test results with pass/fail tree |
| Search | Multi-file grep / symbol search results |

## Keyboard layouts

Three first-class layouts, switchable at runtime:

- **Vim** — modal (normal/insert/visual/command), full vi command set
- **Emacs** — prefix-key/chord system (C-x C-s, M-x), minibuffer
- **Classic** — menu-driven with simple key combos (F4-style)

Implementation: the editor core only knows **commands** (an enum). Each
layout is a **keymap** that translates input events to commands. The keymap
is a Tcl-configurable data structure, not hardcoded logic.

A **command palette** (fuzzy-searchable list of all commands) is the universal
escape hatch for all three layouts.

## Tcl scripting

A Tcl subset interpreter in Rust exposes editor primitives as commands:

```tcl
# keybinding
bind Ctrl-S { buffer save }

# custom command
proc save-all {} {
    foreach buf [buffer list] {
        if {[buffer modified $buf]} { buffer save $buf }
    }
}

# expect-style terminal automation
proc run-tests {} {
    terminal send "cargo test 2>&1\n"
    terminal expect {
        "FAILED" { error-panel focus }
        "test result: ok" { status-bar flash "All tests passed" }
    }
}

# kiro integration
proc explain-function {} {
    set sel [editor selection]
    if {$sel eq ""} { set sel [editor current-file] }
    kiro ask "Explain this:" $sel
}
```

Tcl is the config language (replaces JSON `.kairnrc`):

```tcl
# ~/.kairnrc.tcl
theme load "gruvbox"
set tab-width 4
set auto-save on
set editor-keymap vi
bind Ctrl-S { buffer save }
```

The Tcl subset needs: string substitution, command dispatch, `proc`,
`if`/`foreach`/`while`, `set`/`unset`, lists, `after` (timer events).
Skip: namespaces, OO, traces, full regexp engine (use Rust's regex crate).

## Kiro integration

The Kiro panel is a PTY running `kiro-cli chat` with two modes:

**Interactive** — user types directly, full conversational flow. "Send to kiro"
from the editor pastes context (file path + range + optional prompt).

**Report** — editor invokes kiro non-interactively for specific tasks.
Output streams into the Kiro panel. Diffs are detected and offered as
"apply to buffer" actions.

Key design: no custom API client. Kiro integration is **terminal + a thin
parsing layer**. This automatically gets every Kiro feature for free.

## Rendering

ratatui is replaced with a TXV-style virtual screen buffer (`txv` crate):

- Own a `Cell[rows][cols]` grid
- Write to it directly (no widget abstraction)
- Diff against previous frame
- Emit minimal escape sequences via crossterm

This eliminates the rendering garbage issues seen with ratatui's diff model.

### Border rendering modes

Toggle between:
- **Pretty** — box-drawing characters (─│┌┐└┘)
- **Copy-friendly** — vertical borders are colored spaces, horizontal borders
  are colored space rows. Active window: brighter color. Inactive: dimmer.

## Buffer model

Piece table replacing `Vec<String>`:

- Immutable original buffer + append-only additions buffer
- Edit operations are piece descriptors (source, offset, length)
- O(log n) insert/delete anywhere
- Undo/redo is walking the piece list backward/forward
- Minimal memory copying
- Natural fit for Rust ownership model

## What exists and what to keep

From `feature/mini-ide` (toolbox):
- ✅ Keep: terminal emulation (TermBuf + vte + PTY), git integration (gix),
  fuzzy search (nucleo), import navigation (nav/), content search, session
  persistence, config system, CLI parsing
- 🔄 Rewrite: editor (piece table + new buffer API), rendering (txv replaces
  ratatui), main panel (editor panel replaces viewer)
- ➕ New: txv crate, Tcl interpreter, LSP client, tree-sitter, control panel,
  bottom panel tab system, command palette, three keyboard layouts

## Name

**kairn** — after *cairn*, stacked stones marking a trail. The name stays.

## Success criteria

1. Can edit files with vim, emacs, or classic keybindings — all three work
2. Rendering is clean — no garbage on resize, overlay, or panel switch
3. Can navigate Java/Go/Rust/TS projects via imports and LSP
4. Can build, run tests, navigate errors
5. Can send code to Kiro, get suggestions, apply diffs
6. Works over SSH with no GUI dependencies
7. Single binary, all defaults embedded
8. Tcl scripts can extend behavior without recompilation
