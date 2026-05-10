# v-015: Configuration System

## Overview

kairn reads `~/.config/kairn/init.tcl` (rusticle script) at startup.
The script populates AppSettings and configures the status bar.
If the file doesn't exist, compiled defaults apply.

## File Location

XDG convention:
- `$XDG_CONFIG_HOME/kairn/init.tcl` (if XDG_CONFIG_HOME set)
- `~/.config/kairn/init.tcl` (fallback)

## Settings Commands

```tcl
# Editor defaults (new editors get these)
set editor.wrap on
set editor.list off
set editor.tabstop 4
set editor.number on

# Global
set clock.interval 60
```

## Status Bar Commands

If ANY `statusbar` command appears, the default bar is NOT used.
The config builds the bar from scratch.

```tcl
# Wipe defaults (implicit if any statusbar command exists)
statusbar clear

# Key label items: statusbar add-left <key> <command> <label>
statusbar add-left  F1       show-help    "F1:Help"
statusbar add-left  F2       focus-left   "F2:Tree"
statusbar add-left  F3       focus-center "F3:Main"
statusbar add-left  F4       focus-right  "F4:Term"
statusbar add-left  F5       zoom-toggle  "F5:Zoom"
statusbar add-left  Ctrl-q   quit         "^Q:Quit"

# Hidden hotkeys (no label, just key→command binding)
statusbar bind Ctrl-Shift-Left  focus-prev
statusbar bind Ctrl-Shift-Right focus-next
statusbar bind Ctrl-Shift-Up    tab-dropdown
statusbar bind Ctrl-Shift-Down  tab-dropdown

# Command input (exclusive mode, activated by listed keys)
statusbar command "Alt-x,≈" "M-x"

# Right-side indicators
statusbar add-right position
statusbar add-right mode
statusbar add-right message 5
statusbar add-right branch
statusbar add-right clock 60
```

## Command Name → CommandId Mapping

The config uses string command names. A lookup table maps them:

| Name | CommandId |
|------|-----------|
| show-help | CM_SHOW_HELP |
| focus-left | CM_FOCUS_LEFT |
| focus-center | CM_FOCUS_CENTER |
| focus-right | CM_FOCUS_RIGHT |
| focus-prev | CM_FOCUS_PREV |
| focus-next | CM_FOCUS_NEXT |
| zoom-toggle | CM_ZOOM_TOGGLE |
| tab-dropdown | CM_TAB_DROPDOWN |
| tab-next | CM_TAB_NEXT |
| tab-prev | CM_TAB_PREV |
| tab-close | CM_TAB_CLOSE |
| quit | CM_QUIT |
| show-messages | CM_SHOW_MESSAGES |

## Key Name Parsing

| Syntax | Meaning |
|--------|---------|
| F1-F12 | Function keys |
| Ctrl-x | Ctrl + char |
| Alt-x | Alt + char |
| Ctrl-Shift-Left | Ctrl+Shift+Arrow |
| ≈ | Literal char (macOS Alt-x produces this) |

## Implementation Plan

### Phase 1: Config loading (this task)
1. Create src/config.rs — reads init.tcl, executes via rusticle
2. Register `set` and `statusbar` commands in rusticle interpreter
3. After execution, extract AppSettings and optional StatusBar
4. In main.rs: load config → if statusbar was configured, use it; else use default

### Phase 2: Runtime :set (already partially works)
- `:set wrap` in editor → mutates instance settings
- `:setg wrap` → mutates AppSettings.editor_defaults (future editors)

### Phase 3: :set saves to config (future)
- `:set! wrap` → writes to init.tcl (persistent)

## Rusticle Integration

The interpreter gets custom commands registered:

```rust
interp.register("set", |args| { /* parse key.subkey value, update settings */ });
interp.register("statusbar", |args| { /* parse subcommand, build bar */ });
```

The interpreter runs the script. Side effects accumulate in a ConfigState struct.
After execution, ConfigState is consumed to produce AppSettings + optional StatusBar.
