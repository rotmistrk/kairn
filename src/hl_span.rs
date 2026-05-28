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
}
