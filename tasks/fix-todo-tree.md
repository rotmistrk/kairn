# Bug: Todo Tree Empty and Non-functional

## Problem

1. When `.kairn.todo` doesn't exist, TodoFile::new creates empty items vec → tree shows nothing
2. No keys work on empty tree (no cursor, no add)
3. No integration tests exist for todo tree

## Root Cause

`duir_core::TodoFile::new("Todo")` creates `items: Vec::new()`.
`rebuild_flat` produces zero nodes. View is blank. Key handler requires
a valid cursor position which doesn't exist when empty.

## Fix

### 1. Handle empty state
In `src/views/todo_tree/handle.rs`:
- `n` (new sibling) must work even when tree is empty — insert first item
- Show placeholder text when empty: "Press 'n' to add first item"

### 2. Draw placeholder when empty
In the draw method, if `visible_count() == 0`:
```rust
surface.print(b.x, b.y, "  (empty — press 'a' to add)", dim_style);
```

### 3. Create file on first add
When user adds first item and `.kairn.todo` doesn't exist, create it.
Report error if write fails (status bar).

### 4. Integration tests

Create `tests/todo_tree.rs`:

```rust
#[test]
fn todo_tree_empty_shows_placeholder() {
    // Create TodoTreeView with no file → verify draw shows placeholder
}

#[test]
fn todo_tree_add_item_on_empty() {
    // Create empty tree, simulate 'n' key → verify item added
}

#[test]
fn todo_tree_navigate_and_edit() {
    // Create tree with items, j/k navigate, Enter edits
}

#[test]
fn todo_tree_toggle_completion() {
    // Space toggles completion state
}

#[test]
fn todo_tree_save_creates_file() {
    // Add item to empty tree → verify .kairn.todo created
}

#[test]
fn todo_tree_loads_existing_file() {
    // Write valid JSON, open tree → verify items shown
}
```

## Keybindings (duir standard — MUST match exactly)

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate |
| ←/→ | Collapse/Expand |
| Space | Toggle completed |
| e | Edit item text (inline) |
| n | New sibling |
| b | New child (below) |
| d | Delete item (y to confirm) |
| ! | Toggle important |
| K | Swap up |
| J | Swap down |
| H | Promote (outdent) |
| L | Demote (indent) |
| S | Sort children |
| c | Clone subtree |

## Constraints

- All errors reported (file write failures → status bar)
- Tests must pass
- Empty state must be usable (not dead)
