//! Shared argument extraction helpers for MCP tool handlers.

use serde_json::{Map, Value};

use super::commands::McpCommandQueue;

/// Extract the command queue or return a standard error.
pub fn require_queue(cmd_queue: Option<&McpCommandQueue>) -> Result<&McpCommandQueue, String> {
    cmd_queue.ok_or_else(|| "Write operations disabled".to_string())
}

/// Extract a required string argument by key.
pub fn require_str<'a>(args: &'a Map<String, Value>, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Missing '{key}' argument"))
}

/// Extract an optional string argument with a default value.
pub fn opt_str<'a>(args: &'a Map<String, Value>, key: &str, default: &'a str) -> &'a str {
    args.get(key).and_then(Value::as_str).unwrap_or(default)
}

/// Extract a required u64 argument by key.
pub fn require_u64(args: &Map<String, Value>, key: &str) -> Result<u64, String> {
    args.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("Missing '{key}' argument"))
}

/// Extract an optional bool argument with a default value.
pub fn opt_bool(args: &Map<String, Value>, key: &str, default: bool) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(default)
}
