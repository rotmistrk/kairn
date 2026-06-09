//! Draw logic for TodoTreeView — only draws own pixels (filter row).
//! TreeTableView (child 0) and InputLine (child 1) are rendered by the group pipeline.

use txv_core::prelude::*;

use super::TodoTreeView;

impl TodoTreeView {
    pub(super) fn draw_tree(&mut self) {
        let b = self.group.bounds();
        let w = b.w();
        let h = b.h();
        if w == 0 || h == 0 {
            return;
        }
        if !self.group.is_child_visible(0) {
            // Tree is hidden (empty) — show placeholder
            let dim = palette().style(StyleId::Dim);
            self.group.buffer_mut().fill(' ', Style::default());
            self.group.buffer_mut().print(0, 0, "  (empty — press 'n' to add)", dim);
            return;
        }
        // Set focused state on inner tree
        if self.group.is_focused() {
            self.inner_mut().state_mut().set_focused(true);
        } else {
            self.inner_mut().state_mut().set_focused(false);
        }
        // Draw filter status row if active
        let has_filter = self.filter_active || !self.inner_mut().data_mut().filter_text.is_empty();
        if has_filter {
            let filter_row = h.saturating_sub(1);
            self.draw_filter_status(w, filter_row);
        }
    }

    fn draw_filter_status(&mut self, w: u16, filter_row: u16) {
        let style = palette().style(StyleId::StatusBar);
        self.group.buffer_mut().hline(0, filter_row, w, ' ', style);
        self.group.buffer_mut().print(0, filter_row, "/", style);
        if !self.filter_active {
            let ft = self.inner_mut().data_mut().filter_text.clone();
            self.group.buffer_mut().print(1, filter_row, &ft, style);
        }
    }
}
