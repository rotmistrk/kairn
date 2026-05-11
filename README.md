# kairn

```
╦╔═╔═╗╦╦═╗╔╗╔
╠╩╗╠═╣║╠╦╝║║║
╩ ╩╩ ╩╩╩╚═╝╚╝
```

A TUI IDE oriented around [Kiro](https://kiro.dev) AI. Named after *cairn* — stacked stones marking a trail.

## Features

- **Three-panel layout**: Files/Git/Todo ←→ Editor ←→ Terminal (kiro/shell)
- **Left panel tabs**: File tree, Git changes, Todo tree (cycle with tab dropdown)
- **Full terminal emulation** (vte + PTY) for kiro-cli and shell tabs
- **Scrollback buffer**: PgUp/PgDn to scroll terminal history (configurable size)
- **Inline editing**: Vim-style editor with syntax highlighting, line numbers
- **LSP integration**: completion, go-to-definition, references, hover, diagnostics, rename, code actions
- **Git integration**: diff, file status colors, stage/unstage/untrack/commit from Git panel
- **Todo tree**: hierarchical task management (`.kairn.todo`, duir-compatible format)
- **MCP server**: exposes tabs and terminal content to kiro for AI integration
- **Session persistence**: auto-save on quit, auto-restore on launch
- **Fuzzy file search** (`Ctrl-P`) via nucleo
- **Configurable keybindings** via `.kairnrc` (JSON, sparse overlay with source tracking)
- **Build integration**: build/run/test commands with error navigation (next-error/prev-error)

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
| Tree | `Enter`/`→` on file | Open in editor |
| Tree | `→` on dir | Expand directory |
| Any | `F2`/`F3`/`F4` | Direct focus: Tree/Main/Terminal |
| Any | `F5` | Zoom toggle (maximize focused slot) |
| Any | `Ctrl-Shift-←/→` | Focus prev/next slot |

## Key Bindings

| Key | Action |
|-----|--------|
| `F1` | Help (full docs in main panel) |
| `F2`/`F3`/`F4` | Focus: Tree / Main / Terminal |
| `F5` | Zoom toggle (maximize focused slot) |
| `F6` | Messages window |
| `Ctrl-Q` | Quit |
| `Ctrl-Z` | Suspend to shell |
| `Ctrl-O` | Peek screen (MC style) |
| `Ctrl-D` | Diff vs HEAD (`:diff` for options) |
| `Ctrl-.` | Toggle hidden (dot) files in tree |
| `Ctrl-Shift-←/→` | Focus prev/next slot |
| `Ctrl-Shift-↑/↓` | Open tab dropdown picker |
| `≠/–` (Alt+=/Alt+-) | Grow/shrink panel width |
| `±/—` (Alt+Shift) | Grow/shrink panel height |
| `M-x` (Alt-x/≈) | Command mode prompt |
| `PgUp`/`PgDn` | Scroll back in terminal |

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
