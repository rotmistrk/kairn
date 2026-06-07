//! Key construction helpers.

use txv_core::prelude::*;

pub const ALT_X: KeyEvent = KeyEvent::new(KeyCode::Char('x'), KeyMod::ALT);
pub const APPROX: KeyEvent = KeyEvent::new(KeyCode::Char('≈'), KeyMod::NONE);

pub fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyMod::NONE)
}

pub fn ctrl(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyMod::CTRL)
}

pub fn alt(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyMod::ALT)
}
