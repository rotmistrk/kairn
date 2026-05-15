# Bug: PTY Reflow Loses Lines Above Cursor

<!-- TODO: mark done → todo tree [2][0] -->

## Problem

When the terminal panel is resized (reflow), lines above the cursor are lost.
The reflow logic should preserve content above the cursor position — shrinking
the visible area should push lines into scrollback, not discard them.

## Expected Behavior

- Resize narrower: lines rewrap, content above cursor moves into scrollback
- Resize wider: lines unwrap, scrollback content may become visible again
- No content is ever lost — only moved between visible area and scrollback

## Likely Location

The reflow logic lives in `txv-widgets` (PtyTerminal / VTE layer). The issue
is probably in how the screen buffer is resized — it may be truncating from
the top instead of preserving lines relative to the cursor.

## Fix Strategy

1. On resize, calculate how many rows the content occupies after reflow
2. If content exceeds new height, push excess lines (from top) into scrollback
3. Keep cursor-relative position stable (cursor stays on same logical line)

## Test

1. Open a shell tab, run a command that produces 50+ lines of output
2. Resize the terminal panel narrower (or resize the whole kairn window)
3. Scroll up — all original output should still be in scrollback
4. Resize back wider — content should unwrap correctly

## Constraints

- Fix is likely in txv-widgets, not kairn itself
- Must not break existing scrollback behavior
- Must handle edge case: cursor on last line during shrink
