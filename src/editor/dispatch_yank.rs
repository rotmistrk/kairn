//! Yank/paste/undo dispatch.

use super::command::Command;
use super::{Editor, EditorAction};

impl Editor {
    pub(super) fn dispatch_yank_ops(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::Undo => {
                self.buf().undo();
                self.clamp_cursor();
                EditorAction::ContentChanged
            }
            Command::Redo => {
                self.buf().redo();
                self.clamp_cursor();
                EditorAction::ContentChanged
            }
            Command::YankLine => {
                let line = self.buf().line(self.cursor_line).unwrap_or_default();
                self.yank(line);
                EditorAction::None
            }
            Command::YankWord => {
                self.yank_word();
                EditorAction::None
            }
            Command::YankToEnd => {
                self.yank_to_end();
                EditorAction::None
            }
            Command::Paste => {
                self.paste_after();
                EditorAction::ContentChanged
            }
            Command::PasteBefore => {
                self.paste_before();
                EditorAction::ContentChanged
            }
            Command::OperatorYank => {
                let line = self.buf().line(self.cursor_line).unwrap_or_default();
                self.yank(line);
                EditorAction::None
            }
            _ => EditorAction::None,
        }
    }
}
