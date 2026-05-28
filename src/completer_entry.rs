//! Entry — a concrete completion candidate.

use txv_core::complete::Completion;

/// A concrete completion candidate.
pub(crate) struct Entry {
    pub(crate) text: String,
    pub(crate) display: String,
    pub(crate) kind: &'static str,
}

impl Completion for Entry {
    fn text(&self) -> &str {
        &self.text
    }
    fn display(&self) -> &str {
        &self.display
    }
    fn kind(&self) -> &str {
        self.kind
    }
}
