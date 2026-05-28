//! Diagnostics sub-palette.

use txv_core::cell::Style;

#[derive(Clone, Debug)]
pub struct DiagPalette {
    error: Style,
    warning: Style,
    info: Style,
    hint: Style,
}

impl DiagPalette {
    pub fn new(error: Style, warning: Style, info: Style, hint: Style) -> Self {
        Self {
            error,
            warning,
            info,
            hint,
        }
    }

    pub fn error(&self) -> Style {
        self.error
    }
    pub fn warning(&self) -> Style {
        self.warning
    }
    pub fn info(&self) -> Style {
        self.info
    }
    pub fn hint(&self) -> Style {
        self.hint
    }

    pub fn error_mut(&mut self) -> &mut Style {
        &mut self.error
    }
    pub fn warning_mut(&mut self) -> &mut Style {
        &mut self.warning
    }
    pub fn info_mut(&mut self) -> &mut Style {
        &mut self.info
    }
    pub fn hint_mut(&mut self) -> &mut Style {
        &mut self.hint
    }
}
