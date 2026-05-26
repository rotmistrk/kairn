# Status Bar Redesign

## Core Idea

The status bar is a flat container of **items**. Each item has:

- **priority** — determines visibility when space is tight (higher = shown first)
- **min-sz** — minimum characters needed (default: label length)
- **max-sz** — maximum characters to claim (0 = unbounded)
- **stretch** — weight for distributing extra space (0 = fixed width)
- **gravity** — `left` or `right` alignment
- **label** — display text (may be dynamic, may be empty)
- **action** — optional command to emit on activation key
- **embed** — optional children (sub-keys or input line) shown when active

There is no "exclusive mode" as a special concept. When an item activates
(prefix key pressed, input line opened), its embedded content appears with
its own priority and stretch. Lower-priority items yield space naturally.

## Layout Algorithm

Each item has sizing properties:

- **min-sz** — minimum characters needed (default: label length)
- **max-sz** — maximum characters to claim (0 = unbounded)
- **stretch** — weight for distributing extra space (0 = fixed width)
- **priority** — who survives when space is tight (higher = kept)

Steps:

1. Collect visible items (idle labels + active embedded content)
2. Sort by priority descending
3. Sum min-sz of all items; if exceeds width, drop lowest-priority until fits
4. Allocate min-sz to each surviving item
5. Distribute remaining space to stretch items proportionally (capped by max-sz)
6. Render: left-gravity from x=0 rightward, right-gravity from x=width leftward

## Item Types

### key

A binding that intercepts a key and optionally shows a label.

Properties: key, label, action, priority, gravity, embed.

- Without `-embed`: simple key → command (like current `KeyLabelItem`)
- With `-embed`: prefix key that reveals children when pressed

### cmdline

An input line with completion and history. Always used inside `-embed`.

Properties: priority, stretch, completer, history.

When active: renders the input text, handles typing/Tab/Enter/Esc.
When deactivated: parent key reverts to its idle label.

### indicator

A read-only display driven by editor/system state. Renders a formatted label.

Properties: format, priority, gravity.

Format variables are filled from context updates (cursor moved, mode changed, etc).

### message

Toast area. Shows latest message, auto-dismisses after timeout.

Properties: timeout, priority, gravity, stretch.

## Tcl Configuration API

Follows existing kairn pattern: `namespace verb args ?-option value?`

### status add

```tcl
status add [key ctrl+q -label "^Q" -action quit -priority 10]
status add [key F1 -label "F1:Help" -action help -priority 8]
status add [key F5 -label "F5:Zoom" -action zoom -priority 5]

# Hidden binding (no label, just intercepts)
status add [key ctrl+z -action suspend]
status add [key ctrl+d -action diff]
```

### status add — prefix keys

```tcl
status add [key ctrl+w -label "C-w" -priority 7 -embed {
    [key s -label "s:split" -action split]
    [key v -label "v:vsplit" -action vsplit]
    [key c -label "c:close" -action close]
    [key o -label "o:only" -action only]
    [key w -label "w:cycle" -action cycle]
}]

# Nested prefix (C-x → sub-commands, 5 → frame sub-prefix)
status add [key ctrl+x -label "C-x" -priority 7 -embed {
    [key s -label "s:save" -action save]
    [key 0 -label "0:only" -action only]
    [key 5 -label "5:frame" -embed {
        [key 2 -label "2:new" -action new-frame]
        [key 0 -label "0:close" -action close-frame]
    }]
}]
```

### status add — command line

```tcl
status add [key alt+x -label "M-x" -priority 9 -embed {
    [cmdline -priority 20 -min-sz 10 -stretch 100 -completer commands -history command]
}]

# Vim-style ex command
status add [key : -label ":" -priority 9 -embed {
    [cmdline -priority 20 -min-sz 10 -stretch 100 -completer ex -history ex]
}]

# Search
status add [key / -priority 6 -embed {
    [cmdline -priority 18 -min-sz 10 -stretch 100 -completer search -history search]
}]
```

### status add — indicators

```tcl
status add [indicator position -format "L:{line} C:{col}" -priority 8 -gravity right]
status add [indicator mode -priority 10 -gravity right]
status add [indicator modified -format "{mod}" -priority 9 -gravity right]
status add [indicator language -priority 4 -gravity right]
status add [indicator branch -priority 3 -gravity right]
status add [indicator lsp -priority 2 -gravity right]
status add [indicator clock -format "{H}:{M}" -priority 1 -gravity right]
```

### status add — message area

```tcl
status add [message -timeout 5 -priority 6 -gravity left -stretch 50]
```

### status clear

```tcl
status clear   ;# remove all items, start fresh
```

## Format String Variables

| Variable | Source | Example |
|----------|--------|---------|
| `{line}` | Cursor line (1-indexed) | `42` |
| `{col}` | Cursor column (1-indexed) | `7` |
| `{mode}` | Editor mode | `NOR`, `INS`, `VIS` |
| `{mod}` | Modified flag | `[+]` or empty |
| `{lang}` | File language | `rust`, `go` |
| `{branch}` | Git branch | `main` |
| `{file}` | Current filename | `status.rs` |
| `{H}` | Hour (24h) | `14` |
| `{M}` | Minute | `32` |
| `{S}` | Second | `07` |

## Architecture Split

### txv-core provides:

- `StatusBar` container with priority-based layout
- `StatusItem` trait: `priority()`, `min_width()`, `stretch()`, `gravity()`,
  `label()`, `handle()`, `tick()`, `is_active()`
- Layout algorithm (priority sort → allocate → hide overflow)

### txv-widgets provides:

- `KeyItem` — key binding with optional label and embed
- `CmdlineItem` — input line with completion/history
- `MessageItem` — toast with timeout
- `ClockItem` — time display
- Generic `FormatIndicator` — format-string driven label updated by commands

### kairn provides:

- App-specific indicators: `PositionIndicator`, `ModeIndicator`,
  `ModifiedIndicator`, `LangIndicator`, `BranchIndicator`, `LspIndicator`
- Tcl bridge: `status` namespace commands that construct items and add to bar
- Default status bar config in `doc/example-init.tcl`

## Key Dispatch (unchanged)

1. **Pre-phase** — StatusBar items intercept global keys
2. **Normal phase** — focused widget handles local keys
3. **Post-phase** — fallback handlers

When a prefix key activates, its embedded children become the active
handlers. Esc or timeout deactivates, reverting to idle state.

## Migration

Current `build_status_bar()` in `status.rs` becomes the **default config** —
emitted as Tcl commands if no user config overrides `status`. Users can
`status clear` and rebuild from scratch, or just add/remove individual items.
