# kairn

```
╦╔═╔═╗╦╦═╗╔╗╔
╠╩╗╠═╣║╠╦╝║║║
╩ ╩╩ ╩╩╩╚═╝╚╝
```

A TUI IDE oriented around [Kiro](https://kiro.dev) AI. Named after *cairn* — stacked stones marking a trail.

## Features

- **File tree** with git status colors, filter modes (All/Modified/Untracked), auto-preview
- **Syntax-highlighted** file viewer with line numbers, scrolling, incremental search (`/n/N`)
- **Main panel modes**: File → Diff → Log → Blame (sticky, cycle with `Ctrl-Shift-↑/↓`)
- **Vim-style selection**: `v` stream, `V` line, `Ctrl-V` block — send to kiro/shell with Enter
- **Full terminal emulation** (vte + PTY) for kiro-cli and shell tabs with scrollback
- **Fuzzy file search** (`Ctrl-P`) via nucleo
- **Git integration**: diff vs HEAD, commit log, blame, file status colors
- **Configurable keybindings** via `.kairnrc` (JSON, sparse overlay)
- **Session persistence**: auto-save on quit, auto-restore on launch
- **3 rotatable layouts** with resizable panels

## Layouts

```
Layout 1 (Wide):        Layout 2 (Tall-Right):    Layout 3 (Tall-Bottom):
┌────┬──────┬─────┐    ┌────┬──────────────┐     ┌────┬──────────────┐
│Tree│ Main │Term │    │Tree│    Main      │     │Tree│    Main      │
│    │      │     │    │    ├──────────────┤     ├────┴──────────────┤
└────┴──────┴─────┘    │    │   Terminal   │     │    Terminal       │
                       └────┴──────────────┘     └──────────────────┘
```

## Quick Start

```bash
cargo build --release
./target/release/kairn
```

### Key Bindings

| Key | Action |
|-----|--------|
| `F1` | Help (full docs in main panel) |
| `F2` | Cycle panel focus |
| `F3`/`F4`/`F5` | Focus Tree/Main/Terminal |
| `Ctrl-P` | Fuzzy file search |
| `Ctrl-S` | New shell tab |
| `Ctrl-K` | New Kiro tab |
| `Ctrl-D` | Diff vs HEAD |
| `Ctrl-G` | Git commit log |
| `Ctrl-E` | Open in $EDITOR |
| `Ctrl-L` | Rotate layout |
| `Ctrl-B` | Toggle file tree |
| `Ctrl-T` | Suspend to shell |
| `Ctrl-O` | Peek screen (MC style) |
| `Ctrl-Q` / `Esc Esc` | Quit |
| `Ctrl-Shift-↑/↓` | Cycle mode (per panel) |
| `Space` | Toggle cursor mode (main panel) |
| `/` | Search in main panel |
| `n`/`N` | Next/prev search match |
| `v`/`V`/`Ctrl-V` | Visual select (stream/line/block) |
| `PgUp`/`PgDn` | Scroll back in terminal |

### File Tree

- `j`/`k` or `↑`/`↓` — navigate
- `Enter`/`l`/`→` — open file / expand dir
- `h`/`←` — collapse dir
- Files auto-preview on cursor move
- Git colors: yellow=modified, green=added, red=deleted

## Configuration

```
~/.kairnrc          Global config (auto-created on first run)
$PWD/.kairnrc       Project override (sparse — only set what you change)
$PWD/.kairn.state   Auto-saved on quit, restored on launch
```

Example `.kairnrc`:
```json
{
  "kiro_command": "kiro-cli",
  "line_numbers": true,
  "keys": {
    "quit": "ctrl+q",
    "new_shell_tab": "ctrl+s",
    "prev_tab": "ctrl+shift+left",
    "next_tab": "ctrl+shift+right"
  }
}
```

All keybindings are configurable. Missing keys use built-in defaults. Press `F1` for full documentation including active bindings with their source (default/global/project).

## Environment Variables

| Variable | Description |
|----------|-------------|
| `KAIRN_PID` | Set on start, prevents nested instances |
| `KAIRN_CAPTURE` | Named pipe — `command > $KAIRN_CAPTURE` sends output to main panel |
| `SHELL` | Used for shell tabs |
| `EDITOR` | Used for Ctrl-E |

## Tech Stack

- **Rust** with ratatui, crossterm
- **vte** + **portable-pty** for terminal emulation
- **syntect** for syntax highlighting
- **nucleo** for fuzzy search
- **gix** + **similar** for git operations

## License

MIT
