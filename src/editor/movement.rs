//! Editor movement methods.

use super::keymap::EditorMode;
use super::{motions, Editor};

impl Editor {
    pub(super) fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    pub(super) fn move_right(&mut self) {
        let line_len = self.buf().line_len(self.cursor_line);
        let max = if self.mode == EditorMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1)
        };
        if self.cursor_col < max {
            self.cursor_col += 1;
        }
    }

    pub(super) fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.clamp_col();
        }
    }

    pub(super) fn move_down(&mut self) {
        if self.cursor_line + 1 < self.buf().line_count() {
            self.cursor_line += 1;
            self.clamp_col();
        }
    }

    pub(super) fn move_word_forward(&mut self) {
        let (l, c) = motions::word_forward(&self.buf(), self.cursor_line, self.cursor_col);
        self.cursor_line = l;
        self.cursor_col = c;
    }

    pub(super) fn move_word_backward(&mut self) {
        let (l, c) = motions::word_backward(&self.buf(), self.cursor_line, self.cursor_col);
        self.cursor_line = l;
        self.cursor_col = c;
    }

    pub(super) fn move_word_end(&mut self) {
        let (l, c) = motions::word_end(&self.buf(), self.cursor_line, self.cursor_col);
        self.cursor_line = l;
        self.cursor_col = c;
    }

    pub(super) fn move_line_end(&mut self) {
        let len = self.buf().line_len(self.cursor_line);
        self.cursor_col = len.saturating_sub(1);
    }

    pub(super) fn move_first_non_blank(&mut self) {
        let col = motions::first_non_blank(&self.buf(), self.cursor_line);
        self.cursor_col = col;
    }

    pub(super) fn goto_line(&mut self, n: usize) {
        let target = n.saturating_sub(1).min(self.buf().line_count().saturating_sub(1));
        self.cursor_line = target;
        self.cursor_col = 0;
    }

    pub(super) fn half_page_down(&mut self) {
        let half = self.viewport_height / 2;
        let max_line = self.buf().line_count().saturating_sub(1);
        self.cursor_line = (self.cursor_line + half).min(max_line);
        self.clamp_col();
    }

    pub(super) fn half_page_up(&mut self) {
        let half = self.viewport_height / 2;
        self.cursor_line = self.cursor_line.saturating_sub(half);
        self.clamp_col();
    }

    pub(super) fn page_down(&mut self) {
        let page = self.viewport_height.saturating_sub(2);
        let max_line = self.buf().line_count().saturating_sub(1);
        self.cursor_line = (self.cursor_line + page).min(max_line);
        self.clamp_col();
    }

    pub(super) fn page_up(&mut self) {
        let page = self.viewport_height.saturating_sub(2);
        self.cursor_line = self.cursor_line.saturating_sub(page);
        self.clamp_col();
    }

    pub(super) fn match_bracket(&mut self) {
        let result = motions::match_bracket(&self.buf(), self.cursor_line, self.cursor_col);
        if let Some((l, c)) = result {
            self.cursor_line = l;
            self.cursor_col = c;
        }
    }

    pub(super) fn find_char(&mut self, cmd: char, target: char) {
        self.last_find = Some((cmd, target));
        self.execute_find(cmd, target);
    }

    pub(super) fn execute_find(&mut self, cmd: char, target: char) {
        let result = match cmd {
            'f' => motions::find_char(&self.buf(), self.cursor_line, self.cursor_col, target),
            'F' => motions::find_char_back(&self.buf(), self.cursor_line, self.cursor_col, target),
            't' => motions::find_char(&self.buf(), self.cursor_line, self.cursor_col, target)
                .map(|c| c.saturating_sub(1).max(self.cursor_col + 1)),
            'T' => motions::find_char_back(&self.buf(), self.cursor_line, self.cursor_col, target)
                .map(|c| (c + 1).min(self.cursor_col.saturating_sub(1))),
            _ => None,
        };
        if let Some(col) = result {
            self.cursor_col = col;
        }
    }

    pub(super) fn repeat_find(&mut self, reverse: bool) {
        if let Some((cmd, ch)) = self.last_find {
            let actual_cmd = if reverse {
                match cmd {
                    'f' => 'F',
                    'F' => 'f',
                    't' => 'T',
                    'T' => 't',
                    _ => cmd,
                }
            } else {
                cmd
            };
            self.execute_find(actual_cmd, ch);
        }
    }
}
