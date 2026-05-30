//! Draw logic for TodoTreeView.

use txv_core::buffer::Buffer;
use txv_core::cell::Style;
use txv_core::geometry::Rect;
use txv_core::palette::{palette, StyleId};
use txv_widgets::tree_view::TreeData;

use super::model::{self, Completion, WorkStatus};
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
        let (status_icon, prio_icon, notes_icon) = self.badge_for(id);
        let mut x = indent + 2;
        self.group.buffer_mut().print(x, y, status_icon, style);
        x += 1;
        if !prio_icon.is_empty() {
            self.group.buffer_mut().print(x, y, prio_icon, style);
        }
        x += 1;
        if !notes_icon.is_empty() {
            self.group.buffer_mut().print(x, y, notes_icon, style);
        }
        x += 1;
        let label = self.inner.data.label(id).to_string();
        self.group.buffer_mut().print(x, y, &label, style);
    }

    /// Compute badge icons for a node: (status, priority, notes).
    fn badge_for(&self, id: usize) -> (&'static str, &'static str, &'static str) {
        let Some(item) = self.inner.data.item_at(id) else {
            return ("○", "", "");
        };
        let collapsed = self.inner.data.is_expandable(id) && !self.inner.data.is_expanded(id);
        let status = if item.completed == Completion::Done {
            "✓"
        } else if collapsed && model::effective_in_progress(item) {
            "▶"
        } else if collapsed && model::effective_paused(item) {
            "⏸"
        } else if item.work_status == WorkStatus::InProgress {
            "▶"
        } else if item.work_status == WorkStatus::Paused {
            "⏸"
        } else if item.completed == Completion::Partial {
            "◐"
        } else {
            "○"
        };
        let prio = if collapsed {
            model::effective_priority(item)
        } else {
            item.priority.unwrap_or(0)
        };
        let prio_icon = Self::priority_braille(prio);
        let has_notes = if collapsed {
            model::effective_has_notes(item)
        } else {
            !item.note.is_empty()
        };
        let notes = if has_notes {
            "♪"
        } else {
            ""
        };
        (status, prio_icon, notes)
    }

    fn priority_braille(prio: u8) -> &'static str {
        match prio {
            0 => "",
            1 => "⠁",
            2 => "⠃",
            3 => "⠇",
            4 => "⡇",
            5 => "⣇",
            6 => "⣧",
            7 => "⣷",
            8 => "⣿",
            _ => "⣿",
        }
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
            let indent = (depth * 2 + 5) as u16;
            (indent, screen_y, w.saturating_sub(indent))
        } else {
            return;
        };
        self.group.set_child_bounds(0, Rect::new(x, y, cw, 1));
        if let Some(child) = self.group.child_mut(0) {
            child.draw();
        }
        // Blit at child origin (standard group composite pattern)
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        if let Some(child) = self.group.child(0) {
            let (ox, oy) = self.group.child_origin(0);
            unsafe { (*buf_ptr).blit(child.buffer(), ox, oy) };
        }
    }
}
