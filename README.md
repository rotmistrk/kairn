# kairn

```
╦╔═╔═╗╦╦═╗╔╗╔
╠╩╗╠═╣║╠╦╝║║║
╩ ╩╩ ╩╩╩╚═╝╚╝
```

A TUI IDE oriented around [Kiro](https://kiro.dev) AI. Named after *cairn* — stacked stones marking a trail.

## What It Does

Three-panel TUI with a vim editor, terminal emulator, and file/git/todo tree — all wired together through an MCP server so Kiro AI can see and control everything. Scriptable via Tcl.

## Quick Start

```bash
make setup              # enable pre-commit hook (once per clone)
cargo build --release
./target/release/kairn
```

Press `F1` for interactive help. Press `M-x` (Alt-x) for command mode.

## Layout

```
Wide (>176 cols):        Tall (<176 cols):
┌────┬──────┬─────┐     ┌────┬──────────────┐
│Tree│Editor│Term │     │Tree│   Editor     │
│Git │      │     │     │Git ├──────────────┤
│Todo│      │     │     │Todo│  Terminal    │
└────┴──────┴─────┘     └────┴──────────────┘
```

Panels: Left (tree/git/todo tabs) · Center (editor) · Right/Bottom (terminal).
Auto-switches between Wide and Tall based on terminal width.

## Key Bindings

### Global

| Key | Action |
|-----|--------|
| `F1` | Help |
| `F2` / `F3` / `F4` | Focus: Tree / Editor / Terminal |
| `F5` | Zoom (maximize focused panel) |
| `F6` | Messages |
| `Ctrl-Q` | Quit |
| `Ctrl-Z` | Suspend to shell |
| `Ctrl-O` | Peek (show terminal underneath) |
| `Ctrl-D` | Diff current file vs HEAD |
| `Ctrl-L` | Repaint screen |
| `M-x` (Alt-x / ≈) | Command mode |
| `Ctrl-Shift-←/→` | Focus prev/next panel |
| `Ctrl-Shift-↑/↓` | Tab dropdown picker |
| `Alt-0..9` | Select tab by number |
| `≠/–` (Alt+=/Alt+-) | Grow/shrink panel width |
| `±/—` (Alt+Shift) | Grow/shrink panel height |
| `PgUp/PgDn` | Terminal scrollback |

### Editor (Vim)

Normal mode: `h/j/k/l`, `w/b/e`, `0/$`, `gg/G`, `dd/yy/p`, `u/Ctrl-R`, `.`, `v/V`, `>>/<<`, `f/t`, `%`

LSP: `gd` (definition), `gr` (references), `gR` (rename), `K` (hover)

Visual: extend selection, `d/c/y/>/<`, `:` for ex commands

Search: `/pattern`, `n/N`, `*/#`

Ex: `:w`, `:q`, `:wq`, `:%s/pat/rep/g`, `:set wrap`, `:diff`, `:e <path>`

Insert: `Esc` to exit, `Ctrl-N/P` for completion

### File Tree

`j/k` navigate, `Enter/→` open/expand, `h/←` collapse, `Ctrl-.` toggle hidden

### Git Panel

`s` stage, `u` unstage, `x` untrack, `c` commit

### Todo Panel

`Space` toggle done, `!` toggle important, `e` edit, `n` new sibling, `b` new child, `d` delete, `J/K` swap, `H/L` promote/demote

## Commands (M-x)

| Command | Description |
|---------|-------------|
| `edit <path>` / `e <path>` | Open file |
| `save` | Save current file |
| `close` | Close current tab |
| `shell` | New shell tab |
| `kiro [--agent=name]` | New Kiro AI session |
| `build` / `run` / `test` | Build integration |
| `test-file` / `test-at-cursor` | Targeted tests |
| `next-error` / `prev-error` | Error navigation |
| `grep <pattern>` | Project-wide search |
| `diff` | Diff current file |
| `lsp-rename <name>` | Rename symbol |
| `lsp-status` | Show LSP status |
| `code-action` | LSP code actions |
| `paste` | System clipboard paste |
| `theme dark/light/toggle` | Switch theme |
| `git-stage/unstage/untrack <f>` | Git operations |
| `git-commit <msg>` | Commit |
| `tab-rename <name>` | Rename tab |
| `struct` / `text` | Switch view mode |
| `tab` | Open current file as CSV/TSV table |
| *anything else* | Evaluated as Tcl script |

## Configuration

Config is Tcl. Loaded in order (later overrides earlier):

```
~/.config/kairn/init.tcl     Global settings
~/.kairn/config.tcl          User preferences
~/.kairn/plugins/*/init.tcl  Plugins (alphabetical)
.kairn/init.tcl              Project-local overrides
```

Example (`~/.kairn/config.tcl`):
```tcl
set editor.wrap false
set editor.number true
set editor.tabstop 4
set terminal.scrollback 5000
set theme.mode "dark"
```

See `doc/example-init.tcl` for all settings including colors and LSP config.

## Tcl Scripting

Any M-x command that isn't a built-in is evaluated as Tcl. Available namespaces:

| Namespace | Operations |
|-----------|-----------|
| `editor` | open, save, save-all, close, goto, insert, undo, redo, current-file, current-line, current-col, modified?, filetype, get-selection, replace-selection, get-line, delete-line, replace-word |
| `view` | focus, message, status |
| `build` | run, test |
| `lsp` | hover, definition, references, rename, format |
| `git` | stage, unstage, commit |
| `todo` | add, remove, complete |
| `keymap` | bind, unbind |
| `hook` | add, remove, list |
| `system` | exec, env, set-env, root-dir, home-dir, platform, clipboard-get, clipboard-set |

Example:
```tcl
hook add file-save { build run }
keymap bind ctrl+b { build run }
editor goto 42
```

### Selection Manipulation & Filtered Hooks

Scripts can read and modify the editor selection:
```tcl
# Quote the current selection
keymap bind ctrl+q {
  set sel [editor get-selection]
  editor replace-selection "\"$sel\""
}
```

Hooks support optional `-filter` to fire only on matching input:
```tcl
# Auto-close brackets
hook add char-inserted -filter "(" { editor insert ")" }

# Format on idle
hook add idle { lsp format }
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `KAIRN_MCP_SOCKET` | Set on start — MCP socket path for AI integration |
| `KAIRN_SUSPENDED` | Nesting guard (prevents running inside suspended session) |
| `SHELL` | Used for shell tabs |

## MCP Server

Exposes kairn state to Kiro AI via JSON-RPC over Unix socket. Tools:

- Read/write terminal content
- List/switch tabs
- Open/save files
- Read editor state (cursor, selection, diagnostics)
- Add todo items (including batch `add_subtree`)

## Tech Stack

Rust · txv (custom TUI framework) · crossterm · syntect · git2 · similar · rusticle (Tcl) · duir (todo)

External (in txv-widgets): vte · portable-pty · nucleo

## License

MIT
