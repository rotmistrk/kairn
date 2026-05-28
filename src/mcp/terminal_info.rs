use serde::Serialize;

/// A terminal tab entry with content access.
#[derive(Debug, Clone, Serialize)]
pub struct TerminalInfo {
    pub(crate) name: String,
    pub(crate) terminal_type: String,
    pub(crate) index: usize,
    pub(crate) content: String,
}

impl TerminalInfo {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn terminal_type(&self) -> &str {
        &self.terminal_type
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn content(&self) -> &str {
        &self.content
    }
}
