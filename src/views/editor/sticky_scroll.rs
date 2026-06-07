//! Sticky scroll — detect enclosing scope headers to pin at top of viewport.

use crate::editor::Editor;

/// A pinned header line to display at top of viewport.
pub(super) struct StickyLine {
    pub(super) line_idx: usize,
    pub(super) text: String,
}

/// Find scope headers above the viewport that should be pinned.
/// Returns at most 2 lines (e.g. impl + fn).
pub(super) fn compute_sticky_lines(editor: &Editor, scroll: usize) -> Vec<StickyLine> {
    if scroll == 0 {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current_indent = indent_of_line(editor, scroll);

    // Walk backwards from scroll position looking for lines with less indent
    for line_idx in (0..scroll).rev() {
        let line = editor.buf().line(line_idx).unwrap_or_default();
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let indent = line.len() - trimmed.len();
        if indent < current_indent && is_scope_header(trimmed) {
            result.push(StickyLine {
                line_idx,
                text: line.clone(),
            });
            current_indent = indent;
            if result.len() >= 2 {
                break;
            }
        }
    }
    result.reverse();
    result
}

fn indent_of_line(editor: &Editor, line_idx: usize) -> usize {
    let line = editor.buf().line(line_idx).unwrap_or_default();
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        // Find next non-empty line's indent as reference
        4
    } else {
        line.len() - trimmed.len()
    }
}

fn is_scope_header(trimmed: &str) -> bool {
    // Rust
    trimmed.starts_with("fn ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub(crate) fn ")
        || trimmed.starts_with("pub(super) fn ")
        || trimmed.starts_with("impl ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("struct ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("mod ")
        || trimmed.starts_with("pub mod ")
        || trimmed.starts_with("trait ")
        || trimmed.starts_with("pub trait ")
        // Python
        || trimmed.starts_with("def ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("async def ")
        // JS/TS
        || trimmed.starts_with("function ")
        || trimmed.starts_with("export function ")
        || trimmed.starts_with("export class ")
        || trimmed.starts_with("export default ")
        // Go
        || trimmed.starts_with("func ")
        || trimmed.starts_with("type ")
        // C/C++/Java
        || (trimmed.contains('(') && !trimmed.starts_with("//") && !trimmed.starts_with('#'))
            && (trimmed.ends_with('{') || trimmed.ends_with(") {"))
}

/// Render a single sticky header line.
pub(super) fn draw_sticky(buf: &mut txv_core::buffer::Buffer, sl: &StickyLine, y: u16, w: u16) {
    use crate::app_palette::app_palette;
    let dim = app_palette().editor().gutter();
    buf.hline(0, y, w, ' ', dim);
    let text: String = sl.text.chars().take(w as usize).collect();
    buf.print(0, y, &text, dim);
}
