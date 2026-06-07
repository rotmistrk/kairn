//! HelpView — TextArea populated with help text.

use txv_core::prelude::*;
use txv_widgets::TextArea;

use crate::help::help_text;

pub struct HelpView {
    inner: TextArea,
}

impl Default for HelpView {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpView {
    pub fn new() -> Self {
        let mut ta = TextArea::new();
        ta.show_line_numbers(false);
        ta.set_content(&help_text());
        Self { inner: ta }
    }
}

impl View for HelpView {
    delegate_view!(inner, override { title });

    fn title(&self) -> &str {
        "Help"
    }
}
