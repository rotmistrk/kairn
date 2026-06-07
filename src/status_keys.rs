//! Key bindings for the status bar labels.

use txv_core::prelude::*;

/// Key bindings for the status bar (visible labels).
#[derive(Debug, Clone)]
pub struct StatusKeys {
    pub(crate) help: KeyEvent,
    pub(crate) tree: KeyEvent,
    pub(crate) main: KeyEvent,
    pub(crate) term: KeyEvent,
    pub(crate) zoom: KeyEvent,
    pub(crate) messages: KeyEvent,
    pub(crate) quit: KeyEvent,
    pub(crate) subpanel_focus: KeyEvent,
    pub(crate) subpanel_move: KeyEvent,
    pub(crate) subpanel_grow: KeyEvent,
    pub(crate) subpanel_shrink: KeyEvent,
}

impl Default for StatusKeys {
    fn default() -> Self {
        Self {
            help: fkey(1),
            tree: fkey(2),
            main: fkey(3),
            term: fkey(4),
            zoom: fkey(5),
            messages: fkey(6),
            quit: ctrl_key('q'),
            subpanel_focus: ctrl_key('w'),
            subpanel_move: ctrl_alt_key('w'),
            subpanel_grow: ctrl_alt_key('='),
            subpanel_shrink: ctrl_alt_key('-'),
        }
    }
}

fn fkey(n: u8) -> KeyEvent {
    KeyEvent::new(KeyCode::F(n), KeyMod::NONE)
}

fn ctrl_key(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyMod::CTRL)
}

fn ctrl_alt_key(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyMod::CTRL.with_alt())
}
