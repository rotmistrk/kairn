//! MCP action handler for TodoTreeView.

use serde_json::json;

use super::model::{self, Completion};
use super::TodoTreeView;
use crate::mcp::commands::McpAction;

impl TodoTreeView {
    /// Execute an MCP action on the todo tree. Returns JSON result.
    pub fn mcp_action(&mut self, action: &McpAction) -> Result<serde_json::Value, String> {
        match action {
            McpAction::TodoToggle { path } => {
                let item = model::get_item_mut(&mut self.inner.data.file, path).ok_or("Item not found")?;
                item.completed = match item.completed {
                    Completion::Done => Completion::Open,
                    _ => Completion::Done,
                };
            }
            McpAction::TodoAdd { path, title } => {
                let item = model::TodoItem::new(title);
                if !model::add_sibling(&mut self.inner.data.file, path, item) {
                    return Err("Failed to add item".to_string());
                }
            }
            McpAction::TodoRemove { path } => {
                model::remove_item(&mut self.inner.data.file, path).ok_or("Item not found")?;
            }
            McpAction::TodoMoveUp { path } => {
                model::swap_up(&mut self.inner.data.file, path).ok_or("Cannot move up")?;
            }
            McpAction::TodoMoveDown { path } => {
                model::swap_down(&mut self.inner.data.file, path).ok_or("Cannot move down")?;
            }
            McpAction::TodoPromote { path } => {
                model::promote(&mut self.inner.data.file, path).ok_or("Cannot promote")?;
            }
            McpAction::TodoDemote { path } => {
                model::demote(&mut self.inner.data.file, path).ok_or("Cannot demote")?;
            }
            McpAction::TodoSetNote { path, note } => {
                let item = model::get_item_mut(&mut self.inner.data.file, path).ok_or("Item not found")?;
                item.note.clone_from(note);
            }
            McpAction::TodoAddSubtree { path, items } => {
                fn build_item(val: &serde_json::Value) -> Option<model::TodoItem> {
                    let title = val.get("title")?.as_str()?;
                    let mut item = model::TodoItem::new(title);
                    if let Some(children) = val.get("items").and_then(|v| v.as_array()) {
                        for child_val in children {
                            if let Some(child) = build_item(child_val) {
                                item.items.push(child);
                            }
                        }
                    }
                    Some(item)
                }
                for item_val in items {
                    let item = build_item(item_val).ok_or("Invalid item in subtree")?;
                    if path.is_empty() {
                        // Empty path: add as top-level item
                        self.inner.data.file.items.push(item);
                    } else if !model::add_child(&mut self.inner.data.file, path, item) {
                        return Err("Failed to add subtree item".to_string());
                    }
                }
            }
            _ => return Err("Not a todo action".to_string()),
        }
        self.inner.data.save();
        self.inner.data.rebuild_flat();
        self.inner.mark_dirty();
        Ok(json!({"ok": true}))
    }
}
