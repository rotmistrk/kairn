//! Viewport highlight computation helper for EditorView draw.

use crate::highlight::HlSpan;

use super::EditorView;

impl EditorView {
    /// Compute highlighted spans for the visible viewport range.
    pub(super) fn compute_viewport_spans(&self, scroll: usize, viewport_end: usize) -> Vec<Vec<HlSpan>> {
        let total_lines = self.editor.buf().line_count();
        let mut cache = self.hl_cache.borrow_mut();
        cache.highlight_viewport(
            scroll,
            viewport_end,
            total_lines,
            |i| self.editor.buf().line(i).unwrap_or_default(),
            self.highlighter.syntax_set(),
            self.highlighter.theme(),
        )
    }
}
