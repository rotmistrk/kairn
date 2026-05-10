//! Events flowing through the view tree.

use std::any::Any;

/// Command identifier type.
pub type CommandId = u16;

/// Key codes (terminal-independent).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum KeyCode {
    Char(char),
    F(u8),
    Enter,
    Esc,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
}

/// Key modifiers.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct KeyMod {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

/// A key event.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyMod,
}

/// Mouse button.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Mouse action.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseAction {
    Press(MouseButton),
    Release(MouseButton),
    Move,
    ScrollUp,
    ScrollDown,
}

/// A mouse event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MouseEvent {
    pub x: u16,
    pub y: u16,
    pub action: MouseAction,
    pub modifiers: KeyMod,
}

/// An event flowing through the view tree.
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16),
    Command {
        id: CommandId,
        data: Option<Box<dyn Any + Send>>,
    },
    Tick,
}

impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Key(k) => f.debug_tuple("Key").field(k).finish(),
            Self::Mouse(m) => f.debug_tuple("Mouse").field(m).finish(),
            Self::Paste(s) => f.debug_tuple("Paste").field(&s.len()).finish(),
            Self::Resize(w, h) => f.debug_tuple("Resize").field(w).field(h).finish(),
            Self::Command { id, .. } => f.debug_struct("Command").field("id", id).finish(),
            Self::Tick => write!(f, "Tick"),
        }
    }
}
