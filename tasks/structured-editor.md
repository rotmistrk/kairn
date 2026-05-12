# Structured Editor (Tree-Table View)

## Overview

A format-agnostic tree-table view for structured data files. Opens in the main
panel as an alternative to raw text editing. Navigates and edits the document as
a tree — indentation/syntax errors are impossible.

**Formats (implement in order):**
1. JSON / JSONC (with `//` and `/* */` comments)
2. JSONL (one JSON object per line — shown as array of objects)
3. YAML (with comment preservation)
4. Amazon Ion (with type annotations)
5. XML (with attributes)

## Display: Three-Column Tree-Table

```
Key              │ Value          │ Meta
─────────────────┼────────────────┼──────────────
▼ server         │ {3}            │ # Main config
  ├─ host        │ "localhost"    │
  ├─ port        │ 8080           │ # default
  ▼ ports        │ [2]  ⏎         │
  │ ├─ [0]       │ 443            │
  │ └─ [1]       │ 8080           │
  └─ debug       │ true           │
```

### Columns

1. **Key** — tree-indented with expand/collapse. Arrays show `[i]` indices.
2. **Value** — scalars: editable value. Containers: `{n}` or `[n]` (child count).
3. **Meta** — comments, annotations, formatting hints. Editable.

### Vertical separators (`│`) between columns. No horizontal lines.

### Formatting Hints (in Meta)

Special directives that control serialization:
- `⏎` (or `inline`) — serialize this array/object on one line: `[1, 2, 3]`
- Absence = multi-line (default)

These are shown in Meta column and toggled with a keybinding.

## Navigation (duir-based keybindings)

| Key | Action |
|-----|--------|
| j/k, ↑/↓ | Move cursor row |
| h/l, ←/→ | Collapse/Expand (tree navigation) |
| g/G | First/last row |
| Tab | Cycle focus: Key → Value → Meta columns |
| Enter | Edit focused column (see Editing below) |
| Space | Toggle expand/collapse |
| n | New sibling |
| b | New child (only on containers) |
| d | Delete node (y to confirm) |
| c | Clone subtree (copy below) |
| ! | Toggle inline formatting hint |
| K | Swap up |
| J | Swap down |
| H | Promote (outdent — move to parent's level) |
| L | Demote (indent — make child of previous sibling) |
| t | Cycle scalar type: string → number → bool → null |
| T | Convert container: dict ↔ array |
| s | Sort children by key/value (stable, toggle asc/desc) |
| S | Sort by xpath expression (prompt) |
| f | Filter children (prompt, substring match) |
| F | Clear filter |
| u | Undo |
| Ctrl-R | Redo |
| :w | Save |
| :q | Close |

## Editing

### Enter on Value column:
- Scalar → InlineEditor with current value
- Container → no-op (expand/collapse with Space)

### Enter on Key column:
- Dict entry → InlineEditor to rename key
- Array entry → no-op (indices are auto-assigned)

### Enter on Meta column:
- InlineEditor for comment/annotation text

### New sibling (`n`):
- In array → insert new null element below, no prompt
- In dict → prompt for key name (InlineEditor), then position on value

### Clone (`c`):
- Deep-copies the subtree (including Meta/comments)
- In array → insert copy below silently
- In dict → prompt for new key name (InlineEditor)

### Undo/Redo:
- Full undo stack per document (same granularity as editor: each edit = one undo step)
- Undo covers: value edits, add, delete, move, sort, type changes

## Sorting

- `s` on a container → stable sort its children
  - Dict: sort by key (alphabetic)
  - Array: sort by value (auto-detect numeric vs text)
  - Toggle asc/desc on repeated press
- `S` → prompt for xpath-like expression to sort by
  - Example: `.name` sorts objects in an array by their `name` field
  - Example: `.size` with numeric detection

## Format-Specific Details

### JSON / JSONC
- Parse with serde_json (JSON) or custom parser (JSONC for comments)
- Comments: `//` line comments and `/* */` block comments
- Comments attached to the following node (or trailing on same line)
- Serialize: pretty-print with 2-space indent, respect inline hints

### JSONL
- Each line is a JSON object → present as an array of objects
- Top-level shown as `[n]` array
- Save: one object per line (no pretty-print at top level, pretty within if multi-line)
- Filter/sort across all line-objects by field path

### YAML
- Must preserve comments (attached to nodes)
- Anchors/aliases: show resolved values, mark with `&anchor` / `*alias` in Meta
- Multi-line strings: show first line in Value, full content on Enter
- Serialize: respect original style (flow vs block) where possible

### Amazon Ion (future)
- Type annotations shown in Meta: `user::`, `timestamp::`
- Typed nulls: `null.string`, `null.int`
- Blobs/clobs shown as `<blob:n bytes>`

### XML (future)
- Attributes shown in Meta: `class="x" id="y"`
- Text content as value
- Mixed content (text + elements) shown as interleaved children
- Namespaces shown as prefix in Key

## Implementation

### File Structure

```
src/views/struct_view/
  mod.rs          — StructuredView struct, View trait impl
  draw.rs         — three-column tree-table rendering
  handle.rs       — key handling (nav, edit, structural ops)
  undo.rs         — undo/redo stack
src/structured/
  mod.rs          — StructuredDoc trait, NodeKind, NodeId
  json_doc.rs     — JSON/JSONC implementation
  jsonl_doc.rs    — JSONL implementation (wraps json_doc)
  yaml_doc.rs     — YAML implementation
```

### Core Trait

```rust
pub trait StructuredDoc {
    fn root(&self) -> NodeId;
    fn children(&self, id: NodeId) -> &[NodeId];
    fn node_kind(&self, id: NodeId) -> NodeKind;  // Dict, Array, Scalar
    fn key(&self, id: NodeId) -> Option<&str>;
    fn value_display(&self, id: NodeId) -> &str;
    fn meta(&self, id: NodeId) -> &str;
    fn is_inline(&self, id: NodeId) -> bool;

    fn set_key(&mut self, id: NodeId, key: &str);
    fn set_value(&mut self, id: NodeId, val: &str) -> Result<(), String>;
    fn set_meta(&mut self, id: NodeId, meta: &str);
    fn toggle_inline(&mut self, id: NodeId);

    fn add_sibling(&mut self, id: NodeId) -> NodeId;
    fn add_child(&mut self, id: NodeId) -> NodeId;
    fn clone_node(&mut self, id: NodeId) -> NodeId;  // deep copy
    fn remove(&mut self, id: NodeId);
    fn swap_up(&mut self, id: NodeId) -> Option<NodeId>;
    fn swap_down(&mut self, id: NodeId) -> Option<NodeId>;
    fn promote(&mut self, id: NodeId) -> Option<NodeId>;
    fn demote(&mut self, id: NodeId) -> Option<NodeId>;

    fn cycle_type(&mut self, id: NodeId);         // string→number→bool→null
    fn convert_container(&mut self, id: NodeId);  // dict↔array

    fn serialize(&self) -> String;
}
```

### Undo

```rust
pub struct UndoStack {
    snapshots: Vec<Box<dyn StructuredDoc>>,  // or command-based
    position: usize,
}
```

Command-based undo preferred (less memory): each mutation records an inverse operation.

### File Opening

Detect by extension in `src/handler_open.rs`:
- `.json`, `.jsonc` → StructuredView + JsonDoc
- `.jsonl`, `.ndjson` → StructuredView + JsonlDoc
- `.yaml`, `.yml` → StructuredView + YamlDoc
- `.ion` → (future) StructuredView + IonDoc
- `.xml`, `.html`, `.svg` → (future) StructuredView + XmlDoc

## CSV Viewer Consistency

Both CsvView and StructuredView share patterns:
- Vertical separators, no horizontal lines
- InlineEditor for cell/value editing
- Tab cycles columns
- `s`/`S` for sort
- `f`/`F` for filter
- `:w` to save, `:q` to close
- Undo/redo (add to CSV spec too!)

**Note: Add undo/redo to csv-viewer.md as well.**

## Testing

1. Open `.json` → tree rendered with correct structure
2. Edit scalar value → reflected in tree, undo restores
3. Add sibling to dict → prompts for key
4. Add sibling to array → no prompt, inserts null
5. Clone in array → silent copy
6. Clone in dict → prompts for key
7. Delete → confirm, undo restores
8. Sort dict by key → stable, reversible
9. Sort array by xpath → works on nested field
10. Toggle inline → serializes on one line
11. `:w` → valid JSON output
12. Open `.jsonl` → shows as array, saves one-per-line
13. Comments (JSONC) → preserved through edits
14. Open `.yaml` → comments shown in Meta, preserved on save
15. Undo/redo across all operation types

## Constraints

- 240 code lines per file max
- No unwrap/expect/panic in runtime
- Reuse InlineEditor from txv-widgets
- Errors shown in status bar
- All existing tests must pass
