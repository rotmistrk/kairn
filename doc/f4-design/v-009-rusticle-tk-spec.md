# v-009 — rusticle-tk Spec: TUI Application Framework

## Purpose

rusticle-tk is a TUI application framework where apps are written in
rusticle (Tcl) scripts and rendered via txv/txv-widgets. It is the
terminal equivalent of Tcl/Tk for GUIs.

Use cases:
- Rapid TUI app prototyping without Rust compilation
- Shell script UI (replacement for `dialog`/`whiptail`)
- File managers, dashboards, log viewers, config editors
- kairn plugin panels
- Teaching TUI programming

## Dependencies

- `rusticle` — script interpreter
- `txv` — rendering primitives
- `txv-widgets` — interactive components
- `crossterm` — terminal I/O

## Deliverable

A single binary: `rusticle-tk`

```bash
# Run a script
rusticle-tk myapp.tcl

# One-liner from shell (dialog replacement)
rusticle-tk -e 'dialog confirm "Delete all files?"'

# Interactive REPL (for development)
rusticle-tk -i

# Shebang support
#!/usr/bin/env rusticle-tk
```

## Architecture

```
rusticle-tk binary
├── main.rs           — CLI, script loading, REPL
├── tk_bridge.rs      — registers all widget commands in rusticle
├── widget_mgr.rs     — widget ID registry, lifecycle management
├── layout_mgr.rs     — window/layout commands → txv layout engine
└── event_mgr.rs      — bind/after/on-* → txv-widgets EventLoop
```

The bridge is thin: each txv-widget type gets a rusticle command set.
Widget instances are tracked by string IDs in a registry.

## Widget commands

### Window management

```tcl
# Create a top-level window
set win [window create "Title"]

# Add widgets to window with layout constraints
window add $win $widget -side left -width 20
window add $win $widget -side right -width 30
window add $win $widget -side top -height 3
window add $win $widget -side bottom -height 1
window add $win $widget -side fill    ;# takes remaining space

# Nested layouts
set panel [frame create]
window add $panel $w1 -side left -width 20
window add $panel $w2 -side fill
window add $win $panel -side fill
```

`-side` maps to txv layout Direction + Size:
- `left`/`right` → Horizontal split, Fixed or Percent width
- `top`/`bottom` → Vertical split, Fixed or Percent height
- `fill` → Fill remaining space

### Text display/editing

```tcl
set t [text create]
set t [text create -file "path.txt"]
set t [text create -content "Hello\nWorld"]

text load $t "path.txt"
text set $t "new content"
text append $t "more text\n"
text clear $t
text get $t                    ;# returns content
text save $t ?path?
text readonly $t true
text line-numbers $t true
text syntax $t "rust"          ;# enable highlighting
text cursor $t 10 5            ;# set cursor position
text scroll $t 100             ;# scroll to line
```

### List

```tcl
set l [list create -items %["alpha", "beta", "gamma"]]
set l [list create -data [files "."]]

list set-items $l %["new", "items"]
list selected $l               ;# returns selected item
list index $l                  ;# returns selected index
list filter $l "pattern"

list on-select $l {proc_name}
list on-activate $l {proc_name}  ;# Enter/double-click
```

### Tree

```tcl
set t [tree create -data [files "."]]
set t [tree create -data [json-tree $data]]

tree expand $t $node
tree collapse $t $node
tree selected $t
tree refresh $t

tree on-select $t {proc_name}
tree on-activate $t {proc_name}
```

### Input

```tcl
set i [input create -prompt "Search: "]
set i [input create -prompt "Name: " -default "untitled"]

input get $i
input set $i "new text"
input clear $i
input focus $i

input on-change $i {proc_name}
input on-submit $i {proc_name}
```

### Tab bar

```tcl
set tabs [tabbar create]
tabbar add $tabs "Tab 1"
tabbar add $tabs "Tab 2" -modified true
tabbar active $tabs              ;# returns active index
tabbar set-active $tabs 1
tabbar remove $tabs 0

tabbar on-change $tabs {proc_name}
```

### Status bar

```tcl
set s [statusbar create]
statusbar left $s "Ready"
statusbar left $s %[
    %{ text: "vi:NORMAL", fg: "cyan" },
    %{ text: " main.rs", fg: "white" }
]
statusbar right $s "UTF-8  LF"
```

### Dialog (modal)

```tcl
# Confirmation
set answer [dialog confirm "Delete all files?"]
if {$answer} { ... }

# Input prompt
set name [dialog prompt "Enter name:" "default"]

# Info message
dialog info "Operation complete"

# Error
dialog error "File not found: $path"

# Custom buttons
set choice [dialog choose "Save changes?" %["Save", "Discard", "Cancel"]]
```

### Notification (flash)

```tcl
notify "File saved" -duration 3000
notify "Error: build failed" -style error -duration 5000
```

### Progress bar

```tcl
set p [progress create -title "Building..."]
progress set $p 0.5              ;# 50%
progress set $p 0.75 "Linking..."
progress done $p
```

### Checklist / Radio list

```tcl
# Multi-select
set selected [checklist "Select packages:" %[
    %{ label: "vim", checked: true },
    %{ label: "emacs", checked: false },
    %{ label: "nano", checked: false }
]]

# Single-select
set choice [radiolist "Choose editor:" %["vim", "emacs", "nano"]]
```

### Table

```tcl
set t [table create -columns %["Name", "Size", "Modified"]]
table add-row $t %["main.rs", "4.2K", "2026-04-30"]
table add-row $t %["lib.rs", "1.1K", "2026-04-29"]
table sort $t 0                  ;# sort by first column
table selected $t                ;# returns selected row
```

### Menu

```tcl
set m [menu create %[
    %{ label: "Open", key: "Ctrl-O", action: "file-open" },
    %{ label: "Save", key: "Ctrl-S", action: "file-save" },
    %{ label: "---" },
    %{ label: "Quit", key: "Ctrl-Q", action: "quit" }
]]
menu show $m 10 5                ;# show at position
```

### Splitter

```tcl
set s [splitter create -dir horizontal]
splitter add $s $left_widget -min 10
splitter add $s $right_widget -min 20
# User can drag the split point (future: mouse support)
# Keyboard: splitter resize $s 0 +5
```

### File select

```tcl
# Full file picker dialog
set path [file-select "Open file" -dir "." -filter "*.rs"]

# Save dialog
set path [file-save "Save as" -default "untitled.rs"]
```

## Data source commands

Built-in commands for common data:

```tcl
# File listing (respects .gitignore)
files "."                        ;# returns list of entries
files "." -recursive             ;# recursive
files "." -filter "*.rs"         ;# filtered

# File operations
file read "path.txt"             ;# returns content
file write "path.txt" $content   ;# write
file exists "path.txt"           ;# bool
file isdir "path"                ;# bool
file dirname "a/b/c.txt"        ;# → "a/b"
file basename "a/b/c.txt"       ;# → "c.txt"
file extension "a/b/c.txt"      ;# → ".txt"

# Shell command output
exec "ls -la"                    ;# returns stdout
exec "cargo build" -stream $widget  ;# stream output to widget

# Environment
env HOME                         ;# returns $HOME
env PATH                         ;# returns $PATH
```

## Event handling

```tcl
# Key bindings (global)
bind Ctrl-Q { app quit }
bind Ctrl-S { text save $editor }
bind F1 { dialog info [help-text] }

# Widget-specific events
list on-select $mylist on_item_selected
tree on-activate $mytree on_file_opened
input on-submit $search do_search

# Timers
after 1000 { statusbar left $s [clock] }           ;# one-shot
after 1000 -repeat { statusbar left $s [clock] }    ;# repeating

# App lifecycle
app on-quit { save-state }
app on-resize { layout-refresh }
```

## Example: dialog replacement

```bash
#!/usr/bin/env rusticle-tk

# Equivalent of: dialog --yesno "Proceed?" 7 40
set answer [dialog confirm "Proceed with installation?"]
if {$answer} {
    puts "yes"
} else {
    puts "no"
}
exit [expr {$answer ? 0 : 1}]
```

## Example: log viewer

```tcl
#!/usr/bin/env rusticle-tk

set win [window create "Log Viewer"]
set log [text create -readonly true -syntax "log"]
set filter [input create -prompt "Filter: "]
set status [statusbar create]

window add $win $filter -side top -height 3
window add $win $log -side fill
window add $win $status -side bottom -height 1

proc load-log {path} {
    text load $log $path
    statusbar left $status "$path — [text get $log .len] lines"
}

proc do-filter {} {
    set pattern [input get $filter]
    text filter $log $pattern
}

input on-change $filter do-filter
bind Ctrl-Q { app quit }
bind F5 { load-log [lindex $argv 0] }

load-log [lindex $argv 0]
app run
```

## Example: yazi-style file manager

```tcl
#!/usr/bin/env rusticle-tk

set win [window create "Files — [pwd]"]

set parent [list create -data [files [file dirname [pwd]]]]
set current [list create -data [files "."]]
set preview [text create -readonly true]
set status [statusbar create]

window add $win $parent -side left -width 25
window add $win $current -side left -width 35
window add $win $preview -side fill
window add $win $status -side bottom -height 1

set cwd [pwd]

proc navigate {path} {
    global cwd current parent preview status
    if {[file isdir $path]} {
        set cwd $path
        list set-items $current [files $path]
        list set-items $parent [files [file dirname $path]]
        text clear $preview
        window title $win "Files — $cwd"
    } else {
        text load $preview $path
    }
    statusbar left $status $path
}

proc go-up {} {
    global cwd
    navigate [file dirname $cwd]
}

list on-select $current {path { navigate $path }}
bind q { app quit }
bind h go-up
bind l { navigate [list selected $current] }
bind / { input focus $filter }

statusbar left $status [pwd]
app run
```

## Build order

rusticle-tk is built after txv-widgets and rusticle:

```
Phase 0a: rusticle     ──┐
Phase 0b: txv          ──┤
Phase 1:  txv-widgets  ──┼── Phase 1b: rusticle-tk
                         └── Phase 2+: kairn
```

### Internal build order

1. `widget_mgr.rs` — widget ID registry
2. `tk_bridge.rs` — register core widget commands (text, list, statusbar)
3. `layout_mgr.rs` — window/frame/layout commands
4. `event_mgr.rs` — bind, after, on-* event wiring
5. `main.rs` — CLI, script loading, REPL
6. Additional widget commands (dialog, table, progress, etc.)

### Validation

- The three example scripts above must work
- `dialog confirm/prompt/info` must work as shell one-liners
- Resize must work without artifacts
- All widget commands must be covered by the rusticle manifest for validation
