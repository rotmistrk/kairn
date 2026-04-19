use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use super::{Panel, PanelAction};
use crate::tree::{self, FileNode, NodeKind};

pub struct FileTreePanel {
    pub root_path: PathBuf,
    pub nodes: Vec<FileNode>,
    pub cursor: usize,
    pub scroll_offset: usize,
}

impl FileTreePanel {
    pub fn new(root: String) -> Self {
        let root_path = PathBuf::from(&root);
        let nodes = tree::scan_workspace(&root_path).unwrap_or_default();
        Self {
            root_path,
            nodes,
            cursor: 0,
            scroll_offset: 0,
        }
    }

    fn visible_count(&self) -> usize {
        tree::flatten(&self.nodes).len()
    }
}

impl Panel for FileTreePanel {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_color = if focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        let block = Block::default()
            .title(" Files ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner_height = area.height.saturating_sub(2) as usize;
        let flat = tree::flatten(&self.nodes);
        let items = build_list_items(&flat, self.cursor, self.scroll_offset, inner_height);

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<PanelAction> {
        handle_tree_key(self, key)
    }
}

fn handle_tree_key(panel: &mut FileTreePanel, key: KeyEvent) -> Result<PanelAction> {
    let count = panel.visible_count();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if panel.cursor > 0 {
                panel.cursor -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if panel.cursor + 1 < count {
                panel.cursor += 1;
            }
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            return handle_enter_or_expand(panel);
        }
        KeyCode::Left | KeyCode::Char('h') => {
            collapse_current(panel);
        }
        KeyCode::Home => panel.cursor = 0,
        KeyCode::End => {
            panel.cursor = count.saturating_sub(1);
        }
        _ => {}
    }
    preview_current(panel)
}

fn handle_enter_or_expand(panel: &mut FileTreePanel) -> Result<PanelAction> {
    if let Some(node) = tree::node_at_mut(&mut panel.nodes, panel.cursor) {
        match &node.kind {
            NodeKind::Dir { expanded, .. } => {
                let should_expand = !expanded;
                node.set_expanded(should_expand);
                Ok(PanelAction::None)
            }
            NodeKind::File => {
                let path = node.path.to_string_lossy().to_string();
                Ok(PanelAction::OpenFile(path))
            }
        }
    } else {
        Ok(PanelAction::None)
    }
}

fn preview_current(panel: &FileTreePanel) -> Result<PanelAction> {
    let flat = tree::flatten(&panel.nodes);
    if let Some(entry) = flat.get(panel.cursor) {
        if !entry.node.is_dir() {
            let path = entry.node.path.to_string_lossy().to_string();
            return Ok(PanelAction::PreviewFile(path));
        }
    }
    Ok(PanelAction::None)
}

fn collapse_current(panel: &mut FileTreePanel) {
    if let Some(node) = tree::node_at_mut(&mut panel.nodes, panel.cursor) {
        if node.is_dir() {
            node.set_expanded(false);
        }
    }
}

fn build_list_items<'a>(
    flat: &[tree::FlatEntry<'a>],
    cursor: usize,
    scroll_offset: usize,
    height: usize,
) -> Vec<ListItem<'a>> {
    // Adjust scroll to keep cursor visible
    let scroll = adjust_scroll(cursor, scroll_offset, height);

    flat.iter()
        .enumerate()
        .skip(scroll)
        .take(height)
        .map(|(i, entry)| {
            let indent = "  ".repeat(entry.depth);
            let icon = node_icon(entry.node);
            let name = &entry.node.name;
            let style = if i == cursor {
                Style::default().bg(Color::Blue)
            } else if entry.node.is_dir() {
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let line = Line::from(vec![
                Span::raw(indent),
                Span::styled(format!("{icon} {name}"), style),
            ]);
            ListItem::new(line)
        })
        .collect()
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

fn node_icon(node: &FileNode) -> &'static str {
    match &node.kind {
        NodeKind::Dir { expanded: true, .. } => "📂",
        NodeKind::Dir {
            expanded: false, ..
        } => "📁",
        NodeKind::File => file_icon(&node.name),
    }
}

fn file_icon(name: &str) -> &'static str {
    match name.rsplit('.').next() {
        Some("rs") => "🦀",
        Some("toml") => "⚙️",
        Some("md") => "📝",
        Some("json") => "📋",
        Some("yaml" | "yml") => "📋",
        Some("lock") => "🔒",
        Some("sh") => "🐚",
        Some("py") => "🐍",
        Some("ts" | "tsx") => "📘",
        Some("js" | "jsx") => "📒",
        Some("go") => "🔵",
        _ => "📄",
    }
}
