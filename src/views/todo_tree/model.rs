//! Minimal local todo model — mirrors duir_core::model subset.
//! TODO: Replace with duir-core dependency when feature flags are available.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Completion {
    #[default]
    Open,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    #[serde(default = "NodeId::new")]
    pub id: NodeId,
    pub title: String,
    #[serde(default)]
    pub completed: Completion,
    #[serde(default)]
    pub important: bool,
    #[serde(default)]
    pub folded: bool,
    #[serde(default)]
    pub items: Vec<Self>,
}

impl TodoItem {
    pub fn new(title: &str) -> Self {
        Self {
            id: NodeId::new(),
            title: title.to_owned(),
            completed: Completion::default(),
            important: false,
            folded: false,
            items: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoFile {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub items: Vec<TodoItem>,
}

fn default_version() -> String {
    "1.0".to_owned()
}

impl TodoFile {
    pub fn new(title: &str) -> Self {
        Self {
            version: default_version(),
            title: title.to_owned(),
            items: Vec::new(),
        }
    }
}

/// A path addressing a TodoItem by index at each nesting level.
pub type TreePath = Vec<usize>;

pub fn get_item<'a>(file: &'a TodoFile, path: &TreePath) -> Option<&'a TodoItem> {
    let (last, parents) = path.split_last()?;
    let mut items = &file.items;
    for &idx in parents {
        items = &items.get(idx)?.items;
    }
    items.get(*last)
}

pub fn get_item_mut<'a>(file: &'a mut TodoFile, path: &TreePath) -> Option<&'a mut TodoItem> {
    let (last, parents) = path.split_last()?;
    let mut items = &mut file.items;
    for &idx in parents {
        items = &mut items.get_mut(idx)?.items;
    }
    items.get_mut(*last)
}

fn parent_items<'a>(file: &'a mut TodoFile, path: &TreePath) -> Option<(&'a mut Vec<TodoItem>, usize)> {
    let (&last, parents) = path.split_last()?;
    let mut items = &mut file.items;
    for &idx in parents {
        items = &mut items.get_mut(idx)?.items;
    }
    if last > items.len() {
        return None;
    }
    Some((items, last))
}

pub fn add_sibling(file: &mut TodoFile, path: &TreePath, item: TodoItem) -> bool {
    if let Some((items, idx)) = parent_items(file, path) {
        if idx < items.len() {
            items.insert(idx + 1, item);
            return true;
        }
    }
    false
}

pub fn add_child(file: &mut TodoFile, path: &TreePath, item: TodoItem) -> bool {
    if let Some(parent) = get_item_mut(file, path) {
        parent.items.push(item);
        return true;
    }
    false
}

pub fn remove_item(file: &mut TodoFile, path: &TreePath) -> Option<TodoItem> {
    let (items, idx): (&mut Vec<TodoItem>, usize) = parent_items(file, path)?;
    if idx < items.len() {
        Some(items.remove(idx))
    } else {
        None
    }
}

pub fn swap_up(file: &mut TodoFile, path: &TreePath) -> Option<TreePath> {
    let (items, idx): (&mut Vec<TodoItem>, usize) = parent_items(file, path)?;
    if idx == 0 || idx >= items.len() {
        return None;
    }
    items.swap(idx, idx - 1);
    let mut new_path = path.clone();
    *new_path.last_mut()? = idx - 1;
    Some(new_path)
}

pub fn swap_down(file: &mut TodoFile, path: &TreePath) -> Option<TreePath> {
    let (items, idx): (&mut Vec<TodoItem>, usize) = parent_items(file, path)?;
    if idx + 1 >= items.len() {
        return None;
    }
    items.swap(idx, idx + 1);
    let mut new_path = path.clone();
    *new_path.last_mut()? = idx + 1;
    Some(new_path)
}

pub fn promote(file: &mut TodoFile, path: &TreePath) -> Option<TreePath> {
    if path.len() < 2 {
        return None;
    }
    let item = remove_item(file, path)?;
    let parent_path: TreePath = path[..path.len() - 1].to_vec();
    add_sibling(file, &parent_path, item);
    let mut new_path = parent_path;
    *new_path.last_mut()? += 1;
    Some(new_path)
}

pub fn demote(file: &mut TodoFile, path: &TreePath) -> Option<TreePath> {
    let &idx = path.last()?;
    if idx == 0 {
        return None;
    }
    let item = remove_item(file, path)?;
    let mut sibling_path = path.clone();
    *sibling_path.last_mut()? = idx - 1;
    let sibling = get_item_mut(file, &sibling_path)?;
    let child_idx = sibling.items.len();
    sibling.items.push(item);
    let mut new_path = sibling_path;
    new_path.push(child_idx);
    Some(new_path)
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
        return false;
    };
    fs::write(path, content).is_ok()
}
