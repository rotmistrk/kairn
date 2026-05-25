# Status Bar Redesign

## Core Idea

The status bar is a **group** (container). Status bar items are **views** (teclynnau).
Each item is self-contained: it draws itself AND handles its own key binding.

There is no separation between "what's displayed" and "what's bound" — they are
the same object.

## Teclyn Types

### CommandKey

A leaf that shows a label and intercepts a single key.

```
CommandKey("C-q:Quit", Ctrl+'q', CMD_QUIT)
```

- Draws: `C-q:Quit`
- Intercepts: Ctrl+Q → emits CMD_QUIT
- Pre-phase: catches key before any focused widget sees it

### ModalKey

A prefix key that reveals its children when activated.

```
ModalKey("C-x:...", Ctrl+'x')
├── CommandKey("s:Save", 's', CMD_SAVE)
├── CommandKey("0:Only", '0', CMD_CLOSE_OTHERS)
└── CommandKey("2:HSplit", '2', CMD_HSPLIT)
```

- Draws: `C-x:...` in normal mode
- On Ctrl+X: replaces status bar content with children
- Children are CommandKeys (or nested ModalKeys for cascading prefixes)
- Esc returns to parent level
- Multi-key sequences = nested ModalKeys

### Custom Teclynnau

Any teclyn can live in the status bar:

- **Clock** — shows time, updates on tick
- **EditorStatus** — "Ln 42, Col 7", listens to cursor commands
- **GitBranch** — shows branch name, waker-driven
- **MessageArea** — shows latest toast, auto-dismisses
- **InputLine** — command line (`:` in vim, `M-x` in emacs)

## Structure

```
StatusBar (Group, horizontal PackLayout with gravity)
├── LeftGroup (gravity: Start)
│   ├── CommandKey("q:Quit")
│   ├── ModalKey("C-x:...")
│   └── CommandKey("Esc:Tree")
├── CenterGroup (gravity: Center)
│   └── MessageArea / InputLine (toggled by ModalKey)
└── RightGroup (gravity: End)
    ├── GitBranch
    └── Clock
```

## Modal Behavior

When a ModalKey is activated:
1. Its children become visible (replace or overlay the current group)
2. The status bar shows available continuations
3. User presses a child key → command executes, modal closes
4. User presses Esc → modal closes, normal display restored

This is the same mechanism for:
- Emacs `C-x` prefix (ModalKey with file/buffer commands)
- Vim `:` command mode (ModalKey that activates an InputLine)
- WordStar `C-k` block prefix (ModalKey with block commands)

## Command Line (InputLine in Status Bar)

The command line is an InputLine that lives in the center group.
It starts inactive (hidden). A ModalKey activates it:

```
ModalKey("::cmd", ':', CMD_ACTIVATE_CMDLINE)
```

When activated:
- InputLine becomes visible and active
- InputLine gets focus
- User types command, Tab completes, Up/Down cycles history
- Enter submits → command executes, InputLine deactivates
- Esc cancels → InputLine deactivates

The InputLine has its own micro-bindings (Tab, Enter, arrows) that work
while it's focused. These don't conflict with the status bar CommandKeys
because the InputLine consumes them in Normal phase (after Pre-phase
CommandKeys have had their chance).

## Messages / Toast

Three layers:
1. **Status bar center** — latest message, shown for N seconds
2. **Toast sidekick** — opens when messages stack up, shows recent N
3. **Message viewer** — full ring buffer in a panel, opened on demand

Messages carry timestamp. Toast shows "2 min ago". Viewer shows "14:32:07".
App can also write to its own log from the same MsgEntry data.

## Keymap Switching

Switching keymap = rebuilding the status bar. An `AppKeymap` trait provides:
- Widget-local bindings (InputBindings, TreeBindings, ListBindings)
- Status bar construction (which CommandKeys/ModalKeys to add)

```rust
trait AppKeymap {
    fn input_bindings(&self) -> InputBindings;
    fn tree_bindings(&self) -> TreeBindings;
    fn list_bindings(&self) -> ListBindings;
    fn build_status_bar(&self, engine: &mut Engine, parent: NodeId) -> StatusBarIds;
}
```

One trait, one swap, everything stays consistent.

## Key Dispatch Order

1. **Pre-phase** (StatusBar CommandKeys/ModalKeys) — global shortcuts
2. **Normal phase** (focused widget) — widget-local bindings
3. **Post-phase** — fallback handlers

If a CommandKey claims a key, the focused widget never sees it.
Widget-local bindings handle everything else (arrows, Tab, typing).
