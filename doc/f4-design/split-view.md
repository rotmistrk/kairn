# Split View — Design & Requirements

## Overview

The editor split divides the center panel into two panes showing EditorViews.
Panes share the same buffer (same file) or show different files.

## Commands

| Command | Behavior |
|---------|----------|
| `:split` | If not split: horizontal split (top/bottom), new pane on top. If already split: toggle to horizontal. |
| `:vsplit` | If not split: vertical split (left/right), new pane on left. If already split: toggle to vertical. |
| `:split <file>` | Split + open `<file>` in the new pane. Completer starts at current file's directory. |
| `:vsplit <file>` | Same but vertical. |
| `:only` | Close the split, keep the focused pane. |
| `Ctrl-W w` | Switch focus between panes. |

### Pane placement

When splitting, the **new pane** appears on top (hsplit) or left (vsplit).
The **existing view** stays at its current scroll position in the bottom/right pane.

Rationale: right-handed user — editing hand is below/right, eyes look at
reference material above/left.

### Orientation toggle

`:split` when already in a vsplit → toggles to horizontal.
`:vsplit` when already in a hsplit → toggles to vertical.
No file argument = orientation toggle only.

## `gs` — Go to definition in split

`gs` = "go split". Behavior:

1. Send `textDocument/definition` (same as `gd`).
2. When result arrives:
   - If not in a split: create a vsplit first.
   - Open the target file in the **other pane** (the new/top/left one).
   - Scroll the other pane to the target line.
   - Show a **highlight line** on the target line in the other pane.
   - **Focus stays in the current pane** (where the user edits).

The highlight line uses the `search_match` color and clears on next keypress
in that pane.

## Diff modes

### `:diff` — Unified diff (no split)

Shows diff inline in the current editor view (existing behavior), plus:
- Syntax highlighting applied to diff content
- `+`/`-` markers on the left gutter, colored (green/red)

### `:diff -y` — Side-by-side diff

Opens a split showing:
- Left pane: HEAD version (read-only, reference)
- Right pane: working copy (editable)

With **hunk-aligned synchronized scrolling**.

### `:diff -U<n> -y` — Context control

`-U<n>` sets context lines (default: 3). Examples:
- `:diff -U8 -y` → side-by-side with 8 lines context
- `:diff -U3 -y` → change to 3 lines context
- `:diff -y` → switch from unified to side-by-side (keep current context)
- `:diff` → switch from side-by-side back to unified

### Hunk-aligned scrolling

When scrolling in either pane, the other pane scrolls to the **corresponding
hunk position**, not raw line numbers. Deleted lines in one side align with
blank/filler lines in the other. Both panes stay visually aligned at hunk
boundaries.

### Diff highlighting (both modes)

- Added lines: green background / `+` gutter marker
- Deleted lines: red background / `-` gutter marker
- Changed lines: word-level diff highlighting within the line
- Syntax highlighting applied on top of diff coloring

## Implementation Notes

### EditorSplit changes
- `linked_scroll: bool` — already exists, needs hunk-aligned wiring
- `highlight_line: Option<usize>` — line to highlight in non-focused pane
- `scroll_map: Vec<(usize, usize)>` — maps left line → right line for hunk alignment
- On handle: if scroll changed in focused pane and `linked_scroll`, use scroll_map

### Handler changes
- `:split`/`:vsplit` with no arg + not split: create split, same file, new pane top/left
- `:split`/`:vsplit` with no arg + already split: toggle orientation
- `:split`/`:vsplit` with arg: open file in new pane (or replace if already split)
- New pane placement: top/left (index 0 in SplitPane)

### `gs` flow
1. Editor keymap: `gs` → emit `CM_LSP_GOTO_SPLIT`
2. Handler: same as `CM_LSP_GOTO_DEF` but response uses `CM_OPEN_IN_SPLIT`
3. Response: if not split, create vsplit; open file in other pane; set highlight_line

### Completer
`:split ` and `:vsplit ` trigger path completion starting from the directory
of the currently open file (not project root).

### Unified diff gutter
- New gutter column (1 char wide) showing `+`/`-`/` `
- Colored: added=green, deleted=red, context=dim
- Syntax highlighting applied to the source lines (strip +/- prefix for highlighting)

## Testability

### Synchronized scrolling (unit test)
- Create EditorSplit with two aligned buffers and `linked_scroll: true`
- Build scroll_map from diff hunks
- Inject scroll events into one pane
- Assert other pane's scroll_top matches expected hunk-aligned position

### Highlight line (unit test)
- Set `highlight_line = Some(42)` on EditorSplit
- Draw → assert line 42 has search_match style in the non-focused pane

### Orientation toggle (unit test)
- Create hsplit → send `:vsplit` → assert direction is Horizontal
- Send `:split` → assert direction is Vertical

## Scroll leadership

Both panes are fully navigable. The **focused pane** (where cursor is) leads
synchronized scrolling. `Ctrl-W w` switches focus and thus scroll leadership.

## F5 (Zoom)

F5 zooms the center panel as a whole. The split stays intact — both panes
resize proportionally within the zoomed area.

## Tcl Bindings

All split operations are scriptable via the `split` namespace:

| Command | Description |
|---------|-------------|
| `split open` | Horizontal split (or toggle to horizontal) |
| `split open -vertical` | Vertical split (or toggle to vertical) |
| `split open <file>` | Split + open file in new pane |
| `split open -vertical <file>` | Vertical split + open file |
| `split close` | Close split (`:only`) |
| `split focus` | Switch focus between panes (`Ctrl-W w`) |
| `split direction` | Returns current direction: "horizontal", "vertical", or "none" |
| `split linked` | Returns/sets linked scroll: `split linked true` / `split linked false` |
| `split highlight <line>` | Set highlight line in non-focused pane (0 to clear) |

## MCP Tools

| Tool | Description |
|------|-------------|
| `split_open` | `{"vertical": bool, "file": optional string}` — open/toggle split |
| `split_close` | Close split |
| `split_focus` | Switch focus between panes |
| `split_status` | Returns `{"active": bool, "direction": str, "linked_scroll": bool}` |
