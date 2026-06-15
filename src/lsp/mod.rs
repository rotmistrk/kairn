//! LSP integration — Language Server Protocol client.

pub mod client;
pub mod completion_parse;
pub mod completion_source;
pub mod config_commands;
pub mod diagnostic_store;
pub mod diagnostics;
pub mod handler;
mod jdt_request;
pub mod location;
pub mod messages;
pub(crate) mod pending;
pub mod progress;
pub mod protocol;
pub mod registry;
mod registry_config;
pub mod requests;
pub mod resource_ops;
mod response;
pub mod response_parse;
mod send;
mod send_helpers;
mod send_sync;
pub mod server_config;
pub mod signature_help;
pub(crate) mod text_edit;
pub mod uri;
pub mod workspace_edit;
