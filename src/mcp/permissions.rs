//! MCP tool permission model — allow/confirm/deny per tool.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Permission level for an MCP tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Allow,
    Confirm,
    Deny,
}

/// Per-tool permission table with a default policy.
pub struct PermissionTable {
    tools: HashMap<String, Permission>,
    default_read: Permission,
    default_write: Permission,
}

impl Default for PermissionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionTable {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            default_read: Permission::Allow,
            default_write: Permission::Confirm,
        }
    }

    pub fn set(&mut self, tool: &str, perm: Permission) {
        self.tools.insert(tool.to_string(), perm);
    }

    pub fn get(&self, tool: &str, is_write: bool) -> Permission {
        if let Some(&p) = self.tools.get(tool) {
            return p;
        }
        if is_write {
            self.default_write
        } else {
            self.default_read
        }
    }

    pub fn set_default_write(&mut self, perm: Permission) {
        self.default_write = perm;
    }

    pub fn set_default_read(&mut self, perm: Permission) {
        self.default_read = perm;
    }
}

/// Shared handle to the permission table.
pub type PermissionHandle = Arc<Mutex<PermissionTable>>;

pub fn new_permission_table() -> PermissionHandle {
    Arc::new(Mutex::new(PermissionTable::new()))
}

/// Classify whether a tool is a write (mutating) tool.
pub fn is_write_tool(name: &str) -> bool {
    matches!(
        name,
        "create_file"
            | "edit_buffer"
            | "insert_text"
            | "save_file"
            | "close_tab"
            | "run_build"
            | "diff_revert"
            | "send_terminal_input"
            | "git_ops"
            | "eval_tcl"
            | "undo_redo"
            | "lsp_control"
            | "clipboard_copy"
            | "workspace_roots"
    )
}
