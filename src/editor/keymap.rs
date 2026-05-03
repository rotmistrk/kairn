/// Trait for translating key events into editor commands.
///
/// Implementations are stateful — they track pending multi-key sequences
/// (e.g. vim `d` waiting for a motion, emacs `C-x` prefix).
use crossterm::event::KeyEvent;

use super::command::{Command, EditorMode};

/// Translates key events into editor commands.
pub trait Keymap: Send {
    /// Process a key event. Returns a command (or `Noop`).
    fn handle_key(&mut self, key: KeyEvent, mode: EditorMode, viewport_height: u16) -> Command;

    /// Display name for the status bar.
    fn mode_label(&self, mode: EditorMode) -> &str;

    /// Whether this keymap uses modal editing.
    fn is_modal(&self) -> bool;

    /// Reset any pending state (e.g. after Esc).
    fn reset(&mut self);

    /// Pending key display for status bar (e.g. "d" waiting for motion).
    fn pending_display(&self) -> Option<&str>;
}
