//! Editing, visual, and search command dispatch.

use super::command::Command;
use super::keymap::EditorMode;
use super::motions::first_non_blank;
use super::{Editor, EditorAction};

impl Editor {
    /// Dispatch editing, visual, search, and mode commands.
    /// Returns None if the command is not handled here.
    #[rustfmt::skip]
    pub(super) fn dispatch_edit(&mut self, cmd: Command) -> Option<EditorAction> {
        Some(match cmd {
            Command::EnterInsertMode | Command::EnterInsertAfter
            | Command::EnterInsertLineEnd | Command::EnterInsertLineStart
            | Command::EnterInsertBelow | Command::NewlineBelow
            | Command::EnterInsertAbove | Command::NewlineAbove
            | Command::ExitInsertMode => self.dispatch_mode_ops(cmd),
            Command::InsertChar(_) | Command::InsertNewline
            | Command::DeleteCharForward | Command::DeleteCharBackward
            | Command::DeleteLine | Command::DeleteWord
            | Command::DeleteWordBackward | Command::DeleteToEnd
            | Command::DeleteToStart => self.dispatch_mutate_ops(cmd),
            Command::ChangeWord | Command::ChangeLine | Command::ChangeToEnd
            | Command::Substitute | Command::SubstituteLine | Command::JoinLines
            | Command::ToggleCase | Command::ReplaceChar(_)
            | Command::Indent | Command::Unindent => self.dispatch_change_ops(cmd),
            Command::OperatorDelete | Command::OperatorChange => self.dispatch_operator_ops(cmd),
            Command::Undo | Command::Redo | Command::YankLine | Command::YankWord
            | Command::YankToEnd | Command::Paste | Command::PasteBefore
            | Command::OperatorYank => self.dispatch_yank_ops(cmd),
            Command::EnterVisual | Command::EnterVisualLine | Command::EnterVisualBlock | Command::ExitVisual
            | Command::VisualDelete | Command::VisualYank | Command::VisualChange
            | Command::VisualIndent | Command::VisualUnindent
            | Command::VisualExCommand | Command::BlockInsert | Command::BlockAppend
            | Command::BlockReplace(_) => self.dispatch_visual_ops(cmd),
            Command::EnterSearchMode | Command::SearchForward(_)
            | Command::SearchBackward(_) | Command::SearchNext | Command::SearchPrev
            | Command::SearchWordForward | Command::SearchWordBackward
            | Command::EnterCommandMode | Command::CompletionNext
            | Command::CompletionPrev => self.dispatch_search_and_command(cmd),
            _ => return None,
        })
    }

    fn dispatch_mode_ops(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::EnterInsertMode => {
                self.buf().begin_group();
                self.mode = EditorMode::Insert;
                EditorAction::ModeChanged
            }
            Command::EnterInsertAfter => {
                self.enter_insert_after();
                EditorAction::ModeChanged
            }
            Command::EnterInsertLineEnd => {
                self.buf().begin_group();
                self.mode = EditorMode::Insert;
                self.move_line_end();
                self.cursor_col += 1;
                EditorAction::ModeChanged
            }
            Command::EnterInsertLineStart => {
                self.buf().begin_group();
                self.mode = EditorMode::Insert;
                let col = first_non_blank(&self.buf(), self.cursor_line);
                self.cursor_col = col;
                EditorAction::ModeChanged
            }
            Command::EnterInsertBelow | Command::NewlineBelow => {
                self.open_line_below();
                EditorAction::ContentChanged
            }
            Command::EnterInsertAbove | Command::NewlineAbove => {
                self.open_line_above();
                EditorAction::ContentChanged
            }
            Command::ExitInsertMode => {
                self.exit_insert();
                EditorAction::ModeChanged
            }
            _ => EditorAction::None,
        }
    }

    fn dispatch_mutate_ops(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::InsertChar(ch) => {
                self.insert_char(ch);
                EditorAction::ContentChanged
            }
            Command::InsertNewline => {
                self.insert_newline();
                EditorAction::ContentChanged
            }
            Command::DeleteCharForward => {
                self.delete_char_forward();
                EditorAction::ContentChanged
            }
            Command::DeleteCharBackward => {
                self.delete_char_backward();
                EditorAction::ContentChanged
            }
            Command::DeleteLine => {
                self.delete_line();
                EditorAction::ContentChanged
            }
            Command::DeleteWord => {
                self.delete_word();
                EditorAction::ContentChanged
            }
            Command::DeleteWordBackward => {
                self.delete_word_backward();
                EditorAction::ContentChanged
            }
            Command::DeleteToEnd => {
                self.delete_to_end();
                EditorAction::ContentChanged
            }
            Command::DeleteToStart => {
                self.delete_to_start();
                EditorAction::ContentChanged
            }
            _ => EditorAction::None,
        }
    }

    fn dispatch_change_ops(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::ChangeWord => {
                self.buf().begin_group();
                self.delete_word();
                self.mode = EditorMode::Insert;
                EditorAction::ContentChanged
            }
            Command::ChangeLine => {
                self.change_line();
                EditorAction::ContentChanged
            }
            Command::ChangeToEnd => {
                self.buf().begin_group();
                self.delete_to_end();
                self.mode = EditorMode::Insert;
                EditorAction::ContentChanged
            }
            Command::Substitute => {
                self.buf().begin_group();
                self.delete_char_forward();
                self.mode = EditorMode::Insert;
                EditorAction::ContentChanged
            }
            Command::SubstituteLine => {
                self.change_line();
                EditorAction::ContentChanged
            }
            _ => self.dispatch_misc_edit(cmd),
        }
    }

    fn dispatch_misc_edit(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::JoinLines => self.join_lines(),
            Command::ToggleCase => self.toggle_case(),
            Command::ReplaceChar(ch) => self.replace_char(ch),
            Command::Indent => self.indent_line(),
            Command::Unindent => self.unindent_line(),
            _ => return EditorAction::None,
        }
        EditorAction::ContentChanged
    }

    fn dispatch_operator_ops(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::OperatorDelete => {
                self.delete_word();
                EditorAction::ContentChanged
            }
            Command::OperatorChange => {
                self.buf().begin_group();
                self.delete_word();
                self.mode = EditorMode::Insert;
                EditorAction::ContentChanged
            }
            _ => EditorAction::None,
        }
    }
}
