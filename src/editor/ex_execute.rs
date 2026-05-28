//! Ex command execution — dispatches parsed ex commands to editor actions.

use std::process::Command as ProcessCommand;

use super::ex::{self, ExCommand};
use super::{Editor, EditorAction};

impl Editor {
    pub(super) fn execute_ex(&mut self, input: String) -> EditorAction {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return EditorAction::None;
        }

        if let Some(cmd) = trimmed.strip_prefix('!') {
            let cmd = cmd.trim();
            if !cmd.is_empty() {
                return self.execute_ex_shell_inline(cmd);
            }
        }

        let total = self.buf().line_count();
        let Some(ex_cmd) = ex::parse_ex_full(trimmed, self.cursor_line, total) else {
            return EditorAction::AppCommand(trimmed.to_string());
        };

        self.dispatch_ex_cmd(ex_cmd)
    }

    fn execute_ex_shell_inline(&mut self, cmd: &str) -> EditorAction {
        let output = match ProcessCommand::new("sh").arg("-c").arg(cmd).output() {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(e) => {
                self.status = format!("Shell error: {e}");
                return EditorAction::None;
            }
        };
        EditorAction::ShellOutput(output)
    }

    fn dispatch_ex_cmd(&mut self, ex_cmd: ExCommand) -> EditorAction {
        match ex_cmd {
            ExCommand::Save => EditorAction::SaveRequested,
            ExCommand::Quit => {
                if self.buf().is_dirty() {
                    self.status = "No write since last change (use :q! to override)".to_string();
                    EditorAction::None
                } else {
                    EditorAction::CloseRequested
                }
            }
            ExCommand::QuitForce => EditorAction::ForceCloseRequested,
            ExCommand::SaveQuit => EditorAction::SaveRequested,
            ExCommand::GotoLine(n) => {
                self.goto_line(n);
                EditorAction::CursorMoved
            }
            ExCommand::Edit(filename) => {
                if filename.is_empty() {
                    EditorAction::None
                } else {
                    EditorAction::OpenFile(filename)
                }
            }
            ExCommand::SetGlobal(opt) => {
                if opt.is_empty() {
                    EditorAction::None
                } else {
                    EditorAction::SetGlobal(opt)
                }
            }
            ExCommand::Set(opt) => {
                if !opt.is_empty() {
                    self.apply_set_option(&opt);
                }
                EditorAction::None
            }
            _ => self.dispatch_ex_cmd_range(ex_cmd),
        }
    }

    fn dispatch_ex_cmd_range(&mut self, ex_cmd: ExCommand) -> EditorAction {
        match ex_cmd {
            ExCommand::Diff(args) => EditorAction::Diff(args),
            ExCommand::NoDiff => EditorAction::NoDiff,
            ExCommand::NoBlame => EditorAction::NoBlame,
            ExCommand::Revert => EditorAction::Revert,
            ExCommand::NoHighlight => {
                self.highlight = None;
                EditorAction::None
            }
            ExCommand::Split(arg) => EditorAction::Split(arg),
            ExCommand::Vsplit(arg) => EditorAction::Vsplit(arg),
            ExCommand::Only => EditorAction::Only,
            ExCommand::Delete { start, end } => {
                self.ex_delete(start, end);
                EditorAction::ContentChanged
            }
            ExCommand::Yank { start, end } => {
                self.ex_yank(start, end);
                EditorAction::None
            }
            ExCommand::Substitute {
                start,
                end,
                pattern,
                replacement,
                global,
            } => {
                self.ex_substitute(start, end, &pattern, &replacement, global);
                EditorAction::ContentChanged
            }
            ExCommand::Shell { start, end, command } => {
                self.ex_shell(start, end, &command);
                EditorAction::ContentChanged
            }
            _ => EditorAction::None,
        }
    }
}
