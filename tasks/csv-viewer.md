# CSV Viewer / Editor

## Overview

A tabular view for CSV/TSV files that opens in the main panel. Aligned columns,
smart formatting, sortable, filterable per-column, and inline-editable using the
existing InlineEditor widget.

## Display Rules

### Column Alignment
- **Vertical separators** (`│`) between columns, no horizontal separators
- **Text columns**: left-aligned
- **Numeric columns**: decimal-point aligned (`.` aligned if decimal present, otherwise right-aligned with space padding)
- **Column width**: auto-calculated from content (max width per column, capped at reasonable limit)

### Header Detection
Header row is auto-detected if:
- First row has no numeric-only fields (all text/labels)
- First row differs structurally from data rows (e.g., shorter values, no decimals)
- Header row rendered with bold/reverse/color highlight
- Header stays visible (frozen) when scrolling vertically

### Delimiter Detection
Auto-detect from first few lines:
- `,` (CSV)
- `\t` (TSV)
- `|` (pipe-delimited)
- `;` (European CSV)

### Column Type Detection
Each column is classified as Numeric or Text:
- **Numeric**: majority of non-empty values parse as numbers
- Smart handling of special values in numeric columns:
  - `-` (dash) → treated as null/missing, stays numeric
  - `*` → treated as null/missing, stays numeric
  - `n/a`, `N/A`, `NA` → treated as null/missing, stays numeric
  - Empty cells → treated as null/missing, stays numeric
- A column remains numeric if ≥80% of non-empty, non-special values parse as f64

## Navigation

| Key | Action |
|-----|--------|
| j/k, ↑/↓ | Move cursor row |
| h/l, ←/→ | Move cursor column |
| g/G | First/last row |
| 0/$ | First/last column |
| Enter | Edit cell; on confirm, advance cursor down |
| Tab | Edit cell; on confirm, advance cursor right (no wrap on last column) |
| s | Sort by current column (toggle asc/desc, auto-detect type) |
| f | Set filter on current column |
| F | Clear filter on current column |
| Ctrl-F | Clear all filters |
| / | Search within table |
| :q | Close view |

## Sorting

- Press `s` on a column → stable sort all data rows by that column
- Auto-detects column type: numeric columns sort numerically, text alphabetically
- Press again → reverse sort direction
- Numeric sort: parse as f64; special values (`-`, `*`, `n/a`) sort to bottom
- Header row NEVER moves (always frozen at top)
- Show sort indicator in header: `▲` (asc) / `▼` (desc)

## Filtering

- Press `f` on a column → InlineEditor appears in header area for filter input
- Each column has an independent filter (can filter multiple columns simultaneously)
- Filter is substring match (case-insensitive)
- Rows must match ALL active filters to be visible
- Active filter shown in header: column name becomes `Name [filter]`
- Press `F` on a column → clear that column's filter
- Press `Ctrl-F` → clear all filters
- Row count shown in status: "42/1000 rows" when filtered

## Inline Editing

- Enter on a cell → InlineEditor appears in-place (same widget as todo tree)
- Escape → cancel edit
- Enter → confirm edit, advance cursor one row down
- Tab → confirm edit, advance cursor one column right (no-op on rightmost column)
- Modified cells marked dirty (subtle color change)
- `:w` saves back to file (re-serialize with original delimiter)

## Implementation

### File Structure

```
src/views/csv_view/
  mod.rs          — CsvView struct, View trait impl, delegation
  draw.rs         — draw logic (header, grid, cursor, filters)
  handle.rs       — key handling (navigation, sort, filter, edit)
src/csv_parse.rs  — delimiter detection, parsing, serialization, type detection
```

### `src/views/csv_view/mod.rs`

```rust
pub struct CsvView {
    state: ViewState,
    path: PathBuf,
    delimiter: char,
    headers: Option<Vec<String>>,
    rows: Vec<Vec<String>>,
    col_widths: Vec<u16>,
    col_types: Vec<ColType>,  // Text or Numeric
    cursor_row: usize,
    cursor_col: usize,
    scroll_row: usize,
    scroll_col: usize,
    sort_col: Option<usize>,
    sort_asc: bool,
    filters: Vec<String>,     // per-column filter text (empty = no filter)
    visible_rows: Vec<usize>, // indices into rows[] after filtering
    editing: Option<InlineEditor>,
    dirty: bool,
}

enum ColType {
    Text,
    Numeric { max_before_dot: u16, max_after_dot: u16 },
}
```

### Column Width Calculation
```
for each column:
  scan all rows, find max display width
  cap at terminal_width / visible_columns (min 5, max 40)
  for Numeric columns: track max digits before/after dot
```

### Decimal Alignment
```
  123.45    →  "  123.45"
    0.7     →  "    0.7 "
 1000       →  " 1000   "
```
Align on the `.` position. If no dot, right-align the integer part.

### File Opening

Detect CSV by extension: `.csv`, `.tsv`, `.tab`, `.dat`
Or by content: if first line has consistent delimiters.

Register in `src/handler_open.rs`: if extension matches, open CsvView instead of EditorView.

### Save

Re-serialize using original delimiter. Preserve quoting for fields containing delimiter or newlines.

## Files to Create/Modify

- `src/views/csv_view/mod.rs` — view struct, View impl
- `src/views/csv_view/draw.rs` — rendering
- `src/views/csv_view/handle.rs` — key handling
- `src/csv_parse.rs` — delimiter detection, parsing, serialization, type detection
- `src/views/mod.rs` — add `pub mod csv_view;`
- `src/lib.rs` — add `pub mod csv_parse;`
- `src/handler_open.rs` — detect CSV extension, open CsvView

## Testing

1. Open a `.csv` file → columns aligned, header highlighted, vertical separators
2. Navigate with h/j/k/l → cursor moves between cells
3. Press `s` → sort by column (numeric auto-detected)
4. Press `s` again → reverse sort
5. Special values (`-`, `n/a`) sort to bottom in numeric columns
6. Press `f` → filter input appears, typing filters rows
7. Multiple column filters combine (AND logic)
8. Press Enter → inline edit, confirm advances down
9. Press Tab → inline edit, confirm advances right
10. `:w` → file saved with correct delimiter
11. Open `.tsv` → tab delimiter detected
12. Numeric column → decimal-aligned
13. Large file (1000+ rows) → scrolls smoothly, header frozen
14. Header never moves during sort

## Constraints

- Pure Rust parsing (no external crate needed for basic CSV — just split on delimiter with quote handling)
- No external tools
- Reuse InlineEditor from txv-widgets
- 240 code lines per file max (hence the directory split)
- Errors shown in status bar (file read/write failures)
