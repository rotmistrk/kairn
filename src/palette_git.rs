//! Git sub-palette.

use txv_core::cell::Style;

#[derive(Clone, Debug)]
pub struct GitPalette {
    added: Style,
    modified: Style,
    untracked: Style,
    ignored: Style,
    conflict: Style,
}

impl GitPalette {
    pub fn new(added: Style, modified: Style, untracked: Style, ignored: Style, conflict: Style) -> Self {
        Self {
            added,
            modified,
            untracked,
            ignored,
            conflict,
        }
    }

    pub fn added(&self) -> Style {
        self.added
    }
    pub fn modified(&self) -> Style {
        self.modified
    }
    pub fn untracked(&self) -> Style {
        self.untracked
    }
    pub fn ignored(&self) -> Style {
        self.ignored
    }
    pub fn conflict(&self) -> Style {
        self.conflict
    }

    pub fn added_mut(&mut self) -> &mut Style {
        &mut self.added
    }
    pub fn modified_mut(&mut self) -> &mut Style {
        &mut self.modified
    }
    pub fn untracked_mut(&mut self) -> &mut Style {
        &mut self.untracked
    }
    pub fn ignored_mut(&mut self) -> &mut Style {
        &mut self.ignored
    }
    pub fn conflict_mut(&mut self) -> &mut Style {
        &mut self.conflict
    }
}
