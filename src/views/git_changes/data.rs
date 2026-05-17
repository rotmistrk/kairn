//! GitChangesData — tree data provider for git changes panel.
//! Groups changed files by status category.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use txv_core::cell::{Color, Style};
use txv_widgets::tree_view::TreeData;

use crate::git_status::{collect_git_status, FileStatus};

/// A node in the git changes tree.
#[derive(Clone)]
struct ChangeNode {
    label: String,
    depth: usize,
    expandable: bool,
    expanded: bool,
    file_path: Option<PathBuf>,
    color: Color,
    status: Option<FileStatus>,
}

/// Data provider for the git changes tree.
pub struct GitChangesData {
    nodes: Vec<ChangeNode>,
    visible: Vec<usize>,
}

impl GitChangesData {
    pub fn new(root: &Path) -> Self {
        let mut data = Self {
            nodes: Vec::new(),
            visible: Vec::new(),
        };
        data.rebuild(root);
        data
    }

    /// Rebuild the tree from current git status.
    pub fn rebuild(&mut self, root: &Path) {
        self.nodes.clear();
        self.visible.clear();

        let roots = discover_git_roots(root);
        let multi_root = roots.len() > 1;

        let mut by_status: HashMap<FileStatus, Vec<(String, PathBuf)>> = HashMap::new();
        for git_root in &roots {
            let statuses = collect_git_status(git_root);
            let root_name = git_root
                .strip_prefix(root)
                .unwrap_or(git_root)
                .to_string_lossy()
                .to_string();
            let root_label = if root_name.is_empty() {
                ".".to_string()
            } else {
                root_name
            };
            for (rel_path, status) in statuses {
                if status == FileStatus::Clean || status == FileStatus::Ignored {
                    continue;
                }
                let abs_path = git_root.join(&rel_path);
                let entry = if multi_root {
                    (format!("{root_label}/{rel_path}"), abs_path)
                } else {
                    (rel_path, abs_path)
                };
                by_status.entry(status).or_default().push(entry);
            }
        }

        let app = crate::app_palette::app_palette();
        let categories = [
            (
                FileStatus::Conflict,
                "Conflicts",
                app.git.conflict.fg.unwrap_or(Color::Ansi(5)),
            ),
            (
                FileStatus::Modified,
                "Modified",
                app.git.modified.fg.unwrap_or(Color::Ansi(12)),
            ),
            (FileStatus::Added, "Added", app.git.added.fg.unwrap_or(Color::Ansi(2))),
            (
                FileStatus::Untracked,
                "Untracked",
                app.git.untracked.fg.unwrap_or(Color::Ansi(1)),
            ),
        ];

        for (status, name, color) in &categories {
            let Some(files) = by_status.get(status) else {
                continue;
            };
            if files.is_empty() {
                continue;
            }
            let label = format!("{name} ({})", files.len());
            self.nodes.push(ChangeNode {
                label,
                depth: 0,
                expandable: true,
                expanded: true,
                file_path: None,
                color: *color,
                status: None,
            });
            let mut sorted = files.clone();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            for (rel, abs) in sorted {
                self.nodes.push(ChangeNode {
                    label: rel,
                    depth: 1,
                    expandable: false,
                    expanded: false,
                    file_path: Some(abs),
                    color: *color,
                    status: Some(*status),
                });
            }
        }
        self.rebuild_visible();
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
        Style {
            fg: color,
            ..Style::default()
        }
    }
}

/// Discover all git roots under the workspace root.
fn discover_git_roots(root: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if root.join(".git").exists() {
        roots.push(root.to_path_buf());
    }
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() && path != root {
                roots.push(path);
            }
        }
    }
    if roots.is_empty() {
        if let Ok(repo) = git2::Repository::discover(root) {
            if let Some(workdir) = repo.workdir() {
                roots.push(workdir.to_path_buf());
            }
        }
    }
    roots
}
