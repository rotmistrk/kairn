//! Async reader/writer tasks for LSP server communication over stdio.

use std::collections::HashMap;

use tokio::io::{AsyncRead, AsyncWrite, BufReader};
use tokio::sync::mpsc;

use crate::lsp::protocol::{self, LspIncoming, LspMessage, LspNotification, LspRequest, RequestId};
use crate::lsp::types::{
    CompletionItem, CompletionKind, Diagnostic, DiagnosticSeverity, DocumentSymbol, DocumentUri,
    LanguageId, Location, LspEvent, MessageLevel, SymbolKind, TextEditRange,
};

/// Writer task: reads outgoing messages from a channel, writes to server stdin.
pub async fn writer_task(mut rx: mpsc::Receiver<LspMessage>, mut stdin: impl AsyncWrite + Unpin) {
    while let Some(msg) = rx.recv().await {
        if let Err(e) = protocol::write_message_async(&mut stdin, &msg).await {
            tracing::error!("LSP write error: {e}");
            break;
        }
    }
}

/// Reader task: reads from server stdout, decodes, sends events to main thread.
pub async fn reader_task(
    language_id: LanguageId,
    stdout: impl AsyncRead + Unpin,
    pending_methods: mpsc::Receiver<(RequestId, String, Option<DocumentUri>)>,
    event_tx: mpsc::Sender<LspEvent>,
) {
    let mut reader = BufReader::new(stdout);
    let mut methods: HashMap<RequestId, (String, Option<DocumentUri>)> = HashMap::new();
    let mut pending_rx = pending_methods;

    loop {
        // Drain any newly registered pending methods.
        while let Ok((id, method, uri)) = pending_rx.try_recv() {
            methods.insert(id, (method, uri));
        }

        match protocol::read_message_async(&mut reader).await {
            Ok(msg) => {
                let event = decode_incoming(&language_id, msg, &mut methods);
                if let Some(ev) = event {
                    if event_tx.send(ev).await.is_err() {
                        break; // main thread dropped receiver
                    }
                }
            }
            Err(_) => {
                // EOF or parse error — server crashed.
                let _ = event_tx
                    .send(LspEvent::ServerCrashed(
                        language_id,
                        "connection lost".into(),
                    ))
                    .await;
                break;
            }
        }
    }
}

/// Decode an incoming JSON-RPC message into an `LspEvent`.
fn decode_incoming(
    language_id: &LanguageId,
    msg: LspIncoming,
    pending: &mut HashMap<RequestId, (String, Option<DocumentUri>)>,
) -> Option<LspEvent> {
    match msg {
        LspIncoming::Response(resp) => decode_response(language_id, resp, pending),
        LspIncoming::Notification(notif) => decode_notification(notif),
        LspIncoming::Request(_req) => {
            // Server requests (workspace/configuration, etc.)
            // handled in a future iteration.
            None
        }
    }
}

/// Decode a response using the pending method registry.
fn decode_response(
    language_id: &LanguageId,
    resp: protocol::LspResponse,
    pending: &mut HashMap<RequestId, (String, Option<DocumentUri>)>,
) -> Option<LspEvent> {
    let (method, uri) = pending.remove(&resp.id)?;

    if let Some(err) = resp.error {
        return Some(LspEvent::Response {
            language_id: language_id.clone(),
            id: resp.id,
            result: Err(err),
        });
    }

    let result = resp.result.unwrap_or(serde_json::Value::Null);

    match method.as_str() {
        "initialize" => Some(LspEvent::ServerReady(language_id.clone())),
        "textDocument/completion" => {
            let items = decode_completions(&result);
            Some(LspEvent::Completions {
                uri: uri.unwrap_or_else(|| DocumentUri::new("")),
                items,
            })
        }
        "textDocument/hover" => {
            let contents = decode_hover(&result);
            Some(LspEvent::Hover {
                uri: uri.unwrap_or_else(|| DocumentUri::new("")),
                contents,
            })
        }
        "textDocument/definition" => {
            let locations = decode_locations(&result);
            Some(LspEvent::Definition {
                uri: uri.unwrap_or_else(|| DocumentUri::new("")),
                locations,
            })
        }
        "textDocument/references" => {
            let locations = decode_locations(&result);
            Some(LspEvent::References {
                uri: uri.unwrap_or_else(|| DocumentUri::new("")),
                locations,
            })
        }
        "textDocument/documentSymbol" => {
            let symbols = decode_symbols(&result);
            Some(LspEvent::Symbols {
                uri: uri.unwrap_or_else(|| DocumentUri::new("")),
                symbols,
            })
        }
        "textDocument/formatting" | "textDocument/rangeFormatting" => {
            let edits = decode_text_edits(&result);
            Some(LspEvent::FormattingEdits {
                uri: uri.unwrap_or_else(|| DocumentUri::new("")),
                edits,
            })
        }
        "textDocument/rename" => {
            let edits = decode_workspace_edit(&result);
            Some(LspEvent::WorkspaceEdit { edits })
        }
        _ => Some(LspEvent::Response {
            language_id: language_id.clone(),
            id: resp.id,
            result: Ok(result),
        }),
    }
}

/// Decode a server notification into an event.
fn decode_notification(notif: LspNotification) -> Option<LspEvent> {
    match notif.method.as_str() {
        "textDocument/publishDiagnostics" => decode_diagnostics_notification(&notif.params),
        "window/showMessage" => decode_show_message(&notif.params),
        "window/logMessage" => {
            // Debug-only: log but don't surface to UI.
            tracing::debug!(
                "LSP log: {}",
                notif
                    .params
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            );
            None
        }
        _ => None,
    }
}

// ── Response decoders ───────────────────────────────────────

/// Decode completion items from a completion response.
fn decode_completions(value: &serde_json::Value) -> Vec<CompletionItem> {
    // Response can be CompletionList { items: [...] } or just [...]
    let items = value
        .get("items")
        .and_then(|v| v.as_array())
        .or_else(|| value.as_array());

    let Some(arr) = items else {
        return Vec::new();
    };

    arr.iter()
        .filter_map(|item| {
            let label = item.get("label")?.as_str()?.to_string();
            let kind_num = item.get("kind").and_then(|v| v.as_u64()).unwrap_or(0);
            let detail = item
                .get("detail")
                .and_then(|v| v.as_str())
                .map(String::from);
            let insert_text = item
                .get("insertText")
                .and_then(|v| v.as_str())
                .unwrap_or(&label)
                .to_string();

            Some(CompletionItem {
                label,
                kind: completion_kind_from_lsp(kind_num),
                detail,
                insert_text,
            })
        })
        .collect()
}

/// Decode hover content.
fn decode_hover(value: &serde_json::Value) -> String {
    // Hover result: { contents: string | MarkupContent | MarkedString[] }
    if let Some(contents) = value.get("contents") {
        if let Some(s) = contents.as_str() {
            return s.to_string();
        }
        if let Some(s) = contents.get("value").and_then(|v| v.as_str()) {
            return s.to_string();
        }
        if let Some(arr) = contents.as_array() {
            return arr
                .iter()
                .filter_map(|v| {
                    v.as_str()
                        .map(String::from)
                        .or_else(|| v.get("value").and_then(|s| s.as_str()).map(String::from))
                })
                .collect::<Vec<_>>()
                .join("\n");
        }
    }
    String::new()
}

/// Decode location(s) from definition/references response.
fn decode_locations(value: &serde_json::Value) -> Vec<Location> {
    // Can be a single Location, Location[], or LocationLink[].
    if let Some(arr) = value.as_array() {
        arr.iter().filter_map(decode_single_location).collect()
    } else {
        decode_single_location(value).into_iter().collect()
    }
}

/// Decode a single Location or LocationLink.
fn decode_single_location(value: &serde_json::Value) -> Option<Location> {
    let uri_str = value
        .get("uri")
        .or_else(|| value.get("targetUri"))
        .and_then(|v| v.as_str())?;

    let range = value.get("range").or_else(|| value.get("targetRange"))?;

    let start = range.get("start")?;
    let line = start.get("line")?.as_u64()? as u32;
    let character = start.get("character")?.as_u64()? as u32;

    Some(Location {
        uri: DocumentUri::new(uri_str),
        line,
        character,
    })
}

/// Decode document symbols.
fn decode_symbols(value: &serde_json::Value) -> Vec<DocumentSymbol> {
    let Some(arr) = value.as_array() else {
        return Vec::new();
    };
    arr.iter().filter_map(decode_single_symbol).collect()
}

/// Decode a single DocumentSymbol.
fn decode_single_symbol(value: &serde_json::Value) -> Option<DocumentSymbol> {
    let name = value.get("name")?.as_str()?.to_string();
    let kind_num = value.get("kind")?.as_u64()?;
    let range = value.get("range")?;
    let start_line = range.get("start")?.get("line")?.as_u64()? as u32;
    let end_line = range.get("end")?.get("line")?.as_u64()? as u32;

    let children = value
        .get("children")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(decode_single_symbol).collect())
        .unwrap_or_default();

    Some(DocumentSymbol {
        name,
        kind: symbol_kind_from_lsp(kind_num),
        range_start_line: start_line,
        range_end_line: end_line,
        children,
    })
}

/// Decode text edits from a formatting response.
fn decode_text_edits(value: &serde_json::Value) -> Vec<TextEditRange> {
    let Some(arr) = value.as_array() else {
        return Vec::new();
    };
    arr.iter().filter_map(decode_single_text_edit).collect()
}

/// Decode a single TextEdit.
fn decode_single_text_edit(value: &serde_json::Value) -> Option<TextEditRange> {
    let range = value.get("range")?;
    let start = range.get("start")?;
    let end = range.get("end")?;
    let new_text = value.get("newText")?.as_str()?.to_string();

    Some(TextEditRange {
        start_line: start.get("line")?.as_u64()? as u32,
        start_character: start.get("character")?.as_u64()? as u32,
        end_line: end.get("line")?.as_u64()? as u32,
        end_character: end.get("character")?.as_u64()? as u32,
        new_text,
    })
}

/// Decode workspace edit from a rename response.
fn decode_workspace_edit(value: &serde_json::Value) -> HashMap<DocumentUri, Vec<TextEditRange>> {
    let mut result = HashMap::new();
    let Some(changes) = value.get("changes").and_then(|v| v.as_object()) else {
        // Try documentChanges format.
        if let Some(doc_changes) = value.get("documentChanges").and_then(|v| v.as_array()) {
            for dc in doc_changes {
                let uri_str = dc
                    .get("textDocument")
                    .and_then(|td| td.get("uri"))
                    .and_then(|v| v.as_str());
                let edits = dc.get("edits").and_then(|v| v.as_array());
                if let (Some(uri_str), Some(edits)) = (uri_str, edits) {
                    let uri = DocumentUri::new(uri_str);
                    let text_edits: Vec<TextEditRange> =
                        edits.iter().filter_map(decode_single_text_edit).collect();
                    if !text_edits.is_empty() {
                        result.insert(uri, text_edits);
                    }
                }
            }
        }
        return result;
    };

    for (uri_str, edits_val) in changes {
        let uri = DocumentUri::new(uri_str.as_str());
        let edits: Vec<TextEditRange> = edits_val
            .as_array()
            .map(|arr| arr.iter().filter_map(decode_single_text_edit).collect())
            .unwrap_or_default();
        if !edits.is_empty() {
            result.insert(uri, edits);
        }
    }
    result
}

/// Decode publishDiagnostics notification.
fn decode_diagnostics_notification(params: &serde_json::Value) -> Option<LspEvent> {
    let uri_str = params.get("uri")?.as_str()?;
    let diag_arr = params.get("diagnostics")?.as_array()?;

    let diagnostics: Vec<Diagnostic> = diag_arr
        .iter()
        .filter_map(|d| {
            let range = d.get("range")?;
            let start = range.get("start")?;
            let end = range.get("end")?;
            let severity_num = d.get("severity").and_then(|v| v.as_u64()).unwrap_or(1);
            let message = d.get("message")?.as_str()?.to_string();
            let source = d.get("source").and_then(|v| v.as_str()).map(String::from);

            Some(Diagnostic {
                start_line: start.get("line")?.as_u64()? as u32,
                start_character: start.get("character")?.as_u64()? as u32,
                end_line: end.get("line")?.as_u64()? as u32,
                end_character: end.get("character")?.as_u64()? as u32,
                severity: diagnostic_severity_from_lsp(severity_num),
                message,
                source,
            })
        })
        .collect();

    Some(LspEvent::Diagnostics {
        uri: DocumentUri::new(uri_str),
        diagnostics,
    })
}

/// Decode window/showMessage notification.
fn decode_show_message(params: &serde_json::Value) -> Option<LspEvent> {
    let type_num = params.get("type")?.as_u64()?;
    let message = params.get("message")?.as_str()?.to_string();
    let level = match type_num {
        1 => MessageLevel::Error,
        2 => MessageLevel::Warning,
        3 => MessageLevel::Info,
        _ => MessageLevel::Log,
    };
    Some(LspEvent::ShowMessage { level, message })
}

// ── LSP enum conversions ────────────────────────────────────

/// Map LSP CompletionItemKind number to our enum.
fn completion_kind_from_lsp(kind: u64) -> CompletionKind {
    match kind {
        3 => CompletionKind::Function,
        2 => CompletionKind::Method,
        5 => CompletionKind::Field,
        6 => CompletionKind::Variable,
        7 => CompletionKind::Class,
        8 => CompletionKind::Interface,
        9 => CompletionKind::Module,
        14 => CompletionKind::Keyword,
        15 => CompletionKind::Snippet,
        _ => CompletionKind::Other,
    }
}

/// Map LSP SymbolKind number to our enum.
fn symbol_kind_from_lsp(kind: u64) -> SymbolKind {
    match kind {
        12 => SymbolKind::Function,
        6 => SymbolKind::Method,
        5 => SymbolKind::Class,
        11 => SymbolKind::Interface,
        23 => SymbolKind::Struct,
        10 => SymbolKind::Enum,
        8 => SymbolKind::Field,
        14 => SymbolKind::Constant,
        13 => SymbolKind::Variable,
        2 => SymbolKind::Module,
        _ => SymbolKind::Other,
    }
}

/// Map LSP DiagnosticSeverity number to our enum.
fn diagnostic_severity_from_lsp(sev: u64) -> DiagnosticSeverity {
    match sev {
        1 => DiagnosticSeverity::Error,
        2 => DiagnosticSeverity::Warning,
        3 => DiagnosticSeverity::Info,
        4 => DiagnosticSeverity::Hint,
        _ => DiagnosticSeverity::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_completion_list() {
        let json = serde_json::json!({
            "items": [
                {"label": "foo", "kind": 3, "insertText": "foo()"},
                {"label": "bar", "kind": 5}
            ]
        });
        let items = decode_completions(&json);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].label, "foo");
        assert_eq!(items[0].kind, CompletionKind::Function);
        assert_eq!(items[0].insert_text, "foo()");
        assert_eq!(items[1].label, "bar");
        assert_eq!(items[1].kind, CompletionKind::Field);
        assert_eq!(items[1].insert_text, "bar");
    }

    #[test]
    fn decode_completion_array() {
        let json = serde_json::json!([
            {"label": "x", "kind": 6}
        ]);
        let items = decode_completions(&json);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, CompletionKind::Variable);
    }

    #[test]
    fn decode_hover_string() {
        let json = serde_json::json!({"contents": "fn main()"});
        assert_eq!(decode_hover(&json), "fn main()");
    }

    #[test]
    fn decode_hover_markup() {
        let json = serde_json::json!({
            "contents": {"kind": "plaintext", "value": "fn main()"}
        });
        assert_eq!(decode_hover(&json), "fn main()");
    }

    #[test]
    fn decode_single_location_test() {
        let json = serde_json::json!({
            "uri": "file:///src/main.rs",
            "range": {
                "start": {"line": 10, "character": 5},
                "end": {"line": 10, "character": 9}
            }
        });
        let locs = decode_locations(&json);
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0].uri.as_str(), "file:///src/main.rs");
        assert_eq!(locs[0].line, 10);
        assert_eq!(locs[0].character, 5);
    }

    #[test]
    fn decode_location_array() {
        let json = serde_json::json!([
            {
                "uri": "file:///a.rs",
                "range": {"start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 3}}
            },
            {
                "uri": "file:///b.rs",
                "range": {"start": {"line": 5, "character": 2}, "end": {"line": 5, "character": 8}}
            }
        ]);
        let locs = decode_locations(&json);
        assert_eq!(locs.len(), 2);
    }

    #[test]
    fn decode_symbols_nested() {
        let json = serde_json::json!([{
            "name": "MyStruct",
            "kind": 23,
            "range": {"start": {"line": 0, "character": 0}, "end": {"line": 10, "character": 0}},
            "children": [{
                "name": "method",
                "kind": 6,
                "range": {"start": {"line": 2, "character": 0}, "end": {"line": 5, "character": 0}}
            }]
        }]);
        let syms = decode_symbols(&json);
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].name, "MyStruct");
        assert_eq!(syms[0].kind, SymbolKind::Struct);
        assert_eq!(syms[0].children.len(), 1);
        assert_eq!(syms[0].children[0].name, "method");
    }

    #[test]
    fn decode_text_edits_test() {
        let json = serde_json::json!([{
            "range": {
                "start": {"line": 0, "character": 0},
                "end": {"line": 0, "character": 5}
            },
            "newText": "hello"
        }]);
        let edits = decode_text_edits(&json);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, "hello");
    }

    #[test]
    fn decode_workspace_edit_changes() {
        let json = serde_json::json!({
            "changes": {
                "file:///a.rs": [{
                    "range": {
                        "start": {"line": 1, "character": 0},
                        "end": {"line": 1, "character": 3}
                    },
                    "newText": "new_name"
                }]
            }
        });
        let edits = decode_workspace_edit(&json);
        assert_eq!(edits.len(), 1);
        let key = DocumentUri::new("file:///a.rs");
        assert!(edits.contains_key(&key));
    }

    #[test]
    fn decode_diagnostics_test() {
        let params = serde_json::json!({
            "uri": "file:///main.rs",
            "diagnostics": [{
                "range": {
                    "start": {"line": 5, "character": 0},
                    "end": {"line": 5, "character": 10}
                },
                "severity": 1,
                "message": "expected type",
                "source": "rustc"
            }]
        });
        let event = decode_diagnostics_notification(&params);
        match event {
            Some(LspEvent::Diagnostics { uri, diagnostics }) => {
                assert_eq!(uri.as_str(), "file:///main.rs");
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
                assert_eq!(diagnostics[0].message, "expected type");
            }
            _ => panic!("expected Diagnostics event"),
        }
    }

    #[test]
    fn decode_show_message_test() {
        let params = serde_json::json!({
            "type": 2,
            "message": "indexing complete"
        });
        let event = decode_show_message(&params);
        match event {
            Some(LspEvent::ShowMessage { level, message }) => {
                assert_eq!(level, MessageLevel::Warning);
                assert_eq!(message, "indexing complete");
            }
            _ => panic!("expected ShowMessage event"),
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::lsp::protocol::{self, LspMessage, LspNotification, LspRequest, RequestId};
    use crate::lsp::types::{CompletionKind, DiagnosticSeverity, LanguageId, SymbolKind};
    use tokio::io::{AsyncWriteExt, BufReader, DuplexStream};

    /// Create a duplex pair simulating server stdin/stdout.
    fn mock_streams() -> (DuplexStream, DuplexStream) {
        tokio::io::duplex(8192)
    }

    /// Write a JSON-RPC response to a stream (simulating server output).
    async fn write_response(writer: &mut DuplexStream, id: u64, result: serde_json::Value) {
        let json = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        });
        let body = serde_json::to_vec(&json).unwrap();
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        writer.write_all(header.as_bytes()).await.unwrap();
        writer.write_all(&body).await.unwrap();
        writer.flush().await.unwrap();
    }

    /// Write a JSON-RPC notification to a stream.
    async fn write_notification(
        writer: &mut DuplexStream,
        method: &str,
        params: serde_json::Value,
    ) {
        let json = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        let body = serde_json::to_vec(&json).unwrap();
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        writer.write_all(header.as_bytes()).await.unwrap();
        writer.write_all(&body).await.unwrap();
        writer.flush().await.unwrap();
    }

    #[tokio::test]
    async fn writer_task_sends_request() {
        let (client_write, server_read) = mock_streams();
        let (tx, rx) = mpsc::channel(16);

        let writer_handle = tokio::spawn(writer_task(rx, client_write));

        let msg = LspMessage::Request(LspRequest {
            id: RequestId::new(1),
            method: "initialize".into(),
            params: serde_json::json!({}),
        });
        tx.send(msg).await.unwrap();
        drop(tx); // Close channel to end writer task.

        writer_handle.await.unwrap();

        // Read what was written.
        let mut reader = BufReader::new(server_read);
        let incoming = protocol::read_message_async(&mut reader).await.unwrap();
        match incoming {
            protocol::LspIncoming::Request(req) => {
                assert_eq!(req.id, RequestId::new(1));
                assert_eq!(req.method, "initialize");
            }
            _ => panic!("expected request"),
        }
    }

    #[tokio::test]
    async fn reader_task_receives_initialize_response() {
        let (mut server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");

        // Register the pending initialize request.
        pending_tx
            .send((RequestId::new(1), "initialize".into(), None))
            .await
            .unwrap();

        let reader_handle =
            tokio::spawn(reader_task(lang.clone(), client_read, pending_rx, event_tx));

        // Server sends initialize response.
        write_response(
            &mut server_write,
            1,
            serde_json::json!({"capabilities": {}}),
        )
        .await;

        // Close server to end reader task.
        drop(server_write);

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::ServerReady(l) => assert_eq!(l, lang),
            _ => panic!("expected ServerReady, got {event:?}"),
        }

        reader_handle.await.unwrap();
    }

    #[tokio::test]
    async fn reader_task_receives_completion() {
        let (mut server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");
        let uri = DocumentUri::from_path("/tmp/main.rs");

        pending_tx
            .send((
                RequestId::new(2),
                "textDocument/completion".into(),
                Some(uri.clone()),
            ))
            .await
            .unwrap();

        tokio::spawn(reader_task(lang, client_read, pending_rx, event_tx));

        write_response(
            &mut server_write,
            2,
            serde_json::json!({
                "items": [
                    {"label": "println", "kind": 3, "insertText": "println!"},
                    {"label": "print", "kind": 3}
                ]
            }),
        )
        .await;
        drop(server_write);

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::Completions { uri: u, items } => {
                assert_eq!(u, uri);
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].label, "println");
                assert_eq!(items[0].kind, CompletionKind::Function);
                assert_eq!(items[0].insert_text, "println!");
            }
            _ => panic!("expected Completions, got {event:?}"),
        }
    }

    #[tokio::test]
    async fn reader_task_receives_diagnostics() {
        let (mut server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (_pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");

        tokio::spawn(reader_task(lang, client_read, pending_rx, event_tx));

        write_notification(
            &mut server_write,
            "textDocument/publishDiagnostics",
            serde_json::json!({
                "uri": "file:///tmp/main.rs",
                "diagnostics": [{
                    "range": {
                        "start": {"line": 5, "character": 0},
                        "end": {"line": 5, "character": 10}
                    },
                    "severity": 1,
                    "message": "expected type `usize`",
                    "source": "rustc"
                }]
            }),
        )
        .await;
        drop(server_write);

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::Diagnostics { uri, diagnostics } => {
                assert_eq!(uri.as_str(), "file:///tmp/main.rs");
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
                assert_eq!(diagnostics[0].message, "expected type `usize`");
            }
            _ => panic!("expected Diagnostics, got {event:?}"),
        }
    }

    #[tokio::test]
    async fn reader_task_receives_definition() {
        let (mut server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");
        let uri = DocumentUri::from_path("/tmp/main.rs");

        pending_tx
            .send((
                RequestId::new(3),
                "textDocument/definition".into(),
                Some(uri.clone()),
            ))
            .await
            .unwrap();

        tokio::spawn(reader_task(lang, client_read, pending_rx, event_tx));

        write_response(
            &mut server_write,
            3,
            serde_json::json!({
                "uri": "file:///tmp/lib.rs",
                "range": {
                    "start": {"line": 42, "character": 4},
                    "end": {"line": 42, "character": 10}
                }
            }),
        )
        .await;
        drop(server_write);

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::Definition { locations, .. } => {
                assert_eq!(locations.len(), 1);
                assert_eq!(locations[0].uri.as_str(), "file:///tmp/lib.rs");
                assert_eq!(locations[0].line, 42);
                assert_eq!(locations[0].character, 4);
            }
            _ => panic!("expected Definition, got {event:?}"),
        }
    }

    #[tokio::test]
    async fn reader_task_receives_hover() {
        let (mut server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");
        let uri = DocumentUri::from_path("/tmp/main.rs");

        pending_tx
            .send((
                RequestId::new(4),
                "textDocument/hover".into(),
                Some(uri.clone()),
            ))
            .await
            .unwrap();

        tokio::spawn(reader_task(lang, client_read, pending_rx, event_tx));

        write_response(
            &mut server_write,
            4,
            serde_json::json!({
                "contents": {"kind": "plaintext", "value": "fn main()"}
            }),
        )
        .await;
        drop(server_write);

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::Hover { contents, .. } => {
                assert_eq!(contents, "fn main()");
            }
            _ => panic!("expected Hover, got {event:?}"),
        }
    }

    #[tokio::test]
    async fn reader_task_receives_references() {
        let (mut server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");
        let uri = DocumentUri::from_path("/tmp/main.rs");

        pending_tx
            .send((
                RequestId::new(5),
                "textDocument/references".into(),
                Some(uri.clone()),
            ))
            .await
            .unwrap();

        tokio::spawn(reader_task(lang, client_read, pending_rx, event_tx));

        write_response(
            &mut server_write,
            5,
            serde_json::json!([
                {
                    "uri": "file:///a.rs",
                    "range": {"start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 3}}
                },
                {
                    "uri": "file:///b.rs",
                    "range": {"start": {"line": 10, "character": 5}, "end": {"line": 10, "character": 8}}
                }
            ]),
        )
        .await;
        drop(server_write);

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::References { locations, .. } => {
                assert_eq!(locations.len(), 2);
                assert_eq!(locations[0].uri.as_str(), "file:///a.rs");
                assert_eq!(locations[1].line, 10);
            }
            _ => panic!("expected References, got {event:?}"),
        }
    }

    #[tokio::test]
    async fn reader_task_receives_document_symbols() {
        let (mut server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");
        let uri = DocumentUri::from_path("/tmp/main.rs");

        pending_tx
            .send((
                RequestId::new(6),
                "textDocument/documentSymbol".into(),
                Some(uri.clone()),
            ))
            .await
            .unwrap();

        tokio::spawn(reader_task(lang, client_read, pending_rx, event_tx));

        write_response(
            &mut server_write,
            6,
            serde_json::json!([{
                "name": "main",
                "kind": 12,
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 5, "character": 1}
                },
                "children": []
            }]),
        )
        .await;
        drop(server_write);

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::Symbols { symbols, .. } => {
                assert_eq!(symbols.len(), 1);
                assert_eq!(symbols[0].name, "main");
                assert_eq!(symbols[0].kind, SymbolKind::Function);
            }
            _ => panic!("expected Symbols, got {event:?}"),
        }
    }

    #[tokio::test]
    async fn reader_task_receives_formatting_edits() {
        let (mut server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");
        let uri = DocumentUri::from_path("/tmp/main.rs");

        pending_tx
            .send((
                RequestId::new(7),
                "textDocument/formatting".into(),
                Some(uri.clone()),
            ))
            .await
            .unwrap();

        tokio::spawn(reader_task(lang, client_read, pending_rx, event_tx));

        write_response(
            &mut server_write,
            7,
            serde_json::json!([{
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 0, "character": 5}
                },
                "newText": "hello"
            }]),
        )
        .await;
        drop(server_write);

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::FormattingEdits { edits, .. } => {
                assert_eq!(edits.len(), 1);
                assert_eq!(edits[0].new_text, "hello");
            }
            _ => panic!("expected FormattingEdits, got {event:?}"),
        }
    }

    #[tokio::test]
    async fn reader_task_receives_rename_workspace_edit() {
        let (mut server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");
        let uri = DocumentUri::from_path("/tmp/main.rs");

        pending_tx
            .send((
                RequestId::new(8),
                "textDocument/rename".into(),
                Some(uri.clone()),
            ))
            .await
            .unwrap();

        tokio::spawn(reader_task(lang, client_read, pending_rx, event_tx));

        write_response(
            &mut server_write,
            8,
            serde_json::json!({
                "changes": {
                    "file:///tmp/main.rs": [{
                        "range": {
                            "start": {"line": 1, "character": 4},
                            "end": {"line": 1, "character": 7}
                        },
                        "newText": "new_name"
                    }]
                }
            }),
        )
        .await;
        drop(server_write);

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::WorkspaceEdit { edits } => {
                assert_eq!(edits.len(), 1);
                let key = DocumentUri::new("file:///tmp/main.rs");
                let file_edits = edits.get(&key).unwrap();
                assert_eq!(file_edits[0].new_text, "new_name");
            }
            _ => panic!("expected WorkspaceEdit, got {event:?}"),
        }
    }

    #[tokio::test]
    async fn reader_task_crash_on_eof() {
        let (_server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (_pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");

        // Drop server_write immediately → EOF.
        drop(_server_write);

        tokio::spawn(reader_task(lang.clone(), client_read, pending_rx, event_tx));

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::ServerCrashed(l, msg) => {
                assert_eq!(l, lang);
                assert_eq!(msg, "connection lost");
            }
            _ => panic!("expected ServerCrashed, got {event:?}"),
        }
    }

    #[tokio::test]
    async fn reader_task_receives_show_message() {
        let (mut server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (_pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");

        tokio::spawn(reader_task(lang, client_read, pending_rx, event_tx));

        write_notification(
            &mut server_write,
            "window/showMessage",
            serde_json::json!({
                "type": 3,
                "message": "Indexing complete"
            }),
        )
        .await;
        drop(server_write);

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::ShowMessage { level, message } => {
                assert_eq!(level, MessageLevel::Info);
                assert_eq!(message, "Indexing complete");
            }
            _ => panic!("expected ShowMessage, got {event:?}"),
        }
    }

    #[tokio::test]
    async fn writer_task_sends_notification() {
        let (client_write, server_read) = mock_streams();
        let (tx, rx) = mpsc::channel(16);

        let writer_handle = tokio::spawn(writer_task(rx, client_write));

        let msg = LspMessage::Notification(LspNotification {
            method: "initialized".into(),
            params: serde_json::json!({}),
        });
        tx.send(msg).await.unwrap();
        drop(tx);

        writer_handle.await.unwrap();

        let mut reader = BufReader::new(server_read);
        let incoming = protocol::read_message_async(&mut reader).await.unwrap();
        match incoming {
            protocol::LspIncoming::Notification(n) => {
                assert_eq!(n.method, "initialized");
            }
            _ => panic!("expected notification"),
        }
    }

    #[tokio::test]
    async fn reader_task_error_response() {
        let (mut server_write, client_read) = mock_streams();
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (pending_tx, pending_rx) = mpsc::channel(16);

        let lang = LanguageId::new("rust");
        let uri = DocumentUri::from_path("/tmp/main.rs");

        pending_tx
            .send((
                RequestId::new(9),
                "textDocument/completion".into(),
                Some(uri),
            ))
            .await
            .unwrap();

        tokio::spawn(reader_task(lang.clone(), client_read, pending_rx, event_tx));

        // Server sends error response.
        let json = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 9,
            "error": {
                "code": -32600,
                "message": "Invalid Request"
            }
        });
        let body = serde_json::to_vec(&json).unwrap();
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        server_write.write_all(header.as_bytes()).await.unwrap();
        server_write.write_all(&body).await.unwrap();
        server_write.flush().await.unwrap();
        drop(server_write);

        let event = event_rx.recv().await.unwrap();
        match event {
            LspEvent::Response {
                language_id,
                id,
                result,
            } => {
                assert_eq!(language_id, lang);
                assert_eq!(id, RequestId::new(9));
                assert!(result.is_err());
                let err = result.unwrap_err();
                assert_eq!(err.code, -32600);
            }
            _ => panic!("expected Response error, got {event:?}"),
        }
    }
}
