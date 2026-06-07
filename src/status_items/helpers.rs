//! Shared helpers for context status items.

use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

/// Create ViewState for a status indicator.
pub(super) fn indicator_state(initial_width: u16) -> ViewState {
    let mut state = ViewState::new(ViewOptions::default().with_preprocess());
    state.set_bounds(Rect::new(0, 0, initial_width, 1));
    state
}

/// Update bounds to match label width (excluding ~ markers).
pub(super) fn sync_bounds(state: &mut ViewState, label: &str) {
    let visible_len = label.chars().filter(|c| *c != '~').count();
    let w = if visible_len == 0 {
        0
    } else {
        visible_len as u16 + 2
    };
    let b = state.bounds();
    if b.w() != w {
        state.set_bounds(Rect::new(b.x(), b.y(), w, 1));
    }
}

/// Draw label into buffer with status bar style. ~ toggles bold.
pub(super) fn draw_label(state: &mut ViewState, label: &str) {
    let style = palette().style(StyleId::StatusBar);
    let bold_style = Style::new(style.fg(), style.bg()).with_attrs(style.attrs().bold());
    state.buffer_mut().fill(' ', style);
    if !label.is_empty() {
        render_styled_text(state.buffer_mut(), label, style, bold_style);
    }
    state.mark_redrawn();
}

/// Render text with ~ as style toggle into buffer starting at (1, 0).
fn render_styled_text(buf: &mut Buffer, text: &str, normal: Style, bold: Style) {
    let mut x: u16 = 1;
    let mut in_bold = false;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '~' {
            if chars.peek() == Some(&'~') {
                chars.next();
                buf.put(
                    x,
                    0,
                    '~',
                    if in_bold {
                        bold
                    } else {
                        normal
                    },
                );
                x += 1;
            } else {
                in_bold = !in_bold;
            }
        } else {
            buf.put(
                x,
                0,
                ch,
                if in_bold {
                    bold
                } else {
                    normal
                },
            );
            x += 1;
        }
    }
}
