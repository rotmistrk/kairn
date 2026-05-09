/// Emacs-style chord keymap.
///
/// Non-modal: always in "normal" mode (insert-by-default). Uses prefix
/// keys (`Ctrl-X`, `Ctrl-C`) for two-key chord sequences. `Ctrl-G`
/// cancels any pending prefix.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::command::{Command, EditorMode};
use super::keymap::Keymap;

/// Active prefix for two-key chords.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmacsPrefix {
    /// `C-x` prefix — file/buffer/window commands.
    CtrlX,
    /// `C-c` prefix — user/extension commands.
    CtrlC,
}

/// Emacs keymap state.
pub struct EmacsKeymap {
    prefix: Option<EmacsPrefix>,
}

impl EmacsKeymap {
    /// Create a new emacs keymap.
    pub fn new() -> Self {
        Self { prefix: None }
    }
}

impl Default for EmacsKeymap {
    fn default() -> Self {
        Self::new()
    }
}

impl Keymap for EmacsKeymap {
    fn handle_key(&mut self, key: KeyEvent, _mode: EditorMode, _viewport_height: u16) -> Command {
        // Cancel on Ctrl-G regardless of prefix state.
        if is_ctrl(&key, 'g') {
            self.prefix = None;
            return Command::SelectionCancel;
        }

        if let Some(pfx) = self.prefix.take() {
            return self.handle_prefix(pfx, key);
        }

        self.handle_base(key)
    }

    fn mode_label(&self, _mode: EditorMode) -> &str {
        "EMACS"
    }

    fn is_modal(&self) -> bool {
        false
    }

    fn reset(&mut self) {
        self.prefix = None;
    }

    fn pending_display(&self) -> Option<&str> {
        match self.prefix {
            Some(EmacsPrefix::CtrlX) => Some("C-x"),
            Some(EmacsPrefix::CtrlC) => Some("C-c"),
            None => None,
        }
    }
}

// ── Base (no prefix) ──

impl EmacsKeymap {
    fn handle_base(&mut self, key: KeyEvent) -> Command {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        // Ctrl combos
        if ctrl {
            return self.handle_ctrl(key);
        }

        // Alt combos
        if alt {
            return self.handle_alt(key);
        }

        // Plain keys — insert
        match key.code {
            KeyCode::Enter => Command::InsertNewline,
            KeyCode::Backspace => Command::DeleteCharBackward,
            KeyCode::Delete => Command::DeleteCharForward,
            KeyCode::Tab => Command::Indent,
            KeyCode::BackTab => Command::Dedent,
            KeyCode::Char(ch) => Command::InsertChar(ch),
            _ => Command::Noop,
        }
    }

    fn handle_ctrl(&mut self, key: KeyEvent) -> Command {
        match key.code {
            // Prefix keys
            KeyCode::Char('x') => {
                self.prefix = Some(EmacsPrefix::CtrlX);
                Command::Noop
            }
            KeyCode::Char('c') => {
                self.prefix = Some(EmacsPrefix::CtrlC);
                Command::Noop
            }
            // Movement
            KeyCode::Char('f') => Command::MoveRight,
            KeyCode::Char('b') => Command::MoveLeft,
            KeyCode::Char('n') => Command::MoveDown,
            KeyCode::Char('p') => Command::MoveUp,
            KeyCode::Char('a') => Command::MoveLineStart,
            KeyCode::Char('e') => Command::MoveLineEnd,
            KeyCode::Char('v') => Command::PageDown,
            // Editing
            KeyCode::Char('d') => Command::DeleteCharForward,
            KeyCode::Char('k') => Command::DeleteToLineEnd,
            // Undo
            KeyCode::Char('/') => Command::Undo,
            // Selection
            KeyCode::Char(' ') => Command::SelectionStart,
            // Clipboard
            KeyCode::Char('w') => Command::Yank, // cut (yank region in emacs = kill)
            KeyCode::Char('y') => Command::Paste,
            // Search
            KeyCode::Char('s') => Command::SearchForward(String::new()),
            KeyCode::Char('r') => Command::SearchBackward(String::new()),
            _ => Command::Noop,
        }
    }

    fn handle_alt(&mut self, key: KeyEvent) -> Command {
        match key.code {
            KeyCode::Char('f') => Command::MoveWordForward,
            KeyCode::Char('b') => Command::MoveWordBackward,
            KeyCode::Char('<') => Command::MoveFileStart,
            KeyCode::Char('>') => Command::MoveFileEnd,
            KeyCode::Char('v') => Command::PageUp,
            KeyCode::Char('d') => Command::DeleteWord,
            KeyCode::Char('w') => Command::Yank, // copy
            KeyCode::Char('x') => Command::CommandPalette,
            _ => Command::Noop,
        }
    }
}

// ── Prefix handlers ──

impl EmacsKeymap {
    fn handle_prefix(&self, pfx: EmacsPrefix, key: KeyEvent) -> Command {
        match pfx {
            EmacsPrefix::CtrlX => self.handle_ctrl_x(key),
            EmacsPrefix::CtrlC => Command::Noop, // reserved for extensions
        }
    }

    fn handle_ctrl_x(&self, key: KeyEvent) -> Command {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl {
            return match key.code {
                KeyCode::Char('s') => Command::Save,
                KeyCode::Char('c') => Command::Quit,
                KeyCode::Char('f') => Command::FuzzyFileSearch,
                _ => Command::Noop,
            };
        }
        match key.code {
            KeyCode::Char('k') => Command::CloseBuffer,
            KeyCode::Char('o') => Command::FocusNext,
            KeyCode::Char('1') => Command::ToggleControl,
            _ => Command::Noop,
        }
    }
}

/// Check if a key event is Ctrl + the given character.
fn is_ctrl(key: &KeyEvent, ch: char) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char(c) if c == ch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(ch: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL)
    }

    fn alt(ch: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(ch), KeyModifiers::ALT)
    }

    const M: EditorMode = EditorMode::Normal;
    const VP: u16 = 24;

    #[test]
    fn movement_keys() {
        let mut km = EmacsKeymap::new();
        assert_eq!(km.handle_key(ctrl('f'), M, VP), Command::MoveRight);
        assert_eq!(km.handle_key(ctrl('b'), M, VP), Command::MoveLeft);
        assert_eq!(km.handle_key(ctrl('n'), M, VP), Command::MoveDown);
        assert_eq!(km.handle_key(ctrl('p'), M, VP), Command::MoveUp);
        assert_eq!(km.handle_key(ctrl('a'), M, VP), Command::MoveLineStart);
        assert_eq!(km.handle_key(ctrl('e'), M, VP), Command::MoveLineEnd);
    }

    #[test]
    fn alt_movement() {
        let mut km = EmacsKeymap::new();
        assert_eq!(km.handle_key(alt('f'), M, VP), Command::MoveWordForward);
        assert_eq!(km.handle_key(alt('b'), M, VP), Command::MoveWordBackward);
        assert_eq!(km.handle_key(alt('<'), M, VP), Command::MoveFileStart);
        assert_eq!(km.handle_key(alt('>'), M, VP), Command::MoveFileEnd);
        assert_eq!(km.handle_key(alt('v'), M, VP), Command::PageUp);
    }

    #[test]
    fn page_down() {
        let mut km = EmacsKeymap::new();
        assert_eq!(km.handle_key(ctrl('v'), M, VP), Command::PageDown);
    }

    #[test]
    fn editing_keys() {
        let mut km = EmacsKeymap::new();
        assert_eq!(km.handle_key(ctrl('d'), M, VP), Command::DeleteCharForward);
        assert_eq!(km.handle_key(ctrl('k'), M, VP), Command::DeleteToLineEnd);
        assert_eq!(km.handle_key(alt('d'), M, VP), Command::DeleteWord);
    }

    #[test]
    fn undo() {
        let mut km = EmacsKeymap::new();
        assert_eq!(km.handle_key(ctrl('/'), M, VP), Command::Undo);
    }

    #[test]
    fn selection_and_clipboard() {
        let mut km = EmacsKeymap::new();
        assert_eq!(km.handle_key(ctrl(' '), M, VP), Command::SelectionStart);
        assert_eq!(km.handle_key(ctrl('w'), M, VP), Command::Yank);
        assert_eq!(km.handle_key(alt('w'), M, VP), Command::Yank);
        assert_eq!(km.handle_key(ctrl('y'), M, VP), Command::Paste);
    }

    #[test]
    fn search() {
        let mut km = EmacsKeymap::new();
        assert_eq!(
            km.handle_key(ctrl('s'), M, VP),
            Command::SearchForward(String::new())
        );
        assert_eq!(
            km.handle_key(ctrl('r'), M, VP),
            Command::SearchBackward(String::new())
        );
    }

    #[test]
    fn ctrl_x_save() {
        let mut km = EmacsKeymap::new();
        assert_eq!(km.handle_key(ctrl('x'), M, VP), Command::Noop);
        assert_eq!(km.pending_display(), Some("C-x"));
        assert_eq!(km.handle_key(ctrl('s'), M, VP), Command::Save);
        assert!(km.pending_display().is_none());
    }

    #[test]
    fn ctrl_x_quit() {
        let mut km = EmacsKeymap::new();
        km.handle_key(ctrl('x'), M, VP);
        assert_eq!(km.handle_key(ctrl('c'), M, VP), Command::Quit);
    }

    #[test]
    fn ctrl_x_file_search() {
        let mut km = EmacsKeymap::new();
        km.handle_key(ctrl('x'), M, VP);
        assert_eq!(km.handle_key(ctrl('f'), M, VP), Command::FuzzyFileSearch);
    }

    #[test]
    fn ctrl_x_close_buffer() {
        let mut km = EmacsKeymap::new();
        km.handle_key(ctrl('x'), M, VP);
        assert_eq!(
            km.handle_key(key(KeyCode::Char('k')), M, VP),
            Command::CloseBuffer
        );
    }

    #[test]
    fn ctrl_x_focus_next() {
        let mut km = EmacsKeymap::new();
        km.handle_key(ctrl('x'), M, VP);
        assert_eq!(
            km.handle_key(key(KeyCode::Char('o')), M, VP),
            Command::FocusNext
        );
    }

    #[test]
    fn ctrl_x_toggle_control() {
        let mut km = EmacsKeymap::new();
        km.handle_key(ctrl('x'), M, VP);
        assert_eq!(
            km.handle_key(key(KeyCode::Char('1')), M, VP),
            Command::ToggleControl
        );
    }

    #[test]
    fn ctrl_g_cancels_prefix() {
        let mut km = EmacsKeymap::new();
        km.handle_key(ctrl('x'), M, VP);
        assert!(km.pending_display().is_some());
        assert_eq!(km.handle_key(ctrl('g'), M, VP), Command::SelectionCancel);
        assert!(km.pending_display().is_none());
    }

    #[test]
    fn insert_char() {
        let mut km = EmacsKeymap::new();
        assert_eq!(
            km.handle_key(key(KeyCode::Char('a')), M, VP),
            Command::InsertChar('a')
        );
    }

    #[test]
    fn enter_and_backspace() {
        let mut km = EmacsKeymap::new();
        assert_eq!(
            km.handle_key(key(KeyCode::Enter), M, VP),
            Command::InsertNewline
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Backspace), M, VP),
            Command::DeleteCharBackward
        );
    }

    #[test]
    fn not_modal() {
        let km = EmacsKeymap::new();
        assert!(!km.is_modal());
    }

    #[test]
    fn mode_label() {
        let km = EmacsKeymap::new();
        assert_eq!(km.mode_label(EditorMode::Normal), "EMACS");
    }

    #[test]
    fn reset_clears_prefix() {
        let mut km = EmacsKeymap::new();
        km.handle_key(ctrl('x'), M, VP);
        km.reset();
        assert!(km.pending_display().is_none());
    }

    #[test]
    fn command_palette() {
        let mut km = EmacsKeymap::new();
        assert_eq!(km.handle_key(alt('x'), M, VP), Command::CommandPalette);
    }
}
