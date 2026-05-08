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

## View tree for kairn

```
App (Group, Layout::Split vertical)
├── EditorGroup (Group, Layout::Split horizontal)
│   ├── TreeView (leaf View — file tree)
│   ├── EditorView (leaf View — text editor)
│   └── ControlView (leaf View — outline/diagnostics)
├── BottomGroup (Group, Layout::Stack)
│   ├── TerminalView (leaf View — PTY terminal)
│   ├── ErrorListView (leaf View — compiler errors)
│   ├── SearchResultsView (leaf View — grep results)
│   └── TestResultsView (leaf View — test output)
└── StatusBarView (leaf View — status line, not focusable)
```

Each leaf View is self-contained:
- `TreeView` handles j/k/Enter, draws the tree, emits `cmOpenFile`
- `EditorView` handles vim/emacs keys, draws the buffer, emits `cmSave`
- `TerminalView` handles all keys (forwards to PTY), draws TermBuf
- `StatusBarView` is not focusable, just renders status info

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
