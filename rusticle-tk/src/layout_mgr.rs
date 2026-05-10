//! Window layout tree manager.
//!
//! Translates rusticle `window add` commands into layout computations.
//! Uses a Tk-style pack model: each `add` peels off space from the
//! remaining area in the specified direction.

use txv_core::geometry::Rect;

pub use crate::layout_side::Side;

/// A packed widget entry.
struct PackEntry {
    widget_id: String,
    side: Side,
    size: Option<u16>,
}

/// Manages the window layout using Tk-style packing.
pub struct LayoutManager {
    entries: Vec<PackEntry>,
    title: String,
}

impl LayoutManager {
    /// Create an empty layout.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            title: String::new(),
        }
    }

    /// Set the window title.
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    /// Get the window title.
    #[allow(dead_code)]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Add a widget to the layout.
    pub fn add(&mut self, widget_id: &str, side: Side, size: Option<u16>) {
        self.entries.push(PackEntry {
            widget_id: widget_id.to_string(),
            side,
            size,
        });
    }

    /// Remove a widget from the layout by ID.
    #[allow(dead_code)]
    pub fn remove(&mut self, widget_id: &str) {
        self.entries.retain(|e| e.widget_id != widget_id);
    }

    /// Compute widget positions for the given area.
    pub fn compute(&self, area: Rect) -> Vec<(String, Rect)> {
        let mut result = Vec::new();
        let mut rem = area;

        for entry in &self.entries {
            if rem.w == 0 || rem.h == 0 {
                break;
            }
            let (widget_rect, new_rem) = split_side(rem, entry.side, entry.size);
            result.push((entry.widget_id.clone(), widget_rect));
            rem = new_rem;
        }
        result
    }
}

/// Split a rect by peeling off one side. Returns (widget_rect, remaining).
fn split_side(rem: Rect, side: Side, size: Option<u16>) -> (Rect, Rect) {
    match side {
        Side::Left => {
            let w = size.unwrap_or(rem.w).min(rem.w);
            (
                Rect::new(rem.x, rem.y, w, rem.h),
                Rect::new(rem.x + w, rem.y, rem.w - w, rem.h),
            )
        }
        Side::Right => {
            let w = size.unwrap_or(rem.w).min(rem.w);
            (
                Rect::new(rem.x + rem.w - w, rem.y, w, rem.h),
                Rect::new(rem.x, rem.y, rem.w - w, rem.h),
            )
        }
        Side::Top => {
            let h = size.unwrap_or(rem.h).min(rem.h);
            (
                Rect::new(rem.x, rem.y, rem.w, h),
                Rect::new(rem.x, rem.y + h, rem.w, rem.h - h),
            )
        }
        Side::Bottom => {
            let h = size.unwrap_or(rem.h).min(rem.h);
            (
                Rect::new(rem.x, rem.y + rem.h - h, rem.w, h),
                Rect::new(rem.x, rem.y, rem.w, rem.h - h),
            )
        }
        Side::Fill => (rem, Rect::new(rem.x + rem.w, rem.y + rem.h, 0, 0)),
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect::new(0, 0, 80, 24)
    }

    #[test]
    fn empty_layout_produces_nothing() {
        let mgr = LayoutManager::new();
        assert!(mgr.compute(area()).is_empty());
    }

    #[test]
    fn single_fill_widget() {
        let mut mgr = LayoutManager::new();
        mgr.add("w1", Side::Fill, None);
        let rects = mgr.compute(area());
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].1, area());
    }

    #[test]
    fn left_and_fill() {
        let mut mgr = LayoutManager::new();
        mgr.add("tree", Side::Left, Some(20));
        mgr.add("main", Side::Fill, None);
        let rects = mgr.compute(area());
        assert_eq!(rects[0].1.w, 20);
        assert_eq!(rects[1].1.w, 60);
        assert_eq!(rects[1].1.x, 20);
    }

    #[test]
    fn status_then_fill() {
        let mut mgr = LayoutManager::new();
        mgr.add("status", Side::Bottom, Some(1));
        mgr.add("main", Side::Fill, None);
        let rects = mgr.compute(area());
        assert_eq!(rects[0].1.h, 1);
        assert_eq!(rects[0].1.y, 23);
        assert_eq!(rects[1].1.h, 23);
    }

    #[test]
    fn file_browser_layout() {
        let mut mgr = LayoutManager::new();
        mgr.add("tree", Side::Left, Some(25));
        mgr.add("status", Side::Bottom, Some(1));
        mgr.add("txt", Side::Fill, None);
        let rects = mgr.compute(area());
        assert_eq!(rects[0].1.w, 25);
        assert_eq!(rects[1].1.h, 1);
        assert_eq!(rects[1].1.w, 55);
        assert_eq!(rects[2].1.w, 55);
        assert_eq!(rects[2].1.h, 23);
    }

    #[test]
    fn remove_widget_from_layout() {
        let mut mgr = LayoutManager::new();
        mgr.add("w1", Side::Left, Some(20));
        mgr.add("w2", Side::Fill, None);
        mgr.remove("w1");
        let rects = mgr.compute(area());
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].1.w, 80);
    }

    #[test]
    fn title_management() {
        let mut mgr = LayoutManager::new();
        assert_eq!(mgr.title(), "");
        mgr.set_title("My App");
        assert_eq!(mgr.title(), "My App");
    }

    #[test]
    fn top_and_fill() {
        let mut mgr = LayoutManager::new();
        mgr.add("input", Side::Top, Some(3));
        mgr.add("content", Side::Fill, None);
        let rects = mgr.compute(area());
        assert_eq!(rects[0].1.h, 3);
        assert_eq!(rects[1].1.h, 21);
    }

    #[test]
    fn right_side() {
        let mut mgr = LayoutManager::new();
        mgr.add("sidebar", Side::Right, Some(30));
        mgr.add("main", Side::Fill, None);
        let rects = mgr.compute(area());
        assert_eq!(rects[0].1.x, 50);
        assert_eq!(rects[0].1.w, 30);
        assert_eq!(rects[1].1.w, 50);
    }
}
