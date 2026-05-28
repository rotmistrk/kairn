use serde::Serialize;

use super::cursor_pos::CursorPos;
use super::selection_range::SelectionRange;

/// A tab entry visible to MCP clients.
#[derive(Debug, Clone, Serialize)]
pub struct TabInfo {
    pub(crate) name: String,
    pub(crate) tab_type: String,
    pub(crate) path: Option<String>,
    pub(crate) focused: bool,
    pub(crate) active: bool,
    pub(crate) modified: bool,
    pub(crate) cursor: Option<CursorPos>,
    pub(crate) selection: Option<SelectionRange>,
    pub(crate) order: usize,
}

impl TabInfo {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn tab_type(&self) -> &str {
        &self.tab_type
    }
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
    pub fn focused(&self) -> bool {
        self.focused
    }
    pub fn modified(&self) -> bool {
        self.modified
    }
    pub fn cursor(&self) -> Option<&CursorPos> {
        self.cursor.as_ref()
    }
}
