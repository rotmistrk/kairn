//! TerminalView — placeholder terminal view (PTY integration in later step).

use txv_core::prelude::*;

pub struct TerminalView {
    state: ViewState,
    title: String,
}

impl TerminalView {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            state: ViewState::default(),
            title: title.into(),
        }
    }
}

impl View for TerminalView {
    delegate_view_state!(state, override { title });

    fn title(&self) -> &str {
        &self.title
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let style = Style::default();
        let msg = format!("[{}]", self.title);
        surface.print(b.x, b.y, &msg, style);
    }

    fn handle(
        &mut self,
        _event: &Event,
        _queue: &mut EventQueue,
    ) -> HandleResult {
        HandleResult::Ignored
    }
}
