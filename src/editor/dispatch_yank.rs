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
            Command::YankLine | Command::OperatorYank => {
                let line = self.buf().line(self.cursor_line).unwrap_or_default();
                self.yank_linewise(line);
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
                self.do_paste(false);
                EditorAction::ContentChanged
            }
            Command::PasteBefore => {
                self.do_paste(true);
                EditorAction::ContentChanged
            }
            _ => EditorAction::None,
        }
    }

    fn do_paste(&mut self, before: bool) {
        if self.register_block() {
            if before {
                self.block_paste_before();
            } else {
                self.block_paste_after();
            }
        } else if before {
            self.paste_before();
        } else {
            self.paste_after();
        }
    }
}
