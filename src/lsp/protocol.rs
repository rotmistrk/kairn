//! JSON-RPC 2.0 message types and Content-Length framing for LSP.

use std::io::{self, BufRead, Write};

use serde_json::Value;

/// Request ID — always numeric in kairn's client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(u64);

impl RequestId {
    /// Create a new request ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the numeric value.
    pub fn value(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Outgoing message from client to server.
#[derive(Debug, Clone)]
pub enum LspMessage {
    Request(LspRequest),
    Notification(LspNotification),
}

/// A request expects a response. Identified by a numeric ID.
#[derive(Debug, Clone)]
pub struct LspRequest {
    pub id: RequestId,
    pub method: String,
    pub params: Value,
}

/// A notification is fire-and-forget. No response expected.
#[derive(Debug, Clone)]
pub struct LspNotification {
    pub method: String,
    pub params: Value,
}

/// Incoming message from server to client.
#[derive(Debug, Clone)]
pub enum LspIncoming {
    Response(LspResponse),
    Notification(LspNotification),
    Request(LspRequest),
}

/// Response to a client request.
#[derive(Debug, Clone)]
pub struct LspResponse {
    pub id: RequestId,
    pub result: Option<Value>,
    pub error: Option<LspError>,
}

/// JSON-RPC error object.
#[derive(Debug, Clone)]
pub struct LspError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl std::fmt::Display for LspError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LSP error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for LspError {}

// ── Serialization ───────────────────────────────────────────

/// Serialize an outgoing message to JSON.
fn serialize_message(msg: &LspMessage) -> serde_json::Result<Vec<u8>> {
    let value = match msg {
        LspMessage::Request(req) => serde_json::json!({
            "jsonrpc": "2.0",
            "id": req.id.value(),
            "method": req.method,
            "params": req.params,
        }),
        LspMessage::Notification(notif) => serde_json::json!({
            "jsonrpc": "2.0",
            "method": notif.method,
            "params": notif.params,
        }),
    };
    serde_json::to_vec(&value)
}

/// Deserialize an incoming JSON object into an `LspIncoming`.
fn deserialize_incoming(value: Value) -> io::Result<LspIncoming> {
    let obj = value
        .as_object()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "expected JSON object"))?;

    // Response: has "id" and ("result" or "error"), no "method".
    if obj.contains_key("id") && !obj.contains_key("method") {
        return parse_response(obj);
    }

    let method = obj
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing method"))?
        .to_string();

    let params = obj.get("params").cloned().unwrap_or(Value::Null);

    // Request from server: has "id" and "method".
    if let Some(id_val) = obj.get("id") {
        let id = parse_id(id_val)?;
        return Ok(LspIncoming::Request(LspRequest { id, method, params }));
    }

    // Notification: has "method" but no "id".
    Ok(LspIncoming::Notification(LspNotification {
        method,
        params,
    }))
}

/// Parse a response object.
fn parse_response(obj: &serde_json::Map<String, Value>) -> io::Result<LspIncoming> {
    let id_val = obj
        .get("id")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing id"))?;
    let id = parse_id(id_val)?;

    let error = obj.get("error").and_then(|e| {
        let eo = e.as_object()?;
        Some(LspError {
            code: eo.get("code")?.as_i64()? as i32,
            message: eo.get("message")?.as_str()?.to_string(),
            data: eo.get("data").cloned(),
        })
    });

    let result = obj.get("result").cloned();

    Ok(LspIncoming::Response(LspResponse { id, result, error }))
}

/// Parse a JSON value as a request ID.
fn parse_id(val: &Value) -> io::Result<RequestId> {
    if let Some(n) = val.as_u64() {
        return Ok(RequestId::new(n));
    }
    if let Some(n) = val.as_i64() {
        return Ok(RequestId::new(n as u64));
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "request id must be numeric",
    ))
}

// ── Framing (synchronous, for tests and blocking I/O) ───────

/// Read one LSP message from a byte stream.
/// Parses Content-Length header, reads that many bytes, deserializes JSON.
pub fn read_message(reader: &mut impl BufRead) -> io::Result<LspIncoming> {
    let content_len = read_headers(reader)?;
    let mut body = vec![0u8; content_len];
    reader.read_exact(&mut body)?;
    let value: Value =
        serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    deserialize_incoming(value)
}

/// Write one LSP message to a byte stream.
/// Serializes JSON, prepends Content-Length header.
pub fn write_message(writer: &mut impl Write, msg: &LspMessage) -> io::Result<()> {
    let body = serialize_message(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

/// Parse headers and return the Content-Length value.
fn read_headers(reader: &mut impl BufRead) -> io::Result<usize> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF reading LSP headers",
            ));
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(val) = trimmed.strip_prefix("Content-Length:") {
            content_length = val.trim().parse().ok();
        }
        // Ignore other headers (e.g. Content-Type).
    }

    content_length
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length header"))
}

// ── Async framing (for transport tasks) ─────────────────────

/// Read one LSP message from an async byte stream.
pub async fn read_message_async(
    reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
) -> io::Result<LspIncoming> {
    use tokio::io::AsyncBufReadExt;
    use tokio::io::AsyncReadExt;

    let content_len = read_headers_async(reader).await?;
    let mut body = vec![0u8; content_len];
    reader.read_exact(&mut body).await?;
    let value: Value =
        serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    deserialize_incoming(value)
}

/// Write one LSP message to an async byte stream.
pub async fn write_message_async(
    writer: &mut (impl tokio::io::AsyncWrite + Unpin),
    msg: &LspMessage,
) -> io::Result<()> {
    use tokio::io::AsyncWriteExt;

    let body = serialize_message(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer.write_all(header.as_bytes()).await?;
    writer.write_all(&body).await?;
    writer.flush().await
}

/// Parse headers asynchronously and return Content-Length.
async fn read_headers_async(
    reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
) -> io::Result<usize> {
    use tokio::io::AsyncBufReadExt;

    let mut content_length: Option<usize> = None;
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF reading LSP headers",
            ));
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(val) = trimmed.strip_prefix("Content-Length:") {
            content_length = val.trim().parse().ok();
        }
    }

    content_length
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length header"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn framing_roundtrip_request() {
        let msg = LspMessage::Request(LspRequest {
            id: RequestId::new(1),
            method: "initialize".into(),
            params: serde_json::json!({"rootUri": "file:///tmp"}),
        });

        let mut buf = Vec::new();
        write_message(&mut buf, &msg).unwrap();

        let mut reader = Cursor::new(buf);
        let incoming = read_message(&mut reader).unwrap();

        match incoming {
            LspIncoming::Request(req) => {
                assert_eq!(req.id, RequestId::new(1));
                assert_eq!(req.method, "initialize");
            }
            _ => panic!("expected request"),
        }
    }

    #[test]
    fn framing_roundtrip_notification() {
        let msg = LspMessage::Notification(LspNotification {
            method: "initialized".into(),
            params: serde_json::json!({}),
        });

        let mut buf = Vec::new();
        write_message(&mut buf, &msg).unwrap();

        let mut reader = Cursor::new(buf);
        let incoming = read_message(&mut reader).unwrap();

        match incoming {
            LspIncoming::Notification(n) => {
                assert_eq!(n.method, "initialized");
            }
            _ => panic!("expected notification"),
        }
    }

    #[test]
    fn parse_response_with_result() {
        let json = r#"{"jsonrpc":"2.0","id":42,"result":{"capabilities":{}}}"#;
        let raw = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);
        let mut reader = Cursor::new(raw.as_bytes().to_vec());
        let incoming = read_message(&mut reader).unwrap();

        match incoming {
            LspIncoming::Response(resp) => {
                assert_eq!(resp.id, RequestId::new(42));
                assert!(resp.result.is_some());
                assert!(resp.error.is_none());
            }
            _ => panic!("expected response"),
        }
    }

    #[test]
    fn parse_response_with_error() {
        let json =
            r#"{"jsonrpc":"2.0","id":7,"error":{"code":-32600,"message":"Invalid Request"}}"#;
        let raw = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);
        let mut reader = Cursor::new(raw.as_bytes().to_vec());
        let incoming = read_message(&mut reader).unwrap();

        match incoming {
            LspIncoming::Response(resp) => {
                assert_eq!(resp.id, RequestId::new(7));
                assert!(resp.error.is_some());
                let err = resp.error.as_ref().unwrap();
                assert_eq!(err.code, -32600);
                assert_eq!(err.message, "Invalid Request");
            }
            _ => panic!("expected response"),
        }
    }

    #[test]
    fn malformed_header_returns_error() {
        let raw = b"garbage data without headers\n";
        let mut reader = Cursor::new(raw.to_vec());
        let result = read_message(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn eof_returns_error() {
        let raw = b"";
        let mut reader = Cursor::new(raw.to_vec());
        let result = read_message(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn multiple_messages_in_stream() {
        let msg1 = LspMessage::Notification(LspNotification {
            method: "first".into(),
            params: Value::Null,
        });
        let msg2 = LspMessage::Notification(LspNotification {
            method: "second".into(),
            params: Value::Null,
        });

        let mut buf = Vec::new();
        write_message(&mut buf, &msg1).unwrap();
        write_message(&mut buf, &msg2).unwrap();

        let mut reader = Cursor::new(buf);
        let in1 = read_message(&mut reader).unwrap();
        let in2 = read_message(&mut reader).unwrap();

        match (in1, in2) {
            (LspIncoming::Notification(n1), LspIncoming::Notification(n2)) => {
                assert_eq!(n1.method, "first");
                assert_eq!(n2.method, "second");
            }
            _ => panic!("expected two notifications"),
        }
    }

    #[test]
    fn request_id_display() {
        let id = RequestId::new(42);
        assert_eq!(format!("{id}"), "42");
    }

    #[test]
    fn server_request_parsed() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"workspace/configuration","params":{}}"#;
        let raw = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);
        let mut reader = Cursor::new(raw.as_bytes().to_vec());
        let incoming = read_message(&mut reader).unwrap();

        match incoming {
            LspIncoming::Request(req) => {
                assert_eq!(req.id, RequestId::new(1));
                assert_eq!(req.method, "workspace/configuration");
            }
            _ => panic!("expected server request"),
        }
    }
}
