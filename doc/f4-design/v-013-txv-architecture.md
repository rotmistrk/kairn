# v-013 — TXV Architecture: Core / Render / Widgets

## Principle

The TUI framework is split into three crates with strict dependency rules:

```
txv-core        Pure logic. Zero I/O. Defines the rules.
txv-render      Terminal I/O. Implements the backend trait from core.
txv-widgets     Concrete Views. Only depends on txv-core.
```

```
txv-widgets ──→ txv-core
txv-render  ──→ txv-core
kairn       ──→ txv-core + txv-widgets + txv-render (wiring only)
```

The backend is swappable:
- `txv-render` (crossterm terminal)
- `txv-render-x11` (future: graphical)
- `txv-render-wasm` (future: browser)
- `txv-render-test` (mock for unit tests)

Application startup is the ONLY place that touches the render crate.

## txv-core

**Zero external dependencies.** Pure Rust, no I/O, no async, no crossterm.

### Types

```rust
// ── Geometry ──

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Point { pub x: u16, pub y: u16 }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect { pub x: u16, pub y: u16, pub w: u16, pub h: u16 }

// ── Cells and styles ──

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color { Reset, Ansi(u8), Palette(u8), Rgb(u8, u8, u8) }

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Attrs {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
    pub dim: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Style { pub fg: Color, pub bg: Color, pub attrs: Attrs }

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cell { pub ch: char, pub style: Style, pub width: u8 }

// ── Surface (abstract drawing target) ──

pub struct Surface {
    cells: Vec<Cell>,
    width: u16,
    height: u16,
}

impl Surface {
    pub fn new(w: u16, h: u16) -> Self;
    pub fn put(&mut self, x: u16, y: u16, ch: char, style: Style);
    pub fn print(&mut self, x: u16, y: u16, text: &str, style: Style);
    pub fn fill(&mut self, ch: char, style: Style);
    pub fn hline(&mut self, x: u16, y: u16, len: u16, ch: char, style: Style);
    pub fn vline(&mut self, x: u16, y: u16, len: u16, ch: char, style: Style);
    pub fn sub(&mut self, x: u16, y: u16, w: u16, h: u16) -> SubSurface;
    pub fn cell(&self, x: u16, y: u16) -> &Cell;
    pub fn width(&self) -> u16;
    pub fn height(&self) -> u16;
}
```

### Events

```rust
/// Key modifiers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KeyMod {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

/// A key event.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyMod,
}

/// Key codes (terminal-independent).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum KeyCode {
    Char(char),
    F(u8),
    Enter, Esc, Tab, BackTab,
    Backspace, Delete,
    Left, Right, Up, Down,
    Home, End, PageUp, PageDown,
    Insert,
}

/// Command identifier.
pub type CommandId = u16;

/// An event flowing through the view tree.
#[derive(Debug)]
pub enum Event {
    /// Keyboard input.
    Key(KeyEvent),
    /// Terminal/window resized.
    Resize(u16, u16),
    /// Command (from status bar, menu, or another view).
    Command { id: CommandId, data: Option<Box<dyn std::any::Any + Send>> },
    /// Idle tick.
    Tick,
}

/// Event categories for pre/post processing.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventCategory {
    /// Keyboard events.
    Key,
    /// Command events.
    Command,
    /// All events.
    All,
}
```

### View trait

```rust
/// Options flags (like TXV's ofPreProcess, ofPostProcess).
#[derive(Clone, Copy, Default)]
pub struct ViewOptions {
    /// This view sees events BEFORE the focused view.
    pub preprocess: bool,
    /// This view sees events AFTER the focused view.
    pub postprocess: bool,
    /// This view can receive focus.
    pub focusable: bool,
    /// This view is modal (captures all events when active).
    pub modal: bool,
}

/// Result of handling an event.
pub enum HandleResult {
    /// Event consumed. Stop dispatching.
    Consumed,
    /// Event not handled. Continue dispatching.
    Ignored,
    /// View emits a command. Parent dispatches it as a new Event::Command.
    PutEvent { id: CommandId, data: Option<Box<dyn std::any::Any + Send>> },
}

/// A rectangular UI element.
pub trait View: Send {
    /// Draw into the given surface.
    fn draw(&self, surface: &mut Surface);

    /// Handle an event.
    fn handle(&mut self, event: &Event) -> HandleResult;

    /// View options (preprocess, postprocess, focusable, modal).
    fn options(&self) -> ViewOptions {
        ViewOptions { focusable: true, ..ViewOptions::default() }
    }

    /// Display title (for tab bars, window titles).
    fn title(&self) -> &str { "" }
}
```

### Group

```rust
/// A View that contains and manages child Views.
pub struct Group {
    children: Vec<Box<dyn View>>,
    focused: usize,
    /// Events emitted by children, to be dispatched next cycle.
    event_queue: Vec<Event>,
}

impl Group {
    pub fn new() -> Self;
    pub fn insert(&mut self, view: Box<dyn View>);
    pub fn remove(&mut self, index: usize);
    pub fn focus_next(&mut self);
    pub fn focus_prev(&mut self);
    pub fn focused(&self) -> usize;
    pub fn child_count(&self) -> usize;
    pub fn get(&self, index: usize) -> Option<&dyn View>;
    pub fn get_mut(&mut self, index: usize) -> Option<&mut dyn View>;
}

impl View for Group {
    fn draw(&self, surface: &mut Surface) {
        // Subclass responsibility — Group alone doesn't know layout.
        // Concrete groups (SlottedDesktop, FlatDesktop) override this.
        // Default: draw only focused child into full surface.
        if let Some(child) = self.children.get(self.focused) {
            child.draw(surface);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // THREE-PHASE DISPATCH (TXV model):

        // Phase 1: Preprocess — views with preprocess flag see event first.
        for child in &mut self.children {
            if child.options().preprocess {
                match child.handle(event) {
                    HandleResult::Consumed => return HandleResult::Consumed,
                    HandleResult::PutEvent { id, data } => {
                        self.event_queue.push(Event::Command { id, data });
                        return HandleResult::Consumed;
                    }
                    HandleResult::Ignored => {}
                }
            }
        }

        // Phase 2: Focused child (or modal child if any).
        let target = self.modal_child().unwrap_or(self.focused);
        if let Some(child) = self.children.get_mut(target) {
            match child.handle(event) {
                HandleResult::Consumed => return HandleResult::Consumed,
                HandleResult::PutEvent { id, data } => {
                    self.event_queue.push(Event::Command { id, data });
                    return HandleResult::Consumed;
                }
                HandleResult::Ignored => {}
            }
        }

        // Phase 3: Postprocess — views with postprocess flag see event last.
        for child in &mut self.children {
            if child.options().postprocess {
                match child.handle(event) {
                    HandleResult::Consumed => return HandleResult::Consumed,
                    HandleResult::PutEvent { id, data } => {
                        self.event_queue.push(Event::Command { id, data });
                        return HandleResult::Consumed;
                    }
                    HandleResult::Ignored => {}
                }
            }
        }

        HandleResult::Ignored
    }
}

impl Group {
    /// Drain queued events and dispatch them (called by event loop).
    pub fn dispatch_queued(&mut self) {
        let events: Vec<Event> = self.event_queue.drain(..).collect();
        for event in events {
            self.handle(&event);
        }
    }

    /// Find modal child (if any).
    fn modal_child(&self) -> Option<usize> {
        self.children.iter().position(|c| c.options().modal)
    }
}
```

### Event loop

```rust
/// Backend trait — implemented by txv-render.
pub trait Backend: Send {
    /// Poll for input events. Returns None on timeout.
    fn poll_event(&mut self, timeout: std::time::Duration) -> Option<Event>;
    /// Get current terminal/window size.
    fn size(&self) -> (u16, u16);
    /// Flush a surface to the display.
    fn flush(&mut self, surface: &Surface);
    /// Enter TUI mode (raw mode, alternate screen, etc.)
    fn enter(&mut self);
    /// Leave TUI mode (restore terminal).
    fn leave(&mut self);
}

/// Run the event loop.
pub fn run(root: &mut Group, backend: &mut dyn Backend) {
    backend.enter();
    let (w, h) = backend.size();
    let mut surface = Surface::new(w, h);

    loop {
        // 1. Draw
        surface.fill(' ', Style::default());
        root.draw(&mut surface);
        backend.flush(&surface);

        // 2. Poll event
        let event = backend.poll_event(std::time::Duration::from_millis(50));

        // 3. Handle
        if let Some(event) = event {
            if let Event::Resize(nw, nh) = &event {
                surface = Surface::new(*nw, *nh);
            }
            root.handle(&event);
        }

        // 4. Dispatch queued commands (putEvent results)
        root.dispatch_queued();

        // 5. Tick
        root.handle(&Event::Tick);
    }
}
```

### Well-known commands

```rust
pub mod commands {
    use super::CommandId;
    pub const CM_QUIT: CommandId = 1;
    pub const CM_CLOSE: CommandId = 2;
    pub const CM_FOCUS_NEXT: CommandId = 3;
    pub const CM_FOCUS_PREV: CommandId = 4;
    pub const CM_HELP: CommandId = 5;
    pub const CM_MENU: CommandId = 6;
}
```

Application-specific commands (CM_OPEN_FILE, CM_ZOOM, etc.) are defined
by the application, not by txv-core.

## txv-render

Implements `Backend` for crossterm terminals.

```rust
pub struct CrosstermBackend {
    current: Surface,
    previous: Surface,
    color_mode: ColorMode,
}

impl Backend for CrosstermBackend {
    fn poll_event(&mut self, timeout: Duration) -> Option<Event> {
        // crossterm::event::poll + read, translate to txv_core::Event
    }
    fn size(&self) -> (u16, u16) {
        // crossterm::terminal::size()
    }
    fn flush(&mut self, surface: &Surface) {
        // Diff current vs previous, emit escape sequences
    }
    fn enter(&mut self) {
        // enable_raw_mode, EnterAlternateScreen
    }
    fn leave(&mut self) {
        // disable_raw_mode, LeaveAlternateScreen
    }
}
```

Also provides:
- `TermBuf` — VTE-driven virtual terminal (for embedded terminals)
- Color mode detection and downgrade (RGB → 256 → 16)
- Unicode width handling

Dependencies: `crossterm`, `vte`, `unicode-width`.

## txv-widgets

Concrete View implementations. Depends ONLY on txv-core.

Each widget:
- Implements `View` (draw + handle)
- Uses `Surface` for drawing
- Returns `HandleResult::PutEvent` to emit commands
- Has `ViewOptions` for preprocess/postprocess/focusable

Widgets:
- TreeView, ListView, InputLine, TabBar, StatusBar
- Menu (modal), Dialog (modal)
- TextArea, Table, ProgressBar
- FuzzySelect, Overlay

StatusBar is a View with `preprocess: true` — it translates keys to
commands before anyone else sees them.

Menu is a View with `modal: true` — when active, it captures all events.

## What changes from current code

| Current | New |
|---------|-----|
| txv (cells + surface + screen + layout + termbuf) | Split: Surface/Cell/Style → txv-core, Screen/TermBuf → txv-render |
| txv-widgets (Widget trait + all widgets) | View trait → txv-core, widgets stay in txv-widgets |
| Widget trait (render + handle_key) | View trait (draw + handle + options + title) |
| FocusGroup | Group (in txv-core, three-phase dispatch) |
| EventLoop (in txv-widgets) | run() function in txv-core + Backend trait |
| HandleResult { Consumed, Ignored } | + PutEvent variant |
| No preprocess/postprocess | ViewOptions with preprocess/postprocess/modal |
| crossterm KeyEvent used directly | txv_core::KeyEvent (terminal-independent) |

## File structure

```
txv-core/src/
├── lib.rs
├── cell.rs         — Cell, Color, Attrs, Style
├── surface.rs      — Surface, SubSurface
├── geometry.rs     — Point, Rect
├── event.rs        — Event, KeyEvent, KeyCode, KeyMod, CommandId
├── view.rs         — View trait, ViewOptions, HandleResult
├── group.rs        — Group (three-phase dispatch, event queue)
├── run.rs          — Backend trait, run() function
└── commands.rs     — well-known command IDs

txv-render/src/
├── lib.rs
├── backend.rs      — CrosstermBackend implementing Backend
├── termbuf.rs      — TermBuf (VTE terminal emulator)
├── color.rs        — ColorMode detection, RGB downgrade
└── text.rs         — display_width, truncate (unicode-width)

txv-widgets/src/
├── lib.rs
├── tree_view.rs
├── list_view.rs
├── input_line.rs
├── tab_bar.rs
├── status_bar.rs   — preprocess: true, translates keys → commands
├── menu.rs         — modal: true
├── dialog.rs       — modal: true
├── text_area.rs
├── table.rs
├── fuzzy_select.rs
├── overlay.rs
├── progress_bar.rs
├── scroll_view.rs
├── file_tree.rs    — FileTreeData for TreeView
└── file_list.rs    — FileListData for ListView
```

## Build order

1. **txv-core** — geometry, cell, surface, event, view, group, run, commands (~800-1000 lines)
2. **txv-render** — CrosstermBackend, TermBuf, color, text (~600-800 lines, port from current txv)
3. **txv-widgets** — adapt existing widgets to new View trait (~keep most code, change trait impl)
4. **kairn** — SlottedDesktop (a Group subclass), App (just wiring)

## Testing

txv-core is fully testable without a terminal:
```rust
let mut backend = MockBackend::new(80, 24);
backend.inject_key(KeyCode::Char('j'), KeyMod::default());
txv_core::run_once(&mut root, &mut backend);
assert_eq!(backend.surface().cell(0, 0).ch, 'x');
```

No need for a real terminal in any test.
