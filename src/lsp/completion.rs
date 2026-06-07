//! Completion popup state and rendering.

use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use super::requests::{CompletionItem, CompletionKind};

/// Completion popup state.
pub struct CompletionPopup {
    pub(crate) items: Vec<CompletionItem>,
    pub(crate) selected: usize,
    pub(crate) visible: bool,
    pub(crate) anchor_x: u16,
    pub(crate) anchor_y: u16,
    pub(crate) scroll: usize,
}

impl Default for CompletionPopup {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionPopup {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            visible: false,
            anchor_x: 0,
            anchor_y: 0,
            scroll: 0,
        }
    }

    /// Show the popup with items at the given anchor position.
    pub fn show(&mut self, items: Vec<CompletionItem>, x: u16, y: u16) {
        if items.is_empty() {
            self.hide();
            return;
        }
        self.items = items;
        self.selected = 0;
        self.scroll = 0;
        self.visible = true;
        self.anchor_x = x;
        self.anchor_y = y;
    }

    /// Hide the popup.
    pub fn hide(&mut self) {
        self.visible = false;
        self.items.clear();
        self.selected = 0;
    }

    /// Move selection down.
    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
            self.ensure_visible();
        }
    }

    /// Move selection up.
    pub fn prev(&mut self) {
        if !self.items.is_empty() {
            self.selected = self.selected.checked_sub(1).unwrap_or(self.items.len() - 1);
            self.ensure_visible();
        }
    }

    fn ensure_visible(&mut self) {
        let page = 8;
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + page {
            self.scroll = self.selected + 1 - page;
        }
    }

    /// Get the selected item's insert text (or label).
    pub fn selected_text(&self) -> Option<&str> {
        let item = self.items.get(self.selected)?;
        Some(item.insert_text.as_deref().unwrap_or(&item.label))
    }

    /// Get the selected item's additional text edits (e.g. auto-imports).
    pub fn selected_additional_edits(&self) -> &[super::text_edit::TextEdit] {
        self.items
            .get(self.selected)
            .map(|i| i.additional_edits.as_slice())
            .unwrap_or(&[])
    }

    /// Draw the popup on the buffer.
    pub fn draw(&self, buf: &mut Buffer) {
        if !self.visible || self.items.is_empty() {
            return;
        }
        let max_items = 8.min(self.items.len());
        let (_max_label, _has_detail, max_width) = self.compute_dimensions(max_items);
        let Some((x, y, visible_count, max_width)) = self.compute_position(buf, max_items, max_width) else {
            return;
        };
        self.draw_items(buf, x, y, visible_count, max_width);
    }

    fn compute_dimensions(&self, max_items: usize) -> (usize, bool, u16) {
        let max_label = self
            .items
            .iter()
            .take(max_items)
            .map(|i| self.display_label(i).len())
            .max()
            .unwrap_or(10)
            .min(30);
        let has_detail = self.items.iter().take(max_items).any(|i| i.detail.is_some());
        let max_width = if has_detail {
            (max_label + 20).min(50) as u16 + 2
        } else {
            max_label as u16 + 2
        };
        (max_label, has_detail, max_width)
    }

    fn compute_position(&self, buf: &Buffer, max_items: usize, max_width: u16) -> Option<(u16, u16, usize, u16)> {
        let buf_w = buf.width();
        let buf_h = buf.height();
        let max_width = max_width.min(buf_w.saturating_sub(1));
        let x = if self.anchor_x + max_width > buf_w {
            buf_w.saturating_sub(max_width)
        } else {
            self.anchor_x
        };
        let rows_below = buf_h.saturating_sub(self.anchor_y + 1);
        let rows_above = self.anchor_y;
        let (y, visible) = if rows_below >= max_items as u16 {
            (self.anchor_y + 1, max_items)
        } else if rows_above >= max_items as u16 {
            (self.anchor_y.saturating_sub(max_items as u16), max_items)
        } else if rows_below >= rows_above {
            (self.anchor_y + 1, rows_below as usize)
        } else {
            let n = rows_above as usize;
            (self.anchor_y.saturating_sub(n as u16), n)
        };
        if visible == 0 {
            None
        } else {
            Some((x, y, visible, max_width))
        }
    }

    fn max_label_width(&self, max_items: usize) -> usize {
        self.items
            .iter()
            .take(max_items)
            .map(|i| self.display_label(i).len())
            .max()
            .unwrap_or(10)
            .min(30)
    }

    fn draw_items(&self, buf: &mut Buffer, x: u16, y: u16, max_items: usize, max_width: u16) {
        let max_label = self.max_label_width(max_items);
        let has_detail = self.items.iter().take(max_items).any(|i| i.detail.is_some());
        let pal = palette();
        let normal = pal.style(StyleId::PopupBackground);
        let selected = pal.style(StyleId::PopupSelected);
        let dim_fg = pal.style(StyleId::Dim).fg();
        let detail_style = Style::new(dim_fg, normal.bg()).with_attrs(normal.attrs());
        let detail_sel_style = Style::new(dim_fg, selected.bg()).with_attrs(selected.attrs());
        let detail_col = (max_label + 2) as u16;

        for (i, item) in self.items.iter().skip(self.scroll).take(max_items).enumerate() {
            let row = y + i as u16;
            let is_sel = self.scroll + i == self.selected;
            let style = if is_sel {
                selected
            } else {
                normal
            };
            buf.hline(x, row, max_width, ' ', style);
            let display_label = self.display_label(item);
            let label_end = display_label.len().min(max_label);
            buf.print(x + 1, row, &display_label[..label_end], style);
            if has_detail {
                let ds = if is_sel {
                    detail_sel_style
                } else {
                    detail_style
                };
                self.draw_detail(buf, item, x + detail_col, row, max_width - detail_col, ds);
            }
        }
    }

    fn draw_detail(&self, buf: &mut Buffer, item: &CompletionItem, x: u16, row: u16, avail: u16, style: Style) {
        if let Some(ref detail) = item.detail {
            let max = avail.saturating_sub(1) as usize;
            let d = if detail.len() > max {
                &detail[..max]
            } else {
                detail.as_str()
            };
            buf.print(x, row, d, style);
        }
    }

    /// Format display label: append parens for functions/methods.
    fn display_label(&self, item: &CompletionItem) -> String {
        match item.kind {
            CompletionKind::Function | CompletionKind::Method => {
                let has_params = item
                    .detail
                    .as_deref()
                    .map(|d| {
                        // detail like "fn(x: i32) -> T" or "fn() -> T"
                        d.starts_with("fn(") && !d.starts_with("fn()")
                    })
                    .unwrap_or(false);
                if has_params {
                    format!("{}(…)", item.label)
                } else {
                    format!("{}()", item.label)
                }
            }
            CompletionKind::Other => item.label.clone(),
        }
    }
}
