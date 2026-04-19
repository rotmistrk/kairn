use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use super::{Panel, PanelAction};
use crate::diff::LogEntry;

pub struct CommitTreePanel {
    pub root: PathBuf,
    pub commits: Vec<LogEntry>,
    pub cursor: usize,
    pub scroll: usize,
}

impl CommitTreePanel {
    pub fn new(root: PathBuf) -> Self {
        let commits = crate::diff::git_log(&root, None, 500).unwrap_or_default();
        Self {
            root,
            commits,
            cursor: 0,
            scroll: 0,
        }
    }

    pub fn refresh(&mut self) {
        self.commits = crate::diff::git_log(&self.root, None, 500).unwrap_or_default();
        self.cursor = self.cursor.min(self.commits.len().saturating_sub(1));
    }

    pub fn selected_hash(&self) -> Option<&str> {
        self.commits.get(self.cursor).map(|c| c.hash_short.as_str())
    }
}

impl Panel for CommitTreePanel {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_color = if focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        let block = Block::default()
            .title(" Commits ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner_h = area.height.saturating_sub(2) as usize;
        let scroll = adjust_scroll(self.cursor, self.scroll, inner_h);
        let items: Vec<ListItem<'_>> = self
            .commits
            .iter()
            .enumerate()
            .skip(scroll)
            .take(inner_h)
            .map(|(i, c)| commit_item(c, i == self.cursor))
            .collect();

        frame.render_widget(List::new(items).block(block), area);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<PanelAction> {
        let count = self.commits.len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor + 1 < count {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = count.saturating_sub(1),
            KeyCode::PageUp => {
                self.cursor = self.cursor.saturating_sub(20);
            }
            KeyCode::PageDown => {
                self.cursor = (self.cursor + 20).min(count.saturating_sub(1));
            }
            _ => {}
        }
        // Preview commit diff
        if let Some(hash) = self.selected_hash() {
            return Ok(PanelAction::PreviewCommit(hash.to_string()));
        }
        Ok(PanelAction::None)
    }
}

fn commit_item<'a>(c: &'a LogEntry, selected: bool) -> ListItem<'a> {
    let style = if selected {
        Style::default().bg(Color::Blue)
    } else {
        Style::default()
    };
    let line = Line::from(vec![
        Span::styled(&c.hash_short, style.fg(Color::Yellow)),
        Span::styled(" ", style),
        Span::styled(&c.date, style.fg(Color::DarkGray)),
        Span::styled(" ", style),
        Span::styled(truncate(&c.message, 40), style.fg(Color::White)),
    ]);
    ListItem::new(line)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

fn adjust_scroll(cursor: usize, current: usize, height: usize) -> usize {
    if height == 0 {
        return 0;
    }
    if cursor < current {
        cursor
    } else if cursor >= current + height {
        cursor - height + 1
    } else {
        current
    }
}
