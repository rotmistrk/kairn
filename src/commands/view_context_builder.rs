//! Builder for ViewContext.

use super::context::ViewContext;

#[derive(Default)]
pub struct ViewContextBuilder {
    ctx: ViewContext,
}

impl ViewContextBuilder {
    pub fn file(mut self, f: impl Into<String>) -> Self {
        self.ctx.file = Some(f.into());
        self
    }
    pub fn line(mut self, l: u32) -> Self {
        self.ctx.line = l;
        self
    }
    pub fn col(mut self, c: u32) -> Self {
        self.ctx.col = c;
        self
    }
    pub fn mode(mut self, m: impl Into<String>) -> Self {
        self.ctx.mode = m.into();
        self
    }
    pub fn modified(mut self, m: bool) -> Self {
        self.ctx.modified = m;
        self
    }
    pub fn language(mut self, l: impl Into<String>) -> Self {
        self.ctx.language = l.into();
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.ctx.title = t.into();
        self
    }
    pub fn selection_lines(mut self, s: u32) -> Self {
        self.ctx.selection_lines = s;
        self
    }
    pub fn git_branch(mut self, g: impl Into<String>) -> Self {
        self.ctx.git_branch = g.into();
        self
    }
    pub fn lsp_status(mut self, l: impl Into<String>) -> Self {
        self.ctx.lsp_status = l.into();
        self
    }
    pub fn build(self) -> ViewContext {
        self.ctx
    }
}
