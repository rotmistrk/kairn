//! Editing, visual, and search command dispatch.

use super::command::Command;
use super::keymap::EditorMode;
use super::{Editor, EditorAction};

impl Editor {
    /// Dispatch editing, visual, search, and mode commands.
    /// Returns None if the command is not handled here.
    pub(super) fn dispatch_edit(&mut self, cmd: Command) -> Option<EditorAction> {
        Some(match cmd {
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
                let col = super::motions::first_non_blank(&self.buf(), self.cursor_line);
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
            Command::JoinLines => {
                self.join_lines();
                EditorAction::ContentChanged
            }
            Command::ToggleCase => {
                self.toggle_case();
                EditorAction::ContentChanged
            }
            Command::ReplaceChar(ch) => {
                self.replace_char(ch);
                EditorAction::ContentChanged
            }
            Command::Indent => {
                self.indent_line();
                EditorAction::ContentChanged
            }
            Command::Unindent => {
                self.unindent_line();
                EditorAction::ContentChanged
            }
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
            Command::OperatorYank => {
                let line = self.buf().line(self.cursor_line).unwrap_or_default();
                self.yank(line);
                EditorAction::None
            }
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
            Command::EnterVisual => {
                self.enter_visual();
                EditorAction::ModeChanged
            }
            Command::EnterVisualLine => {
                self.enter_visual_line();
                EditorAction::ModeChanged
            }
            Command::ExitVisual => {
                self.exit_visual();
                EditorAction::ModeChanged
            }
            Command::VisualDelete => {
                self.visual_delete();
                EditorAction::ContentChanged
            }
            Command::VisualYank => {
                self.visual_yank();
                EditorAction::None
            }
            Command::VisualChange => {
                self.visual_change();
                EditorAction::ContentChanged
            }
            Command::VisualIndent => {
                self.visual_indent();
                EditorAction::ContentChanged
            }
            Command::VisualUnindent => {
                self.visual_unindent();
                EditorAction::ContentChanged
            }
            Command::VisualExCommand => {
                self.visual_ex_command();
                EditorAction::ModeChanged
            }
            Command::EnterSearchMode
            | Command::SearchForward(_)
            | Command::SearchBackward(_)
            | Command::SearchNext
            | Command::SearchPrev
            | Command::SearchWordForward
            | Command::SearchWordBackward
            | Command::EnterCommandMode
            | Command::CompletionNext
            | Command::CompletionPrev => self.dispatch_search_and_command(cmd),
            _ => return None,
        })
    }
}
