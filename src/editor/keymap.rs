//! Keymap trait — translates key events into editor commands.

use txv_core::event::KeyEvent;

use super::command::Command;

/// Editor mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Visual,
    VisualLine,
    Command,
    Search,
}

/// Translates key events into editor commands.
pub trait Keymap: Send {
    fn handle_key(&mut self, key: &KeyEvent, mode: EditorMode) -> Command;
    fn mode_label(&self, mode: EditorMode) -> &str;
}
