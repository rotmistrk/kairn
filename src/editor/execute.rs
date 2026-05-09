//! Command dispatch and execution.

use super::command::Command;
use super::keymap::EditorMode;
use super::motions;
use super::{Editor, EditorAction};

impl Editor {
    pub fn execute(&mut self, cmd: Command) -> EditorAction {
        if should_record(&cmd) {
            self.last_command = Some(cmd.clone());
        }
        self.dispatch(cmd)
    }

    pub(super) fn dispatch(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::Noop => EditorAction::None,
            Command::MoveLeft => { self.move_left(); EditorAction::CursorMoved }
            Command::MoveRight => { self.move_right(); EditorAction::CursorMoved }
            Command::MoveUp => { self.move_up(); EditorAction::CursorMoved }
            Command::MoveDown => { self.move_down(); EditorAction::CursorMoved }
            Command::MoveWordForward => { self.move_word_forward(); EditorAction::CursorMoved }
            Command::MoveWordBackward => { self.move_word_backward(); EditorAction::CursorMoved }
            Command::MoveWordEnd => { self.move_word_end(); EditorAction::CursorMoved }
            Command::MoveLineStart => { self.cursor_col = 0; EditorAction::CursorMoved }
            Command::MoveLineEnd => { self.move_line_end(); EditorAction::CursorMoved }
            Command::MoveFirstNonBlank => { self.move_first_non_blank(); EditorAction::CursorMoved }
            Command::MoveFileStart => { self.cursor_line = 0; self.cursor_col = 0; EditorAction::CursorMoved }
            Command::MoveFileEnd => {
                self.cursor_line = self.buffer.line_count().saturating_sub(1);
                self.cursor_col = 0;
                EditorAction::CursorMoved
            }
            Command::GotoLine(n) => { self.goto_line(n); EditorAction::CursorMoved }
            Command::HalfPageDown => { self.half_page_down(); EditorAction::CursorMoved }
            Command::HalfPageUp => { self.half_page_up(); EditorAction::CursorMoved }
            Command::PageDown => { self.page_down(); EditorAction::CursorMoved }
            Command::PageUp => { self.page_up(); EditorAction::CursorMoved }
            Command::MatchBracket => { self.match_bracket(); EditorAction::CursorMoved }
            Command::FindChar(ch) => { self.find_char('f', ch); EditorAction::CursorMoved }
            Command::FindCharBack(ch) => { self.find_char('F', ch); EditorAction::CursorMoved }
            Command::TillChar(ch) => { self.find_char('t', ch); EditorAction::CursorMoved }
            Command::TillCharBack(ch) => { self.find_char('T', ch); EditorAction::CursorMoved }
            Command::RepeatFind => { self.repeat_find(false); EditorAction::CursorMoved }
            Command::RepeatFindReverse => { self.repeat_find(true); EditorAction::CursorMoved }
            Command::EnterInsertMode => { self.mode = EditorMode::Insert; EditorAction::ModeChanged }
            Command::EnterInsertAfter => { self.enter_insert_after(); EditorAction::ModeChanged }
            Command::EnterInsertLineEnd => {
                self.mode = EditorMode::Insert;
                self.cursor_col = self.buffer.line_len(self.cursor_line);
                EditorAction::ModeChanged
            }
            Command::EnterInsertLineStart => {
                self.mode = EditorMode::Insert;
                self.cursor_col = motions::first_non_blank(&self.buffer, self.cursor_line);
                EditorAction::ModeChanged
            }
            Command::EnterInsertBelow | Command::NewlineBelow => { self.open_line_below(); EditorAction::ContentChanged }
            Command::EnterInsertAbove | Command::NewlineAbove => { self.open_line_above(); EditorAction::ContentChanged }
            Command::ExitInsertMode => { self.exit_insert(); EditorAction::ModeChanged }
            Command::InsertChar(ch) => { self.insert_char(ch); EditorAction::ContentChanged }
            Command::InsertNewline => { self.insert_newline(); EditorAction::ContentChanged }
            Command::DeleteCharForward => { self.delete_char_forward(); EditorAction::ContentChanged }
            Command::DeleteCharBackward => { self.delete_char_backward(); EditorAction::ContentChanged }
            Command::DeleteLine => { self.delete_line(); EditorAction::ContentChanged }
            Command::DeleteWord => { self.delete_word(); EditorAction::ContentChanged }
            Command::DeleteWordBackward => { self.delete_word_backward(); EditorAction::ContentChanged }
            Command::DeleteToEnd => { self.delete_to_end(); EditorAction::ContentChanged }
            Command::DeleteToStart => { self.delete_to_start(); EditorAction::ContentChanged }
            Command::ChangeWord => { self.delete_word(); self.mode = EditorMode::Insert; EditorAction::ContentChanged }
            Command::ChangeLine => { self.change_line(); EditorAction::ContentChanged }
            Command::ChangeToEnd => { self.delete_to_end(); self.mode = EditorMode::Insert; EditorAction::ContentChanged }
            Command::Substitute => { self.delete_char_forward(); self.mode = EditorMode::Insert; EditorAction::ContentChanged }
            Command::SubstituteLine => { self.change_line(); EditorAction::ContentChanged }
            Command::JoinLines => { self.join_lines(); EditorAction::ContentChanged }
            Command::ToggleCase => { self.toggle_case(); EditorAction::ContentChanged }
            Command::ReplaceChar(ch) => { self.replace_char(ch); EditorAction::ContentChanged }
            Command::Indent => { self.indent_line(); EditorAction::ContentChanged }
            Command::Unindent => { self.unindent_line(); EditorAction::ContentChanged }
            Command::OperatorDelete => { self.delete_word(); EditorAction::ContentChanged }
            Command::OperatorChange => { self.delete_word(); self.mode = EditorMode::Insert; EditorAction::ContentChanged }
            Command::OperatorYank => { self.register = self.buffer.line(self.cursor_line).unwrap_or_default(); EditorAction::None }
            Command::Undo => { self.buffer.undo(); self.clamp_cursor(); EditorAction::ContentChanged }
            Command::Redo => { self.buffer.redo(); self.clamp_cursor(); EditorAction::ContentChanged }
            Command::YankLine => { self.register = self.buffer.line(self.cursor_line).unwrap_or_default(); EditorAction::None }
            Command::YankWord => { self.yank_word(); EditorAction::None }
            Command::YankToEnd => { self.yank_to_end(); EditorAction::None }
            Command::Paste => { self.paste_after(); EditorAction::ContentChanged }
            Command::PasteBefore => { self.paste_before(); EditorAction::ContentChanged }
            Command::EnterVisual => { self.enter_visual(); EditorAction::ModeChanged }
            Command::EnterVisualLine => { self.enter_visual_line(); EditorAction::ModeChanged }
            Command::ExitVisual => { self.exit_visual(); EditorAction::ModeChanged }
            Command::VisualDelete => { self.visual_delete(); EditorAction::ContentChanged }
            Command::VisualYank => { self.visual_yank(); EditorAction::None }
            Command::VisualChange => { self.visual_change(); EditorAction::ContentChanged }
            Command::VisualIndent => { self.visual_indent(); EditorAction::ContentChanged }
            Command::VisualUnindent => { self.visual_unindent(); EditorAction::ContentChanged }
            Command::VisualExCommand => { self.visual_ex_command(); EditorAction::ModeChanged }
            Command::EnterSearchMode => { self.mode = EditorMode::Search; self.command_buf.clear(); EditorAction::ModeChanged }
            Command::SearchForward(ref pat) => { self.search_forward(pat); EditorAction::CursorMoved }
            Command::SearchBackward(ref pat) => { self.search_backward(pat); EditorAction::CursorMoved }
            Command::SearchNext => { self.search_next(); EditorAction::CursorMoved }
            Command::SearchPrev => { self.search_prev(); EditorAction::CursorMoved }
            Command::SearchWordForward => { self.search_word(true); EditorAction::CursorMoved }
            Command::SearchWordBackward => { self.search_word(false); EditorAction::CursorMoved }
            Command::EnterCommandMode => { self.mode = EditorMode::Command; self.command_buf.clear(); EditorAction::ModeChanged }
            Command::ExCommand(ref input) => self.execute_ex(input.clone()),
            Command::Save => EditorAction::SaveRequested,
            Command::CloseBuffer => EditorAction::CloseRequested,
            Command::DotRepeat => {
                if let Some(last) = self.last_command.clone() { self.dispatch(last) } else { EditorAction::None }
            }
            Command::Repeat(n, cmd) => self.dispatch_repeat(n, *cmd),
        }
    }

    fn dispatch_repeat(&mut self, n: usize, cmd: Command) -> EditorAction {
        match cmd {
            Command::YankLine => { self.yank_lines(n); EditorAction::None }
            Command::DeleteLine => { self.delete_lines(n); EditorAction::ContentChanged }
            Command::ChangeLine => { self.change_lines(n); EditorAction::ContentChanged }
            Command::Indent => { self.indent_lines(n); EditorAction::ContentChanged }
            Command::Unindent => { self.unindent_lines(n); EditorAction::ContentChanged }
            Command::JoinLines => { for _ in 0..n { self.join_lines(); } EditorAction::ContentChanged }
            other => {
                let mut last = EditorAction::None;
                for _ in 0..n { last = self.dispatch(other.clone()); }
                last
            }
        }
    }
}

fn should_record(cmd: &Command) -> bool {
    matches!(cmd,
        Command::InsertChar(_) | Command::InsertNewline | Command::DeleteCharForward
        | Command::DeleteCharBackward | Command::DeleteLine | Command::DeleteWord
        | Command::DeleteToEnd | Command::ChangeWord | Command::ChangeLine
        | Command::ChangeToEnd | Command::Substitute | Command::SubstituteLine
        | Command::JoinLines | Command::ToggleCase | Command::ReplaceChar(_)
        | Command::Indent | Command::Unindent | Command::Paste | Command::PasteBefore
    )
}
