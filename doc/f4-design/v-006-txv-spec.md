# v-006 — txv Spec: Rendering Primitives

## Purpose

txv is a terminal rendering library. It owns a cell grid, diffs it between
frames, and emits minimal escape sequences. No widgets, no event loop, no
opinions about application structure.

Any Rust TUI application can use txv as its rendering backend.

## Dependencies

- `crossterm` — escape sequence output, terminal size query, raw mode
- `vte` — ANSI escape sequence parsing (for TermBuf)
- `unicode-width` — character display width (wide chars = 2 cells)

No tokio, no serde, no async.

## Modules

### cell.rs — Cell, Color, Attrs, Span

```rust
/// Terminal color.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Default terminal color.
    Reset,
    /// ANSI 16 colors (0–15).
    Ansi(u8),
    /// 256-color palette (0–255).
    Palette(u8),
    /// 24-bit RGB.
    Rgb(u8, u8, u8),
}

/// Text attributes (bitflags).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Attrs {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
    pub dim: bool,
    pub strikethrough: bool,
}

/// Style = foreground + background + attributes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub attrs: Attrs,
}

/// A single terminal cell.
#[derive(Clone, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
    /// Display width: 1 for normal, 2 for wide, 0 for continuation.
    pub width: u8,
}

/// A run of styled text.
pub struct Span<'a> {
    pub text: &'a str,
    pub style: Style,
}
```

Color auto-detection at startup:
- `$COLORTERM=truecolor` or `$COLORTERM=24bit` → Rgb mode
- `$TERM` contains `256color` → Palette mode
- Otherwise → Ansi mode

Colors are stored as Rgb internally. At flush time, they are downgraded
to the detected mode:
- Rgb → Rgb: emit `\e[38;2;r;g;bm`
- Rgb → Palette: find nearest in 256-color cube
- Rgb → Ansi: find nearest in 16-color table

### surface.rs — Surface

A bounded, writable rectangular region of cells. Writes outside the
bounds are silently clipped (no panic, no wrap).

```rust
pub struct Surface<'a> {
    cells: &'a mut [Cell],
    stride: u16,        // width of the underlying grid row
    x: u16, y: u16,     // offset within the grid
    w: u16, h: u16,     // dimensions of this surface
}

impl Surface<'_> {
    /// Write a single character at (col, row) with style.
    fn put(&mut self, col: u16, row: u16, ch: char, style: Style);

    /// Write a string starting at (col, row). Truncates at surface edge.
    /// Handles wide characters: a wide char at the last column is replaced
    /// with a space (no half-character rendering).
    fn print(&mut self, col: u16, row: u16, text: &str, style: Style);

    /// Write styled spans starting at (col, row).
    fn print_spans(&mut self, col: u16, row: u16, spans: &[Span]);

    /// Fill the entire surface with a character and style.
    fn fill(&mut self, ch: char, style: Style);

    /// Draw a horizontal line.
    fn hline(&mut self, col: u16, row: u16, len: u16, ch: char, style: Style);

    /// Draw a vertical line.
    fn vline(&mut self, col: u16, row: u16, len: u16, ch: char, style: Style);

    /// Create a sub-surface (further bounded region within this surface).
    /// The sub-surface clips to the intersection of both bounds.
    fn sub(&mut self, col: u16, row: u16, w: u16, h: u16) -> Surface;

    /// Dimensions.
    fn width(&self) -> u16;
    fn height(&self) -> u16;
}
```

### screen.rs — Screen

Owns two cell grids. On flush, diffs them and emits only changes.

```rust
pub struct Screen {
    width: u16,
    height: u16,
    current: Vec<Cell>,
    previous: Vec<Cell>,
    color_mode: ColorMode,
    cursor_pos: Option<(u16, u16)>,  // None = hidden
}

pub enum ColorMode { Ansi, Palette, Rgb }

impl Screen {
    /// Create a new screen. Detects color mode from environment.
    fn new(width: u16, height: u16) -> Self;

    /// Resize the screen. Clears both grids.
    fn resize(&mut self, width: u16, height: u16);

    /// Get a surface covering a rectangular region.
    fn surface(&mut self, col: u16, row: u16, w: u16, h: u16) -> Surface;

    /// Get a surface covering the entire screen.
    fn full_surface(&mut self) -> Surface;

    /// Set cursor position (shown after flush). None = hidden.
    fn set_cursor(&mut self, pos: Option<(u16, u16)>);

    /// Flush changes to the terminal. Only emits diffs.
    fn flush(&mut self, out: &mut impl Write) -> io::Result<()>;

    /// Mark all cells as dirty (forces full redraw on next flush).
    fn force_redraw(&mut self);

    /// Read a cell (for testing).
    fn cell(&self, col: u16, row: u16) -> &Cell;

    /// Dimensions.
    fn width(&self) -> u16;
    fn height(&self) -> u16;
}
```

#### Flush algorithm

```
for each row:
    for each col:
        if current[row][col] != previous[row][col]:
            if cursor not at (col, row): emit CUP(row, col)
            if style changed from last emit: emit SGR(style)
            emit character
            advance cursor
            // batch consecutive dirty cells with same style
    copy current row to previous row
if cursor_pos is Some: emit CUP + show cursor
else: emit hide cursor
```

### layout.rs — Layout engine

Pure math: takes an area and constraints, returns rectangles.

```rust
pub enum Direction { Horizontal, Vertical }

pub enum Size {
    /// Exact number of cells.
    Fixed(u16),
    /// Percentage of parent (0–100).
    Percent(u16),
    /// Take all remaining space.
    Fill,
}

pub struct Constraint {
    pub size: Size,
    pub min: u16,
    pub max: u16,
}

pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    /// Split this rect into sub-rects along a direction.
    fn split(&self, dir: Direction, constraints: &[Constraint]) -> Vec<Rect>;
}
```

Algorithm for `split`:
1. Allocate Fixed sizes first (clamped to min/max)
2. Allocate Percent sizes from remaining space
3. Distribute remaining space to Fill entries
4. If total exceeds available, shrink from last to first, respecting min
5. If total is less than available, expand Fill entries

### border.rs — Box drawing

```rust
pub enum BorderMode { Pretty, CopyFriendly }

pub struct BorderStyle {
    pub mode: BorderMode,
    pub active: Style,    // style for focused panel borders
    pub inactive: Style,  // style for unfocused panel borders
}

/// Draw a box border around a rect on a surface.
/// Returns the inner rect (area inside the border).
pub fn draw_border(
    surface: &mut Surface,
    rect: Rect,
    title: &str,
    style: &BorderStyle,
    focused: bool,
) -> Rect;
```

Pretty mode uses `─│┌┐└┘├┤┬┴┼`. Copy-friendly mode uses colored spaces.
Title is rendered centered on the top border.

### text.rs — Text utilities

```rust
/// Compute display width of a string (handles wide chars).
pub fn display_width(s: &str) -> usize;

/// Truncate a string to fit within `max_width` display columns.
/// Appends `…` if truncated.
pub fn truncate(s: &str, max_width: usize) -> String;

/// Wrap text to fit within `max_width` display columns.
/// Returns a Vec of lines.
pub fn wrap(s: &str, max_width: usize) -> Vec<String>;

/// Compute display column for a byte offset in a string.
pub fn byte_to_col(s: &str, byte_offset: usize) -> usize;

/// Compute byte offset for a display column in a string.
pub fn col_to_byte(s: &str, col: usize) -> usize;
```

### termbuf.rs — Virtual terminal (VTE → cells)

A virtual terminal screen driven by ANSI escape sequences. Used for
embedded terminal widgets (shell, kiro).

```rust
pub struct TermBuf {
    cols: u16,
    rows: u16,
    grid: Vec<Vec<Cell>>,
    scrollback: Vec<Vec<Cell>>,
    max_scrollback: usize,
    cursor_row: u16,
    cursor_col: u16,
    cursor_visible: bool,
    current_style: Style,
    scroll_offset: usize,
    responses: Vec<Vec<u8>>,
    // saved cursor, scroll region, DEC modes, etc.
}

impl TermBuf {
    fn new(cols: u16, rows: u16) -> Self;

    /// Feed raw bytes from PTY output. Parses via vte and updates grid.
    fn process(&mut self, bytes: &[u8]);

    /// Resize the terminal. Reflows content if possible.
    fn resize(&mut self, cols: u16, rows: u16);

    /// Render current visible content to a surface.
    fn render_to(&self, surface: &mut Surface);

    /// Cursor position (col, row) relative to grid.
    fn cursor(&self) -> (u16, u16);
    fn cursor_visible(&self) -> bool;

    /// Drain response bytes (DA1, cursor position reports, etc.).
    fn drain_responses(&mut self) -> Vec<Vec<u8>>;

    /// Scroll offset for scrollback viewing.
    fn scroll_offset(&self) -> usize;
    fn set_scroll_offset(&mut self, offset: usize);
    fn scrollback_len(&self) -> usize;
}
```

TermBuf implements `vte::Perform` internally to handle:
- Print characters (including wide)
- Cursor movement (CUP, CUU, CUD, CUF, CUB, etc.)
- SGR (colors, attributes)
- Erase (ED, EL)
- Scroll (SU, SD, scroll region)
- DEC private modes (cursor visibility, alternate screen, etc.)
- Cursor save/restore
- Cursor position report (response sent back to PTY)

## Testing strategy

### Cell grid comparison (unit tests)

```rust
#[test]
fn put_writes_cell() {
    let mut screen = Screen::new(10, 3);
    let mut s = screen.full_surface();
    s.put(0, 0, 'H', Style::default());
    assert_eq!(screen.cell(0, 0).ch, 'H');
}

#[test]
fn wide_char_occupies_two_cells() {
    let mut screen = Screen::new(10, 1);
    let mut s = screen.full_surface();
    s.put(2, 0, '漢', Style::default());
    assert_eq!(screen.cell(2, 0).ch, '漢');
    assert_eq!(screen.cell(2, 0).width, 2);
    assert_eq!(screen.cell(3, 0).width, 0); // continuation
}

#[test]
fn surface_clips_out_of_bounds() {
    let mut screen = Screen::new(5, 1);
    let mut s = screen.full_surface();
    s.print(3, 0, "Hello", Style::default());
    assert_eq!(screen.cell(3, 0).ch, 'H');
    assert_eq!(screen.cell(4, 0).ch, 'e');
    // "llo" clipped — cells beyond width unchanged
}
```

### Text snapshot (integration tests)

```rust
#[test]
fn layout_splits_correctly() {
    let area = Rect { x: 0, y: 0, w: 80, h: 24 };
    let rects = area.split(Direction::Horizontal, &[
        Constraint { size: Size::Fixed(20), min: 10, max: 40 },
        Constraint { size: Size::Fill, min: 10, max: u16::MAX },
    ]);
    assert_eq!(rects[0], Rect { x: 0, y: 0, w: 20, h: 24 });
    assert_eq!(rects[1], Rect { x: 20, y: 0, w: 60, h: 24 });
}
```

### Screen text dump (visual tests)

```rust
impl Screen {
    /// Dump current grid as plain text (for test assertions).
    fn to_text(&self) -> String;
}

#[test]
fn border_renders_correctly() {
    let mut screen = Screen::new(20, 5);
    let s = screen.full_surface();
    draw_border(&mut s, Rect{x:0,y:0,w:20,h:5}, "Title", &style, true);
    assert_eq!(screen.to_text(),
        "┌─ Title ──────────┐\n\
         │                  │\n\
         │                  │\n\
         │                  │\n\
         └──────────────────┘\n");
}
```

## Performance targets

- Flush with no changes: < 1μs (just compare grids, emit nothing)
- Full screen redraw (80×24): < 500μs
- Full screen redraw (200×60): < 2ms
- Wide character handling: no measurable overhead vs ASCII
- Memory: ~20 bytes per cell × 2 grids = ~200KB for 200×60 terminal
