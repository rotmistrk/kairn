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
make release
./target/release/kairn
```

Press `F1` for interactive help. Press `M-x` (Alt-x) for command mode.

## Layout

```
Wide (≥300 cols):        Tall (≤200 cols):
┌────┬──────┬─────┐     ┌────┬──────────────┐
│Tree│Editor│Term │     │Tree│   Editor     │
│Git │      │     │     │Git ├──────────────┤
│Todo│      │     │     │Todo│  Terminal    │
└────┴──────┴─────┘     └────┴──────────────┘
```

Panels: Left (tree/git/todo tabs) · Center (editor) · Right/Bottom (terminal).
Auto-switches between Wide and Tall based on terminal width (configurable thresholds).

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
| `Alt-0` | Tab dropdown (list all tabs) |
| `Alt-1..9` | Select tab by number |
| `Alt-;` / `Alt-'` | Next / previous tab |
| `Alt-w` | Close active tab |
| `Alt-,` | Toggle tree panel |
| `Alt-.` | Toggle tools panel |
| `Alt-/` | Zoom toggle |
| `Alt-\` | Cycle layout mode |
| `Ctrl-W` | Split prefix (s:split, v:vsplit, c:close, o:only, w:cycle, m:move, +/-:resize, =:equalize) |
| `Alt-=` / `Alt--` | Grow / shrink subpanel |
| `Alt-Shift-←/→` | Resize panel horizontally |
| `Alt-Shift-↑/↓` | Resize panel vertically |
| `≠/–` (macOS Alt+=/Alt+-) | Grow/shrink panel width |
| `±/—` (macOS Alt+Shift) | Grow/shrink panel height |
| `PgUp/PgDn` | Terminal scrollback |

### Editor (Vim)

Normal mode: `h/j/k/l`, `w/b/e`, `0/$`, `gg/G`, `dd/yy/p`, `u/Ctrl-R`, `.`, `v/V`, `>>/<<`, `f/t`, `%`

LSP: `gd` (definition), `gr` (references), `gR` (rename), `K` (hover)

Visual: extend selection, `d/c/y/>/<`, `:` for ex commands

Search: `/pattern`, `n/N`, `*/#`

Ex: `:w`, `:q`, `:wq`, `:%s/pat/rep/g`, `:set wrap`, `:diff`, `:diff -y` (side-by-side), `:diff -w` (ignore ws), `:diff --base <ref>`, `:revert`, `:nodiff`, `:blame`, `:noblame`, `:e <path>`, `:split`, `:vsplit`, `:only`

Insert: `Esc` to exit, `Ctrl-N/P` for completion

### Diff Mode

| Key | Action |
|-----|--------|
| `j/k` | Move down/up |
| `n/N` | Next/previous hunk |
| `g/G` | Jump to start/end |
| `R` | Revert hunk under cursor |
| `Enter` | Exit diff, jump to line |
| `Esc` | Exit diff mode |
| `/` | Search |

Side-by-side diff (`:diff -y`): left=base, right=current, aligned with gaps. Same navigation keys; `q`/`Esc` exits.

### File Tree

`j/k` navigate, `Enter/→` open/expand, `h/←` collapse, `Ctrl-.` toggle hidden

### Git Panel

`s` stage, `u` unstage, `x` untrack, `c` commit

### Todo Panel

`Space` toggle done, `!` toggle important, `e` edit, `n` new sibling, `b` new child, `d` delete, `J/K` swap, `H/L` promote/demote, `N` open note, `/` filter

### Results View (grep/references/errors)

`j/k` navigate, `Enter` open (keep focus), `→` open (focus editor), `n/p` next/prev + open, `q` close

### CSV/Table View

`h/j/k/l` navigate cells, `g/G` first/last row, `0/$` first/last column, `Enter` edit cell, `s` sort, `f/F` filter/clear, `Ctrl-F` clear all filters

### Structured View (JSON tree)

`j/k` navigate, `Space/l/→` expand/collapse, `h/←` collapse/parent, `Tab` cycle column, `Enter` edit, `n` new sibling, `b` new child, `d` delete, `t/T` cycle type, `J/K` swap, `H/L` promote/demote, `u/Ctrl-R` undo/redo

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
| `split [<file>]` / `vsplit [<file>]` | Split editor |
| `struct` / `text` | Switch view mode |
| `tab` | Open current file as CSV/TSV table |
| `blame` / `noblame` | Show/hide git blame |
| `log` | Show git log |
| `zoom` | Zoom toggle (maximize panel) |
| `layout` | Cycle layout mode (auto/wide/tall) |
| `move-tab` | Move tab to other subpanel |
| `toggle-tree` / `toggle-tools` | Show/hide side panels |
| `grow` / `shrink` | Resize panel horizontally |
| `grow-v` / `shrink-v` | Resize panel vertically |
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
See `doc/tcl-reference.md` for the full scripting API reference.

## Tcl Scripting

Any M-x command that isn't a built-in is evaluated as Tcl. Available namespaces:

| Namespace | Operations |
|-----------|-----------|
| `editor` | open, save, save-all, close, goto, insert, undo, redo, search, clear-highlight, current-file, current-line, current-col, modified?, filetype, get-selection, replace-selection, get-line, delete-line, replace-word, diff-revert |
| `view` | focus, message, status, theme, zoom, toggle-tree, toggle-tools, layout |
| `build` | run, test, test-file, test-at-cursor, next-error, prev-error |
| `lsp` | hover, definition, references, rename, format, start, restart, stop, timeout, args |
| `git` | stage, unstage, commit, blame, noblame, untrack, log, diff |
| `todo` | add, remove, complete, toggle-important, edit, swap, promote, demote, list |
| `split` | vsplit, hsplit, close, focus, open, direction, linked |
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

- **Tabs**: list, close, get content
- **Files**: open, create, save
- **Editor**: read state (cursor, selection, diagnostics), edit buffer, insert text, set cursor, undo/redo
- **Terminal**: read content, send input
- **Build**: run build/test, get errors, search project (grep)
- **Diff**: revert hunk under cursor
- **Split**: create/close/focus/open/linked scroll
- **Todo**: add (including batch `add_subtree`), toggle, remove, move, promote/demote, notes
- **Git**: stage, unstage, commit
- **LSP**: start/restart/stop, hover, definition, references, rename, code-action, semantic tokens
- **Scripting**: eval Tcl
- **Messages**: read message log

## Tech Stack

Rust · txv (custom TUI framework) · crossterm · syntect · git2 · similar · rusticle (Tcl) · duir (todo)

External (in txv-widgets): vte · portable-pty · nucleo

## License

MIT
