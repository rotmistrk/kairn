//! Draw logic for TodoTreeView.

use txv_core::cell::Style;
use txv_widgets::tree_view::TreeData;

use super::model;
use super::TodoTreeView;

impl TodoTreeView {
    pub(super) fn draw_tree(&mut self) {
        let w = self.inner.buffer_mut().width();
        let h = self.inner.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        if self.inner.data.visible_count() == 0 {
            let dim = txv_core::palette::palette().style(txv_core::palette::StyleId::Dim);
            self.inner.buffer_mut().print(0, 0, "  (empty — press 'n' to add)", dim);
            return;
        }
        let pal = txv_core::palette::palette();
        let filter_offset: u16 = if self.filter_editor.is_some() || !self.inner.data.filter_text.is_empty() {
            1
        } else {
            0
        };
        let draw_h = h.saturating_sub(filter_offset) as usize;
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
                if self.inner.is_focused() {
                    let cs = pal.style(txv_core::palette::StyleId::CursorFocused);
                    Style {
                        fg: node_style.fg,
                        bg: cs.bg,
                        attrs: cs.attrs,
                    }
                } else {
                    let cs = pal.style(txv_core::palette::StyleId::CursorUnfocused);
                    Style {
                        fg: node_style.fg,
                        bg: cs.bg,
                        attrs: node_style.attrs,
                    }
                }
            } else {
                node_style
            };
            let y = filter_offset + row as u16;
            self.inner.buffer_mut().hline(0, y, w, ' ', style);
            let x = indent;
            self.inner.buffer_mut().print(x, y, marker, style);
            let checkbox = if let Some(item) = self.inner.data.item_at(id) {
                match item.completed {
                    model::Completion::Done => "[x] ",
                    _ => "[ ] ",
                }
            } else {
                "[ ] "
            };
            self.inner.buffer_mut().print(x + 2, y, checkbox, style);
            let label = self.inner.data.label(id).to_string();
            self.inner.buffer_mut().print(x + 6, y, &label, style);
        }
        // Inline editor overlay
        if let Some(ref editor) = self.editing {
            let scroll_offset = self.inner.scroll.offset;
            if editor.row >= scroll_offset {
                let screen_row = (editor.row - scroll_offset) as u16;
                if screen_row < draw_h as u16 {
                    let y = filter_offset + screen_row;
                    let id = self.inner.data.visible_id(editor.row);
                    let depth = self.inner.data.depth(id);
                    let indent = (depth * 2 + 6) as u16;
                    let ex = indent;
                    let ew = w.saturating_sub(indent);
                    let style = pal.style(txv_core::palette::StyleId::EditOverlay);
                    editor.draw(self.inner.buffer_mut(), ex, y, ew, style);
                }
            }
        }
        // Filter bar at top
        if filter_offset > 0 {
            let y = 0;
            let style = pal.style(txv_core::palette::StyleId::EditOverlay);
            self.inner.buffer_mut().hline(0, y, w, ' ', style);
            self.inner.buffer_mut().print(0, y, "/", style);
            if let Some(ref editor) = self.filter_editor {
                editor.draw(self.inner.buffer_mut(), 1, y, w.saturating_sub(1), style);
            } else {
                let ft = self.inner.data.filter_text.clone();
                self.inner.buffer_mut().print(1, y, &ft, style);
            }
        }
    }
}
