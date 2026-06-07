//! Key bindings for the git changes panel.

use txv_core::prelude::*;

/// Key bindings for the git changes panel.
#[derive(Debug, Clone)]
pub struct GitKeys {
    pub(crate) stage: KeyEvent,
    pub(crate) unstage: KeyEvent,
    pub(crate) untrack: KeyEvent,
    pub(crate) commit: KeyEvent,
}

impl Default for GitKeys {
    fn default() -> Self {
        Self {
            stage: KeyEvent::new(KeyCode::Char('s'), KeyMod::NONE),
            unstage: KeyEvent::new(KeyCode::Char('u'), KeyMod::NONE),
            untrack: KeyEvent::new(KeyCode::Char('x'), KeyMod::NONE),
            commit: KeyEvent::new(KeyCode::Char('c'), KeyMod::NONE),
        }
    }
}
