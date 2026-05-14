# Terminal Reflow on Resize

## Problem

Resizing the PTY panel clips content permanently. Long lines that wrapped at the old
width lose their overflow when shrunk, and growing back doesn't restore them. This is
because `TermBuf` stores a fixed-width cell grid and `resize()` just truncates.

## Solution

Store logical lines with a `wrapped: bool` flag. On resize, re-wrap logical lines to
the new width. This is the standard approach used by all modern terminal emulators
(VTE/GNOME Terminal, iTerm2, WezTerm, Kitty, Alacritty, Windows Terminal, Ghostty).

## Design

### Data model change (txv-render/src/termbuf/)

```rust
struct LogicalLine {
    cells: Vec<TCell>,
    wrapped: bool, // true = line continued on next row (soft wrap, no real newline)
}
```

Replace `cells: Vec<Vec<TCell>>` with a logical line store. The rendered grid becomes
a derived cache rebuilt on resize.

### Key changes

1. **VTE handler**: when cursor reaches right margin and wraps, mark current line as
   `wrapped = true`. When a real newline (LF) arrives, `wrapped = false`.

2. **Scrollback**: store `LogicalLine` instead of `Vec<TCell>`. Adjacent wrapped lines
   form one logical unit.

3. **resize()**: iterate logical lines, re-wrap to new width:
   - If growing: merge consecutive wrapped lines back together, split at new width
   - If shrinking: split long lines at new width, mark splits as wrapped

4. **Cursor position**: recompute after reflow (track which logical line the cursor is
   on, then find its new screen row/col).

5. **render_at()**: unchanged — still reads from the grid cache.

### Edge cases

- Scroll regions (CSI r): reflow only applies outside active scroll regions
- Tab stops: stored per logical line position, need adjustment on reflow
- Wide characters (CJK): can't split in the middle of a double-width char

## Estimated LOE

3-4 hours

## Files affected

- `txv-render/src/termbuf/mod.rs` — main refactor
- `txv-render/src/termbuf/scrollback.rs` — store LogicalLine
- `txv-render/src/termbuf/vte_actions.rs` — mark wrapped lines
- `txv-render/src/termbuf/tests.rs` — update/add reflow tests
