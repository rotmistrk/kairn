# Task: Todo Tree UX — Match Duir Behavior

## Objective
Fix multiple UX deficiencies in the todo tree so it behaves like duir's todo
(minus icons). Covers selection colors, checkboxes, inline editing, and
keyboard behavior.

## Issues

### 1. Selection color scheme is painful
**Current:** Cursor row uses `bg: Ansi(4)` (blue) with underline, which clashes
with foreground colors and is hard to read.

**Fix:** Selection/cursor in lists and tree views should keep the node's
foreground color and change background to dark blue (`Ansi(4)`) WITHOUT
underline. This applies to:
- TreeView cursor row (txv-widgets `tree_view.rs`)
- StructuredView cursor row (`struct_view/draw.rs`)
- Todo tree cursor row
- Tab bar active tab (already uses `Ansi(4)` bg — OK)

For the editor visual selection (`visual_style`), use `reverse: true` with
`fg: Ansi(3)` — this is already correct.

**Specific change in txv-widgets `tree_view.rs`:**
```rust
// Focused cursor: keep fg, dark blue bg, NO underline
Style { fg: node_style.fg, bg: Color::Ansi(4), attrs: node_style.attrs }
// Unfocused cursor: keep fg, dark grey bg
Style { fg: node_style.fg, bg: Color::Ansi(8), attrs: node_style.attrs }
```

**In `struct_view/draw.rs`:** Remove `underline: true` from `focus_style`.

### 2. No checkboxes in todo tree
**Current:** Todo items show only their title text with expand/collapse markers.

**Fix:** Render a checkbox before the title:
- `[ ]` — open/incomplete
- `[x]` — done (Completion::Done)
- `[-]` — partial (if supported)

In `TreeData::label()` or in the custom draw override in `TodoTreeView::draw()`,
prepend the checkbox. Prefer custom draw so the checkbox is styled differently
(dim for done items).

### 3. Shift-arrows do not work in todo
**Current:** Shift+Up/Down are not handled — they should select a range or
(in duir behavior) move the item up/down.

**Fix:** In `handle.rs`, handle `Shift+Up` as swap-up (same as `K`) and
`Shift+Down` as swap-down (same as `J`). This matches duir's behavior where
Shift-arrows reorder items.

```rust
KeyCode::Up if key.modifiers.contains(KeyMod::SHIFT) => {
    // same as K — swap up
}
KeyCode::Down if key.modifiers.contains(KeyMod::SHIFT) => {
    // same as J — swap down
}
```

### 4. New todo should open with inline editor active
**Current:** `n` and `b` create an item with hardcoded "New task"/"New subtask"
text and do NOT open the editor.

**Fix:** After creating the item:
1. Move cursor to the new item's row
2. Immediately open InlineEditor with the default text fully selected
3. Default text should be in selection so typing replaces it

This requires InlineEditor to support text selection (see #5).

### 5. InlineEditor selection behavior
**Current:** InlineEditor has no selection concept — just a cursor.

**Fix:** Add selection support to InlineEditor:
- **Selection color:** dark grey bg (`Ansi(8)`) — or dark green bg for
  structural editors (TBD, start with dark grey for all)
- **Nav keys (Left/Right/Home/End without Shift):** clear selection, move cursor
- **Shift+arrows:** extend selection
- **Backspace/Delete with selection:** delete selected text
- **Typing with selection:** replace selected text
- **Esc:** cancel edit entirely (revert)
- **Enter:** commit current buffer
- **Tab (structural editor only):** commit current buffer

The "select all on open" behavior is needed for the "New task" flow:
```rust
InlineEditor::new_selected(row, "New task")
// cursor at end, selection covers entire text
```

### 6. InlineEditor must respect indent
**Current:** Editor draws at `b.x` (column 0 of the view bounds), full width.

**Fix:** InlineEditor x-position must start at the item's indent level:
```
indent = (depth * 2) + 2 (for expand marker) + 4 (for checkbox "[ ] ")
```
Width = `b.w - indent`. This way the editor aligns with the text it's editing.

### 7. Fold/unfold triangles
**Current:** TreeView uses `▼`/`▶` markers — this is correct.

**Verify:** These already render for expandable nodes. Just ensure they show
for todo items that have children.

## Implementation Order
1. Fix selection colors (remove underline from tree/struct cursor styles)
2. Add checkboxes to todo tree draw
3. Handle Shift+Up/Down in todo
4. Add selection support to InlineEditor (txv-widgets change)
5. Open editor on new item creation with text selected
6. Indent-aware editor positioning

## Files to Modify
- `txv-widgets`: `src/tree_view.rs` (cursor style), `src/inline_edit.rs` (selection)
- `src/views/struct_view/draw.rs` (remove underline from focus_style)
- `src/views/todo_tree/mod.rs` (custom draw with checkboxes, editor indent)
- `src/views/todo_tree/handle.rs` (Shift+arrows, open editor on n/b)
- `src/views/todo_tree/data.rs` (label may need adjustment)

## Constraints
- Changes to txv-widgets require a commit+push there, then update Cargo.lock
- 240 lines per file max
- No unwrap/expect/panic
- Must match duir behavior (minus icons)
