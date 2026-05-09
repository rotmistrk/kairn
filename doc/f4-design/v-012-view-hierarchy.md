# v-012 — View Hierarchy Architecture (TXV Model)

## Principle

The application is a **tree of Views**. Each View:
- Owns a rectangular region of the screen
- Draws itself into that region
- Handles events it understands, ignores the rest
- Knows NOTHING about its siblings or parent's other children

A **Group** is a View that contains child Views. It:
- Dispatches events to children (focused child first, then others)
- Manages focus (which child is active)
- Lays out children within its bounds
- Knows NOTHING about what its children ARE

The Application is just the top-level Group.

## From TXV C++ to Rust

| TXV (C++) | Rust equivalent |
|-----------|----------------|
| `View` | `trait View` |
| `Group` | `struct Group` (implements `View`, owns `Vec<Box<dyn View>>`) |
| `Event` | `enum Event` |
| `TextBuff` | `txv::Surface` |
| `View::draw()` | `View::draw(&self, surface)` |
| `View::handle(Event&)` | `View::handle(&mut self, event) -> HandleResult` |
| `View::changeBounds(Rect&)` | `View::set_bounds(Rect)` |
| `Group::current` | `Group::focused: usize` |
| `Group::forEach(handle)` | iterate children, dispatch event |
| `Program` | `App` (a Group with event loop) |
| `clearEvent()` | return `HandleResult::Consumed` |

## Core trait

```rust
/// A rectangular UI element that can draw and handle events.
pub trait View: Send {
    /// Draw this view into the given surface.
    /// The surface is exactly this view's bounds — no need to offset.
    fn draw(&self, surface: &mut Surface, ctx: &DrawContext);

    /// Handle an event. Return whether it was consumed.
    fn handle(&mut self, event: &Event) -> HandleResult;

    /// This view's bounds (position + size) within its parent.
    fn bounds(&self) -> Rect;

    /// Change this view's bounds (called by parent during layout).
    fn set_bounds(&mut self, rect: Rect);

    /// Whether this view can receive focus.
    fn focusable(&self) -> bool { true }

    /// Grow flags: how this view resizes when parent resizes.
    fn grow_flags(&self) -> GrowFlags { GrowFlags::NONE }
}
```

## Events

```rust
/// An event that flows through the view tree.
pub enum Event {
    /// Keyboard input.
    Key(KeyEvent),
    /// Terminal resized.
    Resize(u16, u16),
    /// Command (from menu, keybinding, or another view).
    Command(CommandId),
    /// Timer tick.
    Tick,
    /// Data available from an async source (PTY, LSP, etc.)
    Data { source_id: usize, payload: Vec<u8> },
}

/// Result of handling an event.
pub enum HandleResult {
    /// Event was consumed — stop dispatching.
    Consumed,
    /// Event was not handled — continue dispatching to siblings.
    Ignored,
}

/// A command identifier. Views communicate via commands, not by
/// knowing about each other.
pub type CommandId = u16;
```

### Command-based communication

Views never call methods on siblings. Instead:
- A view handles an event and produces a **command** (via `put_event`)
- The command bubbles up to the nearest Group that handles it
- That Group dispatches it to the appropriate child

Example: Tree selects a file → emits `Command(cmOpenFile)` with path.
The parent Group handles `cmOpenFile` and tells the editor child to open it.
The tree never knows the editor exists.

```rust
/// Well-known commands.
pub mod commands {
    pub const CM_QUIT: CommandId = 1;
    pub const CM_OPEN_FILE: CommandId = 2;
    pub const CM_SAVE: CommandId = 3;
    pub const CM_CLOSE: CommandId = 4;
    pub const CM_FOCUS_NEXT: CommandId = 5;
    pub const CM_FOCUS_PREV: CommandId = 6;
    pub const CM_RESIZE_LEFT: CommandId = 10;
    pub const CM_RESIZE_RIGHT: CommandId = 11;
    pub const CM_RESIZE_UP: CommandId = 12;
    pub const CM_RESIZE_DOWN: CommandId = 13;
    // ... etc
}
```

## Group

```rust
/// A View that contains and manages child Views.
pub struct Group {
    bounds: Rect,
    children: Vec<Box<dyn View>>,
    focused: usize,
    layout: Layout,
    /// Outgoing event queue (commands produced by children).
    outbox: Vec<Event>,
}

impl Group {
    pub fn new(layout: Layout) -> Self;
    pub fn add(&mut self, child: Box<dyn View>);
    pub fn remove(&mut self, index: usize);
    pub fn focus_next(&mut self);
    pub fn focus_prev(&mut self);
    pub fn focused_child(&self) -> Option<&dyn View>;
}

impl View for Group {
    fn draw(&self, surface: &mut Surface, ctx: &DrawContext) {
        let rects = self.layout.compute(self.bounds, &self.children);
        for (i, child) in self.children.iter().enumerate() {
            let child_surface = surface.sub(rects[i]);
            child.draw(&mut child_surface, ctx);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // 1. Try focused child first
        if let Some(child) = self.children.get_mut(self.focused) {
            if child.handle(event) == HandleResult::Consumed {
                return HandleResult::Consumed;
            }
        }
        // 2. Try other children (for pre/post processing)
        for (i, child) in self.children.iter_mut().enumerate() {
            if i == self.focused { continue; }
            if child.handle(event) == HandleResult::Consumed {
                return HandleResult::Consumed;
            }
        }
        // 3. Handle ourselves (group-level commands)
        self.handle_self(event)
    }

    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, rect: Rect) {
        self.bounds = rect;
        self.relayout();
    }
}
```

## Layout

```rust
/// How a Group arranges its children.
pub enum Layout {
    /// Split horizontally or vertically with constraints.
    Split {
        direction: Direction,
        constraints: Vec<Constraint>,
    },
    /// Stack: only one child visible at a time (tabs).
    Stack,
    /// Manual: children have fixed positions (dialogs, overlays).
    Manual,
}
```

## DrawContext

```rust
/// Shared context passed during drawing.
pub struct DrawContext {
    /// Whether the application is focused (vs another terminal tab).
    pub app_focused: bool,
    /// Current color theme.
    pub theme: &Theme,
    /// Tick counter (for animations like cursor blink).
    pub tick: u64,
}
```

## Application

The Application is just a Group with an event loop:

```rust
pub struct App {
    root: Group,       // contains: editor_group, bottom_panel, status_bar
    screen: Screen,
    running: bool,
}

impl App {
    pub fn run(&mut self) {
        loop {
            // 1. Collect events (keyboard, resize, data)
            let events = collect_events();
            // 2. Dispatch each event to root group
            for event in events {
                self.root.handle(&event);
            }
            // 3. Draw
            let mut surface = self.screen.full_surface();
            self.root.draw(&mut surface, &self.draw_ctx());
            // 4. Flush
            self.screen.flush(&mut stdout);
            // 5. Check quit
            if !self.running { break; }
        }
    }
}
```

The App does NOT know what's inside `root`. It could be an editor, a
file manager, a dashboard — doesn't matter. It just dispatches and draws.

## Application structure

The App owns exactly two things: a Desktop (any View) and a StatusBar.

```rust
struct App {
    desktop: Box<dyn View>,  // SlottedDesktop OR FlatDesktop
    status_bar: StatusBarView,
    running: bool,
}
```

The App does NOT know which desktop implementation is in use.
Switching from slotted to flat layout changes ONE constructor call.

### SlottedDesktop (kairn default)

Tiled layout with 4 named slots, each containing tabs:

```
┌──────────┬────────────────────────────┬──────────────┐
│ left     │ center                     │ right        │
│ [tabs]   │ [tabs]                     │ [tabs]       │
├──────────┴────────────────────────────┴──────────────┤
│ bottom [tabs]                                        │
└──────────────────────────────────────────────────────┘
```

- 4 slots: left, center, right, bottom
- Each slot has tabs (stack of views, one visible)
- Slots can be hidden/shown, resized
- Zoom: focused slot expands to fill all space, unzoom restores
- Handles: focus cycling, tab switching, resize, zoom, layout modes

### FlatDesktop (traditional F4/TXV style)

Floating windows, manually positioned:

- Children are windows with position + size
- Windows can overlap, be moved, resized, cascaded, tiled
- Z-order: focused window on top
- Handles: focus cycling, window move/resize, cascade, tile

### Both implement View

```rust
impl View for SlottedDesktop { ... }
impl View for FlatDesktop { ... }
```

The App calls `desktop.handle()` and `desktop.draw()`. It never
inspects the desktop's internals. It only creates views and sends
commands to insert them:

```rust
impl App {
    fn handle_command(&mut self, cmd: CommandId, data: ...) {
        match cmd {
            CM_OPEN_FILE => {
                let view = EditorView::open(path);
                self.desktop.handle(&Event::Command(
                    CM_INSERT_VIEW, Some(Box::new(view))
                ));
            }
            CM_QUIT => self.running = false,
            _ => {} // everything else handled by desktop/status
        }
    }
}
```

### SlottedDesktop details

```rust
struct SlottedDesktop {
    slots: [Slot; 4],       // Left, Center, Right, Bottom
    focused: SlotId,
    zoomed: Option<SlotId>,
    layout: DesktopLayout,
}

struct Slot {
    tabs: Vec<Box<dyn View>>,
    active_tab: usize,
    visible: bool,
    size: u16,
}

enum SlotId { Left, Center, Right, Bottom }

enum DesktopLayout { Standard, Compact }
```

Commands handled by SlottedDesktop:

| Command | Action |
|---------|--------|
| CM_ZOOM_TOGGLE | Zoom/unzoom focused slot |
| CM_FOCUS_LEFT/CENTER/RIGHT/BOTTOM | Direct focus |
| CM_FOCUS_NEXT_SLOT / CM_FOCUS_PREV_SLOT | Cycle |
| CM_TAB_NEXT / CM_TAB_PREV | Cycle tabs in focused slot |
| CM_TAB_CLOSE | Close current tab |
| CM_SLOT_GROW / CM_SLOT_SHRINK | Resize focused slot |
| CM_SLOT_TOGGLE(id) | Show/hide a slot |
| CM_CYCLE_LAYOUT | Switch Standard/Compact |
| CM_INSERT_VIEW(slot_id, view) | Add view as tab in slot |

## What changes in txv-widgets

The current `Widget` trait becomes `View`. The current `FocusGroup` becomes
`Group`. Key changes:

1. **Add `bounds()` and `set_bounds()` to the trait** — views know their size
2. **Add `Layout` to Group** — Group computes child rects from constraints
3. **Replace `EventResult`/`WidgetAction` with `HandleResult` + command events**
4. **Group dispatches to focused child first, then others**
5. **Remove all `WidgetAction` variants** — views communicate via commands only

Individual widgets (TreeView, ListView, InputLine, etc.) stay mostly the
same — they just implement the new `View` trait instead of `Widget`.

## What this means for kairn

kairn's `src/` becomes:

```
src/
├── main.rs          — create App, run event loop
├── views/
│   ├── tree.rs      — TreeView (wraps txv-widgets TreeView + FileTreeData)
│   ├── editor.rs    — EditorView (piece table + keymap + highlighting)
│   ├── terminal.rs  — TerminalView (PTY + TermBuf)
│   ├── errors.rs    — ErrorListView
│   ├── search.rs    — SearchResultsView
│   ├── status.rs    — StatusBarView
│   └── control.rs   — ControlView (outline, diagnostics)
├── commands.rs      — command ID constants
├── editor/          — piece table, keymaps (pure logic, no UI)
├── config/          — rusticle config loading
└── lsp/             — LSP client (async, feeds data events)
```

Each view file is ~100-300 lines. The App is ~50 lines. Total kairn
application code: ~2000-3000 lines (vs the 19K mess we just deleted).

## Rules

1. **A View never references another View.** Communication is via commands.
2. **A Group never inspects its children's types.** It holds `Box<dyn View>`.
3. **The App never knows what views exist.** It just runs the event loop.
4. **Layout is declarative.** Groups specify constraints, not pixel positions.
5. **Events flow down (dispatch) and up (bubbling).** Never sideways.

## StatusLine role (from TXV)

The StatusLine is NOT just a display. It is an **active key-to-command
translator**. In TXV:

1. StatusLine has a list of `(key, command_id, label)` items
2. It sees key events BEFORE other views (via Program::getEvent)
3. When a key matches, it emits the command and consumes the key
4. It also displays the help text of the currently focused view

In our architecture:
- StatusBarView is inserted as the LAST child of the root Group
- But it has `preprocess: true` flag — it sees events before siblings
- It translates configured keybindings into commands
- It renders: key hints on the left, context help on the right

This means the keybinding system lives in the StatusBar, not in the App.
The App has zero knowledge of keybindings.

```rust
struct StatusBarView {
    bindings: Vec<(KeySpec, CommandId, String)>,  // key, command, label
    help_text: String,  // set by the focused view
}

impl View for StatusBarView {
    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Key(key) = event {
            for (spec, cmd, _) in &self.bindings {
                if spec.matches(key) {
                    // Emit command — parent Group will dispatch it
                    return HandleResult::Command(*cmd);
                }
            }
        }
        HandleResult::Ignored
    }
}
```

Wait — this means HandleResult needs a Command variant:

```rust
pub enum HandleResult {
    Consumed,
    Ignored,
    /// View produced a command to be dispatched by the parent.
    Emit(CommandId, Option<Box<dyn Any + Send>>),
}
```

When a view returns `Emit(cmd, data)`, the parent Group dispatches that
command as a new event to all children (including itself).

## Menu role (from TXV)

The MenuBar is a **modal View**. When activated:
1. It takes over the event loop (like a dialog)
2. User navigates menus, selects an item
3. Selected item produces a command
4. Menu closes, command is dispatched normally

In our architecture:
- MenuBar is a View that can enter modal mode
- Modal mode = it captures all events until dismissed
- This is handled by Group: when a child is modal, only that child
  receives events

```rust
pub trait View: Send {
    // ... existing methods ...

    /// Whether this view is currently modal (captures all events).
    fn is_modal(&self) -> bool { false }
}
```

Group dispatch changes when a child is modal:
- If any child is modal, ONLY that child receives events
- Other children are skipped entirely

## Help role (from TXV)

Help is just a Window (a View with a frame and title). It is:
- Created when `cmHelp` command is received
- Inserted into the desktop Group as a new child
- Focused (brought to front)
- Closed when user presses Esc or closes it

No special wiring. The App handles `cmHelp` by creating a HelpView
and inserting it into the appropriate Group. This is the ONLY place
the App knows about specific view types — when creating them in
response to commands.

## Revised event dispatch in Group

```
Group::handle(event):
    1. If any child is modal → dispatch only to that child
    2. Otherwise:
       a. Children with preprocess flag see event first
       b. Focused child sees event
       c. Children with postprocess flag see event last
    3. If child returns Emit(cmd, data):
       → Create Event::Command(cmd, data)
       → Dispatch to all children (including self)
    4. Group handles commands it knows (CM_FOCUS_NEXT, etc.)
```
