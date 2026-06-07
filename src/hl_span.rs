//! A span of highlighted text.

use txv_core::prelude::Style;

/// A span of highlighted text.
pub struct HlSpan {
    pub(crate) text: String,
    pub(crate) style: Style,
}

impl HlSpan {
    pub fn plain(text: String) -> Self {
        Self {
            text,
            style: Style::default(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn style(&self) -> Style {
        self.style
    }
}
