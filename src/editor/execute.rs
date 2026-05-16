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
            Command::MoveLeft => {
                self.move_left();
                EditorAction::CursorMoved
            }
            Command::MoveRight => {
                self.move_right();
                EditorAction::CursorMoved
            }
            Command::MoveUp => {
                self.move_up();
                EditorAction::CursorMoved
            }
            Command::MoveDown => {
                self.move_down();
                EditorAction::CursorMoved
            }
            Command::MoveWordForward => {
                self.move_word_forward();
                EditorAction::CursorMoved
            }
            Command::MoveWordBackward => {
                self.move_word_backward();
                EditorAction::CursorMoved
            }
            Command::MoveWordEnd => {
                self.move_word_end();
                EditorAction::CursorMoved
            }
            Command::MoveLineStart => {
                self.cursor_col = 0;
                EditorAction::CursorMoved
            }
            Command::MoveLineEnd => {
                self.move_line_end();
                EditorAction::CursorMoved
            }
            Command::MoveFirstNonBlank => {
                self.move_first_non_blank();
                EditorAction::CursorMoved
            }
            Command::MoveFileStart => {
                self.cursor_line = 0;
                self.cursor_col = 0;
                EditorAction::CursorMoved
            }
            Command::MoveFileEnd => {
                let last = self.buf().line_count().saturating_sub(1);
                self.cursor_line = last;
                self.cursor_col = 0;
                EditorAction::CursorMoved
            }
            Command::GotoLine(n) => {
                self.goto_line(n);
                EditorAction::CursorMoved
            }
            Command::HalfPageDown => {
                self.half_page_down();
                EditorAction::CursorMoved
            }
            Command::HalfPageUp => {
                self.half_page_up();
                EditorAction::CursorMoved
            }
            Command::PageDown => {
                self.page_down();
                EditorAction::CursorMoved
            }
            Command::PageUp => {
                self.page_up();
                EditorAction::CursorMoved
            }
            Command::MatchBracket => {
                self.match_bracket();
                EditorAction::CursorMoved
            }
            Command::FindChar(ch) => {
                self.find_char('f', ch);
                EditorAction::CursorMoved
            }
            Command::FindCharBack(ch) => {
                self.find_char('F', ch);
                EditorAction::CursorMoved
            }
            Command::TillChar(ch) => {
                self.find_char('t', ch);
                EditorAction::CursorMoved
            }
            Command::TillCharBack(ch) => {
                self.find_char('T', ch);
                EditorAction::CursorMoved
            }
            Command::RepeatFind => {
                self.repeat_find(false);
                EditorAction::CursorMoved
            }
            Command::RepeatFindReverse => {
                self.repeat_find(true);
                EditorAction::CursorMoved
            }
            Command::ExCommand(ref input) => self.execute_ex(input.clone()),
            Command::Save => EditorAction::SaveRequested,
            Command::CloseBuffer => EditorAction::CloseRequested,
            Command::GotoDefinition => EditorAction::LspGotoDefinition,
            Command::GotoShow => EditorAction::LspGotoShow,
            Command::FindReferences => EditorAction::LspFindReferences,
            Command::Hover => EditorAction::LspHover,
            Command::LspRename => {
                let word = motions::word_at(&self.buf(), self.cursor_line, self.cursor_col).unwrap_or_default();
                self.mode = EditorMode::Command;
                self.command_buf = format!("lsp-rename {word}");
                EditorAction::ModeChanged
            }
            Command::DotRepeat => {
                if let Some(last) = self.last_command.clone() {
                    self.dispatch(last)
                } else {
                    EditorAction::None
                }
            }
            Command::Repeat(n, cmd) => self.dispatch_repeat(n, *cmd),
            other => self.dispatch_edit(other).unwrap_or(EditorAction::None),
        }
    }

    fn dispatch_repeat(&mut self, n: usize, cmd: Command) -> EditorAction {
        match cmd {
            Command::YankLine => {
                self.yank_lines(n);
                EditorAction::None
            }
            Command::DeleteLine => {
                self.delete_lines(n);
                EditorAction::ContentChanged
            }
            Command::ChangeLine => {
                self.change_lines(n);
                EditorAction::ContentChanged
            }
            Command::Indent => {
                self.indent_lines(n);
                EditorAction::ContentChanged
            }
            Command::Unindent => {
                self.unindent_lines(n);
                EditorAction::ContentChanged
            }
            Command::JoinLines => {
                for _ in 0..n {
                    self.join_lines();
                }
                EditorAction::ContentChanged
            }
            other => {
                let mut last = EditorAction::None;
                for _ in 0..n {
                    last = self.dispatch(other.clone());
                }
                last
            }
        }
    }
}

fn should_record(cmd: &Command) -> bool {
    matches!(
        cmd,
        Command::InsertChar(_)
            | Command::InsertNewline
            | Command::DeleteCharForward
            | Command::DeleteCharBackward
            | Command::DeleteLine
            | Command::DeleteWord
            | Command::DeleteToEnd
            | Command::ChangeWord
            | Command::ChangeLine
            | Command::ChangeToEnd
            | Command::Substitute
            | Command::SubstituteLine
            | Command::JoinLines
            | Command::ToggleCase
            | Command::ReplaceChar(_)
            | Command::Indent
            | Command::Unindent
            | Command::Paste
            | Command::PasteBefore
    )
}
