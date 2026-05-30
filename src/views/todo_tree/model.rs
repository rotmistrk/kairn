//! Re-export duir-core model + tree_ops for kairn's todo tree.
//! Thin wrappers adapt Result → Option where kairn expects Option.

use std::fs;
use std::path::Path;

pub use duir_core::model::{Completion, TodoFile, TodoItem, WorkStatus};
use duir_core::stats::update_completion;
pub use duir_core::tree_ops::TreePath;
pub use duir_core::{tree_ops, NodeId};

pub fn get_item<'a>(file: &'a TodoFile, path: &TreePath) -> Option<&'a TodoItem> {
    tree_ops::get_item(file, path)
}

pub fn get_item_mut<'a>(file: &'a mut TodoFile, path: &TreePath) -> Option<&'a mut TodoItem> {
    tree_ops::get_item_mut(file, path)
}

pub fn add_sibling(file: &mut TodoFile, path: &TreePath, item: TodoItem) -> bool {
    if let Err(e) = tree_ops::add_sibling(file, path, item) {
        log::warn!("todo add_sibling: {e}");
        return false;
    }
    true
}

pub fn add_child(file: &mut TodoFile, path: &TreePath, item: TodoItem) -> bool {
    if let Err(e) = tree_ops::add_child(file, path, item) {
        log::warn!("todo add_child: {e}");
        return false;
    }
    true
}

pub fn remove_item(file: &mut TodoFile, path: &TreePath) -> Option<TodoItem> {
    tree_ops::remove_item(file, path)
        .map_err(|e| log::warn!("todo remove: {e}"))
        .ok()
}

pub fn swap_up(file: &mut TodoFile, path: &TreePath) -> Option<TreePath> {
    tree_ops::swap_up(file, path)
        .map_err(|e| log::warn!("todo swap_up: {e}"))
        .ok()
}

pub fn swap_down(file: &mut TodoFile, path: &TreePath) -> Option<TreePath> {
    tree_ops::swap_down(file, path)
        .map_err(|e| log::warn!("todo swap_down: {e}"))
        .ok()
}

pub fn promote(file: &mut TodoFile, path: &TreePath) -> Option<TreePath> {
    tree_ops::promote(file, path)
        .map_err(|e| log::warn!("todo promote: {e}"))
        .ok()
}

pub fn demote(file: &mut TodoFile, path: &TreePath) -> Option<TreePath> {
    tree_ops::demote(file, path)
        .map_err(|e| log::warn!("todo demote: {e}"))
        .ok()
}

/// Update completion state for all root items using duir-core's recursive logic.
/// This propagates completion both up and down the tree correctly.
pub fn propagate_completion(file: &mut TodoFile, _path: &TreePath) {
    for item in &mut file.items {
        update_completion(item);
    }
}

pub fn sort_children(file: &mut TodoFile, path: &TreePath) -> bool {
    if let Err(e) = tree_ops::sort_children(file, path) {
        log::warn!("todo sort_children: {e}");
        return false;
    }
    true
}

pub fn clone_subtree(file: &mut TodoFile, path: &TreePath) -> bool {
    if let Err(e) = tree_ops::clone_subtree(file, path) {
        log::warn!("todo clone_subtree: {e}");
        return false;
    }
    true
}

/// Load a TodoFile from path, creating empty if absent.
pub fn load_todo_file(path: &Path) -> TodoFile {
    match fs::read_to_string(path) {
        Ok(content) if !content.trim().is_empty() => {
            serde_json::from_str(&content).unwrap_or_else(|_| TodoFile::new("Todo"))
        }
        _ => TodoFile::new("Todo"),
    }
}

/// Save a TodoFile to path.
pub fn save_todo_file(path: &Path, file: &TodoFile) -> bool {
    let Ok(content) = serde_json::to_string_pretty(file) else {
        log::error!("todo: failed to serialize TodoFile");
        return false;
    };
    if let Err(e) = fs::write(path, content) {
        log::error!("todo: failed to save {}: {e}", path.display());
        return false;
    }
    true
}
