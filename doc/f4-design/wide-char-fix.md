# Wide Character Rendering Bug — Root Cause & Fix

## The Bug

Lines containing wide characters (✅, ←, →, emoji, CJK) render incorrectly:
- Characters after the wide char are shifted or missing
- Scrolling changes which chars are missing (inconsistent)
- Tab switching shows stale content at positions after wide chars

## Root Cause

The editor's draw loop in `src/views/editor/draw.rs` uses a single variable
`col_offset` as BOTH:
1. The character index in the string (for buffer/cursor operations)
2. The cell column position on screen (for `surface.put()` calls)

These are NOT the same when wide characters exist:
- Character index: advances by 1 per char (always)
- Cell column: advances by 1 for normal chars, 2 for wide chars

When `✅` (width=2) is at character index 7, it occupies cells 7 AND 8.
The next character (index 8) should be at cell 9. But the code puts it
at cell 8 (overwriting the continuation cell of ✅).

## The Fix

Separate `col_offset` into two variables:

```rust
let mut char_idx: usize = 0;    // character position in buffer line
let mut visual_col: usize = 0;  // cell column on screen
```

Rules:
- `char_idx` advances by 1 for every character (used for cursor comparison)
- `visual_col` advances by `display_char_width(ch)` (used for surface.put position)
- `surface.put(text_x + visual_col as u16, y, ch, style)` — uses visual_col
- Cursor comparison: `if char_idx == cursor_col` — uses char_idx
- Tab expansion: `visual_col += tab_width` (tabs expand to tab_width cells)
- Padding after line: fill from `text_x + visual_col` to `text_x + avail`

## Where to change

File: `src/views/editor/draw.rs`

The main rendering loop iterates over syntax-highlighted spans:
```rust
for span in &spans {
    for ch in span.text.chars() {
        // Currently uses col_offset for both purposes
        // Fix: use visual_col for put(), char_idx for cursor check
    }
}
```

After the loop, padding fills remaining width:
```rust
// Pad from visual_col to avail (not col_offset to avail)
while visual_col < avail {
    surface.put(text_x + visual_col as u16, y, ' ', normal);
    visual_col += 1;
}
```

## Also fix: vcol for cursor positioning

The cursor position calculation (used to determine which screen cell
to highlight as cursor) must also use visual_col logic:
- For each char before cursor: add display_char_width(ch) to vcol
- For tabs: add tab_width to vcol
- Cursor cell = text_x + vcol

## display_char_width function

Already exists in `txv-core/src/surface.rs` as `pub fn display_char_width(ch: char) -> u16`.
Import it: `use txv_core::surface::display_char_width;`

## Test

After fix:
- Open Makefile containing `@echo "✅ All checks passed"`
- Line must render with correct spacing: ✅ followed by space, then "All"
- Scroll up/down: same line renders identically regardless of position
- Switch tabs: no stale characters from previous tab
- `:set list` mode: $ at correct end-of-line position

## What NOT to change

- Do NOT change `txv-core/src/surface.rs` print_line (it uses put() which handles width)
- Do NOT change `txv-render/src/backend.rs` flush (already fixed, advances by cell.width)
- Do NOT add surface.fill() calls — print_line handles padding
