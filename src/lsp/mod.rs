//! LSP integration — Language Server Protocol client.

pub mod client;
pub mod completion;
#[cfg(test)]
mod completion_tests;
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
mod send_sync;
pub mod server_config;
pub mod signature_help;
pub mod workspace_edit;
