# v-007 — txv-widgets Spec: Reusable Interactive Components

## Purpose

txv-widgets provides interactive TUI components that handle input, manage
state, and render to txv surfaces. It sits between the raw rendering layer
(txv) and the application (kairn).

Any Rust TUI application can use txv-widgets for common UI patterns.

## Dependencies

- `txv` — rendering primitives
- `crossterm` — input events (KeyEvent, etc.)

No tokio, no filesystem, no application-specific logic.

## Core abstractions

### Widget trait

```rust
/// An interactive component that can render and handle input.
pub trait Widget {
    /// Render this widget to a surface.
    fn render(&self, surface: &mut txv::Surface, focused: bool);

    /// Handle an input event. Returns what happened.
    fn handle_input(&mut self, event: &InputEvent) -> EventResult;

    /// Whether this widget can receive focus.
    fn focusable(&self) -> bool { true }

    /// Preferred size (width, height). None = flexible.
    fn preferred_size(&self) -> (Option<u16>, Option<u16>) { (None, None) }
}
```

### EventResult

```rust
/// What happened after a widget processed input.
pub enum EventResult {
    /// Widget consumed the event. No further dispatch.
    Consumed,
    /// Widget did not handle the event. Pass to parent.
    Ignored,
    /// Widget produced an action for the application.
    Action(WidgetAction),
}

/// Actions that widgets can produce.
pub enum WidgetAction {
    /// A selection was made (item index or string).
    Selected(String),
    /// User confirmed (dialog OK, input Enter).
    Confirmed(String),
    /// User cancelled (Esc).
    Cancelled,
    /// User requested close.
    Close,
    /// Focus should move to the next widget.
    FocusNext,
    /// Focus should move to the previous widget.
    FocusPrev,
    /// Custom action (application-defined, carried as boxed Any).
    Custom(Box<dyn std::any::Any + Send>),
}
```

### InputEvent

```rust
/// Normalized input event.
pub enum InputEvent {
    Key(crossterm::event::KeyEvent),
    Resize(u16, u16),
    Tick,  // periodic timer tick for animations/polling
}
```

## EventLoop

The central run loop for any txv-widgets application.

```rust
pub struct EventLoop {
    screen: txv::Screen,
    timers: Vec<TimerEntry>,
    pollers: Vec<Box<dyn Pollable>>,
    tick_interval: Duration,
}

/// Something that can be polled for data (PTY output, channels, etc.).
pub trait Pollable: Send {
    /// Check for available data. Non-blocking.
    fn poll(&mut self) -> Option<Vec<u8>>;
}

pub type TimerId = u64;

impl EventLoop {
    fn new(screen: txv::Screen) -> Self;

    /// Set the tick interval (default 50ms).
    fn set_tick_interval(&mut self, interval: Duration);

    /// Add a one-shot or repeating timer. Returns an ID for cancellation.
    fn add_timer(
        &mut self,
        delay: Duration,
        repeat: bool,
        callback: Box<dyn FnMut() -> bool>,  // return false to cancel
    ) -> TimerId;

    /// Cancel a timer.
    fn cancel_timer(&mut self, id: TimerId);

    /// Add a pollable data source.
    fn add_poller(&mut self, poller: Box<dyn Pollable>);

    /// Run the event loop. Calls the provided closure on each iteration
    /// with the screen, input events, and polled data.
    fn run<F>(&mut self, handler: F) -> Result<()>
    where
        F: FnMut(&mut RunContext) -> LoopControl;
}

pub struct RunContext<'a> {
    pub screen: &'a mut txv::Screen,
    pub events: &'a [InputEvent],
    pub poll_data: &'a [(usize, Vec<u8>)],  // (poller_index, data)
    pub timers: &'a mut TimerManager,
}

pub enum LoopControl {
    Continue,
    Quit,
}
```

The loop cycle:
1. Poll crossterm for input events (with timeout = tick_interval)
2. Fire expired timers
3. Poll all Pollable sources
4. Call handler with collected events/data
5. Handler renders to screen surfaces
6. Flush screen

## Widgets

### ScrollView

Virtual content area larger than the visible surface. Manages scroll
offset and renders a viewport.

```rust
pub struct ScrollView {
    scroll_row: usize,
    scroll_col: usize,
    content_height: usize,
    content_width: usize,
}

impl ScrollView {
    fn scroll_to(&mut self, row: usize, col: usize);
    fn ensure_visible(&mut self, row: usize, col: usize, viewport: (u16, u16));
    fn page_up(&mut self, viewport_height: u16);
    fn page_down(&mut self, viewport_height: u16);
    fn visible_range(&self, viewport_height: u16) -> Range<usize>;
}
```

Not a full Widget — it's a helper that other widgets embed.

### ListView

Scrollable list with single selection.

```rust
pub trait ListData {
    fn len(&self) -> usize;
    fn render_item(&self, index: usize, surface: &mut txv::Surface, selected: bool);
}

pub struct ListView<D: ListData> {
    data: D,
    selected: usize,
    scroll: ScrollView,
}
```

Handles: Up/Down/PgUp/PgDn/Home/End for navigation, Enter for selection,
typing for incremental search (optional).

### TreeView

Expandable/collapsible tree with cursor.

```rust
pub trait TreeData {
    type NodeId: Clone + Eq;
    fn root_nodes(&self) -> Vec<Self::NodeId>;
    fn children(&self, id: &Self::NodeId) -> Vec<Self::NodeId>;
    fn has_children(&self, id: &Self::NodeId) -> bool;
    fn render_node(
        &self,
        id: &Self::NodeId,
        surface: &mut txv::Surface,
        depth: usize,
        expanded: bool,
        selected: bool,
    );
}

pub struct TreeView<D: TreeData> {
    data: D,
    expanded: HashSet<D::NodeId>,
    cursor: D::NodeId,
    scroll: ScrollView,
    flat_cache: Vec<(D::NodeId, usize)>,  // (id, depth) for visible nodes
}
```

Handles: Up/Down for cursor, Right to expand, Left to collapse/go to parent,
Enter for selection.

### InputLine

Single-line text input with cursor, history, and optional completion.

```rust
pub struct InputLine {
    text: String,
    cursor: usize,       // byte offset
    history: Vec<String>,
    history_pos: Option<usize>,
    prompt: String,
}

impl InputLine {
    fn text(&self) -> &str;
    fn set_text(&mut self, text: &str);
    fn clear(&mut self);
}
```

Handles: Left/Right, Home/End, Backspace/Delete, Up/Down for history,
Enter to confirm, Esc to cancel. Emacs-style shortcuts: Ctrl-A/E/K/U/W.

### TabBar

Horizontal tab strip.

```rust
pub struct TabBar {
    tabs: Vec<TabEntry>,
    active: usize,
}

pub struct TabEntry {
    pub title: String,
    pub modified: bool,
    pub closeable: bool,
}

impl TabBar {
    fn add(&mut self, entry: TabEntry);
    fn remove(&mut self, index: usize);
    fn active(&self) -> usize;
    fn set_active(&mut self, index: usize);
}
```

Renders as: ` ▸tab1  tab2  tab3 ` with active tab highlighted.
Handles: click (future), keyboard switching via parent.

### StatusBar

Left/right aligned spans with section separators.

```rust
pub struct StatusBar {
    left: Vec<StatusSpan>,
    right: Vec<StatusSpan>,
}

pub struct StatusSpan {
    pub text: String,
    pub style: txv::Style,
}

impl StatusBar {
    fn set_left(&mut self, spans: Vec<StatusSpan>);
    fn set_right(&mut self, spans: Vec<StatusSpan>);
}
```

Renders left spans left-aligned, right spans right-aligned, fills middle
with background color.

### Dialog

Modal overlay with title, message, and buttons.

```rust
pub enum DialogKind {
    /// Message with OK button.
    Info,
    /// Yes/No confirmation.
    Confirm,
    /// Text input prompt.
    Prompt { default: String },
}

pub struct Dialog {
    title: String,
    message: String,
    kind: DialogKind,
    input: Option<InputLine>,  // for Prompt kind
    selected_button: usize,
}
```

Renders centered on screen. Handles Enter/Esc/Tab (between buttons).
Produces Confirmed/Cancelled actions.

### Notification

Flash message that auto-dismisses.

```rust
pub struct Notification {
    message: String,
    style: txv::Style,
    remaining_ms: u64,
}

impl Notification {
    fn new(message: String, style: txv::Style, duration_ms: u64) -> Self;
    fn tick(&mut self, elapsed_ms: u64) -> bool;  // returns false when expired
}
```

Rendered by the application in the status bar area or as an overlay.

### Overlay

Positioned popup container. Wraps another widget and positions it
relative to an anchor point or centered on screen.

```rust
pub enum Anchor {
    Center,
    Below(u16, u16),   // anchor col, row — popup appears below
    Above(u16, u16),   // anchor col, row — popup appears above
}

pub struct Overlay<W: Widget> {
    inner: W,
    anchor: Anchor,
    width: u16,
    height: u16,
}
```

Renders: clears the overlay area, draws a border, renders the inner widget
inside. Handles input by delegating to inner widget.

### FuzzySelect

Input line + filtered list combo. Used for command palette, file picker.

```rust
pub struct FuzzySelect {
    input: InputLine,
    items: Vec<String>,
    filtered: Vec<(usize, u32)>,  // (original_index, score)
    selected: usize,
    max_visible: usize,
}

impl FuzzySelect {
    fn new(items: Vec<String>) -> Self;
    fn selected_item(&self) -> Option<&str>;
}
```

Handles: typing filters the list, Up/Down selects, Enter confirms, Esc
cancels. Scoring via simple substring match (or nucleo if available as
optional dependency).

## Build order

Widgets are built incrementally as kairn needs them:

1. **Core**: Widget trait, EventResult, EventLoop — needed immediately
2. **First batch**: ScrollView, StatusBar, InputLine — needed for basic editor
3. **Second batch**: ListView, TabBar — needed for bottom panel
4. **Third batch**: TreeView — needed for file tree
5. **Fourth batch**: Dialog, Notification, Overlay, FuzzySelect — needed for overlays

Each widget is independently testable via cell grid assertions.
