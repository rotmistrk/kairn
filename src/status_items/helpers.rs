//! Shared helpers for context status items.

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

/// Update bounds to match label width.
pub(super) fn sync_bounds(state: &mut ViewState, label: &str) {
    let w = if label.is_empty() {
        0
    } else {
        label.len() as u16 + 2
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

/// Draw label into buffer with status bar style.
pub(super) fn draw_label(state: &mut ViewState, label: &str) {
    let style = txv_core::palette::palette().style(txv_core::palette::StyleId::StatusBar);
    state.buffer_mut().fill(' ', style);
    if !label.is_empty() {
        state.buffer_mut().print(1, 0, label, style);
    }
    state.mark_redrawn();
}
