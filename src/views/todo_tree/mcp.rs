//! MCP action handler for TodoTreeView.

use serde_json::json;

use super::model::{self, Completion, TodoItem};
use super::TodoTreeView;
use crate::mcp::commands::McpAction;

impl TodoTreeView {
    /// Execute an MCP action on the todo tree. Returns JSON result.
    pub fn mcp_action(&mut self, action: &McpAction) -> Result<serde_json::Value, String> {
        self.dispatch_mcp_action(action)?;
        self.inner.data_mut().save();
        self.inner.data_mut().rebuild_flat();
        self.inner.state_mut().mark_dirty();
        Ok(json!({"ok": true}))
    }

    fn dispatch_mcp_action(&mut self, action: &McpAction) -> Result<(), String> {
        match action {
            McpAction::TodoToggle { path } => {
                let item = model::get_item_mut(&mut self.inner.data_mut().file, path).ok_or("Item not found")?;
                item.completed = match item.completed {
                    Completion::Done => Completion::Open,
                    _ => Completion::Done,
                };
            }
            McpAction::TodoAdd { path, title } => {
                let item = TodoItem::new(title);
                if !model::add_sibling(&mut self.inner.data_mut().file, path, item) {
                    return Err("Failed to add item".to_string());
                }
            }
            McpAction::TodoRemove { path } => {
                model::remove_item(&mut self.inner.data_mut().file, path).ok_or("Item not found")?;
            }
            McpAction::TodoMoveUp { path } => {
                model::swap_up(&mut self.inner.data_mut().file, path).ok_or("Cannot move up")?;
            }
            McpAction::TodoMoveDown { path } => {
                model::swap_down(&mut self.inner.data_mut().file, path).ok_or("Cannot move down")?;
            }
            McpAction::TodoPromote { path } => {
                model::promote(&mut self.inner.data_mut().file, path).ok_or("Cannot promote")?;
            }
            McpAction::TodoDemote { path } => {
                model::demote(&mut self.inner.data_mut().file, path).ok_or("Cannot demote")?;
            }
            McpAction::TodoSetLoe { path, effort } => {
                let item = model::get_item_mut(&mut self.inner.data_mut().file, path).ok_or("Item not found")?;
                item.effort = if *effort == 0 {
                    None
                } else {
                    Some(*effort)
                };
            }
            _ => self.dispatch_mcp_edit_action(action)?,
        }
        Ok(())
    }

    fn dispatch_mcp_edit_action(&mut self, action: &McpAction) -> Result<(), String> {
        match action {
            McpAction::TodoSetNote { path, note } => {
                let item = model::get_item_mut(&mut self.inner.data_mut().file, path).ok_or("Item not found")?;
                item.note.clone_from(note);
            }
            McpAction::TodoToggleImportant { path } => {
                let item = model::get_item_mut(&mut self.inner.data_mut().file, path).ok_or("Item not found")?;
                item.important = !item.important;
            }
            McpAction::TodoSetPriority { path, priority } => {
                let item = model::get_item_mut(&mut self.inner.data_mut().file, path).ok_or("Item not found")?;
                item.priority = if *priority == 0 {
                    None
                } else {
                    Some(*priority)
                };
            }
            McpAction::TodoSetCompleted { path, state } => {
                let item = model::get_item_mut(&mut self.inner.data_mut().file, path).ok_or("Item not found")?;
                item.completed = match state.as_str() {
                    "done" => Completion::Done,
                    "partial" => Completion::Partial,
                    _ => Completion::Open,
                };
            }

            McpAction::TodoEdit { path, title } => {
                let item = model::get_item_mut(&mut self.inner.data_mut().file, path).ok_or("Item not found")?;
                item.title.clone_from(title);
            }
            McpAction::TodoAddSubtree { path, items } => {
                self.mcp_add_subtree(path, items)?;
            }
            _ => return Err("Not a todo action".to_string()),
        }
        Ok(())
    }

    fn mcp_add_subtree(&mut self, path: &[usize], items: &[serde_json::Value]) -> Result<(), String> {
        fn build_item(val: &serde_json::Value) -> Option<model::TodoItem> {
            let title = val.get("title")?.as_str()?;
            let mut item = TodoItem::new(title);
            if let Some(children) = val.get("items").and_then(|v| v.as_array()) {
                for child_val in children {
                    if let Some(child) = build_item(child_val) {
                        item.items.push(child);
                    }
                }
            }
            Some(item)
        }
        let path_vec: Vec<usize> = path.to_vec();
        for item_val in items {
            let item = build_item(item_val).ok_or("Invalid item in subtree")?;
            if path_vec.is_empty() {
                self.inner.data_mut().file.items.push(item);
            } else if !model::add_child(&mut self.inner.data_mut().file, &path_vec, item) {
                return Err("Failed to add subtree item".to_string());
            }
        }
        Ok(())
    }
}
