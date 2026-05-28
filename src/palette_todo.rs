//! Todo sub-palette.

use txv_core::cell::Style;

#[derive(Clone, Debug)]
pub struct TodoPalette {
    normal: Style,
    done: Style,
    important: Style,
}

impl TodoPalette {
    pub fn new(normal: Style, done: Style, important: Style) -> Self {
        Self {
            normal,
            done,
            important,
        }
    }

    pub fn normal(&self) -> Style {
        self.normal
    }
    pub fn done(&self) -> Style {
        self.done
    }
    pub fn important(&self) -> Style {
        self.important
    }

    pub fn normal_mut(&mut self) -> &mut Style {
        &mut self.normal
    }
    pub fn done_mut(&mut self) -> &mut Style {
        &mut self.done
    }
    pub fn important_mut(&mut self) -> &mut Style {
        &mut self.important
    }
}
