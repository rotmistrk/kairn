//! HelpView — TextArea populated with help text.

use txv_core::prelude::*;
use txv_widgets::TextArea;

use crate::help::help_text;

pub struct HelpView {
    inner: TextArea,
}

impl HelpView {
    pub fn new() -> Self {
        let mut ta = TextArea::new();
        ta.line_numbers = false;
        ta.set_content(&help_text());
        Self { inner: ta }
    }
}

impl View for HelpView {
    fn bounds(&self) -> Rect { self.inner.bounds() }
    fn set_bounds(&mut self, r: Rect) { self.inner.set_bounds(r); }
    fn options(&self) -> ViewOptions { self.inner.options() }
    fn title(&self) -> &str { "Help" }
    fn needs_redraw(&self) -> bool { self.inner.needs_redraw() }
    fn mark_redrawn(&mut self) { self.inner.mark_redrawn(); }
    fn select(&mut self) { self.inner.select(); }
    fn unselect(&mut self) { self.inner.unselect(); }

    fn draw(&self, surface: &mut Surface) {
        self.inner.draw(surface);
    }

    fn handle(
        &mut self,
        event: &Event,
        queue: &mut EventQueue,
    ) -> HandleResult {
        self.inner.handle(event, queue)
    }
}
