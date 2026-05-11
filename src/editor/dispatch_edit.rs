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
                self.buffer.begin_group();
                self.mode = EditorMode::Insert;
                EditorAction::ModeChanged
            }
            Command::EnterInsertAfter => {
                self.enter_insert_after();
                EditorAction::ModeChanged
            }
            Command::EnterInsertLineEnd => {
                self.buffer.begin_group();
                self.mode = EditorMode::Insert;
                self.move_line_end();
                self.cursor_col += 1;
                EditorAction::ModeChanged
            }
            Command::EnterInsertLineStart => {
                self.buffer.begin_group();
                self.mode = EditorMode::Insert;
                self.cursor_col = super::motions::first_non_blank(&self.buffer, self.cursor_line);
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
                self.buffer.begin_group();
                self.delete_word();
                self.mode = EditorMode::Insert;
                EditorAction::ContentChanged
            }
            Command::ChangeLine => {
                self.change_line();
                EditorAction::ContentChanged
            }
            Command::ChangeToEnd => {
                self.buffer.begin_group();
                self.delete_to_end();
                self.mode = EditorMode::Insert;
                EditorAction::ContentChanged
            }
            Command::Substitute => {
                self.buffer.begin_group();
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
                self.buffer.begin_group();
                self.delete_word();
                self.mode = EditorMode::Insert;
                EditorAction::ContentChanged
            }
            Command::OperatorYank => {
                self.yank(self.buffer.line(self.cursor_line).unwrap_or_default());
                EditorAction::None
            }
            Command::Undo => {
                self.buffer.undo();
                self.clamp_cursor();
                EditorAction::ContentChanged
            }
            Command::Redo => {
                self.buffer.redo();
                self.clamp_cursor();
                EditorAction::ContentChanged
            }
            Command::YankLine => {
                self.yank(self.buffer.line(self.cursor_line).unwrap_or_default());
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
            _ => return None,
        })
    }
}
