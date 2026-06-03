//! Shared helpers for context status items.

use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

/// Create ViewState for a status indicator.
pub(super) fn indicator_state(initial_width: u16) -> ViewState {
    let mut state = ViewState::new(ViewOptions {
        preprocess: true,
        focusable: false,
        ..ViewOptions::default()
    });
    state.set_bounds(Rect {
        x: 0,
        y: 0,
        w: initial_width,
        h: 1,
    });
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
    if b.w != w {
        state.set_bounds(Rect {
            x: b.x,
            y: b.y,
            w,
            h: 1,
        });
    }
}

/// Draw label into buffer with status bar style. ~ toggles bold.
pub(super) fn draw_label(state: &mut ViewState, label: &str) {
    let style = palette().style(StyleId::StatusBar);
    let bold_style = Style {
        attrs: Attrs {
            bold: true,
            ..style.attrs
        },
        ..style
    };
    state.buffer_mut().fill(' ', style);
    if !label.is_empty() {
        let mut x: u16 = 1;
        let mut bold = false;
        let mut chars = label.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '~' {
                if chars.peek() == Some(&'~') {
                    chars.next();
                    state.buffer_mut().put(
                        x,
                        0,
                        '~',
                        if bold {
                            bold_style
                        } else {
                            style
                        },
                    );
                    x += 1;
                } else {
                    bold = !bold;
                }
            } else {
                state.buffer_mut().put(
                    x,
                    0,
                    ch,
                    if bold {
                        bold_style
                    } else {
                        style
                    },
                );
                x += 1;
            }
        }
    }
    state.mark_redrawn();
}
