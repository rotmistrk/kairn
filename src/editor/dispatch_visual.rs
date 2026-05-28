//! Editor dispatch: visual mode commands.

use super::command::Command;
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
            _ => EditorAction::None,
        }
    }
}
