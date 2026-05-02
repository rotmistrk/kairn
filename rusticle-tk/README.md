# rusticle-tk

TUI application framework where apps are written in rusticle (Tcl) scripts
and rendered via txv/txv-widgets. The terminal equivalent of Tcl/Tk.

## Quick Start

```bash
cargo build --release -p rusticle-tk

# Run a script
./target/release/rusticle-tk examples/hello.tcl

# One-liner (dialog replacement)
./target/release/rusticle-tk -e 'dialog confirm "Delete all files?"'

# Run with a file argument
./target/release/rusticle-tk examples/log-viewer.tcl /var/log/syslog
```

## Architecture

```
rusticle-tk binary
├── main.rs           — CLI, script loading, event loop launch
├── tk_bridge.rs      — registers all widget commands in rusticle
├── widget_mgr.rs     — widget ID registry, lifecycle management
├── layout_mgr.rs     — window/layout commands → Tk-style pack model
└── event_mgr.rs      — bind/after/on-* → txv-widgets EventLoop
```

## Widget Commands

| Command | Subcommands |
|---------|-------------|
| `window` | `create`, `add`, `title` |
| `text` | `create`, `load`, `set`, `get`, `clear`, `append`, `line-numbers` |
| `list` | `create`, `set-items`, `selected`, `index`, `on-select`, `on-activate` |
| `tree` | `create`, `selected`, `expand`, `collapse`, `refresh`, `on-select` |
| `input` | `create`, `get`, `set`, `clear`, `focus`, `on-change`, `on-submit` |
| `statusbar` | `create`, `left`, `right` |
| `tabbar` | `create`, `add`, `remove`, `active`, `set-active`, `on-change` |
| `table` | `create`, `add-row`, `clear`, `selected` |
| `progress` | `create`, `set`, `done` |
| `dialog` | `confirm`, `prompt`, `info`, `error` |
| `menu` | `create`, `show` |
| `fuzzy-select` | (modal, returns selected item) |
| `bind` | `keyspec script` — global key binding |
| `after` | `ms script`, `ms -repeat script` — timers |
| `notify` | `message ?-duration ms?` |
| `files` | `path ?-recursive? ?-filter pattern?` |
| `app` | `run`, `quit`, `on-quit`, `on-resize` |

## Layout Model

Tk-style pack geometry. Each `window add` peels space from the remaining area:

```tcl
window add $win $tree   -side left   -width 25    ;# fixed left panel
window add $win $status -side bottom -height 1    ;# fixed bottom bar
window add $win $main   -side fill                ;# fills remaining space
```

**Important:** add fixed-size widgets (bottom, left, etc.) *before* the fill
widget. Fill consumes all remaining space.

## Examples

### hello.tcl — minimal app
```tcl
set win [window create "Hello"]
set txt [text create -content "Hello, rusticle-tk!"]
window add $win $txt -side fill
set status [statusbar create]
statusbar left $status "Ready"
window add $win $status -side bottom -height 1
bind Ctrl-Q { app quit }
app run
```

### dialog-demo.tcl — shell dialog replacement
```tcl
set answer [dialog confirm "Proceed with installation?"]
if {$answer} { puts "yes" } else { puts "no" }
```

### Full demos in `examples/`

| Script | Description | Widgets used |
|--------|-------------|--------------|
| `hello.tcl` | Minimal app | text, statusbar |
| `dialog-demo.tcl` | Shell dialog replacement | dialog |
| `file-browser.tcl` | Three-panel file browser | tree, text, statusbar |
| `widget-gallery.tcl` | Every widget on one screen | all widget types |
| `log-viewer.tcl` | File viewer with filter | text, input, statusbar, bind |
| `todo-list.tcl` | Add/remove items | list, input, statusbar |
| `config-editor.tcl` | Section tree + key/value table | list, table, input, statusbar |
| `dashboard.tcl` | Live-updating metrics | progress ×4, statusbar, after -repeat |

## Key Bindings

Scripts register bindings with `bind keyspec { script }`:

```tcl
bind Ctrl-Q { app quit }
bind F5    { load-file $current_file }
bind Tab   { input focus $filter }
```

Key spec format: `Ctrl-Q`, `Shift-F1`, `Alt-Left`, `Escape`, `F5`, etc.

## Known Limitations

- `dialog` commands return defaults in non-interactive mode (no modal event loop yet)
- `menu show` is a stub (no overlay positioning yet)
- `text` is read-only (TextArea widget doesn't support editing)
- `table sort` is a stub
- rusticle lacks the `eq`/`ne` string operators — use `==`/`!=` instead
- `fuzzy-select` returns first item in non-interactive mode
