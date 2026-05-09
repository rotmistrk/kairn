# Step 02: txv-render

**Reference**: `doc/f4-design/v-013-txv-architecture.md`
**Depends on**: Step 01 (txv-core)

## What this is

The terminal backend. Implements txv-core's Backend trait for crossterm.
Also provides TermBuf (VTE terminal emulator) and text utilities.

## Boundary

- **Creates**: `txv-render/` crate
- **Does NOT touch**: txv-core/, txv-widgets/, rusticle/, src/ (kairn)
- **Dependencies**: txv-core (path), crossterm 0.28, vte 0.13, unicode-width 0.2

## Deliverables

```
txv-render/src/
├── lib.rs          — re-exports
├── backend.rs      — CrosstermBackend implementing txv_core::Backend
├── color.rs        — ColorMode detection (RGB/256/16), downgrade functions
├── text.rs         — display_width, truncate, wrap, byte_to_col, col_to_byte
└── termbuf.rs      — TermBuf (VTE → Surface cells)
```

## CrosstermBackend

```rust
impl Backend for CrosstermBackend {
    fn poll_event(&mut self, timeout: Duration) -> Option<Event>;  // crossterm → txv_core::Event
    fn size(&self) -> (u16, u16);
    fn flush(&mut self, surface: &Surface);  // diff previous, emit escape sequences
    fn enter(&mut self);   // raw mode + alternate screen
    fn leave(&mut self);   // restore
}
```

Flush algorithm: dual-buffer diff (current vs previous), emit only changed cells.
Batch consecutive dirty cells with same style.

## TermBuf

Port from existing working code (git history has it).
Implements vte::Perform. Renders to a Surface.

```rust
impl TermBuf {
    pub fn new(cols: u16, rows: u16) -> Self;
    pub fn process(&mut self, bytes: &[u8]);
    pub fn resize(&mut self, cols: u16, rows: u16);
    pub fn render_to(&self, surface: &mut Surface);
    pub fn cursor(&self) -> (u16, u16);
    pub fn cursor_visible(&self) -> bool;
}
```

## Text utilities

```rust
pub fn display_width(s: &str) -> usize;
pub fn truncate(s: &str, max_width: usize) -> String;
pub fn wrap(s: &str, max_width: usize) -> Vec<String>;
pub fn byte_to_col(s: &str, byte_offset: usize) -> usize;
pub fn col_to_byte(s: &str, col: usize) -> usize;
```

## Verification

```bash
cargo test -p txv-render
cargo clippy -p txv-render -- -D warnings
dupfinder txv-render/src
```

## Do NOT

- Do NOT define View/Group/Event here (that's txv-core)
- Do NOT add widgets here (that's txv-widgets)
- Do NOT duplicate Surface/Cell types (import from txv-core)
