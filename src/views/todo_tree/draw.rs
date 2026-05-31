//! Draw logic for TodoTreeView — delegates tree rendering to TreeTableView.

use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;

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
        let draw_h = h.saturating_sub(filter_offset);
        // Set bounds on inner TreeTableView and draw it
        let inner_bounds = Rect::new(0, filter_offset, w, draw_h);
        self.inner.state.set_bounds(inner_bounds);
        if self.group.is_focused() {
            self.inner.state.set_focused(true);
        } else {
            self.inner.state.set_focused(false);
        }
        self.inner.draw();
        // Blit inner buffer onto group buffer
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        unsafe { (*buf_ptr).blit(self.inner.state.buffer(), 0, filter_offset) };
        self.draw_filter_status(w, filter_offset);
        self.position_and_blit_child(w, draw_h as usize, filter_offset);
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
            let indent = (depth * 2 + 2) as u16;
            (indent, screen_y, w.saturating_sub(indent))
        } else {
            return;
        };
        self.group.set_child_bounds(0, Rect::new(x, y, cw, 1));
        if let Some(child) = self.group.child_mut(0) {
            child.draw();
        }
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        if let Some(child) = self.group.child(0) {
            let (ox, oy) = self.group.child_origin(0);
            unsafe { (*buf_ptr).blit(child.buffer(), ox, oy) };
        }
    }
}
