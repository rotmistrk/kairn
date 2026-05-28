//! Tree sub-palette.

use txv_core::cell::Style;

#[derive(Clone, Debug)]
pub struct TreePalette {
    directory: Style,
}

impl TreePalette {
    pub fn new(directory: Style) -> Self {
        Self { directory }
    }

    pub fn directory(&self) -> Style {
        self.directory
    }

    pub fn directory_mut(&mut self) -> &mut Style {
        &mut self.directory
    }
}
