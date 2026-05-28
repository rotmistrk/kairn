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
            stage: KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyMod::default(),
            },
            unstage: KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyMod::default(),
            },
            untrack: KeyEvent {
                code: KeyCode::Char('x'),
                modifiers: KeyMod::default(),
            },
            commit: KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyMod::default(),
            },
        }
    }
}
