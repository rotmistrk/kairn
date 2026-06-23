//! Application views — concrete View implementations for kairn.

pub mod clipboard_viewer;
pub mod csv_view;
pub mod diff_view;
pub mod editor;
pub mod git_changes;
pub mod git_log;
mod git_log_draw;
pub mod help;
pub mod messages;
pub mod notes;
pub mod problems;
pub mod result_entry;
pub mod results;
pub mod scroll_map;
pub mod search_replace;
pub mod struct_view;
pub mod terminal;
pub mod todo_tree;
pub mod tree;
mod tree_handlers;
pub mod welcome;
