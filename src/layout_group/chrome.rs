//! Chrome background for LayoutGroup — horizontal rule lines and connectors.
//!
//! TabPanel's TabBar renders the actual tab content with transparent fill,
//! so the `─` line drawn here shows through in non-tab areas.

use txv_core::prelude::*;

use super::{LayoutGroup, SlotId, PANEL_COUNT};

fn chrome_style() -> Style {
    txv_core::palette::palette().chrome.bar.to_style()
}

/// Truncate a title to fit within `max_chars`, appending `…` if needed.
/// For paths, collapses leading segments: `…/last/segments`.
pub(super) fn truncate_title(title: &str, max_chars: usize) -> String {
    let char_count = title.chars().count();
    if char_count <= max_chars {
        return title.to_string();
    }
    if max_chars <= 1 {
        return "…".to_string();
    }
    if title.contains('/') {
        let parts: Vec<&str> = title.split('/').collect();
        for skip in 1..parts.len() {
            let candidate = format!("…/{}", parts[skip..].join("/"));
            if candidate.chars().count() <= max_chars {
                return candidate;
            }
        }
    }
    let mut s: String = title.chars().take(max_chars - 1).collect();
    s.push('…');
    s
}

impl LayoutGroup {
    /// Draw the chrome background: horizontal `─` lines at row 0 (and bottom
    /// divider in tall mode) plus `┬`/`┴` connectors where vertical dividers
    /// meet horizontal lines.
    pub(super) fn draw_chrome_background(&mut self) {
        let b = self.group.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let rects = self.compute_rects(b);
        let cs = chrome_style();

        // Top row: full-width horizontal rule
        self.group.buffer_mut().hline(0, 0, b.w, '─', cs);

        // ┬ connectors at top where vertical dividers start
        let left_r = rects[SlotId::Left as usize];
        let center_r = rects[SlotId::Center as usize];
        let right_r = rects[SlotId::Right as usize];
        if left_r.w > 0 && center_r.w > 0 {
            let x = (left_r.x + left_r.w).saturating_sub(b.x);
            self.group.buffer_mut().put(x, 0, '┬', cs);
        }
        if right_r.w > 0 && center_r.w > 0 && !self.is_tall() {
            let x = right_r.x.saturating_sub(1).saturating_sub(b.x);
            self.group.buffer_mut().put(x, 0, '┬', cs);
        }

        // Bottom chrome (horizontal divider above bottom/right-below panel)
        self.draw_bottom_chrome_background(&rects, b);
    }

    fn draw_bottom_chrome_background(&mut self, rects: &[Rect; PANEL_COUNT], b: Rect) {
        let tall = self.is_tall();
        let bottom_r = rects[SlotId::Bottom as usize];
        if bottom_r.h == 0 && !(tall && self.panel(SlotId::Right).tab_count() > 0) {
            return;
        }
        let div_y = if tall {
            let right_bounds = self
                .group
                .child(SlotId::Right as usize)
                .map(|c| c.bounds())
                .unwrap_or_default();
            if right_bounds.h == 0 {
                return;
            }
            right_bounds.y.saturating_sub(b.y)
        } else if bottom_r.h > 0 {
            bottom_r.y.saturating_sub(b.y)
        } else {
            return;
        };

        let cs = chrome_style();
        self.group.buffer_mut().hline(0, div_y, b.w, '─', cs);

        // ┴ connectors where vertical dividers meet bottom horizontal
        let left_r = rects[SlotId::Left as usize];
        let center_r = rects[SlotId::Center as usize];
        let right_r = rects[SlotId::Right as usize];
        if left_r.w > 0 && center_r.w > 0 {
            let x = (left_r.x + left_r.w).saturating_sub(b.x);
            self.group.buffer_mut().put(x, div_y, '┴', cs);
        }
        if right_r.w > 0 && center_r.w > 0 && !tall {
            let x = right_r.x.saturating_sub(1).saturating_sub(b.x);
            self.group.buffer_mut().put(x, div_y, '┴', cs);
        }
    }
}

#[cfg(test)]
#[path = "chrome_tests.rs"]
mod tests;
