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

/// A line from git log --graph output.
pub struct GraphLine {
    pub graph: String,
    pub hash: String,
    pub rest: String,
}

pub struct CommitTreePanel {
    pub root: PathBuf,
    pub lines: Vec<GraphLine>,
    pub cursor: usize,
    pub scroll: usize,
}

impl CommitTreePanel {
    pub fn new(root: PathBuf) -> Self {
        let lines = load_graph(&root);
        Self {
            root,
            lines,
            cursor: 0,
            scroll: 0,
        }
    }

    pub fn refresh(&mut self) {
        self.lines = load_graph(&self.root);
        self.cursor = self.cursor.min(self.lines.len().saturating_sub(1));
    }

    fn selected_hash(&self) -> Option<&str> {
        self.lines.get(self.cursor).and_then(|l| {
            if l.hash.is_empty() {
                None
            } else {
                Some(l.hash.as_str())
            }
        })
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
        let scroll = super::adjust_scroll(self.cursor, self.scroll, inner_h);
        let items: Vec<ListItem<'_>> = self
            .lines
            .iter()
            .enumerate()
            .skip(scroll)
            .take(inner_h)
            .map(|(i, gl)| graph_item(gl, i == self.cursor))
            .collect();

        frame.render_widget(List::new(items).block(block), area);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<PanelAction> {
        let count = self.lines.len();
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
            KeyCode::PageUp => self.cursor = self.cursor.saturating_sub(20),
            KeyCode::PageDown => {
                self.cursor = (self.cursor + 20).min(count.saturating_sub(1));
            }
            KeyCode::Right => {
                return Ok(PanelAction::FocusRight);
            }
            _ => {}
        }
        if let Some(hash) = self.selected_hash() {
            return Ok(PanelAction::PreviewCommit(hash.to_string()));
        }
        Ok(PanelAction::None)
    }
}

fn graph_item<'a>(gl: &'a GraphLine, selected: bool) -> ListItem<'a> {
    let bg = if selected { Color::Blue } else { Color::Reset };
    let mut spans = vec![Span::styled(
        &gl.graph,
        Style::default().fg(Color::Red).bg(bg),
    )];
    if !gl.hash.is_empty() {
        spans.push(Span::styled(
            &gl.hash,
            Style::default().fg(Color::Yellow).bg(bg),
        ));
        spans.push(Span::styled(" ", Style::default().bg(bg)));
        spans.push(Span::styled(
            &gl.rest,
            Style::default().fg(Color::White).bg(bg),
        ));
    }
    ListItem::new(Line::from(spans))
}

fn load_graph(root: &std::path::Path) -> Vec<GraphLine> {
    let output = std::process::Command::new("git")
        .args([
            "log",
            "--graph",
            "--oneline",
            "--all",
            "--decorate",
            "--abbrev-commit",
            "-200",
        ])
        .current_dir(root)
        .output();
    let text = match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => return Vec::new(),
    };
    text.lines().map(parse_graph_line).collect()
}

fn parse_graph_line(line: &str) -> GraphLine {
    // Graph chars: * | / \ space, then optional hash + message
    let mut graph_end = 0;
    for (i, ch) in line.char_indices() {
        if matches!(ch, '*' | '|' | '/' | '\\' | ' ' | '_') {
            graph_end = i + ch.len_utf8();
        } else {
            break;
        }
    }
    let graph = line[..graph_end].to_string();
    let rest = line[graph_end..].trim_start();

    // Try to extract hash (first word if it looks like hex)
    if let Some(space) = rest.find(' ') {
        let word = &rest[..space];
        if word.len() >= 6 && word.chars().all(|c| c.is_ascii_hexdigit()) {
            return GraphLine {
                graph,
                hash: word.to_string(),
                rest: rest[space + 1..].to_string(),
            };
        }
    }

    GraphLine {
        graph,
        hash: String::new(),
        rest: rest.to_string(),
    }
}
