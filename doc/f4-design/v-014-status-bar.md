# v-014: StatusBar Architecture

## Intent

StatusBar is a composable container of typed items. Apps configure it by
adding items — no subclassing, no monolithic wrappers. Each item is a small
focused object that handles one concern.

## Traits (txv-core)

```rust
/// An item that translates events into commands.
/// Does NOT have to be visible (e.g., a pure hotkey binding).
pub trait ActiveItem {
    /// Handle an event. Return Consumed if the item handled it.
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult;
}

/// An item that renders a label on the status bar.
/// Does NOT have to handle events (e.g., a passive display).
pub trait VisibleItem {
    fn label(&self) -> &str;
    fn gravity(&self) -> Gravity;
    /// Called on every tick/event so the item can update its label.
    fn tick(&mut self) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gravity { Left, Right }
```

Items can implement one or both traits.

## StatusBar Container (txv-core)

```rust
pub struct StatusBar {
    items: Vec<Box<dyn StatusBarItem>>,  // trait object combining both
    exclusive: Option<usize>,            // index of item in exclusive mode
    bounds: Rect,
    dirty: bool,
}

/// Combined trait for items that are both active and visible.
/// Items implementing only one trait use adapter wrappers.
pub trait StatusBarItem: ActiveItem + VisibleItem {}

impl StatusBar {
    fn add(&mut self, item: impl StatusBarItem + 'static);
    fn add_active_only(&mut self, item: impl ActiveItem + 'static);
    fn add_visible_only(&mut self, item: impl VisibleItem + 'static);
}
```

StatusBar implements `View` with `preprocess: true`.

### Event routing

1. If an item is in exclusive mode → only that item gets events
2. Otherwise → all ActiveItems get the event (first match wins)

### Rendering

1. If exclusive → render only that item (full width)
2. Otherwise → lay out VisibleItems: left-gravity from left, right-gravity from right, gap in middle

### Exclusive mode

An item signals exclusive mode by returning a special HandleResult or
calling a method on a context. When exclusive ends, normal layout resumes.

Use case: CommandItem activates on M-x, takes full bar for input, deactivates on Enter/Esc.

## Item Implementations (txv-widgets)

### KeyLabelItem (Active + Visible)
- Translates one key → one command
- Label: "F1:Help"
- Config: key, command, label

### ClockItem (Visible only)
- Label: "HH:MM"
- Updates on tick when interval elapsed
- Config: interval_seconds

### CommandItem (Active + Visible, exclusive when active)
- Dormant: zero-width, invisible
- Activated by key (M-x, :, ≈) → enters exclusive mode
- Renders ": " + input line with cursor
- Supports Tab completion (takes a Completer)
- Emits CM_EXECUTE_COMMAND on Enter, deactivates on Esc

### MessageItem (Visible only)
- Shows last message, clears after timeout
- Updated via CM_STATUS_MESSAGE command
- Config: timeout_seconds (0 = permanent until next message)

### ModeItem (Visible only)
- Shows editor mode: NORMAL, INSERT, VISUAL
- Updated via CM_MODE_CHANGED command

### PositionItem (Visible only)
- Shows "Ln 42, Col 7"
- Updated via CM_CURSOR_MOVED command

### FileInfoItem (Visible only)
- Shows "[+]" (modified), filetype, encoding
- Updated via CM_FILE_INFO_CHANGED command

## Future items (not implemented, justify design)

- **BranchItem** — git branch from working dir
- **LoadAvgItem** — system load (useful over SSH)
- **SpinnerItem** — activity indicator during async ops
- **DiagnosticsItem** — "⚠3 ✗1" from LSP
- **TimerItem** — pomodoro/stopwatch
- **NetworkItem** — connection status
- **BatteryItem** — laptop battery level
- **MailItem** — unread mail count

## Kairn Configuration Example

```rust
let mut bar = StatusBar::new();
// Left side: key labels
bar.add(KeyLabelItem::new(F1, CM_SHOW_HELP, "F1:Help"));
bar.add(KeyLabelItem::new(F2, CM_FOCUS_LEFT, "F2:Tree"));
bar.add(KeyLabelItem::new(F3, CM_FOCUS_CENTER, "F3:Main"));
bar.add(KeyLabelItem::new(F4, CM_FOCUS_RIGHT, "F4:Term"));
bar.add(KeyLabelItem::new(F5, CM_ZOOM_TOGGLE, "F5:Zoom"));
bar.add(KeyLabelItem::new(CTRL_Q, CM_QUIT, "^Q:Quit"));
// Hidden hotkeys (active only, no label)
bar.add_active_only(KeyLabelItem::hidden(CTRL_SHIFT_LEFT, CM_FOCUS_PREV));
bar.add_active_only(KeyLabelItem::hidden(CTRL_SHIFT_RIGHT, CM_FOCUS_NEXT));
bar.add_active_only(KeyLabelItem::hidden(CTRL_SHIFT_UP, CM_TAB_DROPDOWN));
// Command input (exclusive mode on activation)
bar.add(CommandItem::new(&[ALT_X, COLON, APPROX]));
// Right side: passive displays
bar.add(ModeItem::new().gravity(Gravity::Right));
bar.add(PositionItem::new().gravity(Gravity::Right));
bar.add(MessageItem::new(5).gravity(Gravity::Right));
bar.add(ClockItem::new(60).gravity(Gravity::Right));
```

No KairnStatusBar. No subclassing. Pure composition.

## Event Flow

```
Key pressed
  → StatusBar.handle() [preprocess phase]
    → if exclusive: exclusive_item.handle()
    → else: iterate ActiveItems, first Consumed wins
      → KeyLabelItem matches F1 → emits CM_SHOW_HELP
      → CommandItem matches M-x → enters exclusive mode

Editor moves cursor
  → emits CM_CURSOR_MOVED { line: 42, col: 7 }
  → StatusBar receives in handle()
    → PositionItem.handle() updates its label

Tick arrives
  → StatusBar calls tick() on all VisibleItems
    → ClockItem checks elapsed, updates label
    → MessageItem checks timeout, clears label
```

## Migration Plan

1. Add traits (ActiveItem, VisibleItem, Gravity) to txv-core
2. Rewrite StatusBar in txv-core as the container
3. Implement items in txv-widgets (KeyLabel, Clock, Command, Message)
4. Replace KairnStatusBar with composed StatusBar in kairn
5. Add ModeItem + PositionItem, wire editor to emit update commands
