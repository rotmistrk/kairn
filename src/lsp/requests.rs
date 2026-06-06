//! LSP request helpers — send definition, references, hover, completion, rename, code-action.

use serde_json::json;

use super::client::LspClient;

// Re-export parse functions and types from response_parse module.
pub use super::response_parse::{
    parse_code_actions, parse_completion, parse_hover, parse_locations, CompletionItem, CompletionKind, Location,
};
pub use super::text_edit::TextEdit;

/// Send `textDocument/definition` request. Returns request id.
pub fn goto_definition(client: &mut LspClient, uri: &str, line: u32, character: u32) -> u64 {
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": character }
    });
    client.send_request("textDocument/definition", params)
}

/// Send `textDocument/references` request. Returns request id.
pub fn find_references(client: &mut LspClient, uri: &str, line: u32, character: u32) -> u64 {
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": character },
        "context": { "includeDeclaration": true }
    });
    client.send_request("textDocument/references", params)
}

/// Send `textDocument/hover` request. Returns request id.
pub fn hover(client: &mut LspClient, uri: &str, line: u32, character: u32) -> u64 {
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": character }
    });
    client.send_request("textDocument/hover", params)
}

/// Send `textDocument/completion` request. Returns request id.
pub fn completion(client: &mut LspClient, uri: &str, line: u32, character: u32) -> u64 {
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": character }
    });
    client.send_request("textDocument/completion", params)
}

/// Send `textDocument/rename` request. Returns request id.
pub fn rename(client: &mut LspClient, uri: &str, line: u32, character: u32, new_name: &str) -> u64 {
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": character },
        "newName": new_name
    });
    client.send_request("textDocument/rename", params)
}

/// Send `textDocument/codeAction` request. Returns request id.
pub fn code_action(client: &mut LspClient, uri: &str, line: u32, character: u32) -> u64 {
    let params = json!({
        "textDocument": { "uri": uri },
        "range": {
            "start": { "line": line, "character": character },
            "end": { "line": line, "character": character }
        },
        "context": { "diagnostics": [] }
    });
    client.send_request("textDocument/codeAction", params)
}

/// Apply a workspace edit from a rename response. Returns number of files changed.
pub use super::workspace_edit::apply_workspace_edit;

/// Send `textDocument/formatting` request. Returns request id.
pub fn formatting(client: &mut LspClient, uri: &str, tab_size: u32) -> u64 {
    let params = json!({
        "textDocument": { "uri": uri },
        "options": { "tabSize": tab_size, "insertSpaces": true }
    });
    client.send_request("textDocument/formatting", params)
}

/// Send `textDocument/rangeFormatting` request. Returns request id.
pub fn range_formatting(client: &mut LspClient, uri: &str, start_line: u32, end_line: u32, tab_size: u32) -> u64 {
    let params = json!({
        "textDocument": { "uri": uri },
        "range": {
            "start": { "line": start_line, "character": 0 },
            "end": { "line": end_line, "character": 0 }
        },
        "options": { "tabSize": tab_size, "insertSpaces": true }
    });
    client.send_request("textDocument/rangeFormatting", params)
}

pub use super::signature_help::{parse_signature_help, signature_help, SignatureHelp};
