//! Style helpers for editor draw — computes per-character style overlays.

use txv_core::prelude::{Color, Style};

/// Compute the effective style for a character at `byte_pos`, applying
/// visual selection and search highlight overlays.
pub(super) fn char_style(
    base: Style,
    byte_pos: usize,
    visual_range: Option<(usize, usize)>,
    visual_bg: Color,
    highlight: Option<&crate::editor::highlight_state::HighlightState>,
    hl_match: Style,
    hl_other_bg: Color,
) -> Style {
    if let Some((vs, ve)) = visual_range {
        if byte_pos >= vs && byte_pos < ve {
            return Style { bg: visual_bg, ..base };
        }
    } else if let Some(is_current) = highlight.and_then(|h| h.match_at(byte_pos)) {
        if is_current {
            return hl_match;
        }
        return Style {
            bg: hl_other_bg,
            ..base
        };
    }
    base
}

#[cfg(test)]
mod tests {
    use crate::views::editor::EditorView;
    use txv_core::prelude::*;

    #[test]
    fn wide_char_positions_correct() {
        // "A✅B" — ✅ is width 2, so B should be at visual column 4
        let mut view = EditorView::from_text("A✅B");
        view.editor.options.number = false;
        view.set_bounds(Rect::new(0, 0, 20, 1));
        let mut surface = Surface::new(20, 1);
        view.draw(&mut surface);
        assert_eq!(surface.cell(0, 0).ch, 'A');
        assert_eq!(surface.cell(1, 0).ch, '✅');
        assert_eq!(surface.cell(3, 0).ch, 'B');
    }
}
