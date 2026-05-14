//! Draw logic for TodoTreeView.

use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;

use super::model;
use super::TodoTreeView;

impl TodoTreeView {
    pub(super) fn draw_tree(&self, surface: &mut Surface) {
        if self.inner.data.visible_count() == 0 {
            let b = self.inner.state.bounds();
            let dim = txv_core::palette::palette().base.dim.to_style();
            surface.print(b.x, b.y, "  (empty — press 'n' to add)", dim);
            return;
        }
        let pal = txv_core::palette::palette();
        let b = self.inner.state.bounds();
        let filter_offset: u16 = if self.filter_editor.is_some() || !self.inner.data.filter_text.is_empty() {
            1
        } else {
            0
        };
        let draw_h = b.h.saturating_sub(filter_offset) as usize;
        for row in 0..draw_h {
            let idx = self.inner.scroll.offset + row;
            if idx >= self.inner.data.visible_count() {
                break;
            }
            let id = self.inner.data.visible_id(idx);
            let depth = self.inner.data.depth(id);
            let indent = (depth * 2) as u16;
            let marker = if self.inner.data.is_expandable(id) {
                if self.inner.data.is_expanded(id) {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };
            let mut node_style = self.inner.data.style(id);
            if self.inner.data.is_in_important_subtree(id) {
                node_style.attrs.bold = true;
            }
            let style = if idx == self.inner.cursor {
                if self.inner.state.is_focused() {
                    pal.interactive.cursor_focused.resolve(&node_style)
                } else {
                    pal.interactive.cursor_unfocused.resolve(&node_style)
                }
            } else {
                node_style
            };
            let y = b.y + filter_offset + row as u16;
            surface.hline(b.x, y, b.w, ' ', style);
            let x = b.x + indent;
            surface.print(x, y, marker, style);
            let checkbox = if let Some(item) = self.inner.data.item_at(id) {
                match item.completed {
                    model::Completion::Done => "[x] ",
                    _ => "[ ] ",
                }
            } else {
                "[ ] "
            };
            surface.print(x + 2, y, checkbox, style);
            surface.print(x + 6, y, self.inner.data.label(id), style);
        }
        // Inline editor overlay
        if let Some(ref editor) = self.editing {
            let scroll_offset = self.inner.scroll.offset;
            if editor.row >= scroll_offset {
                let screen_row = (editor.row - scroll_offset) as u16;
                if screen_row < draw_h as u16 {
                    let y = b.y + filter_offset + screen_row;
                    let id = self.inner.data.visible_id(editor.row);
                    let depth = self.inner.data.depth(id);
                    let indent = (depth * 2 + 6) as u16;
                    let ex = b.x + indent;
                    let ew = b.w.saturating_sub(indent);
                    let style = pal.interactive.edit_overlay.to_style();
                    editor.draw(surface, ex, y, ew, style);
                }
            }
        }
        // Filter bar at top
        if filter_offset > 0 {
            let y = b.y;
            let style = pal.interactive.edit_overlay.to_style();
            surface.hline(b.x, y, b.w, ' ', style);
            surface.print(b.x, y, "/", style);
            if let Some(ref editor) = self.filter_editor {
                editor.draw(surface, b.x + 1, y, b.w.saturating_sub(1), style);
            } else {
                surface.print(b.x + 1, y, &self.inner.data.filter_text, style);
            }
        }
    }
}
