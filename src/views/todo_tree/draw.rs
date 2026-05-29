//! Draw logic for TodoTreeView.

use txv_core::buffer::Buffer;
use txv_core::cell::Style;
use txv_core::geometry::Rect;
use txv_core::palette::{palette, StyleId};
use txv_widgets::tree_view::TreeData;

use super::model::Completion;
use super::TodoTreeView;

impl TodoTreeView {
    pub(super) fn draw_tree(&mut self) {
        let b = self.group.bounds();
        let w = b.w;
        let h = b.h;
        if w == 0 || h == 0 {
            return;
        }
        self.group.buffer_mut().fill(' ', Style::default());
        if self.inner.data.visible_count() == 0 {
            let dim = palette().style(StyleId::Dim);
            self.group.buffer_mut().print(0, 0, "  (empty — press 'n' to add)", dim);
            return;
        }
        let has_filter = self.filter_active || !self.inner.data.filter_text.is_empty();
        let filter_offset: u16 = u16::from(has_filter);
        let draw_h = h.saturating_sub(filter_offset) as usize;
        self.inner.scroll.set_viewport(draw_h);
        self.inner.scroll.set_total(self.inner.data.visible_count());
        self.inner.scroll.ensure_visible(self.inner.cursor);
        self.draw_tree_rows(w, draw_h, filter_offset);
        self.draw_filter_status(w, filter_offset);
        self.position_and_blit_child(w, draw_h, filter_offset);
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
        self.group.buffer_mut().hline(0, y, w, ' ', style);
        self.group.buffer_mut().print(indent, y, marker, style);
        let checkbox = if let Some(item) = self.inner.data.item_at(id) {
            match item.completed {
                Completion::Done => "[x] ",
                _ => "[ ] ",
            }
        } else {
            "[ ] "
        };
        self.group.buffer_mut().print(indent + 2, y, checkbox, style);
        let label = self.inner.data.label(id).to_string();
        self.group.buffer_mut().print(indent + 6, y, &label, style);
    }

    fn row_style(&self, idx: usize, id: usize) -> Style {
        let pal = palette();
        let mut node_style = self.inner.data.style(id);
        if self.inner.data.is_in_important_subtree(id) {
            node_style.attrs.bold = true;
        }
        if idx == self.inner.cursor {
            let cs = if self.group.is_focused() {
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

    fn draw_filter_status(&mut self, w: u16, filter_offset: u16) {
        if filter_offset == 0 {
            return;
        }
        let style = palette().style(StyleId::StatusBar);
        self.group.buffer_mut().hline(0, 0, w, ' ', style);
        self.group.buffer_mut().print(0, 0, "/", style);
        if !self.filter_active {
            let ft = self.inner.data.filter_text.clone();
            self.group.buffer_mut().print(1, 0, &ft, style);
        }
    }

    /// Position the InputLine child and blit it onto the group buffer.
    fn position_and_blit_child(&mut self, w: u16, draw_h: usize, filter_offset: u16) {
        if self.group.child_count() == 0 {
            return;
        }
        let (x, y, cw) = if self.filter_active {
            (1u16, 0u16, w.saturating_sub(1))
        } else if let Some(row) = self.editing_row {
            let scroll_offset = self.inner.scroll.offset;
            if row < scroll_offset || (row - scroll_offset) >= draw_h {
                return;
            }
            let screen_y = filter_offset + (row - scroll_offset) as u16;
            let id = self.inner.data.visible_id(row);
            let depth = self.inner.data.depth(id);
            let indent = (depth * 2 + 6) as u16;
            (indent, screen_y, w.saturating_sub(indent))
        } else {
            return;
        };
        self.group.set_child_bounds(0, Rect::new(x, y, cw, 1));
        if let Some(child) = self.group.child_mut(0) {
            child.draw();
        }
        // Safety: child (immutable borrow) and buffer (mutable) are disjoint.
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        if let Some(child) = self.group.child(0) {
            let cb = child.bounds();
            unsafe { (*buf_ptr).blit(child.buffer(), cb.x, cb.y) };
        }
    }
}
