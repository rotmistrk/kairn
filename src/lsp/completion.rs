//! Completion popup state and rendering.

use txv_core::prelude::*;

use super::requests::{CompletionItem, CompletionKind};

/// Completion popup state.
pub struct CompletionPopup {
    pub items: Vec<CompletionItem>,
    pub selected: usize,
    pub visible: bool,
    pub anchor_x: u16,
    pub anchor_y: u16,
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

    /// Draw the popup as an overlay on the buffer.
    pub fn draw(&self, buf: &mut Buffer) {
        if !self.visible || self.items.is_empty() {
            return;
        }
        let max_items = 8.min(self.items.len());
        // Compute width: label + detail (capped)
        let max_label = self
            .items
            .iter()
            .take(max_items)
            .map(|i| self.display_label(i).len())
            .max()
            .unwrap_or(10)
            .min(30);
        let has_detail = self.items.iter().take(max_items).any(|i| i.detail.is_some());
        let detail_col = max_label + 2; // 1 padding + label + 1 space
        let max_width = if has_detail {
            (max_label + 20).min(50) as u16 + 2
        } else {
            max_label as u16 + 2
        };

        let buf_w = buf.width();
        let buf_h = buf.height();

        // Clamp popup within buffer bounds
        let max_width = max_width.min(buf_w.saturating_sub(1));
        let x = if self.anchor_x + max_width > buf_w {
            buf_w.saturating_sub(max_width)
        } else {
            self.anchor_x
        };
        // Show above cursor if not enough room below
        let rows_below = buf_h.saturating_sub(self.anchor_y + 1);
        let rows_above = self.anchor_y;
        let (y, max_items) = if rows_below >= max_items as u16 {
            (self.anchor_y + 1, max_items)
        } else if rows_above >= max_items as u16 {
            (self.anchor_y.saturating_sub(max_items as u16), max_items)
        } else {
            // Use whichever side has more room
            if rows_below >= rows_above {
                (self.anchor_y + 1, rows_below as usize)
            } else {
                let n = rows_above as usize;
                (self.anchor_y.saturating_sub(n as u16), n)
            }
        };
        if max_items == 0 {
            return;
        }

        let pal = txv_core::palette::palette();
        let normal = pal.popup().background();
        let selected = pal.popup().selected();
        let dim_fg = pal.base().dim().fg;
        let detail_style = Style { fg: dim_fg, ..normal };
        let detail_sel_style = Style { fg: dim_fg, ..selected };

        for (i, item) in self.items.iter().skip(self.scroll).take(max_items).enumerate() {
            let row = y + i as u16;
            let abs_idx = self.scroll + i;
            let style = if abs_idx == self.selected {
                selected
            } else {
                normal
            };
            buf.hline(x, row, max_width, ' ', style);
            let display_label = self.display_label(item);
            let label = if display_label.len() > max_label {
                &display_label[..max_label]
            } else {
                display_label.as_str()
            };
            buf.print(x + 1, row, label, style);
            // Draw detail (type) in grey
            if let Some(ref detail) = item.detail {
                let ds = if abs_idx == self.selected {
                    detail_sel_style
                } else {
                    detail_style
                };
                let avail = max_width.saturating_sub(detail_col as u16 + 1) as usize;
                let d = if detail.len() > avail {
                    &detail[..avail]
                } else {
                    detail.as_str()
                };
                buf.print(x + detail_col as u16, row, d, ds);
            }
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
