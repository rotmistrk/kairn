//! FileTreeData — TreeData implementation for filesystem navigation.
//! Uses the `ignore` crate to respect .gitignore rules.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use txv_core::cell::{Color, Style};

use crate::tree_view::TreeData;

#[derive(Clone)]
struct TreeNode {
    path: PathBuf,
    label: String,
    depth: usize,
    is_dir: bool,
    expanded: bool,
    parent: Option<usize>,
}

/// Filesystem tree data provider.
pub struct FileTreeData {
    root: PathBuf,
    nodes: Vec<TreeNode>,
    visible: Vec<usize>,
    /// Per-path foreground color (relative path → color).
    colors: HashMap<String, Color>,
}

impl FileTreeData {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let mut data = Self {
            root: root.clone(),
            nodes: Vec::new(),
            visible: Vec::new(),
            colors: HashMap::new(),
        };
        data.load_children(root, None, 0);
        data.rebuild_visible();
        data
    }

    pub fn path(&self, id: usize) -> &Path {
        &self.nodes[id].path
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Set per-path foreground colors (relative path → color).
    pub fn set_colors(&mut self, colors: HashMap<String, Color>) {
        self.colors = colors;
    }

    /// Rebuild the tree from disk, preserving expanded directories.
    pub fn refresh(&mut self) {
        // Collect expanded paths before clearing
        let expanded_paths: Vec<PathBuf> = self
            .nodes
            .iter()
            .filter(|n| n.is_dir && n.expanded)
            .map(|n| n.path.clone())
            .collect();

        let root = self.root.clone();
        self.nodes.clear();
        self.visible.clear();
        self.load_children(root, None, 0);

        // Re-expand previously expanded directories
        for path in &expanded_paths {
            if let Some(idx) = self.nodes.iter().position(|n| n.path == *path) {
                if self.nodes[idx].is_dir && !self.nodes[idx].expanded {
                    self.nodes[idx].expanded = true;
                    let depth = self.nodes[idx].depth;
                    self.load_children(path.clone(), Some(idx), depth + 1);
                }
            }
        }

        self.rebuild_visible();
    }

    fn load_children(&mut self, dir: PathBuf, parent: Option<usize>, depth: usize) {
        let walker = WalkBuilder::new(&dir)
            .max_depth(Some(1))
            .sort_by_file_name(|a, b| a.cmp(b))
            .build();

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in walker.flatten() {
            let path = entry.path().to_path_buf();
            if path == dir {
                continue;
            }
            let label = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let is_dir = path.is_dir();
            let node = TreeNode {
                path,
                label,
                depth,
                is_dir,
                expanded: false,
                parent,
            };
            if is_dir {
                dirs.push(node);
            } else {
                files.push(node);
            }
        }

        // Dirs first, then files
        self.nodes.extend(dirs);
        self.nodes.extend(files);
    }

    fn rebuild_visible(&mut self) {
        self.visible.clear();
        self.collect_visible(None, 0);
    }

    fn collect_visible(&mut self, parent: Option<usize>, depth: usize) {
        let ids: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.parent == parent && n.depth == depth)
            .map(|(i, _)| i)
            .collect();
        for id in ids {
            self.visible.push(id);
            if self.nodes[id].is_dir && self.nodes[id].expanded {
                self.collect_visible(Some(id), depth + 1);
            }
        }
    }

    fn expand_node(&mut self, id: usize) {
        if !self.nodes[id].is_dir || self.nodes[id].expanded {
            return;
        }
        self.nodes[id].expanded = true;
        let path = self.nodes[id].path.clone();
        let depth = self.nodes[id].depth + 1;
        // Only load if not already loaded
        let has_children = self.nodes.iter().any(|n| n.parent == Some(id));
        if !has_children {
            self.load_children(path, Some(id), depth);
        }
        self.rebuild_visible();
    }

    fn collapse_node(&mut self, id: usize) {
        if !self.nodes[id].expanded {
            return;
        }
        self.nodes[id].expanded = false;
        self.rebuild_visible();
    }
}

impl TreeData for FileTreeData {
    fn root_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.parent.is_none()).count()
    }

    fn child_count(&self, id: usize) -> usize {
        self.nodes.iter().filter(|n| n.parent == Some(id)).count()
    }

    fn label(&self, id: usize) -> &str {
        &self.nodes[id].label
    }

    fn is_expandable(&self, id: usize) -> bool {
        self.nodes[id].is_dir
    }

    fn is_expanded(&self, id: usize) -> bool {
        self.nodes[id].expanded
    }

    fn toggle(&mut self, id: usize) {
        if self.nodes[id].expanded {
            self.collapse_node(id);
        } else {
            self.expand_node(id);
        }
    }

    fn depth(&self, id: usize) -> usize {
        self.nodes[id].depth
    }

    fn visible_count(&self) -> usize {
        self.visible.len()
    }

    fn visible_id(&self, row: usize) -> usize {
        self.visible[row]
    }

    fn style(&self, id: usize) -> Style {
        let node = &self.nodes[id];
        if node.is_dir {
            return Style {
                fg: Color::Ansi(14),
                ..Style::default()
            };
        }
        let rel = node.path.strip_prefix(&self.root).ok().and_then(|p| p.to_str());
        if let Some(rel_path) = rel {
            if let Some(&color) = self.colors.get(rel_path) {
                return Style {
                    fg: color,
                    ..Style::default()
                };
            }
        }
        Style::default()
    }
}
