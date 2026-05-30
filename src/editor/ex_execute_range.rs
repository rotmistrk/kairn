//! Ex command execution — range operations (delete, yank, substitute, shell filter).

use std::process::{Command as ProcessCommand, Stdio};

use regex::Regex;

use super::Editor;

impl Editor {
    pub(super) fn ex_delete(&mut self, start: usize, end: usize) {
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
            self.yank_linewise(content[start_off..end_off].to_string());
            self.buf().delete(start_off, end_off);
        }
        let target = start.min(self.buf().line_count().saturating_sub(1));
        self.cursor_line = target;
        self.cursor_col = 0;
        let count = end - start + 1;
        self.status = format!("{count} line(s) deleted");
    }

    pub(super) fn ex_yank(&mut self, start: usize, end: usize) {
        let total = self.buf().line_count();
        let end = end.min(total.saturating_sub(1));
        let start_off = self.buf().line_col_to_offset(start, 0).unwrap_or(0);
        let end_off = if end + 1 < total {
            self.buf().line_col_to_offset(end + 1, 0).unwrap_or(start_off)
        } else {
            self.buf().content().len()
        };
        let content = self.buf().content();
        self.yank_linewise(content[start_off..end_off].to_string());
        let count = end - start + 1;
        self.status = format!("{count} line(s) yanked");
    }

    pub(super) fn ex_substitute(&mut self, start: usize, end: usize, pattern: &str, replacement: &str, global: bool) {
        let total = self.buf().line_count();
        let end = end.min(total.saturating_sub(1));
        let Ok(re) = Regex::new(pattern) else {
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

    pub(super) fn ex_shell(&mut self, start: usize, end: usize, command: &str) {
        let total = self.buf().line_count();
        let end = end.min(total.saturating_sub(1));
        let input = self.collect_lines(start, end);

        let output = match self.run_shell_filter(command, &input) {
            Ok(out) => out,
            Err(e) => {
                self.status = format!("Shell error: {e}");
                return;
            }
        };

        self.replace_range_with_output(start, end, total, &output);
        self.cursor_line = start;
        self.cursor_col = 0;
    }

    fn collect_lines(&self, start: usize, end: usize) -> String {
        let mut lines = Vec::new();
        for i in start..=end {
            lines.push(self.buf().line(i).unwrap_or_default());
        }
        lines.join("\n")
    }

    fn run_shell_filter(&self, command: &str, input: &str) -> Result<String, String> {
        let mut child = ProcessCommand::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        use std::io::Write;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(input.as_bytes()).ok();
        }
        drop(child.stdin.take());

        let out = child.wait_with_output().map_err(|e| e.to_string())?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn replace_range_with_output(&mut self, start: usize, end: usize, total: usize, output: &str) {
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
    }
}
