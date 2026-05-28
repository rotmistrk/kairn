//! Badge sub-palette.

use txv_core::cell::Style;

#[derive(Clone, Debug)]
pub struct BadgePalette {
    busy: Style,
    idle: Style,
    exited: Style,
}

impl BadgePalette {
    pub fn new(busy: Style, idle: Style, exited: Style) -> Self {
        Self { busy, idle, exited }
    }

    pub fn busy(&self) -> Style {
        self.busy
    }
    pub fn idle(&self) -> Style {
        self.idle
    }
    pub fn exited(&self) -> Style {
        self.exited
    }
}
