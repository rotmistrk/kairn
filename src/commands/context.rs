//! ViewContext — status snapshot collected each tick.

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
