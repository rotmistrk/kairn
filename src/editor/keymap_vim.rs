/// Vim-style modal keymap.
///
/// Translates crossterm key events into [`Command`] values based on the
/// current [`EditorMode`]. Supports numeric prefixes, two-key sequences
/// (`dd`, `gg`, `dw`, etc.), and command-line / search input.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::command::{Command, EditorMode, VisualKind};
use super::keymap::Keymap;

/// Vim keymap state.
pub struct VimKeymap {
    pending: Option<char>,
    count: Option<usize>,
    command_buf: String,
    search_buf: String,
    in_search: bool,
    search_backward: bool,
    in_command: bool,
}

impl VimKeymap {
    /// Create a new vim keymap.
    pub fn new() -> Self {
        Self {
            pending: None,
            count: None,
            command_buf: String::new(),
            search_buf: String::new(),
            in_search: false,
            search_backward: false,
            in_command: false,
        }
    }

    fn accumulate_count(&mut self, digit: char) {
        let d = digit as usize - '0' as usize;
        self.count = Some(self.count.unwrap_or(0) * 10 + d);
    }

    fn take_count(&mut self) -> usize {
        self.count.take().unwrap_or(1)
    }

    fn with_count(&mut self, cmd: Command) -> Command {
        let n = self.take_count();
        if n <= 1 {
            return cmd;
        }
        // For movement commands, wrap in Repeat (editor handles it).
        // Since Command has no Repeat variant, we return the base command
        // and let the editor check the count via a separate mechanism.
        // For now, return the command — the Editor will query pending_count.
        cmd
    }
}

impl Default for VimKeymap {
    fn default() -> Self {
        Self::new()
    }
}

impl Keymap for VimKeymap {
    fn handle_key(&mut self, key: KeyEvent, mode: EditorMode, _viewport_height: u16) -> Command {
        match mode {
            EditorMode::Normal => self.handle_normal(key),
            EditorMode::Insert => self.handle_insert(key),
            EditorMode::Visual(_) => self.handle_visual(key),
            EditorMode::CommandLine => self.handle_cmdline(key),
        }
    }

    fn mode_label(&self, mode: EditorMode) -> &str {
        match mode {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Visual(VisualKind::Stream) => "VISUAL",
            EditorMode::Visual(VisualKind::Line) => "V-LINE",
            EditorMode::Visual(VisualKind::Block) => "V-BLOCK",
            EditorMode::CommandLine => "COMMAND",
        }
    }

    fn is_modal(&self) -> bool {
        true
    }

    fn reset(&mut self) {
        self.pending = None;
        self.count = None;
        self.in_search = false;
        self.in_command = false;
        self.command_buf.clear();
        self.search_buf.clear();
    }

    fn pending_display(&self) -> Option<&str> {
        if self.in_command {
            return Some(&self.command_buf);
        }
        if self.in_search {
            return Some(&self.search_buf);
        }
        match self.pending {
            Some('d') => Some("d"),
            Some('c') => Some("c"),
            Some('y') => Some("y"),
            Some('g') => Some("g"),
            Some('>') => Some(">"),
            Some('<') => Some("<"),
            _ => None,
        }
    }
}

// ── Normal mode ──

impl VimKeymap {
    fn handle_normal(&mut self, key: KeyEvent) -> Command {
        // Handle pending two-key sequences first
        if let Some(prefix) = self.pending.take() {
            return self.handle_pending(prefix, key);
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            // Numeric prefix (1-9 start, 0 only extends)
            KeyCode::Char(c @ '1'..='9') if !ctrl => {
                self.accumulate_count(c);
                Command::Noop
            }
            KeyCode::Char('0') if !ctrl && self.count.is_some() => {
                self.accumulate_count('0');
                Command::Noop
            }
            // Movement
            KeyCode::Char('h') if !ctrl => self.with_count(Command::MoveLeft),
            KeyCode::Char('l') if !ctrl => self.with_count(Command::MoveRight),
            KeyCode::Char('j') if !ctrl => self.with_count(Command::MoveDown),
            KeyCode::Char('k') if !ctrl => self.with_count(Command::MoveUp),
            KeyCode::Char('w') if !ctrl => self.with_count(Command::MoveWordForward),
            KeyCode::Char('b') if !ctrl => self.with_count(Command::MoveWordBackward),
            KeyCode::Char('0') if !ctrl => Command::MoveLineStart,
            KeyCode::Char('$') if !ctrl => Command::MoveLineEnd,
            KeyCode::Char('G') if !ctrl => {
                if let Some(n) = self.count.take() {
                    Command::GotoLine(n.saturating_sub(1))
                } else {
                    Command::MoveFileEnd
                }
            }
            KeyCode::Char('%') if !ctrl => Command::BracketMatch,
            // Ctrl movement
            KeyCode::Char('d') if ctrl => Command::HalfPageDown,
            KeyCode::Char('u') if ctrl => Command::HalfPageUp,
            KeyCode::Char('f') if ctrl => Command::PageDown,
            KeyCode::Char('b') if ctrl => Command::PageUp,
            KeyCode::Char('r') if ctrl => Command::Redo,
            // Insert mode entry
            KeyCode::Char('i') if !ctrl => Command::EnterInsertMode,
            KeyCode::Char('a') if !ctrl => Command::EnterInsertAfter,
            KeyCode::Char('I') if !ctrl => Command::EnterInsertLineStart,
            KeyCode::Char('A') if !ctrl => Command::EnterInsertLineEnd,
            KeyCode::Char('o') if !ctrl => Command::EnterInsertBelow,
            KeyCode::Char('O') if !ctrl => Command::EnterInsertAbove,
            // Single-key editing
            KeyCode::Char('x') if !ctrl => Command::DeleteCharForward,
            KeyCode::Char('J') if !ctrl => Command::JoinLines,
            KeyCode::Char('u') if !ctrl => Command::Undo,
            KeyCode::Char('p') if !ctrl => Command::Paste,
            KeyCode::Char('P') if !ctrl => Command::PasteBefore,
            // Visual mode
            KeyCode::Char('v') if !ctrl => Command::SelectionStart,
            KeyCode::Char('V') if !ctrl => Command::SelectionLineStart,
            KeyCode::Char('v') if ctrl => Command::SelectionBlockStart,
            // Search
            KeyCode::Char('n') if !ctrl => Command::SearchNext,
            KeyCode::Char('N') if !ctrl => Command::SearchPrev,
            KeyCode::Char('*') if !ctrl => Command::SearchWordUnderCursor,
            KeyCode::Char('/') if !ctrl => {
                self.in_search = true;
                self.search_backward = false;
                self.search_buf.clear();
                Command::Noop
            }
            KeyCode::Char('?') if !ctrl => {
                self.in_search = true;
                self.search_backward = true;
                self.search_buf.clear();
                Command::Noop
            }
            // Command mode
            KeyCode::Char(':') if !ctrl => {
                self.in_command = true;
                self.command_buf.clear();
                Command::EnterCommandMode
            }
            // Two-key prefixes
            KeyCode::Char(c @ ('d' | 'c' | 'y' | 'g' | '>' | '<')) if !ctrl => {
                self.pending = Some(c);
                Command::Noop
            }
            KeyCode::Esc => Command::SelectionCancel,
            _ => Command::Noop,
        }
    }

    fn handle_pending(&mut self, prefix: char, key: KeyEvent) -> Command {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match (prefix, key.code) {
            // dd → DeleteLine
            ('d', KeyCode::Char('d')) if !ctrl => {
                self.take_count();
                Command::DeleteLine
            }
            // dw → DeleteWord
            ('d', KeyCode::Char('w')) if !ctrl => {
                self.take_count();
                Command::DeleteWord
            }
            // d$ → DeleteToLineEnd
            ('d', KeyCode::Char('$')) if !ctrl => {
                self.take_count();
                Command::DeleteToLineEnd
            }
            // cc → DeleteLine + enter insert
            ('c', KeyCode::Char('c')) if !ctrl => {
                self.take_count();
                Command::DeleteLine
            }
            // cw → DeleteWord (editor enters insert after)
            ('c', KeyCode::Char('w')) if !ctrl => {
                self.take_count();
                Command::DeleteWord
            }
            // c$ → DeleteToLineEnd
            ('c', KeyCode::Char('$')) if !ctrl => {
                self.take_count();
                Command::DeleteToLineEnd
            }
            // yy → YankLine
            ('y', KeyCode::Char('y')) if !ctrl => {
                self.take_count();
                Command::YankLine
            }
            // gg → MoveFileStart
            ('g', KeyCode::Char('g')) if !ctrl => {
                self.take_count();
                Command::MoveFileStart
            }
            // gd → GotoDefinition
            ('g', KeyCode::Char('d')) if !ctrl => {
                self.take_count();
                Command::GotoDefinition
            }
            // gr → GotoReferences
            ('g', KeyCode::Char('r')) if !ctrl => {
                self.take_count();
                Command::GotoReferences
            }
            // >> → Indent
            ('>', KeyCode::Char('>')) if !ctrl => {
                self.take_count();
                Command::Indent
            }
            // << → Dedent
            ('<', KeyCode::Char('<')) if !ctrl => {
                self.take_count();
                Command::Dedent
            }
            // Unrecognized second key — cancel
            _ => {
                self.count = None;
                Command::Noop
            }
        }
    }
}

// ── Insert mode ──

impl VimKeymap {
    fn handle_insert(&mut self, key: KeyEvent) -> Command {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Esc => Command::ExitInsertMode,
            KeyCode::Enter => Command::InsertNewline,
            KeyCode::Backspace => Command::DeleteCharBackward,
            KeyCode::Delete => Command::DeleteCharForward,
            KeyCode::Char('w') if ctrl => Command::DeleteWordBackward,
            KeyCode::Char('u') if ctrl => Command::DeleteToLineStart,
            KeyCode::Char(ch) if !ctrl => Command::InsertChar(ch),
            _ => Command::Noop,
        }
    }
}

// ── Visual mode ──

impl VimKeymap {
    fn handle_visual(&mut self, key: KeyEvent) -> Command {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Esc => Command::SelectionCancel,
            // Movement (same as normal)
            KeyCode::Char('h') if !ctrl => Command::MoveLeft,
            KeyCode::Char('l') if !ctrl => Command::MoveRight,
            KeyCode::Char('j') if !ctrl => Command::MoveDown,
            KeyCode::Char('k') if !ctrl => Command::MoveUp,
            KeyCode::Char('w') if !ctrl => Command::MoveWordForward,
            KeyCode::Char('b') if !ctrl => Command::MoveWordBackward,
            KeyCode::Char('0') if !ctrl => Command::MoveLineStart,
            KeyCode::Char('$') if !ctrl => Command::MoveLineEnd,
            KeyCode::Char('G') if !ctrl => Command::MoveFileEnd,
            // Actions on selection
            KeyCode::Char('y') if !ctrl => Command::Yank,
            KeyCode::Char('d') if !ctrl => Command::DeleteCharForward,
            KeyCode::Char('x') if !ctrl => Command::DeleteCharForward,
            // Mode switching
            KeyCode::Char('v') if !ctrl => Command::SelectionStart,
            KeyCode::Char('V') if !ctrl => Command::SelectionLineStart,
            KeyCode::Char('v') if ctrl => Command::SelectionBlockStart,
            _ => Command::Noop,
        }
    }
}

// ── Command-line mode ──

impl VimKeymap {
    fn handle_cmdline(&mut self, key: KeyEvent) -> Command {
        match key.code {
            KeyCode::Esc => {
                self.in_command = false;
                self.in_search = false;
                self.command_buf.clear();
                self.search_buf.clear();
                Command::ExitInsertMode
            }
            KeyCode::Enter if self.in_search => {
                self.in_search = false;
                let pattern = std::mem::take(&mut self.search_buf);
                if self.search_backward {
                    Command::SearchBackward(pattern)
                } else {
                    Command::SearchForward(pattern)
                }
            }
            KeyCode::Enter if self.in_command => {
                self.in_command = false;
                let cmd = std::mem::take(&mut self.command_buf);
                Command::ExCommand(cmd)
            }
            KeyCode::Backspace if self.in_search => {
                self.search_buf.pop();
                Command::Noop
            }
            KeyCode::Backspace if self.in_command => {
                self.command_buf.pop();
                Command::Noop
            }
            KeyCode::Char(ch) if self.in_search => {
                self.search_buf.push(ch);
                Command::Noop
            }
            KeyCode::Char(ch) if self.in_command => {
                self.command_buf.push(ch);
                Command::Noop
            }
            _ => Command::Noop,
        }
    }
}

/// Return the pending numeric count (for the Editor to apply repetition).
impl VimKeymap {
    /// Peek at the current count without consuming it.
    pub fn pending_count(&self) -> Option<usize> {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn normal_movement() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        assert_eq!(
            km.handle_key(key(KeyCode::Char('h')), m, 24),
            Command::MoveLeft
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Char('l')), m, 24),
            Command::MoveRight
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Char('j')), m, 24),
            Command::MoveDown
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Char('k')), m, 24),
            Command::MoveUp
        );
    }

    #[test]
    fn normal_word_movement() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        assert_eq!(
            km.handle_key(key(KeyCode::Char('w')), m, 24),
            Command::MoveWordForward
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Char('b')), m, 24),
            Command::MoveWordBackward
        );
    }

    #[test]
    fn normal_insert_modes() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        assert_eq!(
            km.handle_key(key(KeyCode::Char('i')), m, 24),
            Command::EnterInsertMode
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Char('a')), m, 24),
            Command::EnterInsertAfter
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Char('o')), m, 24),
            Command::EnterInsertBelow
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Char('O')), m, 24),
            Command::EnterInsertAbove
        );
    }

    #[test]
    fn two_key_dd() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        assert_eq!(km.handle_key(key(KeyCode::Char('d')), m, 24), Command::Noop);
        assert_eq!(
            km.handle_key(key(KeyCode::Char('d')), m, 24),
            Command::DeleteLine
        );
    }

    #[test]
    fn two_key_gg() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        assert_eq!(km.handle_key(key(KeyCode::Char('g')), m, 24), Command::Noop);
        assert_eq!(
            km.handle_key(key(KeyCode::Char('g')), m, 24),
            Command::MoveFileStart
        );
    }

    #[test]
    fn two_key_yy() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        assert_eq!(km.handle_key(key(KeyCode::Char('y')), m, 24), Command::Noop);
        assert_eq!(
            km.handle_key(key(KeyCode::Char('y')), m, 24),
            Command::YankLine
        );
    }

    #[test]
    fn two_key_indent_dedent() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        assert_eq!(km.handle_key(key(KeyCode::Char('>')), m, 24), Command::Noop);
        assert_eq!(
            km.handle_key(key(KeyCode::Char('>')), m, 24),
            Command::Indent
        );
        assert_eq!(km.handle_key(key(KeyCode::Char('<')), m, 24), Command::Noop);
        assert_eq!(
            km.handle_key(key(KeyCode::Char('<')), m, 24),
            Command::Dedent
        );
    }

    #[test]
    fn ctrl_keys() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        assert_eq!(
            km.handle_key(ctrl(KeyCode::Char('d')), m, 24),
            Command::HalfPageDown
        );
        assert_eq!(
            km.handle_key(ctrl(KeyCode::Char('u')), m, 24),
            Command::HalfPageUp
        );
        assert_eq!(
            km.handle_key(ctrl(KeyCode::Char('r')), m, 24),
            Command::Redo
        );
    }

    #[test]
    fn insert_mode_keys() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Insert;
        assert_eq!(
            km.handle_key(key(KeyCode::Char('a')), m, 24),
            Command::InsertChar('a')
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Enter), m, 24),
            Command::InsertNewline
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Backspace), m, 24),
            Command::DeleteCharBackward
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Esc), m, 24),
            Command::ExitInsertMode
        );
    }

    #[test]
    fn insert_ctrl_keys() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Insert;
        assert_eq!(
            km.handle_key(ctrl(KeyCode::Char('w')), m, 24),
            Command::DeleteWordBackward
        );
        assert_eq!(
            km.handle_key(ctrl(KeyCode::Char('u')), m, 24),
            Command::DeleteToLineStart
        );
    }

    #[test]
    fn visual_mode_keys() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Visual(VisualKind::Stream);
        assert_eq!(km.handle_key(key(KeyCode::Char('y')), m, 24), Command::Yank);
        assert_eq!(
            km.handle_key(key(KeyCode::Esc), m, 24),
            Command::SelectionCancel
        );
    }

    #[test]
    fn search_mode() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        // Enter search
        assert_eq!(km.handle_key(key(KeyCode::Char('/')), m, 24), Command::Noop);
        assert!(km.in_search);
        // Type search term in command-line mode
        let cm = EditorMode::CommandLine;
        assert_eq!(
            km.handle_key(key(KeyCode::Char('f')), cm, 24),
            Command::Noop
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Char('o')), cm, 24),
            Command::Noop
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Char('o')), cm, 24),
            Command::Noop
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Enter), cm, 24),
            Command::SearchForward("foo".into())
        );
    }

    #[test]
    fn command_mode() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        assert_eq!(
            km.handle_key(key(KeyCode::Char(':')), m, 24),
            Command::EnterCommandMode
        );
        let cm = EditorMode::CommandLine;
        assert_eq!(
            km.handle_key(key(KeyCode::Char('w')), cm, 24),
            Command::Noop
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Char('q')), cm, 24),
            Command::Noop
        );
        assert_eq!(
            km.handle_key(key(KeyCode::Enter), cm, 24),
            Command::ExCommand("wq".into())
        );
    }

    #[test]
    fn mode_labels() {
        let km = VimKeymap::new();
        assert_eq!(km.mode_label(EditorMode::Normal), "NORMAL");
        assert_eq!(km.mode_label(EditorMode::Insert), "INSERT");
        assert_eq!(
            km.mode_label(EditorMode::Visual(VisualKind::Block)),
            "V-BLOCK"
        );
    }

    #[test]
    fn pending_display_shows_prefix() {
        let mut km = VimKeymap::new();
        assert!(km.pending_display().is_none());
        km.handle_key(key(KeyCode::Char('d')), EditorMode::Normal, 24);
        assert_eq!(km.pending_display(), Some("d"));
    }

    #[test]
    fn goto_line_with_count() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        km.handle_key(key(KeyCode::Char('5')), m, 24);
        let cmd = km.handle_key(key(KeyCode::Char('G')), m, 24);
        assert_eq!(cmd, Command::GotoLine(4)); // 5G → line 4 (0-indexed)
    }

    #[test]
    fn numeric_prefix_accumulates() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        km.handle_key(key(KeyCode::Char('1')), m, 24);
        km.handle_key(key(KeyCode::Char('2')), m, 24);
        assert_eq!(km.pending_count(), Some(12));
    }

    #[test]
    fn is_modal() {
        let km = VimKeymap::new();
        assert!(km.is_modal());
    }

    #[test]
    fn reset_clears_state() {
        let mut km = VimKeymap::new();
        km.handle_key(key(KeyCode::Char('d')), EditorMode::Normal, 24);
        assert!(km.pending_display().is_some());
        km.reset();
        assert!(km.pending_display().is_none());
    }

    #[test]
    fn gd_goto_definition() {
        let mut km = VimKeymap::new();
        let m = EditorMode::Normal;
        km.handle_key(key(KeyCode::Char('g')), m, 24);
        assert_eq!(
            km.handle_key(key(KeyCode::Char('d')), m, 24),
            Command::GotoDefinition
        );
    }
}
