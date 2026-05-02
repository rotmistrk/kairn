//! Two-child split pane with a keyboard-resizable divider.
//!
//! `SplitPane` does not own child widgets — it computes rects and tracks
//! focus side. The parent renders children into the computed rects.

use txv::cell::{Color, Style};
use txv::layout::Rect;
use txv::surface::Surface;

/// Split orientation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitDirection {
    /// Left | Right with a vertical divider.
    Horizontal,
    /// Top / Bottom with a horizontal divider.
    Vertical,
}

/// A resizable split between two child areas.
pub struct SplitPane {
    direction: SplitDirection,
    /// Divider position in absolute cells from the start.
    divider_pos: u16,
    /// Minimum size for each child.
    min_size: u16,
    /// Which side has focus: 0 = first, 1 = second.
    focused_side: usize,
}

impl SplitPane {
    /// Create a new split pane.
    pub fn new(direction: SplitDirection, divider_pos: u16, min_size: u16) -> Self {
        Self {
            direction,
            divider_pos,
            min_size,
            focused_side: 0,
        }
    }

    /// Current divider position in cells.
    pub fn divider_pos(&self) -> u16 {
        self.divider_pos
    }

    /// Set the divider position in cells.
    pub fn set_divider_pos(&mut self, pos: u16) {
        self.divider_pos = pos;
    }

    /// Which side currently has focus (0 or 1).
    pub fn focused_side(&self) -> usize {
        self.focused_side
    }

    /// Set which side has focus (clamped to 0 or 1).
    pub fn set_focused_side(&mut self, side: usize) {
        self.focused_side = side.min(1);
    }

    /// The split direction.
    pub fn direction(&self) -> SplitDirection {
        self.direction
    }

    /// Compute the two child rects given the total area.
    ///
    /// The divider occupies 1 cell between the two children.
    pub fn child_rects(&self, area: Rect) -> (Rect, Rect) {
        let total = match self.direction {
            SplitDirection::Horizontal => area.w,
            SplitDirection::Vertical => area.h,
        };
        // Clamp divider so both sides get at least min_size
        let max_pos = total.saturating_sub(self.min_size + 1);
        let pos = self.divider_pos.clamp(self.min_size, max_pos);

        match self.direction {
            SplitDirection::Horizontal => {
                let first = Rect {
                    x: area.x,
                    y: area.y,
                    w: pos,
                    h: area.h,
                };
                let second_x = area.x + pos + 1; // +1 for divider
                let second_w = area.w.saturating_sub(pos + 1);
                let second = Rect {
                    x: second_x,
                    y: area.y,
                    w: second_w,
                    h: area.h,
                };
                (first, second)
            }
            SplitDirection::Vertical => {
                let first = Rect {
                    x: area.x,
                    y: area.y,
                    w: area.w,
                    h: pos,
                };
                let second_y = area.y + pos + 1;
                let second_h = area.h.saturating_sub(pos + 1);
                let second = Rect {
                    x: area.x,
                    y: second_y,
                    w: area.w,
                    h: second_h,
                };
                (first, second)
            }
        }
    }

    /// Grow or shrink the divider by `delta` cells, respecting min_size.
    pub fn resize_divider(&mut self, delta: i16, total: u16) {
        let max_pos = total.saturating_sub(self.min_size + 1);
        let new_pos = (self.divider_pos as i32 + delta as i32)
            .max(self.min_size as i32)
            .min(max_pos as i32) as u16;
        self.divider_pos = new_pos;
    }

    /// Render the divider line into the surface.
    pub fn render_divider(&self, surface: &mut Surface<'_>, area: Rect) {
        let total = match self.direction {
            SplitDirection::Horizontal => area.w,
            SplitDirection::Vertical => area.h,
        };
        let max_pos = total.saturating_sub(self.min_size + 1);
        let pos = self.divider_pos.clamp(self.min_size, max_pos);

        let style = Style {
            fg: Color::Ansi(if self.focused_side == 2 { 15 } else { 8 }),
            ..Style::default()
        };

        match self.direction {
            SplitDirection::Horizontal => {
                let col = area.x + pos;
                surface.vline(col, area.y, area.h, '│', style);
            }
            SplitDirection::Vertical => {
                let row = area.y + pos;
                surface.hline(area.x, row, area.w, '─', style);
            }
        }
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
    fn horizontal_child_rects() {
        let sp = SplitPane::new(SplitDirection::Horizontal, 30, 5);
        let (left, right) = sp.child_rects(area());
        assert_eq!(left.x, 0);
        assert_eq!(left.w, 30);
        assert_eq!(right.x, 31); // 30 + 1 divider
        assert_eq!(right.w, 49); // 80 - 31
        assert_eq!(left.h, 24);
        assert_eq!(right.h, 24);
    }

    #[test]
    fn vertical_child_rects() {
        let sp = SplitPane::new(SplitDirection::Vertical, 10, 3);
        let (top, bottom) = sp.child_rects(area());
        assert_eq!(top.y, 0);
        assert_eq!(top.h, 10);
        assert_eq!(bottom.y, 11);
        assert_eq!(bottom.h, 13); // 24 - 11
        assert_eq!(top.w, 80);
        assert_eq!(bottom.w, 80);
    }

    #[test]
    fn child_rects_clamps_to_min() {
        let sp = SplitPane::new(SplitDirection::Horizontal, 2, 10);
        let (left, right) = sp.child_rects(area());
        // divider_pos clamped to min_size=10
        assert_eq!(left.w, 10);
        assert!(right.w > 0);
    }

    #[test]
    fn child_rects_clamps_to_max() {
        let sp = SplitPane::new(SplitDirection::Horizontal, 200, 10);
        let (left, right) = sp.child_rects(area());
        // divider_pos clamped so right side gets at least min_size
        assert!(right.w >= 10);
        assert!(left.w <= 80 - 10 - 1);
    }

    #[test]
    fn resize_divider_grows() {
        let mut sp = SplitPane::new(SplitDirection::Horizontal, 30, 5);
        sp.resize_divider(5, 80);
        assert_eq!(sp.divider_pos(), 35);
    }

    #[test]
    fn resize_divider_shrinks() {
        let mut sp = SplitPane::new(SplitDirection::Horizontal, 30, 5);
        sp.resize_divider(-10, 80);
        assert_eq!(sp.divider_pos(), 20);
    }

    #[test]
    fn resize_divider_clamps_min() {
        let mut sp = SplitPane::new(SplitDirection::Horizontal, 10, 5);
        sp.resize_divider(-100, 80);
        assert_eq!(sp.divider_pos(), 5);
    }

    #[test]
    fn resize_divider_clamps_max() {
        let mut sp = SplitPane::new(SplitDirection::Horizontal, 70, 5);
        sp.resize_divider(100, 80);
        assert_eq!(sp.divider_pos(), 74); // 80 - 5 - 1
    }

    #[test]
    fn focused_side_default() {
        let sp = SplitPane::new(SplitDirection::Horizontal, 30, 5);
        assert_eq!(sp.focused_side(), 0);
    }

    #[test]
    fn set_focused_side_clamps() {
        let mut sp = SplitPane::new(SplitDirection::Horizontal, 30, 5);
        sp.set_focused_side(5);
        assert_eq!(sp.focused_side(), 1);
    }

    #[test]
    fn render_divider_no_panic() {
        let sp = SplitPane::new(SplitDirection::Horizontal, 30, 5);
        let mut screen = txv::screen::Screen::with_color_mode(80, 24, txv::cell::ColorMode::Rgb);
        let mut s = screen.full_surface();
        sp.render_divider(&mut s, area());
    }

    #[test]
    fn render_vertical_divider_no_panic() {
        let sp = SplitPane::new(SplitDirection::Vertical, 10, 3);
        let mut screen = txv::screen::Screen::with_color_mode(80, 24, txv::cell::ColorMode::Rgb);
        let mut s = screen.full_surface();
        sp.render_divider(&mut s, area());
    }

    #[test]
    fn horizontal_with_offset_area() {
        let sp = SplitPane::new(SplitDirection::Horizontal, 20, 5);
        let a = Rect {
            x: 10,
            y: 5,
            w: 60,
            h: 15,
        };
        let (left, right) = sp.child_rects(a);
        assert_eq!(left.x, 10);
        assert_eq!(left.w, 20);
        assert_eq!(right.x, 31); // 10 + 20 + 1
        assert_eq!(left.y, 5);
        assert_eq!(right.y, 5);
    }
}
