//! Scroll utility — shared cursor-follows-viewport logic.

/// Adjusts `scroll` so that `cursor` is visible within a viewport of `viewport_h` lines.
///
/// If cursor is above the viewport, scroll snaps up. If cursor is below, scroll
/// snaps down so cursor sits on the last visible row.
///
/// Returns the new scroll value (unchanged if already visible or `viewport_h` is 0).
pub(crate) fn ensure_visible(cursor: usize, scroll: usize, viewport_h: usize) -> usize {
    if viewport_h == 0 {
        return scroll;
    }
    if cursor < scroll {
        cursor
    } else if cursor >= scroll + viewport_h {
        cursor - viewport_h + 1
    } else {
        scroll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_visible() {
        assert_eq!(ensure_visible(5, 3, 10), 3);
    }

    #[test]
    fn cursor_above_viewport() {
        assert_eq!(ensure_visible(2, 5, 10), 2);
    }

    #[test]
    fn cursor_below_viewport() {
        // viewport shows rows 0..10, cursor at 12 → scroll to 3
        assert_eq!(ensure_visible(12, 0, 10), 3);
    }

    #[test]
    fn zero_viewport_returns_unchanged() {
        assert_eq!(ensure_visible(5, 3, 0), 3);
    }

    #[test]
    fn cursor_at_bottom_edge() {
        // cursor at scroll + h - 1 → still visible
        assert_eq!(ensure_visible(9, 0, 10), 0);
    }

    #[test]
    fn cursor_just_past_bottom() {
        // cursor at scroll + h → needs scroll
        assert_eq!(ensure_visible(10, 0, 10), 1);
    }
}
