//! Diff sub-palette.

use txv_core::cell::Style;

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
