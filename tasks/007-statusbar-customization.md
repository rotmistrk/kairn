# Statusbar Customization

<!-- TODO: mark done → todo tree [2][6] -->

## Overview

Let users configure which items appear in the status bar, their order,
and optionally define custom items via Tcl.

## Tcl API

```tcl
# Configure left and right sections
status-bar left {mode position file-modified}
status-bar right {git-branch lsp-status encoding line-ending clock}

# Custom item (evaluated every tick, result displayed)
status-bar add-custom "build" { if {[build running?]} { "⟳" } else { "" } }
```

## Available Items

| Name | Description |
|------|-------------|
| mode | NOR/INS/VIS/CMD |
| position | Ln N, Col N |
| file-modified | [+] when dirty |
| git-branch | Current branch |
| lsp-status | ◉/○ indicator |
| encoding | UTF-8 |
| line-ending | LF/CRLF |
| clock | HH:MM |
| file-type | rust/go/etc |
| selection | N lines selected |

## Implementation

- Parse `status-bar` Tcl commands during config load
- Store ordered list of item IDs per side (left/right)
- On tick: render only configured items in configured order
- Custom items: evaluate Tcl expression, display result string

## Constraints

- Default config matches current hardcoded layout (no visible change without config)
- 240 code line max per file
- Custom items must not block the main thread (timeout after 1ms)
