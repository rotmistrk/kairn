//! Shared yank register — allows yank in one editor to paste in another.

use std::sync::{Arc, Mutex};

/// Shared yank register state.
#[derive(Default)]
pub(crate) struct SharedRegister {
    pub(crate) text: String,
    pub(crate) linewise: bool,
    pub(crate) block: bool,
}

/// Thread-safe handle to a shared register.
pub(crate) type RegisterHandle = Arc<Mutex<SharedRegister>>;

/// Create a new shared register handle.
pub(crate) fn new_register() -> RegisterHandle {
    Arc::default()
}
