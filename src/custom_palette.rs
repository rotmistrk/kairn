//! Configurable palette — wraps a base palette with user overrides.

use std::sync::Arc;

use txv_core::cell::Style;
use txv_core::palette::{Palette, StyleId};

/// A palette that wraps a base and overrides specific styles from config.
pub struct CustomPalette {
    base: Arc<dyn Palette>,
    overrides: [Option<Style>; StyleId::COUNT],
}

impl CustomPalette {
    pub fn new(base: Arc<dyn Palette>) -> Self {
        Self {
            base,
            overrides: [None; StyleId::COUNT],
        }
    }

    pub fn set_override(&mut self, id: StyleId, style: Style) {
        self.overrides[id as usize] = Some(style);
    }
}

impl Palette for CustomPalette {
    fn style(&self, id: StyleId) -> Style {
        if let Some(s) = self.overrides[id as usize] {
            return s;
        }
        self.base.style(id)
    }
}
