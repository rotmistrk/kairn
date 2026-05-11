//! TodoTreeData — tree data provider backed by local TodoFile model.

use std::path::{Path, PathBuf};

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
}

impl TodoTreeData {
    pub fn new(file_path: &Path) -> Self {
        let file = model::load_todo_file(file_path);
        let mut data = Self {
            file,
            file_path: file_path.to_path_buf(),
            nodes: Vec::new(),
            visible: Vec::new(),
        };
        data.rebuild_flat();
        data
    }

    pub fn save(&self) {
        model::save_todo_file(&self.file_path, &self.file);
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
        for i in 0..self.nodes.len() {
            self.visible.push(i);
        }
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
        let fg = match (&item.completed, item.important) {
            (Completion::Done, _) => Color::Ansi(8), // dim for done
            (_, true) => Color::Ansi(1),             // red for important
            _ => Color::Ansi(7),                     // default
        };
        Style { fg, ..Style::default() }
    }
}
