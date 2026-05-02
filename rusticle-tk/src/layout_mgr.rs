//! Window layout tree manager.
//!
//! Translates rusticle `window add` commands into txv layout computations.
//! Uses a Tk-style pack model: each `add` peels off space from the
//! remaining area in the specified direction.

use txv::layout::{Constraint, Direction, Rect, Size};

/// Side specification from the script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Horizontal split, widget on left.
    Left,
    /// Horizontal split, widget on right.
    Right,
    /// Vertical split, widget on top.
    Top,
    /// Vertical split, widget on bottom.
    Bottom,
    /// Takes remaining space.
    Fill,
}

impl Side {
    /// Parse a side string.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "top" => Ok(Self::Top),
            "bottom" => Ok(Self::Bottom),
            "fill" => Ok(Self::Fill),
            _ => Err(format!("unknown side: {s}")),
        }
    }
}

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
    /// Processes entries in order, peeling off space from each side.
    pub fn compute(&self, area: Rect) -> Vec<(String, Rect)> {
        let mut result = Vec::new();
        let mut remaining = area;

        for entry in &self.entries {
            if remaining.w == 0 || remaining.h == 0 {
                break;
            }
            match entry.side {
                Side::Left => {
                    let w = entry.size.unwrap_or(remaining.w).min(remaining.w);
                    let rects = remaining.split(
                        Direction::Horizontal,
                        &[fixed_constraint(w), fill_constraint()],
                    );
                    result.push((entry.widget_id.clone(), rects[0]));
                    remaining = rects[1];
                }
                Side::Right => {
                    let w = entry.size.unwrap_or(remaining.w).min(remaining.w);
                    let rects = remaining.split(
                        Direction::Horizontal,
                        &[fill_constraint(), fixed_constraint(w)],
                    );
                    remaining = rects[0];
                    result.push((entry.widget_id.clone(), rects[1]));
                }
                Side::Top => {
                    let h = entry.size.unwrap_or(remaining.h).min(remaining.h);
                    let rects = remaining.split(
                        Direction::Vertical,
                        &[fixed_constraint(h), fill_constraint()],
                    );
                    result.push((entry.widget_id.clone(), rects[0]));
                    remaining = rects[1];
                }
                Side::Bottom => {
                    let h = entry.size.unwrap_or(remaining.h).min(remaining.h);
                    let rects = remaining.split(
                        Direction::Vertical,
                        &[fill_constraint(), fixed_constraint(h)],
                    );
                    remaining = rects[0];
                    result.push((entry.widget_id.clone(), rects[1]));
                }
                Side::Fill => {
                    result.push((entry.widget_id.clone(), remaining));
                    remaining = Rect {
                        x: remaining.x + remaining.w,
                        y: remaining.y + remaining.h,
                        w: 0,
                        h: 0,
                    };
                }
            }
        }
        result
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

fn fixed_constraint(size: u16) -> Constraint {
    Constraint {
        size: Size::Fixed(size),
        min: 0,
        max: u16::MAX,
    }
}

fn fill_constraint() -> Constraint {
    Constraint {
        size: Size::Fill,
        min: 0,
        max: u16::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: 80,
            h: 24,
        }
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
        assert_eq!(rects[0].0, "w1");
        assert_eq!(rects[0].1, area());
    }

    #[test]
    fn left_and_fill() {
        let mut mgr = LayoutManager::new();
        mgr.add("tree", Side::Left, Some(20));
        mgr.add("main", Side::Fill, None);
        let rects = mgr.compute(area());
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].0, "tree");
        assert_eq!(rects[0].1.w, 20);
        assert_eq!(rects[0].1.x, 0);
        assert_eq!(rects[1].0, "main");
        assert_eq!(rects[1].1.w, 60);
        assert_eq!(rects[1].1.x, 20);
    }

    #[test]
    fn fill_then_bottom_exhausts_space() {
        let mut mgr = LayoutManager::new();
        mgr.add("main", Side::Fill, None);
        mgr.add("status", Side::Bottom, Some(1));
        let rects = mgr.compute(area());
        // Fill consumes all space; status gets nothing useful
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].0, "main");
    }

    #[test]
    fn status_then_fill() {
        let mut mgr = LayoutManager::new();
        mgr.add("status", Side::Bottom, Some(1));
        mgr.add("main", Side::Fill, None);
        let rects = mgr.compute(area());
        assert_eq!(rects.len(), 2);
        let status = rects.iter().find(|(id, _)| id == "status");
        let main = rects.iter().find(|(id, _)| id == "main");
        assert_eq!(status.map(|(_, r)| r.h), Some(1));
        assert_eq!(status.map(|(_, r)| r.y), Some(23));
        assert_eq!(main.map(|(_, r)| r.h), Some(23));
    }

    #[test]
    fn three_panel_hello_layout() {
        // Matches the hello.tcl example: fill text, bottom status
        let mut mgr = LayoutManager::new();
        mgr.add("status", Side::Bottom, Some(1));
        mgr.add("txt", Side::Fill, None);
        let rects = mgr.compute(area());
        assert_eq!(rects.len(), 2);
        let txt = rects.iter().find(|(id, _)| id == "txt");
        assert_eq!(txt.map(|(_, r)| r.h), Some(23));
    }

    #[test]
    fn file_browser_layout() {
        // tree(left,25) + txt(fill) + status(bottom,1)
        let mut mgr = LayoutManager::new();
        mgr.add("tree", Side::Left, Some(25));
        mgr.add("status", Side::Bottom, Some(1));
        mgr.add("txt", Side::Fill, None);
        let rects = mgr.compute(area());
        assert_eq!(rects.len(), 3);
        let tree = rects.iter().find(|(id, _)| id == "tree");
        let txt = rects.iter().find(|(id, _)| id == "txt");
        let status = rects.iter().find(|(id, _)| id == "status");
        assert_eq!(tree.map(|(_, r)| r.w), Some(25));
        assert_eq!(tree.map(|(_, r)| r.h), Some(24));
        // Status is bottom of remaining (55 wide, 24 tall)
        assert_eq!(status.map(|(_, r)| r.h), Some(1));
        assert_eq!(status.map(|(_, r)| r.w), Some(55));
        // txt fills the rest
        assert_eq!(txt.map(|(_, r)| r.w), Some(55));
        assert_eq!(txt.map(|(_, r)| r.h), Some(23));
    }

    #[test]
    fn remove_widget_from_layout() {
        let mut mgr = LayoutManager::new();
        mgr.add("w1", Side::Left, Some(20));
        mgr.add("w2", Side::Fill, None);
        mgr.remove("w1");
        let rects = mgr.compute(area());
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].0, "w2");
        assert_eq!(rects[0].1.w, 80);
    }

    #[test]
    fn side_parse() {
        assert_eq!(Side::parse("left"), Ok(Side::Left));
        assert_eq!(Side::parse("right"), Ok(Side::Right));
        assert_eq!(Side::parse("top"), Ok(Side::Top));
        assert_eq!(Side::parse("bottom"), Ok(Side::Bottom));
        assert_eq!(Side::parse("fill"), Ok(Side::Fill));
        assert!(Side::parse("center").is_err());
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
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].0, "input");
        assert_eq!(rects[0].1.h, 3);
        assert_eq!(rects[1].0, "content");
        assert_eq!(rects[1].1.h, 21);
    }

    #[test]
    fn right_side() {
        let mut mgr = LayoutManager::new();
        mgr.add("main", Side::Fill, None);
        // Note: fill consumes all, so right gets nothing
        // Correct: add right first
        let mut mgr2 = LayoutManager::new();
        mgr2.add("sidebar", Side::Right, Some(30));
        mgr2.add("main", Side::Fill, None);
        let rects = mgr2.compute(area());
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].0, "sidebar");
        assert_eq!(rects[0].1.x, 50);
        assert_eq!(rects[0].1.w, 30);
        assert_eq!(rects[1].0, "main");
        assert_eq!(rects[1].1.w, 50);
    }
}
