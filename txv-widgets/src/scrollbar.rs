//! Proportional vertical scrollbar indicator.

use txv::cell::Style;
use txv::surface::Surface;

/// Renders a proportional vertical scrollbar on a surface column.
pub struct Scrollbar {
    /// Style for the track (background).
    pub track_style: Style,
    /// Style for the thumb (indicator).
    pub thumb_style: Style,
    /// Character used for the track.
    pub track_char: char,
    /// Character used for the thumb.
    pub thumb_char: char,
}

impl Scrollbar {
    /// Create a scrollbar with default block characters.
    pub fn new() -> Self {
        Self {
            track_style: Style::default(),
            thumb_style: Style {
                attrs: txv::cell::Attrs {
                    reverse: true,
                    ..txv::cell::Attrs::default()
                },
                ..Style::default()
            },
            track_char: '│',
            thumb_char: '█',
        }
    }

    /// Compute thumb position and size.
    /// Returns `(start_row, thumb_height)` within the given `track_height`.
    pub fn compute(
        &self,
        offset: usize,
        visible: usize,
        total: usize,
        track_height: u16,
    ) -> (u16, u16) {
        let th = track_height as usize;
        if total == 0 || visible >= total || th == 0 {
            return (0, track_height);
        }
        let thumb_h = ((visible * th) / total).max(1).min(th);
        let scrollable = total.saturating_sub(visible);
        let track_range = th.saturating_sub(thumb_h);
        let start = if scrollable == 0 {
            0
        } else {
            (offset * track_range / scrollable).min(track_range)
        };
        (start as u16, thumb_h as u16)
    }

    /// Render the scrollbar on a single column of the surface.
    /// Draws from row 0 to `track_height - 1` at the given column.
    pub fn render(
        &self,
        surface: &mut Surface<'_>,
        col: u16,
        track_height: u16,
        offset: usize,
        visible: usize,
        total: usize,
    ) {
        let (start, thumb_h) = self.compute(offset, visible, total, track_height);
        for row in 0..track_height {
            if row >= start && row < start + thumb_h {
                surface.put(col, row, self.thumb_char, self.thumb_style);
            } else {
                surface.put(col, row, self.track_char, self.track_style);
            }
        }
    }
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use txv::cell::ColorMode;
    use txv::screen::Screen;

    #[test]
    fn full_content_visible() {
        let sb = Scrollbar::new();
        let (start, thumb_h) = sb.compute(0, 20, 10, 10);
        // visible >= total, thumb fills track
        assert_eq!(start, 0);
        assert_eq!(thumb_h, 10);
    }

    #[test]
    fn empty_content() {
        let sb = Scrollbar::new();
        let (start, thumb_h) = sb.compute(0, 10, 0, 10);
        assert_eq!(start, 0);
        assert_eq!(thumb_h, 10);
    }

    #[test]
    fn half_visible() {
        let sb = Scrollbar::new();
        let (start, thumb_h) = sb.compute(0, 10, 20, 10);
        assert_eq!(start, 0);
        assert_eq!(thumb_h, 5); // 10/20 * 10
    }

    #[test]
    fn scrolled_to_bottom() {
        let sb = Scrollbar::new();
        let (start, thumb_h) = sb.compute(10, 10, 20, 10);
        assert_eq!(thumb_h, 5);
        assert_eq!(start, 5); // at bottom
    }

    #[test]
    fn scrolled_midway() {
        let sb = Scrollbar::new();
        let (start, thumb_h) = sb.compute(5, 10, 20, 10);
        assert_eq!(thumb_h, 5);
        assert_eq!(start, 2); // 5/10 * 5 = 2.5 → 2
    }

    #[test]
    fn minimum_thumb_size() {
        let sb = Scrollbar::new();
        let (_, thumb_h) = sb.compute(0, 1, 1000, 10);
        assert!(thumb_h >= 1);
    }

    #[test]
    fn zero_track_height() {
        let sb = Scrollbar::new();
        let (start, thumb_h) = sb.compute(0, 10, 20, 0);
        assert_eq!(start, 0);
        assert_eq!(thumb_h, 0);
    }

    #[test]
    fn render_draws_track_and_thumb() {
        let sb = Scrollbar::new();
        let mut screen = Screen::with_color_mode(1, 10, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            sb.render(&mut s, 0, 10, 0, 5, 20);
        }
        // Thumb at top (rows 0-1), track below
        let (start, thumb_h) = sb.compute(0, 5, 20, 10);
        for row in 0..10u16 {
            let ch = screen.cell(0, row).ch;
            if row >= start && row < start + thumb_h {
                assert_eq!(ch, '█', "row {row} should be thumb");
            } else {
                assert_eq!(ch, '│', "row {row} should be track");
            }
        }
    }

    #[test]
    fn render_at_offset_column() {
        let sb = Scrollbar::new();
        let mut screen = Screen::with_color_mode(5, 5, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            sb.render(&mut s, 4, 5, 0, 5, 5);
        }
        // Full thumb at col 4
        assert_eq!(screen.cell(4, 0).ch, '█');
        assert_eq!(screen.cell(0, 0).ch, ' '); // untouched
    }
}
