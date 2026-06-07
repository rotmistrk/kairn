//! Search & Replace view — project-wide find-and-replace with confirmation.

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use txv_core::cell::Style;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use crate::commands::{OpenFileRequest, CM_OPEN_FILE, CM_TAB_CLOSE};
use crate::views::result_entry::ResultEntry;

/// (entry, replacement_line, applied)
type ReplaceMatch = (ResultEntry, String, bool);

/// Search & Replace view — displays matches, allows per-match or bulk apply.
pub struct SearchReplaceView {
    state: ViewState,
    matches: Vec<ReplaceMatch>,
    pattern: String,
    replacement: String,
    cursor: usize,
    scroll: usize,
    title: String,
    root: PathBuf,
}

impl SearchReplaceView {
    pub fn new(pattern: &str, replacement: &str, root: &Path, entries: Vec<ResultEntry>) -> Self {
        let re = Regex::new(pattern).ok();
        let matches: Vec<ReplaceMatch> = entries
            .into_iter()
            .map(|entry| {
                let repl = re
                    .as_ref()
                    .map(|r| r.replace_all(&entry.text, replacement).to_string())
                    .unwrap_or_else(|| entry.text.replace(pattern, replacement));
                (entry, repl, false)
            })
            .collect();
        Self {
            state: ViewState::default(),
            matches,
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
            cursor: 0,
            scroll: 0,
            title: format!("replace:{pattern}"),
            root: root.to_path_buf(),
        }
    }

    fn apply_current(&mut self) {
        if let Some(m) = self.matches.get_mut(self.cursor) {
            if !m.2 {
                apply_replacement(m, &self.pattern, &self.replacement);
            }
        }
        self.advance_cursor();
    }

    fn apply_all(&mut self) {
        for m in &mut self.matches {
            if !m.2 {
                apply_replacement(m, &self.pattern, &self.replacement);
            }
        }
    }

    fn advance_cursor(&mut self) {
        for i in (self.cursor + 1)..self.matches.len() {
            if !self.matches[i].2 {
                self.cursor = i;
                self.ensure_visible();
                return;
            }
        }
    }

    fn ensure_visible(&mut self) {
        let h = self.state.bounds().h.saturating_sub(1) as usize;
        if h == 0 {
            return;
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + h {
            self.scroll = self.cursor - h + 1;
        }
    }

    fn draw_header(&mut self, w: u16, dim: Style) {
        let applied = self.matches.iter().filter(|m| m.2).count();
        let header = format!(
            " /{}/{}/ {}/{} applied [Enter n A q]",
            self.pattern,
            self.replacement,
            applied,
            self.matches.len(),
        );
        let buf = self.state.buffer_mut();
        buf.hline(0, 0, w, ' ', dim);
        buf.print(0, 0, &header[..header.len().min(w as usize)], dim);
    }
}

fn apply_replacement(m: &mut ReplaceMatch, pattern: &str, replacement: &str) {
    let line_idx = m.0.line as usize;
    let Ok(content) = fs::read_to_string(&m.0.path) else {
        return;
    };
    let mut lines: Vec<&str> = content.lines().collect();
    if line_idx >= lines.len() {
        return;
    }
    let re = Regex::new(pattern).ok();
    let new_line = re
        .as_ref()
        .map(|r| r.replace_all(lines[line_idx], replacement).to_string())
        .unwrap_or_else(|| lines[line_idx].replace(pattern, replacement));
    lines[line_idx] = &new_line;
    let new_content = lines.join("\n") + "\n";
    let _ = fs::write(&m.0.path, new_content);
    m.2 = true;
}

impl View for SearchReplaceView {
    delegate_view_state!(state, override { title, draw, handle });

    fn title(&self) -> &str {
        &self.title
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        let pal = palette();
        let normal = Style::default();
        let selected = pal.style(StyleId::CursorFocused);
        let dim = pal.style(StyleId::Dim);
        let success = pal.style(StyleId::StateSuccess);
        self.draw_header(w, dim);
        let buf = self.state.buffer_mut();
        for row in 1..h {
            let idx = self.scroll + (row as usize - 1);
            buf.hline(0, row, w, ' ', normal);
            let Some(m) = self.matches.get(idx) else {
                continue;
            };
            let style = if m.2 {
                success
            } else if idx == self.cursor {
                selected
            } else {
                normal
            };
            let rel = m.0.path.strip_prefix(&self.root).unwrap_or(&m.0.path);
            let prefix = format!("{}:{}: ", rel.display(), m.0.line + 1);
            let text = if m.2 {
                format!("{prefix}✓ {}", m.1)
            } else {
                format!("{prefix}{} → {}", m.0.text, m.1)
            };
            buf.print(0, row, &text[..text.len().min(w as usize)], style);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor + 1 < self.matches.len() {
                    self.cursor += 1;
                    self.ensure_visible();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.cursor = self.cursor.saturating_sub(1);
                self.ensure_visible();
            }
            KeyCode::Enter => self.apply_current(),
            KeyCode::Char('n') => self.advance_cursor(),
            KeyCode::Char('A') => self.apply_all(),
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state.put_command(CM_TAB_CLOSE, None);
            }
            KeyCode::Right => {
                if let Some(m) = self.matches.get(self.cursor) {
                    let req = OpenFileRequest::at(m.0.path.clone(), m.0.line, 0);
                    self.state.put_command(CM_OPEN_FILE, Some(Box::new(req)));
                }
            }
            _ => return HandleResult::Ignored,
        }
        self.state.mark_dirty();
        HandleResult::Consumed
    }
}
