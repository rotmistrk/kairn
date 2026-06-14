//! JSON-RPC 2.0 MCP server implementation.

use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};

use super::commands::{McpAction, McpCommandQueue};
use super::log::log as mcp_log;
use super::permissions::{is_write_tool, Permission, PermissionHandle};
use super::snapshot::McpSnapshot;
use super::tools;

fn jsonrpc_error(id: Option<&Value>, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id.cloned().unwrap_or(Value::Null),
        "error": {"code": code, "message": message},
    })
}

fn jsonrpc_result(id: &Value, result: &Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

/// MCP server holding a shared snapshot of kairn state.
pub struct McpServer {
    snapshot: Arc<Mutex<McpSnapshot>>,
    cmd_queue: Option<McpCommandQueue>,
    permissions: Option<PermissionHandle>,
}

impl McpServer {
    pub fn new(
        snapshot: Arc<Mutex<McpSnapshot>>,
        cmd_queue: Option<McpCommandQueue>,
        permissions: Option<PermissionHandle>,
    ) -> Self {
        Self {
            snapshot,
            cmd_queue,
            permissions,
        }
    }

    /// Run the server loop: read JSON-RPC lines, dispatch, write responses.
    ///
    /// # Errors
    /// Returns `io::Error` on read/write failure.
    pub fn run<R: BufRead, W: Write>(&self, reader: R, mut writer: W) -> io::Result<()> {
        for line_result in reader.lines() {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }
            let Ok(request) = serde_json::from_str::<Value>(&line) else {
                let err = jsonrpc_error(None, -32700, "Parse error");
                writeln!(writer, "{err}")?;
                writer.flush()?;
                continue;
            };
            let method = request.get("method").and_then(Value::as_str).unwrap_or("?");
            mcp_log("mcp", &format!("← {method}"));
            if let Some(response) = self.handle_request(&request) {
                writeln!(writer, "{response}")?;
                writer.flush()?;
            }
        }
        Ok(())
    }

    fn handle_request(&self, request: &Value) -> Option<Value> {
        let id = request.get("id");
        let method = request.get("method").and_then(Value::as_str);

        let Some(method) = method else {
            return Some(jsonrpc_error(id, -32600, "Missing method"));
        };

        // Notifications (no id) get no response.
        if method == "notifications/initialized" {
            return None;
        }

        let id = id?;

        match method {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "kairn", "version": "0.1.0"},
                });
                Some(jsonrpc_result(id, &result))
            }
            "tools/list" => {
                let result = json!({"tools": tools::tool_definitions()});
                Some(jsonrpc_result(id, &result))
            }
            "tools/call" => Some(self.handle_tools_call(id, request)),
            _ => Some(jsonrpc_error(Some(id), -32601, &format!("Method not found: {method}"))),
        }
    }

    fn handle_tools_call(&self, id: &Value, request: &Value) -> Value {
        let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
        let tool_name = params.get("name").and_then(Value::as_str).unwrap_or("");
        let empty_map = Map::new();
        let arguments = params.get("arguments").and_then(Value::as_object).unwrap_or(&empty_map);

        // Permission check
        if let Err(msg) = self.check_permission(tool_name, arguments) {
            let r = json!({"isError": true, "content": [{"type": "text", "text": msg}]});
            return jsonrpc_result(id, &r);
        }

        match tools::handle_tool_call(&self.snapshot, self.cmd_queue.as_ref(), tool_name, arguments) {
            Ok(result) => {
                let text = if result.is_string() {
                    result.as_str().unwrap_or("").to_owned()
                } else {
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                };
                let r = json!({"content": [{"type": "text", "text": text}]});
                jsonrpc_result(id, &r)
            }
            Err(msg) => {
                let r = json!({
                    "isError": true,
                    "content": [{"type": "text", "text": msg}],
                });
                jsonrpc_result(id, &r)
            }
        }
    }

    fn check_permission(&self, tool_name: &str, args: &Map<String, Value>) -> Result<(), String> {
        let Some(ref perms) = self.permissions else {
            return Ok(());
        };
        let perm = if let Ok(table) = perms.lock() {
            table.get(tool_name, is_write_tool(tool_name))
        } else {
            Permission::Allow
        };
        match perm {
            Permission::Allow => Ok(()),
            Permission::Deny => Err(format!("Tool '{tool_name}' is denied by permission policy")),
            Permission::Confirm => self.request_confirmation(tool_name, args),
        }
    }

    fn request_confirmation(&self, tool_name: &str, args: &Map<String, Value>) -> Result<(), String> {
        let Some(ref cq) = self.cmd_queue else {
            return Ok(()); // no queue = headless mode, allow
        };
        let summary = summarize_args(args);
        let action = McpAction::ConfirmTool {
            tool_name: tool_name.to_string(),
            args_summary: summary,
        };
        let result = cq.send(action)?;
        if result.as_bool() == Some(true) {
            Ok(())
        } else {
            Err(format!("Tool '{tool_name}' denied by user"))
        }
    }
}

fn summarize_args(args: &Map<String, Value>) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (k, v) in args.iter().take(3) {
        let val = match v {
            Value::String(s) if s.len() > 40 => format!("{}...", &s[..37]),
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        parts.push(format!("{k}={val}"));
    }
    if args.len() > 3 {
        parts.push(format!("(+{} more)", args.len() - 3));
    }
    parts.join(", ")
}
