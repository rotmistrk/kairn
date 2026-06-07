//! GitChangesData — tree data provider for git changes panel.
//! Groups changed files by status category, optionally per-root.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use txv_core::cell::{Color, Style};
use txv_widgets::tree_view::TreeData;

use crate::app_palette::app_palette;
use crate::git_status::FileStatus;

use super::builders::{build_category_nodes, collect_statuses_flat, discover_git_roots};
use super::change_node::ChangeNode;

/// Data provider for the git changes tree.
pub struct GitChangesData {
    nodes: Vec<ChangeNode>,
    visible: Vec<usize>,
    /// Badge colors for root header nodes (only in multi-root mode).
    root_badge_colors: Vec<Color>,
    /// Disambiguated display labels for root headers.
    root_labels: Vec<String>,
}

impl GitChangesData {
    pub fn new(root: &Path) -> Self {
        let mut data = Self {
            nodes: Vec::new(),
            visible: Vec::new(),
            root_badge_colors: Vec::new(),
            root_labels: Vec::new(),
        };
        data.rebuild(root);
        data
    }

    /// Rebuild the tree from current git status.
    pub fn rebuild(&mut self, root: &Path) {
        self.rebuild_roots(&[root.to_path_buf()]);
    }

    /// Set badge colors for root headers (one per root, in order).
    pub fn set_root_badge_colors(&mut self, colors: Vec<Color>) {
        self.root_badge_colors = colors;
    }

    /// Set disambiguated display labels for root headers.
    pub fn set_root_labels(&mut self, labels: Vec<String>) {
        self.root_labels = labels;
    }

    /// Rebuild from multiple workspace roots.
    pub fn rebuild_roots(&mut self, roots: &[PathBuf]) {
        // Preserve collapsed state using stable identity keys
        let collapsed: HashSet<String> = self
            .nodes
            .iter()
            .filter(|n| n.expandable && !n.expanded)
            .filter_map(|n| n.key.clone())
            .collect();

        self.nodes.clear();
        self.visible.clear();

        if roots.len() > 1 {
            self.rebuild_multi_root(roots);
        } else {
            self.rebuild_single_root(roots);
        }

        // Restore collapsed state
        for node in &mut self.nodes {
            if node.expandable && node.key.as_ref().is_some_and(|k| collapsed.contains(k)) {
                node.expanded = false;
            }
        }
        self.rebuild_visible();
    }

    fn rebuild_single_root(&mut self, roots: &[PathBuf]) {
        let git_roots: Vec<PathBuf> = roots.iter().flat_map(|r| discover_git_roots(r)).collect();
        let parent = roots.first().map(|r| r.as_path()).unwrap_or(Path::new("."));
        let by_status = collect_statuses_flat(parent, &git_roots);
        let root = roots.first().cloned().unwrap_or_default();
        build_category_nodes(&mut self.nodes, &by_status, 0, 1, &root);
    }

    fn rebuild_multi_root(&mut self, roots: &[PathBuf]) {
        let app = app_palette();
        for (root_idx, root) in roots.iter().enumerate() {
            let git_roots = discover_git_roots(root);
            if git_roots.is_empty() {
                continue;
            }
            let by_status = collect_statuses_flat(root, &git_roots);
            if by_status.is_empty() {
                continue;
            }
            let root_name = self.root_labels.get(root_idx).cloned().unwrap_or_else(|| {
                root.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| root.to_string_lossy().to_string())
            });
            let color = self
                .root_badge_colors
                .get(root_idx)
                .copied()
                .unwrap_or(app.git().modified().fg());
            // Root header node
            self.nodes.push(ChangeNode {
                label: root_name,
                depth: 0,
                expandable: true,
                expanded: true,
                file_path: None,
                color,
                status: None,
                key: Some(root.to_string_lossy().to_string()),
            });
            build_category_nodes(&mut self.nodes, &by_status, 1, 2, root);
        }
    }

    fn rebuild_visible(&mut self) {
        self.visible.clear();
        let mut skip_depth: Option<usize> = None;
        for (i, node) in self.nodes.iter().enumerate() {
            if let Some(sd) = skip_depth {
                if node.depth > sd {
                    continue;
                }
                skip_depth = None;
            }
            self.visible.push(i);
            if node.expandable && !node.expanded {
                skip_depth = Some(node.depth);
            }
        }
    }

    /// Get the file path for a node (if it's a leaf).
    pub fn file_path(&self, id: usize) -> Option<&Path> {
        self.nodes.get(id).and_then(|n| n.file_path.as_deref())
    }

    /// Check if a node is an untracked file.
    pub fn is_untracked(&self, id: usize) -> bool {
        self.nodes
            .get(id)
            .is_some_and(|n| n.status == Some(FileStatus::Untracked))
    }
}

impl TreeData for GitChangesData {
    fn root_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.depth == 0).count()
    }
    fn child_count(&self, _id: usize) -> usize {
        0
    }
    fn label(&self, id: usize) -> &str {
        self.nodes.get(id).map(|n| n.label.as_str()).unwrap_or("")
    }
    fn is_expandable(&self, id: usize) -> bool {
        self.nodes.get(id).is_some_and(|n| n.expandable)
    }
    fn is_expanded(&self, id: usize) -> bool {
        self.nodes.get(id).is_some_and(|n| n.expanded)
    }
    fn toggle(&mut self, id: usize) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.expanded = !node.expanded;
        }
        self.rebuild_visible();
    }
    fn depth(&self, id: usize) -> usize {
        self.nodes.get(id).map(|n| n.depth).unwrap_or(0)
    }
    fn visible_count(&self) -> usize {
        self.visible.len()
    }
    fn visible_id(&self, row: usize) -> usize {
        self.visible.get(row).copied().unwrap_or(0)
    }
    fn style(&self, id: usize) -> Style {
        let color = self.nodes.get(id).map(|n| n.color).unwrap_or(Color::Ansi(7));
        Style::new(color, Color::Reset)
    }
    fn badge_color(&self, id: usize) -> Option<Color> {
        if self.root_badge_colors.is_empty() {
            return None;
        }
        let node = self.nodes.get(id)?;
        // Only root headers (depth 0, expandable, no file_path) get badges
        if node.depth != 0 || node.file_path.is_some() {
            return None;
        }
        Some(node.color)
    }
}
