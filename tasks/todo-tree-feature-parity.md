# Todo Tree: Feature Parity with Duir

## Priority 1 — Required

### Completion propagation
- When all children are marked Done → parent auto-marks Done
- When a new child is added → parent becomes Open
- When any child is unchecked → parent becomes Open
- Implementation: after any toggle/add/remove, walk up and recompute parent state
- duir-core has `stats::compute_stats` — use it to determine if all leaves are done

### Sort children (S key)
- Already in duir-core: `tree_ops::sort_children(file, path)`
- Just wire up 'S' key in handle.rs

### Clone subtree (c key)
- Already in duir-core: `tree_ops::clone_subtree(file, path)`
- Wire up 'c' key

### Filter (/ key)
- duir-core has `filter` module
- Enter filter mode → inline editor at top → filter visible items by title match
- Esc exits filter, shows all items again
- Already have inline editor infrastructure

### Important mark rendering
- When item.important = true, render the entire subtree in bold/bright style
- Currently only the item itself is styled differently
- On draw: check if any ancestor is important, apply bold style

### MCP access
- Ensure all operations (sort, clone, filter, notes) are accessible via MCP tools
- Current MCP tools: list_todos, update_todo (toggle, add, remove, move, promote, demote)
- Add: sort, clone, set_note, get_note

## Priority 2 — Notes Pane

### Design
- "Notes" tab in the center/main pane (like an editor tab)
- Content syncs to the currently selected todo item's `note` field
- On todo tree cursor move → update Notes content
- Editing uses the existing EditorView (reuse vim keybindings, :w saves note back)
- Command: `CM_TODO_NOTE_CHANGED` with node_id + content

### Implementation
- New command: `CM_TODO_CURSOR_MOVED` emitted by todo tree on navigation
- Handler opens/updates a "Notes" editor tab with the item's note content
- On editor save → write back to TodoItem.note, save todo file
- EditorView already supports in-memory buffers (from_text)

## Priority 3 — Encryption (nice to have)

- duir-core has `crypto` module (feature-gated)
- Lock/unlock subtrees with a passphrase
- Encrypted nodes show as locked in tree, children hidden
- Unlock prompts for passphrase via status bar input
- Requires enabling `crypto` feature in Cargo.toml

## Implementation Order

1. Completion propagation (affects correctness)
2. Sort children (S) — trivial
3. Clone subtree (c) — trivial
4. Important subtree rendering — small draw change
5. Filter (/) — moderate (reuse inline editor)
6. MCP extensions — small
7. Notes pane — moderate (reuse EditorView)
8. Encryption — later

## Estimated LOE

- Items 1-4: ~1 hour
- Item 5 (filter): ~45 min
- Item 6 (MCP): ~30 min
- Item 7 (notes pane): ~1.5 hours
- Item 8 (encryption): ~1 hour

Total: ~5 hours
