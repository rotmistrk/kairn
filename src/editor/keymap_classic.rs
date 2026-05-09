/// Classic (menu-driven) keymap.
///
/// Stateless — no pending keys, no modes. Inspired by nano/mcedit/F4.
/// Arrow keys for movement, Ctrl combos for actions, function keys for
/// search navigation and help.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::command::{Command, EditorMode};
use super::keymap::Keymap;

/// Classic keymap — no state needed.
pub struct ClassicKeymap;

impl ClassicKeymap {
    /// Create a new classic keymap.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClassicKeymap {
    fn default() -> Self {
        Self::new()
    }
}

impl Keymap for ClassicKeymap {
    fn handle_key(&mut self, key: KeyEvent, _mode: EditorMode, _viewport_height: u16) -> Command {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        if ctrl {
            return handle_ctrl(key.code);
        }

        match key.code {
            // Arrow movement
            KeyCode::Left => Command::MoveLeft,
            KeyCode::Right => Command::MoveRight,
            KeyCode::Up => Command::MoveUp,
            KeyCode::Down => Command::MoveDown,
            KeyCode::Home => Command::MoveLineStart,
            KeyCode::End => Command::MoveLineEnd,
            KeyCode::PageUp => Command::PageUp,
            KeyCode::PageDown => Command::PageDown,
            // Editing
            KeyCode::Backspace => Command::DeleteCharBackward,
            KeyCode::Delete => Command::DeleteCharForward,
            KeyCode::Enter => Command::InsertNewline,
            KeyCode::Tab => Command::Indent,
            KeyCode::BackTab => Command::Dedent,
            // Function keys
            KeyCode::F(2) => Command::Save,
            KeyCode::F(3) if shift => Command::SearchPrev,
            KeyCode::F(3) => Command::SearchNext,
            // Insert characters
            KeyCode::Char(ch) => Command::InsertChar(ch),
            _ => Command::Noop,
        }
    }

    fn mode_label(&self, _mode: EditorMode) -> &str {
        "CLASSIC"
    }

    fn is_modal(&self) -> bool {
        false
    }

    fn reset(&mut self) {}

    fn pending_display(&self) -> Option<&str> {
        None
    }
}

/// Handle Ctrl+key combinations.
fn handle_ctrl(code: KeyCode) -> Command {
    match code {
        // Movement
        KeyCode::Left => Command::MoveWordBackward,
        KeyCode::Right => Command::MoveWordForward,
        KeyCode::Home => Command::MoveFileStart,
        KeyCode::End => Command::MoveFileEnd,
        // Editing
        KeyCode::Char('d') => Command::DeleteLine,
        KeyCode::Char('k') => Command::DeleteToLineEnd,
        // Undo/redo
        KeyCode::Char('z') => Command::Undo,
        KeyCode::Char('y') => Command::Redo,
        // Clipboard
        KeyCode::Char('c') => Command::Yank,
        KeyCode::Char('x') => Command::Yank, // cut
        KeyCode::Char('v') => Command::Paste,
        // File
        KeyCode::Char('s') => Command::Save,
        KeyCode::Char('q') => Command::Quit,
        KeyCode::Char('o') => Command::FuzzyFileSearch,
        KeyCode::Char('w') => Command::CloseBuffer,
        // Search
        KeyCode::Char('f') => Command::SearchForward(String::new()),
        KeyCode::Char('g') => Command::GotoLine(0),
        // Application
        KeyCode::Char('p') => Command::FuzzyFileSearch,
        KeyCode::Char('a') => Command::SelectAll,
        _ => Command::Noop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    fn shift_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::SHIFT)
    }

    const M: EditorMode = EditorMode::Normal;
    const VP: u16 = 24;

    #[test]
    fn arrow_movement() {
        let mut km = ClassicKeymap::new();
        assert_eq!(km.handle_key(key(KeyCode::Left), M, VP), Command::MoveLeft);
        assert_eq!(
            km.handle_key(key(KeyCode::Right), M, VP),
            Command::MoveRight
        );
        assert_eq!(km.handle_key(key(KeyCode::Up), M, VP), Command::MoveUp);
        assert_eq!(km.handle_key(key(KeyCode::Down), M, VP), Command::MoveDown);
    }

    #[test]
    fn home_end() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(key(KeyCode::Home), M, VP),
            Command::MoveLineStart
        );
        assert_eq!(
            km.handle_key(key(KeyCode::End), M, VP),
            Command::MoveLineEnd
        );
    }

    #[test]
    fn ctrl_home_end() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Home), M, VP),
            Command::MoveFileStart
        );
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::End), M, VP),
            Command::MoveFileEnd
        );
    }

    #[test]
    fn ctrl_word_movement() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Left), M, VP),
            Command::MoveWordBackward
        );
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Right), M, VP),
            Command::MoveWordForward
        );
    }

    #[test]
    fn page_up_down() {
        let mut km = ClassicKeymap::new();
        assert_eq!(km.handle_key(key(KeyCode::PageUp), M, VP), Command::PageUp);
        assert_eq!(
            km.handle_key(key(KeyCode::PageDown), M, VP),
            Command::PageDown
        );
    }

    #[test]
    fn editing_keys() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(key(KeyCode::Backspace), M, VP),
            Command::DeleteCharBackward
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Delete), M, VP),
            Command::DeleteCharForward
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Enter), M, VP),
            Command::InsertNewline
        );
    }

    #[test]
    fn ctrl_editing() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('d')), M, VP),
            Command::DeleteLine
        );
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('k')), M, VP),
            Command::DeleteToLineEnd
        );
    }

    #[test]
    fn undo_redo() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('z')), M, VP),
            Command::Undo
        );
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('y')), M, VP),
            Command::Redo
        );
    }

    #[test]
    fn clipboard() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('c')), M, VP),
            Command::Yank
        );
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('x')), M, VP),
            Command::Yank
        );
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('v')), M, VP),
            Command::Paste
        );
    }

    #[test]
    fn file_ops() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('s')), M, VP),
            Command::Save
        );
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('q')), M, VP),
            Command::Quit
        );
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('w')), M, VP),
            Command::CloseBuffer
        );
    }

    #[test]
    fn search() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('f')), M, VP),
            Command::SearchForward(String::new())
        );
        assert_eq!(
            km.handle_key(key(KeyCode::F(3)), M, VP),
            Command::SearchNext
        );
        assert_eq!(
            km.handle_key(shift_key(KeyCode::F(3)), M, VP),
            Command::SearchPrev
        );
    }

    #[test]
    fn select_all() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('a')), M, VP),
            Command::SelectAll
        );
    }

    #[test]
    fn insert_char() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(key(KeyCode::Char('z')), M, VP),
            Command::InsertChar('z')
        );
    }

    #[test]
    fn f2_save() {
        let mut km = ClassicKeymap::new();
        assert_eq!(km.handle_key(key(KeyCode::F(2)), M, VP), Command::Save);
    }

    #[test]
    fn indent_dedent() {
        let mut km = ClassicKeymap::new();
        assert_eq!(km.handle_key(key(KeyCode::Tab), M, VP), Command::Indent);
        assert_eq!(km.handle_key(key(KeyCode::BackTab), M, VP), Command::Dedent);
    }

    #[test]
    fn not_modal() {
        let km = ClassicKeymap::new();
        assert!(!km.is_modal());
    }

    #[test]
    fn mode_label() {
        let km = ClassicKeymap::new();
        assert_eq!(km.mode_label(EditorMode::Normal), "CLASSIC");
    }

    #[test]
    fn no_pending() {
        let km = ClassicKeymap::new();
        assert!(km.pending_display().is_none());
    }

    #[test]
    fn fuzzy_search() {
        let mut km = ClassicKeymap::new();
        assert_eq!(
            km.handle_key(ctrl_key(KeyCode::Char('p')), M, VP),
            Command::FuzzyFileSearch
        );
    }
}
