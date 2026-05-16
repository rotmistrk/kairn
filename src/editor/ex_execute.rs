//! Ex command execution — dispatches parsed ex commands to editor actions.

use super::ex;
use super::{Editor, EditorAction};

impl Editor {
    pub(super) fn execute_ex(&mut self, input: String) -> EditorAction {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return EditorAction::None;
        }

        // Shell command without range: :!cmd
        if let Some(cmd) = trimmed.strip_prefix('!') {
            let cmd = cmd.trim();
            if !cmd.is_empty() {
                let output = match std::process::Command::new("sh").arg("-c").arg(cmd).output() {
                    Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                    Err(e) => {
                        self.status = format!("Shell error: {e}");
                        return EditorAction::None;
                    }
                };
                return EditorAction::ShellOutput(output);
            }
        }

        let total = self.buf().line_count();
        let Some(ex_cmd) = ex::parse_ex_full(trimmed, self.cursor_line, total) else {
            return EditorAction::AppCommand(trimmed.to_string());
        };

        match ex_cmd {
            ex::ExCommand::Save => EditorAction::SaveRequested,
            ex::ExCommand::Quit => {
                if self.buf().is_dirty() {
                    self.status = "No write since last change (use :q! to override)".to_string();
                    EditorAction::None
                } else {
                    EditorAction::CloseRequested
                }
            }
            ex::ExCommand::QuitForce => EditorAction::ForceCloseRequested,
            ex::ExCommand::SaveQuit => EditorAction::SaveRequested,
            ex::ExCommand::GotoLine(n) => {
                self.goto_line(n);
                EditorAction::CursorMoved
            }
            ex::ExCommand::Edit(filename) => {
                if filename.is_empty() {
                    EditorAction::None
                } else {
                    EditorAction::OpenFile(filename)
                }
            }
            ex::ExCommand::SetGlobal(opt) => {
                if opt.is_empty() {
                    EditorAction::None
                } else {
                    EditorAction::SetGlobal(opt)
                }
            }
            ex::ExCommand::Set(opt) => {
                if !opt.is_empty() {
                    self.apply_set_option(&opt);
                }
                EditorAction::None
            }
            ex::ExCommand::Diff(args) => EditorAction::Diff(args),
            ex::ExCommand::NoDiff => EditorAction::NoDiff,
            ex::ExCommand::NoHighlight => {
                self.highlight = None;
                EditorAction::None
            }
            ex::ExCommand::Split(arg) => EditorAction::Split(arg),
            ex::ExCommand::Vsplit(arg) => EditorAction::Vsplit(arg),
            ex::ExCommand::Only => EditorAction::Only,
            ex::ExCommand::Delete { start, end } => {
                self.ex_delete(start, end);
                EditorAction::ContentChanged
            }
            ex::ExCommand::Yank { start, end } => {
                self.ex_yank(start, end);
                EditorAction::None
            }
            ex::ExCommand::Substitute {
                start,
                end,
                pattern,
                replacement,
                global,
            } => {
                self.ex_substitute(start, end, &pattern, &replacement, global);
                EditorAction::ContentChanged
            }
            ex::ExCommand::Shell { start, end, command } => {
                self.ex_shell(start, end, &command);
                EditorAction::ContentChanged
            }
        }
    }

    fn ex_delete(&mut self, start: usize, end: usize) {
        let total = self.buf().line_count();
        let end = end.min(total.saturating_sub(1));
        let start_off = self.buf().line_col_to_offset(start, 0).unwrap_or(0);
        let end_off = if end + 1 < total {
            self.buf().line_col_to_offset(end + 1, 0).unwrap_or(start_off)
        } else {
            self.buf().content().len()
        };
        if end_off > start_off {
            let content = self.buf().content();
            self.yank(content[start_off..end_off].to_string());
            self.buf().delete(start_off, end_off);
        }
        let target = start.min(self.buf().line_count().saturating_sub(1));
        self.cursor_line = target;
        self.cursor_col = 0;
        let count = end - start + 1;
        self.status = format!("{count} line(s) deleted");
    }

    fn ex_yank(&mut self, start: usize, end: usize) {
        let total = self.buf().line_count();
        let end = end.min(total.saturating_sub(1));
        let start_off = self.buf().line_col_to_offset(start, 0).unwrap_or(0);
        let end_off = if end + 1 < total {
            self.buf().line_col_to_offset(end + 1, 0).unwrap_or(start_off)
        } else {
            self.buf().content().len()
        };
        let content = self.buf().content();
        self.yank(content[start_off..end_off].to_string());
        let count = end - start + 1;
        self.status = format!("{count} line(s) yanked");
    }

    fn ex_substitute(&mut self, start: usize, end: usize, pattern: &str, replacement: &str, global: bool) {
        let total = self.buf().line_count();
        let end = end.min(total.saturating_sub(1));
        let Ok(re) = regex::Regex::new(pattern) else {
            self.status = format!("Invalid regex: {pattern}");
            return;
        };
        self.buf().begin_group();
        let mut count = 0usize;
        for line_idx in (start..=end).rev() {
            let line = self.buf().line(line_idx).unwrap_or_default();
            let new_line = if global {
                re.replace_all(&line, replacement).to_string()
            } else {
                re.replace(&line, replacement).to_string()
            };
            if new_line != line {
                count += 1;
                let line_start = self.buf().line_col_to_offset(line_idx, 0).unwrap_or(0);
                let line_end = self
                    .buf()
                    .line_col_to_offset(line_idx, line.chars().count())
                    .unwrap_or(line_start);
                self.buf().delete(line_start, line_end);
                self.buf().insert(line_start, &new_line);
            }
        }
        self.buf().end_group();
        self.status = format!("{count} substitution(s)");
    }

    fn ex_shell(&mut self, start: usize, end: usize, command: &str) {
        let total = self.buf().line_count();
        let end = end.min(total.saturating_sub(1));
        let mut input_lines = Vec::new();
        for i in start..=end {
            input_lines.push(self.buf().line(i).unwrap_or_default());
        }
        let input = input_lines.join("\n");

        let output = match std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(input.as_bytes()).ok();
                }
                drop(child.stdin.take());
                match child.wait_with_output() {
                    Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                    Err(e) => {
                        self.status = format!("Shell error: {e}");
                        return;
                    }
                }
            }
            Err(e) => {
                self.status = format!("Shell error: {e}");
                return;
            }
        };

        self.buf().begin_group();
        let start_off = self.buf().line_col_to_offset(start, 0).unwrap_or(0);
        let end_off = if end + 1 < total {
            self.buf().line_col_to_offset(end + 1, 0).unwrap_or(start_off)
        } else {
            self.buf().content().len()
        };
        if end_off > start_off {
            self.buf().delete(start_off, end_off);
        }
        let trimmed_output = output.trim_end_matches('\n');
        if !trimmed_output.is_empty() {
            let insert_text = if start_off < self.buf().content().len() || start_off == 0 {
                format!("{trimmed_output}\n")
            } else {
                format!("\n{trimmed_output}")
            };
            self.buf().insert(start_off, &insert_text);
        }
        self.buf().end_group();
        self.cursor_line = start;
        self.cursor_col = 0;
    }
}
