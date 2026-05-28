//! Configuration for a language server.

use std::collections::HashMap;

/// Configuration for a language server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: HashMap<String, String>,
}
