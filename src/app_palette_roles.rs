//! Sub-palette structs: git, diff, editor.

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

#[derive(Clone, Debug)]
pub struct DiffPalette {
    added: Style,
    deleted: Style,
    fold: Style,
}

impl DiffPalette {
    pub fn new(added: Style, deleted: Style, fold: Style) -> Self {
        Self { added, deleted, fold }
    }

    pub fn added(&self) -> Style {
        self.added
    }
    pub fn deleted(&self) -> Style {
        self.deleted
    }
    pub fn fold(&self) -> Style {
        self.fold
    }

    pub fn added_mut(&mut self) -> &mut Style {
        &mut self.added
    }
    pub fn deleted_mut(&mut self) -> &mut Style {
        &mut self.deleted
    }
    pub fn fold_mut(&mut self) -> &mut Style {
        &mut self.fold
    }
}

#[derive(Clone, Debug)]
pub struct EditorPalette {
    gutter: Style,
    list_chars: Style,
    cursor: Style,
    highlight_match: Style,
    highlight_other: Style,
    matchparen: Style,
}

impl EditorPalette {
    pub fn new(
        gutter: Style,
        list_chars: Style,
        cursor: Style,
        highlight_match: Style,
        highlight_other: Style,
        matchparen: Style,
    ) -> Self {
        Self {
            gutter,
            list_chars,
            cursor,
            highlight_match,
            highlight_other,
            matchparen,
        }
    }

    pub fn gutter(&self) -> Style {
        self.gutter
    }
    pub fn list_chars(&self) -> Style {
        self.list_chars
    }
    pub fn cursor(&self) -> Style {
        self.cursor
    }
    pub fn highlight_match(&self) -> Style {
        self.highlight_match
    }
    pub fn highlight_other(&self) -> Style {
        self.highlight_other
    }
    pub fn matchparen(&self) -> Style {
        self.matchparen
    }

    pub fn gutter_mut(&mut self) -> &mut Style {
        &mut self.gutter
    }
    pub fn list_chars_mut(&mut self) -> &mut Style {
        &mut self.list_chars
    }
}
