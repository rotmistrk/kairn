//! Shared hunk application logic for diff revert operations.
//!
//! Used by both `handle_deferred` (KairnDelegate) and `methods_diff` (EditorView).

use crate::editor::Editor;

/// Apply a hunk revert to the buffer: remove added lines and insert deleted text.
///
/// - `buf_lines`: buffer line indices that were added (to be deleted on revert)
/// - `deleted_text`: original lines that were deleted (to be re-inserted on revert)
/// - `insert_line`: fallback insertion point when `buf_lines` is empty
pub(crate) fn apply_hunk_to_buffer(
    editor: &mut Editor,
    buf_lines: &[usize],
    deleted_text: &[String],
    insert_line: usize,
) {
    let mut buf = editor.buf();
    buf.begin_group();
    if !buf_lines.is_empty() {
        let first = buf_lines[0];
        let last = buf_lines[buf_lines.len() - 1];
        let start_off = buf.line_col_to_offset(first, 0).unwrap_or(0);
        let end_off = if last + 1 < buf.line_count() {
            buf.line_col_to_offset(last + 1, 0).unwrap_or(buf.len())
        } else {
            buf.len()
        };
        if end_off > start_off {
            buf.delete(start_off, end_off);
        }
        if !deleted_text.is_empty() {
            let insert = deleted_text.join("\n") + "\n";
            let off = buf.line_col_to_offset(first, 0).unwrap_or(buf.len());
            buf.insert(off, &insert);
        }
    } else if !deleted_text.is_empty() {
        let insert = deleted_text.join("\n") + "\n";
        let off = buf.line_col_to_offset(insert_line, 0).unwrap_or(buf.len());
        buf.insert(off, &insert);
    }
    buf.end_group();
    drop(buf);
    editor.clamp_cursor();
}
