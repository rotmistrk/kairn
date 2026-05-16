//! LSP integration — Language Server Protocol client.

pub mod client;
pub mod completion;
pub mod config_commands;
pub mod diagnostics;
pub mod handler;
pub mod messages;
pub mod progress;
pub mod protocol;
pub mod registry;
pub mod requests;
pub mod resource_ops;
mod response;
mod send;
mod send_sync;
pub mod workspace_edit;
