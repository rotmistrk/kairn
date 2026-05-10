# rusticle-tk

TUI application framework where apps are written in rusticle (Tcl) scripts
and rendered via txv-widgets. The terminal equivalent of Tcl/Tk.

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

Built on the TXV framework (txv-core + txv-widgets):

```
Program (txv_core::program)
├── StatusBar (preprocess) — key→command translation via KeyLabelItem
└── TkDesktop (GroupState) — holds script widgets as Box<dyn View> children
```

Source layout:

```
src/
  main.rs              — CLI, panic handler, Program::run
  desktop.rs           — TkDesktop (GroupState + name→index mapping)
  event_mgr.rs         — builds StatusBar, runs Program event loop
  keyspec.rs           — parse/format key specifications
  layout_mgr.rs        — Tk-style pack layout computation
  layout_side.rs       — Side enum (left/right/top/bottom/fill)
  widget_mgr.rs        — StringListData helper for ListView
  tk_bridge/
    mod.rs             — SharedState, register_all, helpers
    window_app.rs      — window + app commands
    text_list.rs       — text + list widget commands
    tree_input.rs      — tree + input + statusbar commands
    tabbar_table.rs    — tabbar + table + progress commands
    commands.rs        — dialog, menu, bind, after, focus, notify, files
    tests.rs           — unit tests
```

## Widget Commands

| Command | Subcommands |
|---------|-------------|
| `window` | `create`, `add`, `title` |
| `text` | `create`, `set`, `get`, `clear`, `append`, `line-numbers` |
| `list` | `create`, `set-items`, `selected`, `index`, `on-select`, `on-activate` |
| `tree` | `create`, `selected`, `refresh`, `on-select`, `on-activate` |
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
| `notify` | `message` — status notification |
| `focus` | `widget_id` — set keyboard focus |
| `files` | `path ?-recursive? ?-filter pattern?` |
| `app` | `run`, `quit`, `on-quit` |

## Layout Model

Tk-style pack geometry. Each `window add` peels space from the remaining area:

```tcl
window add $win $tree   -side left   -width 25    ;# fixed left panel
window add $win $status -side bottom -height 1    ;# fixed bottom bar
window add $win $main   -side fill                ;# fills remaining space
```

**Important:** add fixed-size widgets (bottom, left, etc.) *before* the fill
widget. Fill consumes all remaining space.

Sides: `left`, `right`, `top`, `bottom`, `fill`.

## Key Bindings

Scripts register bindings with `bind keyspec { script }`. These become
`KeyLabelItem` entries in the StatusBar (preprocess phase):

```tcl
bind Ctrl-Q { app quit }
bind F5    { load-file $current_file }
bind Tab   { focus $filter }
```

Key spec format: modifiers joined with `-`, then key name.
- Modifiers: `Ctrl`, `Alt`, `Shift`
- Keys: `A`-`Z`, `F1`-`F12`, `Enter`, `Escape`, `Tab`, `Backspace`, `Delete`,
  `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`

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

| Script | Description |
|--------|-------------|
| `hello.tcl` | Minimal app — text + statusbar |
| `dialog-demo.tcl` | Shell dialog replacement |
| `file-browser.tcl` | Three-panel file browser |
| `widget-gallery.tcl` | Every widget type on one screen |
| `log-viewer.tcl` | File viewer with filter input |
| `todo-list.tcl` | Add/remove list items |
| `config-editor.tcl` | Section tree + key/value table |
| `dashboard.tcl` | Live-updating progress bars |

## Known Limitations

- `dialog` commands return defaults in non-interactive mode (no modal yet)
- `menu show` is a stub (no overlay positioning)
- `text` is read-only (TextArea widget)
- `fuzzy-select` returns first item in non-interactive mode
- Binding redesign planned — see `doc/design-binding-architecture.md`
