//! Scrollbar — vertical scroll indicator View.

use txv_core::prelude::*;

use crate::scroll_view::ScrollView;

pub struct Scrollbar {
    state: ViewState,
    pub scroll: ScrollView,
}

impl Scrollbar {
    pub fn new() -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                focusable: false,
                ..ViewOptions::default()
            }),
            scroll: ScrollView::new(),
        }
    }
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Scrollbar {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let track_style = Style {
            fg: Color::Ansi(8),
            ..Style::default()
        };
        let thumb_style = Style {
            fg: Color::Reset,
            attrs: Attrs { reverse: true, ..Attrs::default() },
            ..Style::default()
        };
        let (thumb_pos, thumb_size) = self.scroll.thumb(b.h);
        for row in 0..b.h {
            let style = if row >= thumb_pos && row < thumb_pos + thumb_size {
                thumb_style
            } else {
                track_style
            };
            surface.put(b.x, b.y + row, '│', style);
        }
    }

    fn handle(
        &mut self,
        _event: &Event,
        _queue: &mut EventQueue,
    ) -> HandleResult {
        HandleResult::Ignored
    }
}
