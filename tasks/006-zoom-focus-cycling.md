# C-S-left/right in Zoom Mode

<!-- TODO: mark done → todo tree [2][5] -->

## Problem

When a panel is zoomed (F5), Ctrl-Shift-Left/Right (focus cycling) either
does nothing or behaves unexpectedly.

## Expected Behavior

In zoom mode, C-S-left/right should:
- Unzoom the current panel
- Move focus to the adjacent panel
- Optionally: zoom the newly focused panel (so you cycle between zoomed views)

Or alternatively: C-S-left/right in zoom mode cycles tabs within the zoomed
panel (since there's only one panel visible).

## Implementation

In `src/layout_group/mod.rs` (or wherever focus cycling is handled):
- Check if zoomed
- If zoomed: either unzoom+move or cycle tabs within panel
- User preference via config

## Constraints

- 240 code line max
- Must not break non-zoomed focus cycling
