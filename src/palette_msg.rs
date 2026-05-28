//! Message sub-palette.

use txv_core::cell::Style;

#[derive(Clone, Debug)]
pub struct MsgPalette {
    error: Style,
    warning: Style,
    info: Style,
    debug: Style,
}

impl MsgPalette {
    pub fn new(error: Style, warning: Style, info: Style, debug: Style) -> Self {
        Self {
            error,
            warning,
            info,
            debug,
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
    pub fn debug(&self) -> Style {
        self.debug
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
    pub fn debug_mut(&mut self) -> &mut Style {
        &mut self.debug
    }
}
