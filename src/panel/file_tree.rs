use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::{Panel, PanelAction};
use crate::tree::{self, FileNode, NodeKind};

/// Git file status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
    Clean,
}

/// Filter mode for the file tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreeFilter {
    #[default]
    All,
    Modified,
    Untracked,
}

impl TreeFilter {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Modified,
            Self::Modified => Self::Untracked,
            Self::Untracked => Self::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Modified => "Modified",
            Self::Untracked => "Untracked",
        }
    }
}

pub struct FileTreePanel {
    pub root_path: PathBuf,
    pub nodes: Vec<FileNode>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub filter: TreeFilter,
    pub git_status: HashMap<String, GitStatus>,
    viewport_height: std::cell::Cell<usize>,
    last_refresh: std::time::Instant,
}

impl FileTreePanel {
    pub fn new(root: String) -> Self {
        let root_path = PathBuf::from(&root);
        let nodes = tree::scan_workspace(&root_path).unwrap_or_default();
        let git_status = collect_git_status(&root_path);
        Self {
            root_path,
            nodes,
            cursor: 0,
            scroll_offset: 0,
            filter: TreeFilter::default(),
            git_status,
            viewport_height: std::cell::Cell::new(0),
            last_refresh: std::time::Instant::now(),
        }
    }

    /// Re-scan the workspace, preserving cursor on the same entry by path.
    pub fn refresh(&mut self) {
        let prev_path = self
            .filtered_flat()
            .get(self.cursor)
            .map(|e| e.node.path.clone());
        let prev_idx = self.cursor;
        let expanded = tree::expanded_paths(&self.nodes);
        self.nodes = tree::scan_workspace(&self.root_path).unwrap_or_default();
        tree::restore_expanded(&mut self.nodes, &expanded);
        self.git_status = collect_git_status(&self.root_path);
        let count = self.visible_count();
        if let Some(ref target) = prev_path {
            self.cursor = self
                .filtered_flat()
                .iter()
                .position(|e| e.node.path == *target)
                .unwrap_or(prev_idx.min(count.saturating_sub(1)));
        }
        if self.cursor >= count {
            self.cursor = count.saturating_sub(1);
        }
        self.last_refresh = std::time::Instant::now();
    }

    /// Auto-refresh if enough time has elapsed. Returns true if refreshed.
    pub fn maybe_refresh(&mut self, interval: std::time::Duration) -> bool {
        if self.last_refresh.elapsed() >= interval {
            self.refresh();
            return true;
        }
        false
    }

    fn visible_count(&self) -> usize {
        self.filtered_flat().len()
    }

    fn filtered_flat(&self) -> Vec<tree::FlatEntry<'_>> {
        let flat = tree::flatten(&self.nodes);
        match self.filter {
            TreeFilter::All => flat,
            _ => {
                let matching = self.matching_paths();
                if matching.is_empty() {
                    return Vec::new();
                }
                let prefix = |dir: &str, path: &str| -> bool {
                    path.starts_with(dir)
                        && path.as_bytes().get(dir.len()) == Some(&b'/')
                };
                flat.into_iter()
                    .filter(|e| {
                        let rel = self.rel_path(e.node);
                        if e.node.is_dir() {
                            matching.iter().any(|m| prefix(&rel, m))
                        } else {
                            matching.contains(&rel)
                        }
                    })
                    .collect()
            }
        }
    }

    /// Collect relative paths of files matching the current filter.
    fn matching_paths(&self) -> std::collections::HashSet<String> {
        self.git_status
            .iter()
            .filter(|(_, status)| match self.filter {
                TreeFilter::Modified => {
                    matches!(status, GitStatus::Modified | GitStatus::Deleted)
                }
                TreeFilter::Untracked => {
                    matches!(status, GitStatus::Untracked | GitStatus::Added)
                }
                TreeFilter::All => true,
            })
            .map(|(path, _)| path.clone())
            .collect()
    }

    fn rel_path(&self, node: &FileNode) -> String {
        node.path
            .strip_prefix(&self.root_path)
            .unwrap_or(&node.path)
            .to_string_lossy()
            .to_string()
    }
}

impl Panel for FileTreePanel {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_color = if focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        let title = format!(" Files [{}] ", self.filter.label());
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner_height = area.height.saturating_sub(2) as usize;
        self.viewport_height.set(inner_height);
        let flat = self.filtered_flat();
        if flat.is_empty() && self.filter != TreeFilter::All {
            let msg = Paragraph::new("  (no matching files)")
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            frame.render_widget(msg, area);
            return;
        }
        let items = build_list_items(
            &flat,
            self.cursor,
            self.scroll_offset,
            inner_height,
            &self.git_status,
            &self.root_path,
        );

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
        KeyCode::Enter | KeyCode::Char('l') => {
            return handle_enter_or_expand(panel);
        }
        KeyCode::Right => return handle_right(panel),
        KeyCode::Left | KeyCode::Char('h') => {
            collapse_current(panel);
        }
        KeyCode::Home => panel.cursor = 0,
        KeyCode::End => {
            panel.cursor = count.saturating_sub(1);
        }
        _ => {}
    }
    panel.scroll_offset = super::adjust_scroll(
        panel.cursor,
        panel.scroll_offset,
        panel.viewport_height.get(),
    );
    preview_current(panel)
}

fn handle_right(panel: &mut FileTreePanel) -> Result<PanelAction> {
    let flat = panel.filtered_flat();
    if let Some(entry) = flat.get(panel.cursor) {
        if entry.node.is_dir() {
            return handle_enter_or_expand(panel);
        }
    }
    Ok(PanelAction::FocusRight)
}

fn handle_enter_or_expand(panel: &mut FileTreePanel) -> Result<PanelAction> {
    // Resolve cursor through filtered view to get the actual node path.
    let target = {
        let flat = panel.filtered_flat();
        flat.get(panel.cursor).map(|e| e.node.path.clone())
    };
    let target = match target {
        Some(p) => p,
        None => return Ok(PanelAction::None),
    };
    if let Some(node) = tree::node_by_path_mut(&mut panel.nodes, &target) {
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
    let flat = panel.filtered_flat();
    if let Some(entry) = flat.get(panel.cursor) {
        if !entry.node.is_dir() {
            let path = entry.node.path.to_string_lossy().to_string();
            return Ok(PanelAction::PreviewFile(path));
        }
    }
    Ok(PanelAction::None)
}

fn collapse_current(panel: &mut FileTreePanel) {
    let flat = panel.filtered_flat();
    let entry = match flat.get(panel.cursor) {
        Some(e) => e,
        None => return,
    };
    let is_expanded_dir = matches!(
        entry.node.kind,
        tree::NodeKind::Dir { expanded: true, .. }
    );
    if is_expanded_dir {
        let target = entry.node.path.clone();
        drop(flat);
        if let Some(node) = tree::node_by_path_mut(&mut panel.nodes, &target) {
            node.set_expanded(false);
        }
        return;
    }
    // Leaf or collapsed dir — jump cursor to parent directory.
    let parent_idx = find_parent_index(&flat, panel.cursor);
    drop(flat);
    if let Some(idx) = parent_idx {
        panel.cursor = idx;
    }
}

fn find_parent_index(flat: &[tree::FlatEntry<'_>], cursor: usize) -> Option<usize> {
    let depth = flat.get(cursor)?.depth;
    if depth == 0 {
        return None;
    }
    (0..cursor)
        .rev()
        .find(|&i| flat[i].depth < depth && flat[i].node.is_dir())
}

fn build_list_items<'a>(
    flat: &[tree::FlatEntry<'a>],
    cursor: usize,
    scroll_offset: usize,
    height: usize,
    git_status: &HashMap<String, GitStatus>,
    root: &std::path::Path,
) -> Vec<ListItem<'a>> {
    let scroll = super::adjust_scroll(cursor, scroll_offset, height);

    flat.iter()
        .enumerate()
        .skip(scroll)
        .take(height)
        .map(|(i, entry)| {
            let indent = "  ".repeat(entry.depth);
            let icon = node_icon(entry.node);
            let name = &entry.node.name;
            let rel = entry
                .node
                .path
                .strip_prefix(root)
                .unwrap_or(&entry.node.path)
                .to_string_lossy();
            let style = entry_style(i == cursor, entry.node, git_status.get(rel.as_ref()));
            ListItem::new(Line::from(vec![
                Span::raw(indent),
                Span::styled(format!("{icon} {name}"), style),
            ]))
        })
        .collect()
}

fn entry_style(is_cursor: bool, node: &FileNode, status: Option<&GitStatus>) -> Style {
    let status_color = match status {
        Some(GitStatus::Modified) => Some(Color::Yellow),
        Some(GitStatus::Added) => Some(Color::Green),
        Some(GitStatus::Untracked) => Some(Color::LightGreen),
        Some(GitStatus::Deleted) => Some(Color::Red),
        _ => None,
    };
    if is_cursor {
        let mut s = Style::default().bg(Color::Blue);
        if let Some(c) = status_color {
            s = s.fg(c);
        }
        s
    } else if let Some(c) = status_color {
        Style::default().fg(c)
    } else if node.is_dir() {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    }
}

/// Collect git status by running `git status --porcelain`.
pub fn collect_git_status(root: &std::path::Path) -> HashMap<String, GitStatus> {
    let mut map = HashMap::new();
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain", "-uall"])
        .current_dir(root)
        .output();
    let output = match output {
        Ok(o) => o,
        Err(_) => return map,
    };
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.len() < 4 {
            continue;
        }
        let xy = &line[..2];
        let path = line[3..].trim().to_string();
        let status = match xy {
            "??" => GitStatus::Untracked,
            " M" | "MM" | "AM" => GitStatus::Modified,
            "M " | "A " => GitStatus::Added,
            " D" | "D " => GitStatus::Deleted,
            _ => GitStatus::Modified,
        };
        map.insert(path, status);
    }
    map
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
