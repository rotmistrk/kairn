//! Editor dispatch: search and command-mode commands.

use super::command::Command;
use super::keymap::EditorMode;
use super::{Editor, EditorAction};

impl Editor {
    pub(super) fn dispatch_search_and_command(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::EnterSearchMode => {
                self.mode = EditorMode::Search;
                self.command_buf.clear();
                EditorAction::ModeChanged
            }
            Command::SearchForward(ref pat) => {
                self.search_forward(pat);
                EditorAction::CursorMoved
            }
            Command::SearchBackward(ref pat) => {
                self.search_backward(pat);
                EditorAction::CursorMoved
            }
            Command::SearchNext => {
                self.search_next();
                EditorAction::CursorMoved
            }
            Command::SearchPrev => {
                self.search_prev();
                EditorAction::CursorMoved
            }
            Command::SearchWordForward => {
                self.search_word(true);
                EditorAction::CursorMoved
            }
            Command::SearchWordBackward => {
                self.search_word(false);
                EditorAction::CursorMoved
            }
            Command::EnterCommandMode => {
                self.mode = EditorMode::Command;
                self.command_buf.clear();
                EditorAction::ModeChanged
            }
            Command::CompletionNext | Command::CompletionPrev => EditorAction::LspCompletion,
            _ => EditorAction::None,
        }
    }
}
