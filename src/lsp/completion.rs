//! Completion popup state and rendering.

use txv_core::prelude::*;

use super::requests::{CompletionItem, CompletionKind};

/// Completion popup state.
pub struct CompletionPopup {
    pub items: Vec<CompletionItem>,
    pub selected: usize,
    pub visible: bool,
    /// Anchor position (screen x, y) for the popup.
    pub anchor_x: u16,
    pub anchor_y: u16,
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
        }
    }

    /// Move selection up.
    pub fn prev(&mut self) {
        if !self.items.is_empty() {
            self.selected = self.selected.checked_sub(1).unwrap_or(self.items.len() - 1);
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

        let x = self.anchor_x;
        let y = self.anchor_y + 1; // below cursor

        let pal = txv_core::palette::palette();
        let normal = pal.popup.background.to_style();
        let selected = pal.popup.selected.to_style();
        let dim_fg = pal.base.dim.to_style().fg;
        let detail_style = Style { fg: dim_fg, ..normal };
        let detail_sel_style = Style { fg: dim_fg, ..selected };

        for (i, item) in self.items.iter().take(max_items).enumerate() {
            let row = y + i as u16;
            let style = if i == self.selected {
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
                let ds = if i == self.selected {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn items(labels: &[&str]) -> Vec<CompletionItem> {
        labels
            .iter()
            .map(|l| CompletionItem {
                label: l.to_string(),
                detail: None,
                insert_text: None,
                kind: CompletionKind::Other,
            })
            .collect()
    }

    #[test]
    fn show_and_navigate() {
        let mut popup = CompletionPopup::new();
        popup.show(items(&["foo", "bar", "baz"]), 5, 10);
        assert!(popup.visible);
        assert_eq!(popup.selected, 0);
        assert_eq!(popup.selected_text(), Some("foo"));

        popup.next();
        assert_eq!(popup.selected_text(), Some("bar"));

        popup.prev();
        assert_eq!(popup.selected_text(), Some("foo"));

        popup.prev(); // wraps
        assert_eq!(popup.selected_text(), Some("baz"));
    }

    #[test]
    fn show_empty_hides() {
        let mut popup = CompletionPopup::new();
        popup.show(Vec::new(), 0, 0);
        assert!(!popup.visible);
    }

    #[test]
    fn draw_renders_items() {
        let mut popup = CompletionPopup::new();
        popup.show(items(&["hello", "world"]), 0, 0);
        let mut buf = Buffer::new(20, 5);
        popup.draw(&mut buf);
        // First item at row 1 (anchor_y + 1)
        let cell = buf.cell(1, 1);
        assert_eq!(cell.ch, 'h');
    }
}
