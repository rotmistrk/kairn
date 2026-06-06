//! Editor accessor methods — getters and setters.

use super::options::EditorOptions;
use super::Editor;
use crate::clipboard_ring::ClipboardHandle;
use crate::shared_register::RegisterHandle;

impl Editor {
    pub(crate) fn set_shared_state(&mut self, register: RegisterHandle, clipboard: ClipboardHandle) {
        self.shared_register = register;
        self.clipboard = Some(clipboard);
    }
    pub fn search_pattern(&self) -> &str {
        &self.search_pattern
    }
    pub fn set_search_pattern(&mut self, pat: impl Into<String>) {
        self.search_pattern = pat.into();
    }
    pub fn visual_anchor(&self) -> Option<(usize, usize)> {
        self.visual_anchor
    }
    pub fn options(&self) -> &EditorOptions {
        &self.options
    }
    pub fn options_mut(&mut self) -> &mut EditorOptions {
        &mut self.options
    }
    pub fn command_buf(&self) -> &str {
        &self.command_buf
    }
}
