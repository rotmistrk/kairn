//! TodoTreeData — tree data provider backed by local TodoFile model.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use txv_core::cell::{Color, Style};
use txv_widgets::tree_view::TreeData;

use super::model::{self, Completion, TodoFile, TodoItem, TreePath};

/// Flattened node for display in the tree.
struct FlatNode {
    depth: usize,
    path: TreePath,
    expandable: bool,
    expanded: bool,
}

/// Data provider for the todo tree.
pub struct TodoTreeData {
    pub file: TodoFile,
    file_path: PathBuf,
    nodes: Vec<FlatNode>,
    visible: Vec<usize>,
    pub filter_text: String,
    /// Last known mtime of the file on disk.
    last_mtime: Option<SystemTime>,
}

impl TodoTreeData {
    pub fn new(file_path: &Path) -> Self {
        let file = model::load_todo_file(file_path);
        let mtime = Self::read_mtime(file_path);
        let mut data = Self {
            file,
            file_path: file_path.to_path_buf(),
            nodes: Vec::new(),
            visible: Vec::new(),
            filter_text: String::new(),
            last_mtime: mtime,
        };
        data.rebuild_flat();
        data
    }

    /// Save to disk. Reloads first if file was modified externally.
    /// Returns true if saved, false if only reloaded (caller should rebuild).
    pub fn save(&mut self) -> bool {
        self.reload_if_changed();
        if model::save_todo_file(&self.file_path, &self.file) {
            self.last_mtime = Self::read_mtime(&self.file_path);
            true
        } else {
            false
        }
    }

    /// Check if the file on disk has changed since we last read/wrote it.
    /// If so, reload from disk and rebuild. Returns true if reloaded.
    pub fn reload_if_changed(&mut self) -> bool {
        let current_mtime = Self::read_mtime(&self.file_path);
        if current_mtime != self.last_mtime {
            self.file = model::load_todo_file(&self.file_path);
            self.last_mtime = current_mtime;
            self.rebuild_flat();
            true
        } else {
            false
        }
    }

    fn read_mtime(path: &Path) -> Option<SystemTime> {
        std::fs::metadata(path).ok().and_then(|m| m.modified().ok())
    }

    /// Rebuild the flat node list from the TodoFile tree.
    pub fn rebuild_flat(&mut self) {
        self.nodes.clear();
        self.visible.clear();
        self.flatten_items(&self.file.items.clone(), &[], 0);
        self.rebuild_visible();
    }

    fn flatten_items(&mut self, items: &[TodoItem], parent_path: &[usize], depth: usize) {
        for (i, item) in items.iter().enumerate() {
            let mut path = parent_path.to_vec();
            path.push(i);
            let expandable = !item.items.is_empty();
            let expanded = expandable && !item.folded;
            self.nodes.push(FlatNode {
                depth,
                path,
                expandable,
                expanded,
            });
            if expanded {
                let p: Vec<usize> = parent_path.iter().copied().chain(std::iter::once(i)).collect();
                self.flatten_items(&item.items, &p, depth + 1);
            }
        }
    }

    fn rebuild_visible(&mut self) {
        self.visible.clear();
        if self.filter_text.is_empty() {
            for i in 0..self.nodes.len() {
                self.visible.push(i);
            }
        } else {
            let query = self.filter_text.to_lowercase();
            for i in 0..self.nodes.len() {
                if let Some(item) = self.item_at_node(i) {
                    if item.title.to_lowercase().contains(&query) {
                        self.visible.push(i);
                    }
                }
            }
        }
    }

    /// Get item for a node index (not visible row).
    fn item_at_node(&self, node_idx: usize) -> Option<&TodoItem> {
        let node = self.nodes.get(node_idx)?;
        model::get_item(&self.file, &node.path)
    }

    /// Get the TodoItem for a given flat node id.
    pub fn item_at(&self, id: usize) -> Option<&TodoItem> {
        let node = self.nodes.get(id)?;
        model::get_item(&self.file, &node.path)
    }

    /// Get the TreePath for a given flat node id.
    pub fn path_at(&self, id: usize) -> Option<&TreePath> {
        self.nodes.get(id).map(|n| &n.path)
    }

    /// Return the visible row for a given TreePath, if any.
    pub fn row_for_path(&self, path: &TreePath) -> Option<usize> {
        self.visible
            .iter()
            .position(|&nid| self.nodes.get(nid).is_some_and(|n| n.path == *path))
    }

    /// Check if the node at `id` or any of its ancestors has `important = true`.
    pub fn is_in_important_subtree(&self, id: usize) -> bool {
        let Some(node) = self.nodes.get(id) else {
            return false;
        };
        // Check self
        if let Some(item) = model::get_item(&self.file, &node.path) {
            if item.important {
                return true;
            }
        }
        // Check ancestors
        let path = &node.path;
        for len in 1..path.len() {
            let ancestor_path: Vec<usize> = path[..len].to_vec();
            if let Some(item) = model::get_item(&self.file, &ancestor_path) {
                if item.important {
                    return true;
                }
            }
        }
        false
    }

    /// Add the first item to an empty tree. Creates the file if needed.
    pub fn add_first_item(&mut self) {
        let item = model::TodoItem::new("<new task>");
        self.file.items.push(item);
        self.save();
        self.rebuild_flat();
    }

    /// Update the title of the item at the given visible row.
    pub fn update_title(&mut self, row: usize, title: String) {
        let id = self.visible_id(row);
        if let Some(node) = self.nodes.get(id) {
            let path = node.path.clone();
            if let Some(item) = model::get_item_mut(&mut self.file, &path) {
                item.title = title;
            }
        }
        self.save();
        self.rebuild_flat();
    }
}

impl TreeData for TodoTreeData {
    fn root_count(&self) -> usize {
        self.file.items.len()
    }

    fn child_count(&self, _id: usize) -> usize {
        0
    }

    fn label(&self, id: usize) -> &str {
        self.item_at(id).map(|i| i.title.as_str()).unwrap_or("")
    }

    fn is_expandable(&self, id: usize) -> bool {
        self.nodes.get(id).is_some_and(|n| n.expandable)
    }

    fn is_expanded(&self, id: usize) -> bool {
        self.nodes.get(id).is_some_and(|n| n.expanded)
    }

    fn toggle(&mut self, id: usize) {
        if let Some(node) = self.nodes.get(id) {
            let path = node.path.clone();
            if let Some(item) = model::get_item_mut(&mut self.file, &path) {
                item.folded = !item.folded;
            }
        }
        self.rebuild_flat();
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
        let Some(item) = self.item_at(id) else {
            return Style::default();
        };
        let app = crate::app_palette::app_palette();
        let fg = match (&item.completed, item.important) {
            (Completion::Done, _) => app.todo.done.fg.unwrap_or(Color::Ansi(8)),
            (_, true) => app.todo.important.fg.unwrap_or(Color::Ansi(1)),
            _ => app.todo.normal.fg.unwrap_or(Color::Ansi(7)),
        };
        Style { fg, ..Style::default() }
    }
}
