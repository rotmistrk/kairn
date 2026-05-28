//! LSP protocol helpers — initialize handshake and document sync.

use std::path::Path;

use serde_json::{json, Value};

use super::client::LspClient;

/// Send the `initialize` request. Returns the request id.
pub fn initialize(client: &mut LspClient, root_uri: &str) -> u64 {
    let params = json!({
        "processId": std::process::id(),
        "rootUri": root_uri,
        "capabilities": {
            "textDocument": {
                "synchronization": { "dynamicRegistration": false, "didSave": true },
                "completion": { "completionItem": { "snippetSupport": false } },
                "hover": { "contentFormat": ["plaintext"] },
                "definition": { "dynamicRegistration": false },
                "references": { "dynamicRegistration": false },
                "publishDiagnostics": { "relatedInformation": false }
            },
            "window": { "workDoneProgress": true }
        }
    });
    client.send_request("initialize", params)
}

/// Send the `initialized` notification (after receiving initialize response).
pub fn initialized(client: &mut LspClient) {
    log::info!("Sending initialized notification to server");
    client.send_notification("initialized", json!({}));
}

/// Send `textDocument/didOpen`.
pub fn did_open(client: &mut LspClient, uri: &str, language_id: &str, text: &str) {
    let params = json!({
        "textDocument": {
            "uri": uri,
            "languageId": language_id,
            "version": 1,
            "text": text
        }
    });
    client.send_notification("textDocument/didOpen", params);
}

/// Send `textDocument/didChange` (full sync — sends entire content).
pub fn did_change(client: &mut LspClient, uri: &str, version: i64, text: &str) {
    let params = json!({
        "textDocument": { "uri": uri, "version": version },
        "contentChanges": [{ "text": text }]
    });
    client.send_notification("textDocument/didChange", params);
}

/// Send `textDocument/didClose`.
pub fn did_close(client: &mut LspClient, uri: &str) {
    let params = json!({
        "textDocument": { "uri": uri }
    });
    client.send_notification("textDocument/didClose", params);
}

/// Convert a filesystem path to a file:// URI.
pub fn path_to_uri(path: &Path) -> String {
    let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    format!("file://{}", abs.display())
}

/// Infer language ID from file extension.
pub fn language_id(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => "rust",
        Some("ts") | Some("tsx") => "typescript",
        Some("js") | Some("jsx") => "javascript",
        Some("go") => "go",
        Some("py") => "python",
        Some("c") | Some("h") => "c",
        Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") => "cpp",
        Some("java") => "java",
        Some("rb") => "ruby",
        Some("lua") => "lua",
        Some("sh") | Some("bash") => "shellscript",
        Some("tcl") => "tcl",
        Some("toml") => "toml",
        Some("json") => "json",
        Some("yaml") | Some("yml") => "yaml",
        Some("md") => "markdown",
        _ => "plaintext",
    }
}

/// Parse the initialize response to extract server capabilities.
pub fn parse_capabilities(result: &Value) -> ServerCapabilities {
    let caps = result.get("capabilities").unwrap_or(result);
    ServerCapabilities {
        completion: caps.get("completionProvider").is_some(),
        hover: caps
            .get("hoverProvider")
            .map(|v| v.as_bool().unwrap_or(true))
            .unwrap_or(false),
        definition: caps
            .get("definitionProvider")
            .map(|v| v.as_bool().unwrap_or(true))
            .unwrap_or(false),
        references: caps
            .get("referencesProvider")
            .map(|v| v.as_bool().unwrap_or(true))
            .unwrap_or(false),
    }
}

/// Server capabilities extracted from initialize response.
#[derive(Debug, Default, Clone)]
pub struct ServerCapabilities {
    pub(crate) completion: bool,
    pub(crate) hover: bool,
    pub(crate) definition: bool,
    pub(crate) references: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_to_uri_format() {
        let uri = path_to_uri(Path::new("/tmp/test.rs"));
        assert!(uri.starts_with("file:///"));
        assert!(uri.contains("test.rs"));
    }

    #[test]
    fn language_id_rust() {
        assert_eq!(language_id(Path::new("main.rs")), "rust");
    }

    #[test]
    fn language_id_unknown() {
        assert_eq!(language_id(Path::new("file.xyz")), "plaintext");
    }

    #[test]
    fn parse_capabilities_basic() {
        let result = json!({
            "capabilities": {
                "completionProvider": {},
                "hoverProvider": true,
                "definitionProvider": true,
                "referencesProvider": true
            }
        });
        let caps = parse_capabilities(&result);
        assert!(caps.completion);
        assert!(caps.hover);
        assert!(caps.definition);
        assert!(caps.references);
    }

    #[test]
    fn parse_capabilities_empty() {
        let caps = parse_capabilities(&json!({}));
        assert!(!caps.completion);
        assert!(!caps.hover);
    }
}
