# CSV Viewer / Editor

## Overview

A tabular view for CSV/TSV files that opens in the main panel. Aligned columns,
smart formatting, sortable, and inline-editable using the existing InlineEditor widget.

## Display Rules

### Column Alignment
- **No delimiter characters shown** — columns separated by whitespace only
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

## Navigation

| Key | Action |
|-----|--------|
| j/k, ↑/↓ | Move cursor row |
| h/l, ←/→ | Move cursor column |
| g/G | First/last row |
| 0/$ | First/last column |
| Enter | Edit cell (inline editor) |
| Tab | Next cell |
| Shift+Tab | Previous cell |
| s | Sort by current column (toggle asc/desc) |
| S | Sort by current column (numeric) |
| / | Search within table |
| q/Esc | Close view |

## Sorting

- Press `s` on a column → sort all data rows by that column (alphabetic)
- Press again → reverse sort
- Press `S` → numeric sort (parse as f64, non-numeric at bottom)
- Header row never moves
- Show sort indicator in header: `▲` / `▼`

## Inline Editing

- Enter on a cell → InlineEditor appears in-place (same widget as todo tree)
- Escape → cancel edit
- Enter → confirm edit, update cell value
- Tab → confirm and move to next cell
- Modified cells marked dirty (subtle color change)
- `:w` saves back to file (re-serialize with original delimiter)

## Implementation

### `src/views/csv_view.rs`

```rust
pub struct CsvView {
    state: ViewState,
    path: PathBuf,
    delimiter: char,
    headers: Option<Vec<String>>,
    rows: Vec<Vec<String>>,
    col_widths: Vec<u16>,
    col_types: Vec<ColType>,  // Text, Integer, Decimal
    cursor_row: usize,
    cursor_col: usize,
    scroll_row: usize,
    scroll_col: usize,
    sort_col: Option<usize>,
    sort_asc: bool,
    editing: Option<InlineEditor>,
    dirty: bool,
}

enum ColType {
    Text,
    Integer,
    Decimal { max_before_dot: u16, max_after_dot: u16 },
}
```

### Column Width Calculation
```
for each column:
  scan all rows, find max display width
  cap at terminal_width / visible_columns (min 5, max 40)
  for Decimal columns: track max digits before/after dot
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

Register in file open handler: if extension matches, open CsvView instead of EditorView.

### Save

Re-serialize using original delimiter. Preserve quoting for fields containing delimiter or newlines.

## Files to Create/Modify

- `src/views/csv_view.rs` — main view implementation
- `src/views/mod.rs` — add `pub mod csv_view;`
- `src/handler_open.rs` — detect CSV extension, open CsvView
- `src/csv_parse.rs` — delimiter detection, parsing, serialization

## Testing

1. Open a `.csv` file → columns aligned, header highlighted
2. Navigate with h/j/k/l → cursor moves between cells
3. Press `s` → sort by column
4. Press Enter → inline edit, confirm with Enter
5. `:w` → file saved with correct delimiter
6. Open `.tsv` → tab delimiter detected
7. Numeric column → decimal-aligned
8. Large file (1000+ rows) → scrolls smoothly, header frozen

## Constraints

- Pure Rust parsing (no external crate needed for basic CSV — just split on delimiter with quote handling)
- No external tools
- Reuse InlineEditor from txv-widgets
- Errors shown in status bar (file read/write failures)
