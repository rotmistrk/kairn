//! MCP action handler for TodoTreeView.

use serde_json::json;

use super::model::{self, Completion, TodoItem, TreePath};
use super::TodoTreeView;
use crate::mcp::commands::McpAction;

impl TodoTreeView {
    /// Execute an MCP action on the todo tree. Returns JSON result.
    pub fn mcp_action(&mut self, action: &McpAction) -> Result<serde_json::Value, String> {
        self.dispatch_mcp_action(action)?;
        // Propagate completion state after any mutation.
        model::reconcile_all(self.inner_mut().data_mut().file_mut());
        self.inner_mut().data_mut().save();
        self.inner_mut().data_mut().rebuild_flat();
        self.inner_mut().state_mut().mark_dirty();
        Ok(json!({"ok": true}))
    }

    fn dispatch_mcp_action(&mut self, action: &McpAction) -> Result<(), String> {
        match action {
            McpAction::TodoToggle { path } => self.mcp_toggle(path),
            McpAction::TodoAdd { path, title } => self.mcp_add_sibling(path, title),
            McpAction::TodoRemove { path } => self.mcp_remove(path),
            McpAction::TodoMoveUp { path } => self.mcp_move_up(path),
            McpAction::TodoMoveDown { path } => self.mcp_move_down(path),
            McpAction::TodoPromote { path } => self.mcp_promote(path),
            McpAction::TodoDemote { path } => self.mcp_demote(path),
            McpAction::TodoSetLoe { path, effort } => self.mcp_set_loe(path, *effort),
            _ => self.dispatch_mcp_edit_action(action),
        }
    }

    fn mcp_toggle(&mut self, path: &TreePath) -> Result<(), String> {
        if model::has_children(self.inner_mut().data_mut().file(), path) {
            return Err("Cannot toggle parent; toggle children instead".to_string());
        }
        let item = model::get_item_mut(self.inner_mut().data_mut().file_mut(), path).ok_or("Item not found")?;
        item.completed = match item.completed {
            Completion::Done => Completion::Open,
            _ => Completion::Done,
        };
        Ok(())
    }

    fn mcp_add_sibling(&mut self, path: &TreePath, title: &str) -> Result<(), String> {
        let item = TodoItem::new(title);
        if !model::add_sibling(self.inner_mut().data_mut().file_mut(), path, item) {
            return Err("Failed to add item".to_string());
        }
        Ok(())
    }

    fn mcp_remove(&mut self, path: &TreePath) -> Result<(), String> {
        model::remove_item(self.inner_mut().data_mut().file_mut(), path).ok_or("Item not found")?;
        Ok(())
    }

    fn mcp_move_up(&mut self, path: &TreePath) -> Result<(), String> {
        model::swap_up(self.inner_mut().data_mut().file_mut(), path).ok_or("Cannot move up")?;
        Ok(())
    }

    fn mcp_move_down(&mut self, path: &TreePath) -> Result<(), String> {
        model::swap_down(self.inner_mut().data_mut().file_mut(), path).ok_or("Cannot move down")?;
        Ok(())
    }

    fn mcp_promote(&mut self, path: &TreePath) -> Result<(), String> {
        model::promote(self.inner_mut().data_mut().file_mut(), path).ok_or("Cannot promote")?;
        Ok(())
    }

    fn mcp_demote(&mut self, path: &TreePath) -> Result<(), String> {
        model::demote(self.inner_mut().data_mut().file_mut(), path).ok_or("Cannot demote")?;
        Ok(())
    }

    fn mcp_set_loe(&mut self, path: &TreePath, effort: u8) -> Result<(), String> {
        let item = model::get_item_mut(self.inner_mut().data_mut().file_mut(), path).ok_or("Item not found")?;
        item.effort = if effort == 0 {
            None
        } else {
            Some(effort)
        };
        Ok(())
    }

    fn dispatch_mcp_edit_action(&mut self, action: &McpAction) -> Result<(), String> {
        match action {
            McpAction::TodoSetNote { path, note } => self.mcp_set_note(path, note),
            McpAction::TodoToggleImportant { path } => self.mcp_toggle_important(path),
            McpAction::TodoSetPriority { path, priority } => self.mcp_set_priority(path, *priority),
            McpAction::TodoSetCompleted { path, state } => self.mcp_set_completed(path, state),
            McpAction::TodoEdit { path, title } => self.mcp_edit_title(path, title),
            McpAction::TodoAddSubtree { path, items } => self.mcp_add_subtree(path, items),
            _ => Err("Not a todo action".to_string()),
        }
    }

    fn mcp_set_note(&mut self, path: &TreePath, note: &str) -> Result<(), String> {
        let item = model::get_item_mut(self.inner_mut().data_mut().file_mut(), path).ok_or("Item not found")?;
        item.note = note.to_string();
        Ok(())
    }

    fn mcp_toggle_important(&mut self, path: &TreePath) -> Result<(), String> {
        let item = model::get_item_mut(self.inner_mut().data_mut().file_mut(), path).ok_or("Item not found")?;
        item.important = !item.important;
        Ok(())
    }

    fn mcp_set_priority(&mut self, path: &TreePath, priority: u8) -> Result<(), String> {
        let item = model::get_item_mut(self.inner_mut().data_mut().file_mut(), path).ok_or("Item not found")?;
        item.priority = if priority == 0 {
            None
        } else {
            Some(priority)
        };
        Ok(())
    }

    fn mcp_set_completed(&mut self, path: &TreePath, state: &str) -> Result<(), String> {
        if model::has_children(self.inner_mut().data_mut().file(), path) {
            return Err("Cannot set completion on parent; toggle children instead".to_string());
        }
        let item = model::get_item_mut(self.inner_mut().data_mut().file_mut(), path).ok_or("Item not found")?;
        item.completed = match state {
            "done" => Completion::Done,
            "partial" => Completion::Partial,
            _ => Completion::Open,
        };
        Ok(())
    }

    fn mcp_edit_title(&mut self, path: &TreePath, title: &str) -> Result<(), String> {
        let item = model::get_item_mut(self.inner_mut().data_mut().file_mut(), path).ok_or("Item not found")?;
        item.title = title.to_string();
        Ok(())
    }

    fn mcp_add_subtree(&mut self, path: &TreePath, items: &[serde_json::Value]) -> Result<(), String> {
        for item_val in items {
            let item = Self::build_item(item_val).ok_or("Invalid item in subtree")?;
            if path.is_empty() {
                let file = self.inner_mut().data_mut().file_mut();
                file.items.push(item);
            } else if !model::add_child(self.inner_mut().data_mut().file_mut(), path, item) {
                return Err("Failed to add subtree item".to_string());
            }
        }
        Ok(())
    }

    fn build_item(val: &serde_json::Value) -> Option<TodoItem> {
        let title_val = val.get("title")?;
        let title = title_val.as_str()?;
        let mut item = TodoItem::new(title);
        if let Some(children) = val.get("items").and_then(|v| v.as_array()) {
            for child_val in children {
                if let Some(child) = Self::build_item(child_val) {
                    item.items.push(child);
                }
            }
        }
        Some(item)
    }
}
