//! LSP capability negotiation and notification/request parameter builders.
//!
//! Provides reusable builders for LSP notification params (didOpen,
//! didChange, didSave, didClose) and request params (completion, hover,
//! definition, references, documentSymbol, formatting, rename).

use crate::buffer::TextChange;
use crate::lsp::types::{DocumentUri, DocumentVersion, LanguageId};

// ── Notification param builders ─────────────────────────────

/// Build `textDocument/didOpen` params.
pub fn did_open_params(
    uri: &DocumentUri,
    language_id: &LanguageId,
    version: DocumentVersion,
    text: &str,
) -> serde_json::Value {
    serde_json::json!({
        "textDocument": {
            "uri": uri.as_str(),
            "languageId": language_id.as_str(),
            "version": version.value(),
            "text": text,
        }
    })
}

/// Build `textDocument/didChange` params with incremental changes.
pub fn did_change_params(
    uri: &DocumentUri,
    version: DocumentVersion,
    changes: &[TextChange],
) -> serde_json::Value {
    let content_changes: Vec<serde_json::Value> = changes
        .iter()
        .map(|c| {
            serde_json::json!({
                "range": {
                    "start": {
                        "line": c.start_line,
                        "character": c.start_col,
                    },
                    "end": {
                        "line": c.end_line,
                        "character": c.end_col,
                    },
                },
                "text": c.new_text,
            })
        })
        .collect();

    serde_json::json!({
        "textDocument": {
            "uri": uri.as_str(),
            "version": version.value(),
        },
        "contentChanges": content_changes,
    })
}

/// Build `textDocument/didSave` params.
pub fn did_save_params(uri: &DocumentUri, text: Option<&str>) -> serde_json::Value {
    let mut params = serde_json::json!({
        "textDocument": {"uri": uri.as_str()}
    });
    if let Some(t) = text {
        params["text"] = serde_json::Value::String(t.to_string());
    }
    params
}

/// Build `textDocument/didClose` params.
pub fn did_close_params(uri: &DocumentUri) -> serde_json::Value {
    serde_json::json!({
        "textDocument": {"uri": uri.as_str()}
    })
}

// ── Request param builders ──────────────────────────────────

/// Build `textDocument/completion` params.
pub fn completion_params(uri: &DocumentUri, line: u32, character: u32) -> serde_json::Value {
    serde_json::json!({
        "textDocument": {"uri": uri.as_str()},
        "position": {"line": line, "character": character},
    })
}

/// Build `textDocument/hover` params.
pub fn hover_params(uri: &DocumentUri, line: u32, character: u32) -> serde_json::Value {
    serde_json::json!({
        "textDocument": {"uri": uri.as_str()},
        "position": {"line": line, "character": character},
    })
}

/// Build `textDocument/definition` params.
pub fn definition_params(uri: &DocumentUri, line: u32, character: u32) -> serde_json::Value {
    serde_json::json!({
        "textDocument": {"uri": uri.as_str()},
        "position": {"line": line, "character": character},
    })
}

/// Build `textDocument/references` params.
pub fn references_params(uri: &DocumentUri, line: u32, character: u32) -> serde_json::Value {
    serde_json::json!({
        "textDocument": {"uri": uri.as_str()},
        "position": {"line": line, "character": character},
        "context": {"includeDeclaration": true},
    })
}

/// Build `textDocument/documentSymbol` params.
pub fn document_symbol_params(uri: &DocumentUri) -> serde_json::Value {
    serde_json::json!({
        "textDocument": {"uri": uri.as_str()},
    })
}

/// Build `textDocument/formatting` params.
pub fn formatting_params(
    uri: &DocumentUri,
    tab_size: u32,
    insert_spaces: bool,
) -> serde_json::Value {
    serde_json::json!({
        "textDocument": {"uri": uri.as_str()},
        "options": {
            "tabSize": tab_size,
            "insertSpaces": insert_spaces,
        },
    })
}

/// Build `textDocument/rename` params.
pub fn rename_params(
    uri: &DocumentUri,
    line: u32,
    character: u32,
    new_name: &str,
) -> serde_json::Value {
    serde_json::json!({
        "textDocument": {"uri": uri.as_str()},
        "position": {"line": line, "character": character},
        "newName": new_name,
    })
}

/// Build `initialize` params.
pub fn initialize_params(
    root_uri: Option<&DocumentUri>,
    capabilities: serde_json::Value,
    init_options: Option<serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({
        "processId": std::process::id(),
        "rootUri": root_uri.map(|u| u.as_str()),
        "capabilities": capabilities,
        "initializationOptions": init_options,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn did_open_params_structure() {
        let uri = DocumentUri::from_path("/tmp/main.rs");
        let lang = LanguageId::new("rust");
        let ver = DocumentVersion::new(0);
        let params = did_open_params(&uri, &lang, ver, "fn main() {}");

        let td = &params["textDocument"];
        assert_eq!(td["uri"].as_str().unwrap(), uri.as_str());
        assert_eq!(td["languageId"].as_str().unwrap(), "rust");
        assert_eq!(td["version"].as_i64().unwrap(), 0);
        assert_eq!(td["text"].as_str().unwrap(), "fn main() {}");
    }

    #[test]
    fn did_change_params_with_changes() {
        let uri = DocumentUri::from_path("/tmp/main.rs");
        let ver = DocumentVersion::new(1);
        let changes = vec![TextChange {
            start_line: 0,
            start_col: 5,
            end_line: 0,
            end_col: 5,
            new_text: " beautiful".into(),
        }];
        let params = did_change_params(&uri, ver, &changes);

        assert_eq!(params["textDocument"]["version"].as_i64().unwrap(), 1);
        let cc = params["contentChanges"].as_array().unwrap();
        assert_eq!(cc.len(), 1);
        assert_eq!(cc[0]["range"]["start"]["character"].as_u64().unwrap(), 5);
        assert_eq!(cc[0]["text"].as_str().unwrap(), " beautiful");
    }

    #[test]
    fn did_save_params_with_text() {
        let uri = DocumentUri::from_path("/tmp/main.rs");
        let params = did_save_params(&uri, Some("content"));
        assert_eq!(params["text"].as_str().unwrap(), "content");
    }

    #[test]
    fn did_save_params_without_text() {
        let uri = DocumentUri::from_path("/tmp/main.rs");
        let params = did_save_params(&uri, None);
        assert!(params.get("text").is_none());
    }

    #[test]
    fn did_close_params_structure() {
        let uri = DocumentUri::from_path("/tmp/main.rs");
        let params = did_close_params(&uri);
        assert_eq!(
            params["textDocument"]["uri"].as_str().unwrap(),
            uri.as_str()
        );
    }

    #[test]
    fn completion_params_structure() {
        let uri = DocumentUri::from_path("/tmp/main.rs");
        let params = completion_params(&uri, 10, 5);
        assert_eq!(params["position"]["line"].as_u64().unwrap(), 10);
        assert_eq!(params["position"]["character"].as_u64().unwrap(), 5);
    }

    #[test]
    fn references_params_includes_declaration() {
        let uri = DocumentUri::from_path("/tmp/main.rs");
        let params = references_params(&uri, 1, 2);
        assert!(params["context"]["includeDeclaration"].as_bool().unwrap());
    }

    #[test]
    fn formatting_params_options() {
        let uri = DocumentUri::from_path("/tmp/main.rs");
        let params = formatting_params(&uri, 2, true);
        assert_eq!(params["options"]["tabSize"].as_u64().unwrap(), 2);
        assert!(params["options"]["insertSpaces"].as_bool().unwrap());
    }

    #[test]
    fn rename_params_structure() {
        let uri = DocumentUri::from_path("/tmp/main.rs");
        let params = rename_params(&uri, 5, 10, "new_name");
        assert_eq!(params["newName"].as_str().unwrap(), "new_name");
        assert_eq!(params["position"]["line"].as_u64().unwrap(), 5);
    }

    #[test]
    fn initialize_params_structure() {
        let root = DocumentUri::from_path("/tmp/project");
        let caps = serde_json::json!({"textDocument": {}});
        let params = initialize_params(Some(&root), caps, None);
        assert!(params["processId"].as_u64().is_some());
        assert_eq!(params["rootUri"].as_str().unwrap(), root.as_str());
    }
}
