//! Completion popup state and rendering.

use txv_core::prelude::*;

use super::requests::CompletionItem;

/// Completion popup state.
pub struct CompletionPopup {
    pub items: Vec<CompletionItem>,
    pub selected: usize,
    pub visible: bool,
    /// Anchor position (screen x, y) for the popup.
    pub anchor_x: u16,
    pub anchor_y: u16,
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

    /// Draw the popup as an overlay on the surface.
    pub fn draw(&self, surface: &mut Surface) {
        if !self.visible || self.items.is_empty() {
            return;
        }
        let max_items = 8.min(self.items.len());
        let max_width = self
            .items
            .iter()
            .take(max_items)
            .map(|i| i.label.len())
            .max()
            .unwrap_or(10)
            .min(40) as u16
            + 2;

        let x = self.anchor_x;
        let y = self.anchor_y + 1; // below cursor

        let normal = Style {
            bg: Color::Ansi(0),
            fg: Color::Ansi(7),
            ..Style::default()
        };
        let selected = Style {
            bg: Color::Ansi(4),
            fg: Color::Ansi(15),
            ..Style::default()
        };

        for (i, item) in self.items.iter().take(max_items).enumerate() {
            let row = y + i as u16;
            let style = if i == self.selected {
                selected
            } else {
                normal
            };
            surface.hline(x, row, max_width, ' ', style);
            let label = if item.label.len() > max_width as usize - 1 {
                &item.label[..max_width as usize - 1]
            } else {
                &item.label
            };
            surface.print(x + 1, row, label, style);
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
        let mut surface = Surface::new(20, 5);
        popup.draw(&mut surface);
        // First item at row 1 (anchor_y + 1)
        let cell = surface.cell(1, 1);
        assert_eq!(cell.ch, 'h');
    }
}
