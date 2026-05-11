//! Mock LSP server for integration tests.
//!
//! Reads JSON-RPC from stdin, responds with canned data.
//! Supports: initialize, textDocument/definition, textDocument/references,
//! textDocument/hover, textDocument/completion, textDocument/didChange.
//!
//! Build: cargo build --bin mock_lsp
//! Usage: echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | mock_lsp

use std::io::{self, BufRead, Write};

use serde_json::{Value, json};

fn main() {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    let mut did_change_count: u32 = 0;

    loop {
        let Some(request) = read_message(&mut reader) else {
            break;
        };
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let id = request.get("id");

        // Notifications (no id) — just track them
        if method == "notifications/initialized" || method == "initialized" {
            continue;
        }
        if method == "textDocument/didChange" {
            did_change_count += 1;
            continue;
        }
        if method == "textDocument/didOpen" {
            continue;
        }
        if method == "exit" {
            break;
        }

        let Some(id) = id else { continue };

        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "capabilities": {
                        "textDocumentSync": 1,
                        "completionProvider": {},
                        "definitionProvider": true,
                        "referencesProvider": true,
                        "hoverProvider": true,
                        "renameProvider": true
                    }
                }
            }),
            "textDocument/definition" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "uri": "file:///src/lib.rs",
                    "range": {"start": {"line": 10, "character": 4}, "end": {"line": 10, "character": 12}}
                }
            }),
            "textDocument/references" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": [
                    {"uri": "file:///src/main.rs", "range": {"start": {"line": 5, "character": 0}, "end": {"line": 5, "character": 3}}},
                    {"uri": "file:///src/lib.rs", "range": {"start": {"line": 10, "character": 4}, "end": {"line": 10, "character": 7}}}
                ]
            }),
            "textDocument/hover" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {"contents": {"kind": "markdown", "value": "```rust\nfn hello() -> &str\n```"}}
            }),
            "textDocument/completion" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "items": [
                        {"label": "println!", "detail": "macro", "insertText": "println!($0)"},
                        {"label": "print!", "detail": "macro"}
                    ]
                }
            }),
            "textDocument/rename" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {"changes": {}}
            }),
            "textDocument/codeAction" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": [{"title": "Extract function", "kind": "refactor.extract"}]
            }),
            "shutdown" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": null
            }),
            "mock/didChangeCount" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": did_change_count
            }),
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("Method not found: {method}")}
            }),
        };

        write_message(&mut writer, &response);
    }
}

fn read_message(reader: &mut impl BufRead) -> Option<Value> {
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
            content_length = len_str.parse().ok()?;
        }
    }
    if content_length == 0 {
        return None;
    }
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}

fn write_message(writer: &mut impl Write, msg: &Value) {
    let body = serde_json::to_string(msg).unwrap_or_default();
    let _ = write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = writer.flush();
}
