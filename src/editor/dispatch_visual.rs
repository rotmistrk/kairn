//! Editor dispatch: visual mode commands.

use super::command::Command;
use super::keymap::EditorMode;
use super::{Editor, EditorAction};

impl Editor {
    pub(super) fn dispatch_visual_ops(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::EnterVisual => {
                self.enter_visual();
                EditorAction::ModeChanged
            }
            Command::EnterVisualLine => {
                self.enter_visual_line();
                EditorAction::ModeChanged
            }
            Command::EnterVisualBlock => {
                self.enter_visual_block();
                EditorAction::ModeChanged
            }
            Command::ExitVisual => {
                self.exit_visual();
                EditorAction::ModeChanged
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
            _ => self.dispatch_visual_block_ops(cmd),
        }
    }

    fn dispatch_visual_block_ops(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::VisualDelete => {
                self.do_visual_delete();
                EditorAction::ContentChanged
            }
            Command::VisualYank => {
                self.do_visual_yank();
                EditorAction::None
            }
            Command::VisualChange => {
                self.do_visual_change();
                EditorAction::ContentChanged
            }
            Command::BlockInsert => {
                self.dispatch_block_insert();
                EditorAction::ModeChanged
            }
            Command::BlockAppend => {
                self.dispatch_block_append();
                EditorAction::ModeChanged
            }
            Command::BlockReplace(ch) => {
                if self.mode == EditorMode::VisualBlock {
                    self.block_replace(ch);
                }
                EditorAction::ContentChanged
            }
            _ => EditorAction::None,
        }
    }

    fn do_visual_delete(&mut self) {
        if self.mode == EditorMode::VisualBlock {
            self.block_delete();
        } else {
            self.visual_delete();
        }
    }

    fn do_visual_yank(&mut self) {
        if self.mode == EditorMode::VisualBlock {
            self.block_yank();
        } else {
            self.visual_yank();
        }
    }

    fn do_visual_change(&mut self) {
        if self.mode == EditorMode::VisualBlock {
            self.block_change();
        } else {
            self.visual_change();
        }
    }

    fn dispatch_block_insert(&mut self) {
        if self.mode == EditorMode::VisualBlock {
            if let Some((sl, _, sc, _)) = self.block_range() {
                self.exit_visual();
                self.cursor_line = sl;
                self.cursor_col = sc;
                self.mode = EditorMode::Insert;
            }
        }
    }

    fn dispatch_block_append(&mut self) {
        if self.mode == EditorMode::VisualBlock {
            if let Some((sl, _, _, ec)) = self.block_range() {
                self.exit_visual();
                self.cursor_line = sl;
                self.cursor_col = ec + 1;
                self.mode = EditorMode::Insert;
            }
        }
    }
}
