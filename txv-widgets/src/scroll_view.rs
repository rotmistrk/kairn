//! ScrollView — scroll state helper embedded by scrollable widgets.
//! Not a View itself.

/// Tracks scroll offset and viewport for a scrollable region.
pub struct ScrollView {
    pub offset: usize,
    pub total: usize,
    pub viewport: usize,
}

impl ScrollView {
    pub fn new() -> Self {
        Self {
            offset: 0,
            total: 0,
            viewport: 0,
        }
    }

    pub fn set_total(&mut self, total: usize) {
        self.total = total;
        self.clamp();
    }

    pub fn set_viewport(&mut self, viewport: usize) {
        self.viewport = viewport;
        self.clamp();
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.offset = self.offset.saturating_sub(n);
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.offset = self.offset.saturating_add(n);
        self.clamp();
    }

    pub fn scroll_to(&mut self, pos: usize) {
        self.offset = pos;
        self.clamp();
    }

    /// Ensure the given index is visible, scrolling minimally.
    pub fn ensure_visible(&mut self, index: usize) {
        if index < self.offset {
            self.offset = index;
        } else if self.viewport > 0 && index >= self.offset + self.viewport {
            self.offset = index.saturating_sub(self.viewport.saturating_sub(1));
        }
    }

    pub fn max_offset(&self) -> usize {
        self.total.saturating_sub(self.viewport)
    }

    /// Thumb position and size for a scrollbar (0-based row, height).
    pub fn thumb(&self, track_height: u16) -> (u16, u16) {
        if self.total == 0 || self.viewport >= self.total {
            return (0, track_height);
        }
        let th = track_height as usize;
        let size = (self.viewport * th / self.total).max(1);
        let max_off = self.max_offset();
        let pos = if max_off == 0 {
            0
        } else {
            self.offset * (th.saturating_sub(size)) / max_off
        };
        (pos as u16, size as u16)
    }

    fn clamp(&mut self) {
        let max = self.max_offset();
        if self.offset > max {
            self.offset = max;
        }
    }
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}
