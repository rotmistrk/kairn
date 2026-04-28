# kairn

```
╦╔═╔═╗╦╦═╗╔╗╔
╠╩╗╠═╣║╠╦╝║║║
╩ ╩╩ ╩╩╩╚═╝╚╝
```

A TUI IDE oriented around [Kiro](https://kiro.dev) AI. Named after *cairn* — stacked stones marking a trail.

## Features

- **Three-panel layout**: File tree ←→ Main viewer ←→ Terminal (kiro/shell)
- **Spatial navigation**: Left/Right arrows move between panels naturally
- **Full terminal emulation** (vte + PTY) for kiro-cli and shell tabs
- **Syntax-highlighted** file viewer with line numbers, search (`/n/N`)
- **Main panel modes**: File → Diff → Log → Blame (sticky, cycle with `Ctrl-Shift-↑/↓`)
- **Vim-style selection**: `v` stream, `V` line, `Ctrl-V` block → send to kiro/shell
- **Git integration**: diff, commit log, blame, file status colors, commit graph
- **Fuzzy file search** (`Ctrl-P`) via nucleo
- **Template macros**: `@file`, `@name`, `@dir`, `@line` expand in terminal input
- **Two-chord key sequences**: Emacs-style `Ctrl-X` prefix bindings (status bar shows pending chord)
- **Terminal capture**: scrape terminal output into main panel for review/saving
- **Configurable keybindings** via `.kairnrc` (JSON, sparse overlay with source tracking)
- **Session persistence**: auto-save on quit, auto-restore on launch

## Quick Start

```bash
cargo build --release
./target/release/kairn
```

Press `F1` for full interactive help.

## Navigation

```
← Tree ←→ Main ←→ Terminal →
```

| Context | Key | Action |
|---------|-----|--------|
| Tree | `→` on file | Focus main panel |
| Tree | `→` on dir | Expand directory |
| Main (scroll) | `←` | Focus tree |
| Main (scroll) | `→` | Focus terminal |
| Main | `Space` | Toggle cursor mode (double-line border) |
| Terminal | `Esc Esc` | Escape to main panel |
| Terminal | `Ctrl-]` | Escape to main panel |
| Any | `F3`/`F4`/`F5` | Direct focus: Tree/Main/Terminal |
| Any | `F2` | Cycle focus |

## Key Bindings

Some bindings use a two-chord sequence: press the prefix (e.g. `Ctrl-X`), then the second key. The status bar shows the pending prefix.

| Key | Action |
|-----|--------|
| `F1` | Help (full docs in main panel) |
| `F6` | Toggle left panel: Files / Commits |
| `F7`/`F8` | Resize tree (Shift: ×5; in stacked layouts, resizes terminal vertically when focused) |
| `F9`/`F10` | Resize terminal (Shift: ×5) |
| `Ctrl-P` | Fuzzy file search |
| `Ctrl-X T` | New shell tab |
| `Ctrl-X N` | New Kiro tab |
| `Ctrl-X K` | Close tab |
| `Ctrl-R` | Rename tab |
| `Ctrl-D` | Diff vs HEAD |
| `Ctrl-G` | Git commit log |
| `Ctrl-E` | Open in $EDITOR |
| `Ctrl-L` | Rotate layout |
| `Ctrl-B` | Toggle file tree |
| `Ctrl-T` | Suspend to shell |
| `Ctrl-O` | Peek screen (MC style) |
| `Ctrl-Q` | Quit |
| `Ctrl-Shift-↑/↓` | Cycle mode/filter/tabs (context-aware) |
| `Ctrl-Enter` | Expand @macros in terminal |
| `/` | Search in main panel |
| `n`/`N` | Next/prev search match |
| `v`/`V`/`Ctrl-V` | Visual select (stream/line/block) |
| `Enter` | Send selection to terminal |
| `PgUp`/`PgDn` | Scroll back in terminal |
| `Ctrl-X O` | Capture last command output to main panel |
| `Ctrl-X A` | Capture full terminal content to main panel |
| `Ctrl-X Ctrl-S` | Save main panel buffer to file |
| `Ctrl-X S` | Save session |
| `Ctrl-Shift-O` | Load session |

## Layouts

```
Wide:                   Tall-Right:               Tall-Bottom:
┌────┬──────┬─────┐    ┌────┬──────────────┐     ┌────┬──────────────┐
│Tree│ Main │Term │    │Tree│    Main      │     │Tree│    Main      │
│    │      │     │    │    ├──────────────┤     ├────┴──────────────┤
└────┴──────┴─────┘    │    │   Terminal   │     │    Terminal       │
                       └────┴──────────────┘     └──────────────────┘
```

## Configuration

```
~/.kairnrc          Global config (auto-created on first run)
$PWD/.kairnrc       Project override (sparse — only set what you change)
$PWD/.kairn.state   Auto-saved on quit, restored on launch
```

```json
{
  "kiro_command": "kiro-cli",
  "line_numbers": true,
  "keys": {
    "quit": "ctrl+q",
    "new_shell_tab": "ctrl+x t",
    "prev_tab": "alt+left",
    "next_tab": "alt+right"
  }
}
```

All keybindings configurable. `F1` shows active bindings with source (default/global/project).

## Environment Variables

| Variable | Description |
|----------|-------------|
| `KAIRN_PID` | Set on start, prevents nested instances |
| `KAIRN_CAPTURE` | Named pipe — `command > $KAIRN_CAPTURE` sends output to main panel |
| `SHELL` | Used for shell tabs |
| `EDITOR` | Used for Ctrl-E |

## Tech Stack

Rust · ratatui · crossterm · vte · portable-pty · syntect · nucleo · gix · similar

## License

MIT
