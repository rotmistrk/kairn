//! ViewContext — status snapshot collected each tick.

use super::view_context_builder::ViewContextBuilder;

/// Context collected from the active view each tick.
#[derive(Debug, Clone, Default)]
pub struct ViewContext {
    pub(crate) file: Option<String>,
    pub(crate) line: u32,
    pub(crate) col: u32,
    pub(crate) mode: String,
    pub(crate) modified: bool,
    pub(crate) language: String,
    pub(crate) title: String,
    pub(crate) selection_lines: u32,
    pub(crate) git_branch: String,
    pub(crate) lsp_status: String,
}

impl ViewContext {
    pub fn builder() -> ViewContextBuilder {
        ViewContextBuilder::default()
    }
    pub fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }
    pub fn line(&self) -> u32 {
        self.line
    }
    pub fn col(&self) -> u32 {
        self.col
    }
    pub fn mode(&self) -> &str {
        &self.mode
    }
    pub fn modified(&self) -> bool {
        self.modified
    }
    pub fn language(&self) -> &str {
        &self.language
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn selection_lines(&self) -> u32 {
        self.selection_lines
    }
    pub fn git_branch(&self) -> &str {
        &self.git_branch
    }
    pub fn lsp_status(&self) -> &str {
        &self.lsp_status
    }
}
