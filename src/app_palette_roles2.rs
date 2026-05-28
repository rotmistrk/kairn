//! Sub-palette structs: diagnostics, tree, todo, messages, badges.

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
