//! JSON-RPC message types for LSP communication.

use serde_json::Value;

/// An incoming message from the LSP server.
#[derive(Debug, Clone)]
pub enum LspMessage {
    /// Response to a request (id, result or error).
    Response {
        id: u64,
        result: Option<Value>,
        error: Option<RpcError>,
    },
    /// Server-initiated notification (no id).
    Notification { method: String, params: Value },
    /// Server-initiated request (has id + method, needs response).
    ServerRequest { id: u64, method: String },
}

/// JSON-RPC error object.
#[derive(Debug, Clone)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

/// Encode a JSON-RPC request with Content-Length header.
pub fn encode_request(id: u64, method: &str, params: Value) -> Vec<u8> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    encode_body(&body)
}

/// Encode a JSON-RPC notification (no id).
pub fn encode_notification(method: &str, params: Value) -> Vec<u8> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    });
    encode_body(&body)
}

/// Encode a JSON-RPC response (null result) to a server request.
pub fn encode_response(id: u64) -> Vec<u8> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": null,
    });
    encode_body(&body)
}

fn encode_body(body: &Value) -> Vec<u8> {
    let json = match serde_json::to_string(body) {
        Ok(s) => s,
        Err(e) => {
            log::error!("LSP serialize bug: {e}");
            return Vec::new();
        }
    };
    format!("Content-Length: {}\r\n\r\n{}", json.len(), json).into_bytes()
}

/// Parse a raw JSON-RPC message into an LspMessage.
pub fn parse_message(json: &Value) -> Option<LspMessage> {
    if let Some(id) = json.get("id").and_then(|v| v.as_u64()) {
        // It has an id — could be a response (has result/error) or a request from server
        if let Some(method) = json.get("method").and_then(|v| v.as_str()) {
            return Some(LspMessage::ServerRequest {
                id,
                method: method.to_string(),
            });
        }
        let result = json.get("result").cloned();
        let error = json.get("error").and_then(|e| {
            Some(RpcError {
                code: e.get("code")?.as_i64()?,
                message: e.get("message")?.as_str()?.to_string(),
            })
        });
        Some(LspMessage::Response { id, result, error })
    } else if let Some(method) = json.get("method").and_then(|v| v.as_str()) {
        let params = json.get("params").cloned().unwrap_or(Value::Null);
        Some(LspMessage::Notification {
            method: method.to_string(),
            params,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn encode_request_has_content_length() {
        let data = encode_request(1, "initialize", json!({}));
        let s = String::from_utf8(data).unwrap();
        assert!(s.starts_with("Content-Length: "));
        assert!(s.contains("\"id\":1"));
        assert!(s.contains("\"method\":\"initialize\""));
    }

    #[test]
    fn encode_notification_no_id() {
        let data = encode_notification("initialized", json!({}));
        let s = String::from_utf8(data).unwrap();
        assert!(s.contains("\"method\":\"initialized\""));
        assert!(!s.contains("\"id\""));
    }

    #[test]
    fn parse_response() {
        let json = json!({"jsonrpc": "2.0", "id": 1, "result": {"capabilities": {}}});
        let msg = parse_message(&json);
        assert!(matches!(msg, Some(LspMessage::Response { id: 1, .. })));
    }

    #[test]
    fn parse_notification() {
        let json = json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {}});
        let msg = parse_message(&json);
        match msg {
            Some(LspMessage::Notification { method, .. }) => {
                assert_eq!(method, "textDocument/publishDiagnostics");
            }
            _ => panic!("expected notification"),
        }
    }

    #[test]
    fn parse_error_response() {
        let json = json!({"jsonrpc": "2.0", "id": 2, "error": {"code": -32600, "message": "Invalid"}});
        let msg = parse_message(&json);
        match msg {
            Some(LspMessage::Response {
                id: 2, error: Some(e), ..
            }) => {
                assert_eq!(e.code, -32600);
                assert_eq!(e.message, "Invalid");
            }
            _ => panic!("expected error response"),
        }
    }
}
