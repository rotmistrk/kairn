//! MCP server module — Unix socket server for kiro integration.

pub mod agent_file;
pub mod agent_patch;
pub mod bridge;
pub mod collect;
pub mod command_queue;
pub mod commands;
pub mod cursor_pos;
pub mod listener;
pub mod log;
pub mod selection_range;
pub mod server;
pub mod snapshot;
pub mod socket_path;
pub mod tab_info;
pub mod terminal_info;
pub mod tools;
pub mod tools_defs;
pub mod tools_defs_extra;
pub mod tools_defs_write;
pub mod tools_extra;
pub mod tools_todo;
pub mod tools_write;

pub use cursor_pos::CursorPos;
pub use selection_range::SelectionRange;
pub use snapshot::McpSnapshot;
pub use tab_info::TabInfo;
pub use terminal_info::TerminalInfo;
