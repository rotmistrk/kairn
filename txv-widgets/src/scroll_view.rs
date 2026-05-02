//! Scroll state helper — embedded by scrollable widgets.

use std::ops::Range;

/// Tracks scroll offset for a virtual content area larger than the viewport.
pub struct ScrollView {
    /// Current vertical scroll offset.
    pub scroll_row: usize,
    /// Current horizontal scroll offset.
    pub scroll_col: usize,
    content_height: usize,
    content_width: usize,
}

impl ScrollView {
    /// Create a new scroll view at origin.
    pub fn new() -> Self {
        Self {
            scroll_row: 0,
            scroll_col: 0,
            content_height: 0,
            content_width: 0,
        }
    }

    /// Update the content dimensions.
    pub fn set_content_size(&mut self, height: usize, width: usize) {
        self.content_height = height;
        self.content_width = width;
    }

    /// Adjust scroll so that `row` is visible within the viewport.
    pub fn ensure_visible(&mut self, row: usize, viewport_height: u16) {
        let vh = viewport_height as usize;
        if vh == 0 {
            return;
        }
        if row < self.scroll_row {
            self.scroll_row = row;
        } else if row >= self.scroll_row + vh {
            self.scroll_row = row.saturating_sub(vh - 1);
        }
    }

    /// Scroll up by `amount` rows.
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_row = self.scroll_row.saturating_sub(amount);
    }

    /// Scroll down by `amount` rows, clamped to content.
    pub fn scroll_down(&mut self, amount: usize, viewport_height: u16) {
        let max = self.max_scroll_row(viewport_height);
        self.scroll_row = (self.scroll_row + amount).min(max);
    }

    /// Scroll up by one page.
    pub fn page_up(&mut self, viewport_height: u16) {
        self.scroll_up(viewport_height as usize);
    }

    /// Scroll down by one page.
    pub fn page_down(&mut self, viewport_height: u16) {
        self.scroll_down(viewport_height as usize, viewport_height);
    }

    /// Jump to the top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_row = 0;
    }

    /// Jump to the bottom.
    pub fn scroll_to_bottom(&mut self, viewport_height: u16) {
        self.scroll_row = self.max_scroll_row(viewport_height);
    }

    /// Range of content rows visible in the viewport.
    pub fn visible_range(&self, viewport_height: u16) -> Range<usize> {
        let end = (self.scroll_row + viewport_height as usize).min(self.content_height);
        self.scroll_row..end
    }

    fn max_scroll_row(&self, viewport_height: u16) -> usize {
        self.content_height.saturating_sub(viewport_height as usize)
    }
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_zero() {
        let sv = ScrollView::new();
        assert_eq!(sv.scroll_row, 0);
        assert_eq!(sv.scroll_col, 0);
    }

    #[test]
    fn ensure_visible_scrolls_down() {
        let mut sv = ScrollView::new();
        sv.set_content_size(100, 80);
        sv.ensure_visible(15, 10);
        assert_eq!(sv.scroll_row, 6);
    }

    #[test]
    fn ensure_visible_scrolls_up() {
        let mut sv = ScrollView::new();
        sv.set_content_size(100, 80);
        sv.scroll_row = 20;
        sv.ensure_visible(5, 10);
        assert_eq!(sv.scroll_row, 5);
    }

    #[test]
    fn ensure_visible_noop_when_in_view() {
        let mut sv = ScrollView::new();
        sv.set_content_size(100, 80);
        sv.scroll_row = 5;
        sv.ensure_visible(8, 10);
        assert_eq!(sv.scroll_row, 5);
    }

    #[test]
    fn ensure_visible_zero_viewport() {
        let mut sv = ScrollView::new();
        sv.set_content_size(100, 80);
        sv.ensure_visible(50, 0);
        assert_eq!(sv.scroll_row, 0);
    }

    #[test]
    fn scroll_up_clamps_to_zero() {
        let mut sv = ScrollView::new();
        sv.scroll_row = 3;
        sv.scroll_up(10);
        assert_eq!(sv.scroll_row, 0);
    }

    #[test]
    fn scroll_down_clamps_to_max() {
        let mut sv = ScrollView::new();
        sv.set_content_size(20, 80);
        sv.scroll_down(100, 10);
        assert_eq!(sv.scroll_row, 10);
    }

    #[test]
    fn page_up_down() {
        let mut sv = ScrollView::new();
        sv.set_content_size(50, 80);
        sv.page_down(10);
        assert_eq!(sv.scroll_row, 10);
        sv.page_up(10);
        assert_eq!(sv.scroll_row, 0);
    }

    #[test]
    fn scroll_to_top_and_bottom() {
        let mut sv = ScrollView::new();
        sv.set_content_size(50, 80);
        sv.scroll_to_bottom(10);
        assert_eq!(sv.scroll_row, 40);
        sv.scroll_to_top();
        assert_eq!(sv.scroll_row, 0);
    }

    #[test]
    fn visible_range_basic() {
        let mut sv = ScrollView::new();
        sv.set_content_size(50, 80);
        sv.scroll_row = 5;
        assert_eq!(sv.visible_range(10), 5..15);
    }

    #[test]
    fn visible_range_clamps_to_content() {
        let mut sv = ScrollView::new();
        sv.set_content_size(8, 80);
        assert_eq!(sv.visible_range(10), 0..8);
    }

    #[test]
    fn visible_range_at_bottom() {
        let mut sv = ScrollView::new();
        sv.set_content_size(50, 80);
        sv.scroll_row = 45;
        assert_eq!(sv.visible_range(10), 45..50);
    }
}
