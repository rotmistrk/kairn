# Todo Tree Improvements: Badge Column, Status, Priority, LOE

## Problem

The todo tree currently has a flat status model (Open/Done/Partial) with no way
to express priority, effort, in-progress state, or paused state. Users cannot
see at a glance which items are hot, blocked, or actively being worked on.

Additionally, when the todo panel is focused, there's no status bar context
showing available actions or selected item metadata.

## Solution

1. **Extended data model** — add priority, effort, in_progress, paused fields
2. **Badge column** — 3-char column showing status/priority/notes at a glance
3. **Effective values** — collapsed nodes bubble up children's state
4. **FocusGatedGroup** — TXV widget for panel-specific status bar sections
5. **Status bar integration** — minihelp + live stats when todo is focused

## Data Model

New fields on each todo item (backward compatible — absent = default):

```json
{
  "title": "Fix auth",
  "completed": "Open",
  "important": false,
  "in_progress": false,
  "paused": false,
  "priority": 0,
  "effort": 0,
  "note": "",
  "items": []
}
```

- `priority`: 0-9 (0 = unset, 9 = critical)
- `effort`: fibonacci values (0,1,2,3,5,8,13,21), 0 = unset
- `in_progress`: leaf-only, actively being worked on
- `paused`: leaf-only, work suspended (see notes for reason)

Status is derived from fields:
- `completed == "Done"` → ✓
- `in_progress == true` → ▶
- `paused == true` → ⏸
- else → ○
- Parent with mixed children → ◐

## Effective Values (for collapsed display)

When a node is collapsed, display effective (aggregated) values:

- `effectivePriority` = max(self.priority, children.effectivePriority...)
- `effectiveEffort` = self.effort + sum(children.effectiveEffort...)
- `effectiveInProgress` = self.in_progress || any(children.effectiveInProgress)
- `effectivePaused` = self.paused || (any child paused && no child in_progress)
- `effectiveHasNotes` = has_note(self) || any(children.effectiveHasNotes)

Display rule: show effective values on the most specific *visible* node.
Expanded nodes show own values; collapsed nodes show effective.

## Badge Column (3 chars)

Layout: `[status][priority][notes]`

### Status Icons
- `○` open
- `▶` in-progress (green)
- `⏸` paused (yellow)
- `✓` done (dim green)
- `◐` partial (parent, auto-derived)

### Priority (Braille fill, 0-9)
- 0: (blank)
- 1-8: `⠁⠃⠇⡇⣇⣧⣷⣿` (progressive fill)
- 9: `⣿` (full, red/bold)

### Notes Indicator
- `♪` if node has notes, or collapsed children have notes
- (blank) otherwise

### Examples
```
○     open, no priority, no notes
▶⡇♪  in-progress, prio 4, has notes
⏸⣿   paused, prio 9
✓     done (dim everything)
◐⡆♪  partial, effective prio 5, child has notes
```

## LOE Column (optional, toggled with `L`)

- 2 chars wide, right-aligned
- Shows effort value (own when expanded, effective when collapsed)
- 0 = blank (not shown)
- Values: 1, 2, 3, 5, 8, 13, 21

## Key Bindings

| Key | Action | Constraint |
|-----|--------|-----------|
| `Space` | Toggle done (○↔✓) | Leaf-only |
| `i` | Toggle in-progress (○↔▶, ⏸→▶) | Leaf-only |
| `\` | Toggle paused (○↔⏸, ▶→⏸) | Leaf-only |
| `+` / `-` | Priority up/down (0-9) | Any node |
| `>` / `<` | LOE up/down (fibonacci cycle) | Any node |
| `L` | Toggle LOE column visibility | Global |
| `!` | Toggle important (existing) | Any node |
| `c` | Copy (reset status, keep LOE+notes) | Any node |
| `C` | Copy (keep everything) | Any node |

Status conflicts:
- Done + press `i` → ignored (must Space to reopen first)
- In-progress + press `\` → switch to paused
- Paused + press `i` → switch to in-progress

## FocusGatedGroup (txv-widgets)

A Group widget that is active/inactive based on commands from associated views.

### Behavior
- **Inactive**: `size() = (0, h)`, `draw()` = noop, `handle_event()` = Ignored
- **Active**: renders children, dispatches events to children normally

### Activation
- Listens for `CM_ACTIVATE_GROUP(group_id)` → active = true
- Listens for `CM_DEACTIVATE_GROUP(group_id)` → active = false
- Associated widget sends these on focus/blur (select/unselect)

### API
```rust
pub struct FocusGatedGroup {
    group_state: GroupState,
    active: bool,
    group_id: u16,
}

impl FocusGatedGroup {
    pub fn new(group_id: u16) -> Self;
    pub fn add_child(&mut self, child: Box<dyn View>);
}
```

### Integration
The TodoTreeView sends:
- `CM_ACTIVATE_GROUP(TODO_GROUP_ID)` in `on_select()`
- `CM_DEACTIVATE_GROUP(TODO_GROUP_ID)` in `on_unselect()`

The group lives in the status bar and contains:
- Badge indicator (mirrors selected item's status)
- LOE indicator (effective effort of selected subtree)
- Key label items (minihelp)

## File Changes

### txv-widgets
- New: `src/focus_gated_group.rs`
- Modify: `src/lib.rs` (export)

### kairn
- Modify: `src/views/todo_tree/model.rs` (new fields, parse/serialize)
- Modify: `src/views/todo_tree/data.rs` (effective values, badge rendering)
- Modify: `src/views/todo_tree/mod.rs` (key handling, activate/deactivate)
- Modify: `src/status.rs` (add FocusGatedGroup for todo)
- New command IDs: CM_TODO_TOGGLE_PROGRESS, CM_TODO_TOGGLE_PAUSE,
  CM_TODO_PRIORITY_UP/DOWN, CM_TODO_LOE_UP/DOWN, CM_TODO_TOGGLE_LOE
