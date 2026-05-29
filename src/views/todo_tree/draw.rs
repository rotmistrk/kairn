//! Draw logic for TodoTreeView.

use txv_core::cell::Style;
use txv_core::geometry::Rect;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::View;
use txv_widgets::tree_view::TreeData;

use super::model::Completion;
use super::TodoTreeView;

impl TodoTreeView {
    pub(super) fn draw_tree(&mut self) {
        let w = self.inner.buffer_mut().width();
        let h = self.inner.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        if self.inner.data.visible_count() == 0 {
            let dim = palette().style(StyleId::Dim);
            self.inner.buffer_mut().print(0, 0, "  (empty — press 'n' to add)", dim);
            return;
        }
        let has_filter = self.filter_editor.is_some() || !self.inner.data.filter_text.is_empty();
        let filter_offset: u16 = if has_filter {
            1
        } else {
            0
        };
        let draw_h = h.saturating_sub(filter_offset) as usize;
        self.draw_tree_rows(w, draw_h, filter_offset);
        self.draw_edit_overlay(w, draw_h, filter_offset);
        self.draw_filter_bar(w, filter_offset);
    }

    fn draw_tree_rows(&mut self, w: u16, draw_h: usize, filter_offset: u16) {
        for row in 0..draw_h {
            let idx = self.inner.scroll.offset + row;
            if idx >= self.inner.data.visible_count() {
                break;
            }
            self.draw_single_row(w, row, idx, filter_offset);
        }
    }

    fn draw_single_row(&mut self, w: u16, row: usize, idx: usize, filter_offset: u16) {
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
        let style = self.row_style(idx, id);
        let y = filter_offset + row as u16;
        self.inner.buffer_mut().hline(0, y, w, ' ', style);
        self.inner.buffer_mut().print(indent, y, marker, style);
        let checkbox = if let Some(item) = self.inner.data.item_at(id) {
            match item.completed {
                Completion::Done => "[x] ",
                _ => "[ ] ",
            }
        } else {
            "[ ] "
        };
        self.inner.buffer_mut().print(indent + 2, y, checkbox, style);
        let label = self.inner.data.label(id).to_string();
        self.inner.buffer_mut().print(indent + 6, y, &label, style);
    }

    fn row_style(&self, idx: usize, id: usize) -> Style {
        let pal = palette();
        let mut node_style = self.inner.data.style(id);
        if self.inner.data.is_in_important_subtree(id) {
            node_style.attrs.bold = true;
        }
        if idx == self.inner.cursor {
            let cs = if self.inner.is_focused() {
                pal.style(StyleId::CursorFocused)
            } else {
                pal.style(StyleId::CursorUnfocused)
            };
            Style {
                fg: node_style.fg,
                bg: cs.bg,
                attrs: cs.attrs,
            }
        } else {
            node_style
        }
    }

    fn draw_edit_overlay(&mut self, w: u16, draw_h: usize, filter_offset: u16) {
        let Some((row, ref mut input)) = self.editing else {
            return;
        };
        let scroll_offset = self.inner.scroll.offset;
        if row < scroll_offset || (row - scroll_offset) as u16 >= draw_h as u16 {
            return;
        }
        let screen_row = (row - scroll_offset) as u16;
        let y = filter_offset + screen_row;
        let id = self.inner.data.visible_id(row);
        let depth = self.inner.data.depth(id);
        let indent = (depth * 2 + 6) as u16;
        let ew = w.saturating_sub(indent);
        input.set_bounds(Rect::new(0, 0, ew, 1));
        input.draw();
        self.inner.buffer_mut().blit(input.buffer(), indent, y);
    }

    fn draw_filter_bar(&mut self, w: u16, filter_offset: u16) {
        if filter_offset == 0 {
            return;
        }
        let style = palette().style(StyleId::EditOverlay);
        self.inner.buffer_mut().hline(0, 0, w, ' ', style);
        self.inner.buffer_mut().print(0, 0, "/", style);
        if let Some(ref mut input) = self.filter_editor {
            let fw = w.saturating_sub(1);
            input.set_bounds(Rect::new(0, 0, fw, 1));
            input.draw();
            self.inner.buffer_mut().blit(input.buffer(), 1, 0);
        } else {
            let ft = self.inner.data.filter_text.clone();
            self.inner.buffer_mut().print(1, 0, &ft, style);
        }
    }
}
