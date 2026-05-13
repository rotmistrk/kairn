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
        }
        self.inner.data.save();
        self.inner.data.rebuild_flat();
        self.inner.state.mark_dirty();
        Ok(json!({"ok": true}))
    }
}
