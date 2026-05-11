//! LSP request helpers — definition, references, hover.

use serde_json::{json, Value};

use super::client::LspClient;

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

/// A completion item from the server.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
}

/// Parse a completion response into items.
pub fn parse_completion(result: &Value) -> Vec<CompletionItem> {
    let items = if let Some(arr) = result.as_array() {
        arr
    } else if let Some(arr) = result.get("items").and_then(|v| v.as_array()) {
        arr
    } else {
        return Vec::new();
    };
    items.iter().filter_map(parse_one_completion).collect()
}

fn parse_one_completion(val: &Value) -> Option<CompletionItem> {
    let label = val.get("label")?.as_str()?.to_string();
    let detail = val.get("detail").and_then(|v| v.as_str()).map(|s| s.to_string());
    let insert_text = val.get("insertText").and_then(|v| v.as_str()).map(|s| s.to_string());
    Some(CompletionItem {
        label,
        detail,
        insert_text,
    })
}

/// A location result from definition/references responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub uri: String,
    pub line: u32,
    pub character: u32,
}

/// Parse a definition/references response into locations.
pub fn parse_locations(result: &Value) -> Vec<Location> {
    if let Some(obj) = result.as_object() {
        if let Some(loc) = parse_one_location(result) {
            return vec![loc];
        }
        let _ = obj;
    }
    if let Some(arr) = result.as_array() {
        return arr.iter().filter_map(parse_one_location).collect();
    }
    Vec::new()
}

fn parse_one_location(val: &Value) -> Option<Location> {
    let uri = val.get("uri")?.as_str()?.to_string();
    let range = val.get("range")?;
    let start = range.get("start")?;
    let line = start.get("line")?.as_u64()? as u32;
    let character = start.get("character")?.as_u64()? as u32;
    Some(Location { uri, line, character })
}

/// Parse a hover response into plain text content.
pub fn parse_hover(result: &Value) -> Option<String> {
    let contents = result.get("contents")?;
    if let Some(s) = contents.as_str() {
        return Some(s.to_string());
    }
    if let Some(obj) = contents.as_object() {
        return obj.get("value").and_then(|v| v.as_str()).map(|s| s.to_string());
    }
    if let Some(arr) = contents.as_array() {
        let parts: Vec<String> = arr
            .iter()
            .filter_map(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .or_else(|| v.get("value").and_then(|x| x.as_str()).map(|s| s.to_string()))
            })
            .collect();
        if parts.is_empty() {
            return None;
        }
        return Some(parts.join("\n"));
    }
    None
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

/// Parse code actions response into action titles.
pub fn parse_code_actions(result: &Value) -> Vec<String> {
    let arr = result.as_array();
    match arr {
        Some(items) => items
            .iter()
            .filter_map(|v| v.get("title").and_then(|t| t.as_str()).map(|s| s.to_string()))
            .collect(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_single_location() {
        let result = json!({
            "uri": "file:///src/lib.rs",
            "range": {"start": {"line": 10, "character": 4}, "end": {"line": 10, "character": 8}}
        });
        let locs = parse_locations(&result);
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0].uri, "file:///src/lib.rs");
        assert_eq!(locs[0].line, 10);
        assert_eq!(locs[0].character, 4);
    }

    #[test]
    fn parse_location_array() {
        let result = json!([
            {"uri": "file:///a.rs", "range": {"start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 5}}},
            {"uri": "file:///b.rs", "range": {"start": {"line": 2, "character": 3}, "end": {"line": 2, "character": 7}}}
        ]);
        let locs = parse_locations(&result);
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[1].uri, "file:///b.rs");
    }

    #[test]
    fn parse_null_result() {
        let locs = parse_locations(&json!(null));
        assert!(locs.is_empty());
    }

    #[test]
    fn parse_hover_string() {
        let result = json!({"contents": "fn main()"});
        assert_eq!(parse_hover(&result), Some("fn main()".to_string()));
    }

    #[test]
    fn parse_hover_markup() {
        let result = json!({"contents": {"kind": "plaintext", "value": "pub fn foo()"}});
        assert_eq!(parse_hover(&result), Some("pub fn foo()".to_string()));
    }

    #[test]
    fn parse_hover_null() {
        assert_eq!(parse_hover(&json!(null)), None);
    }

    #[test]
    fn parse_completion_array() {
        let result = json!([
            {"label": "println!", "detail": "macro"},
            {"label": "print!", "insertText": "print!($0)"}
        ]);
        let items = parse_completion(&result);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].label, "println!");
        assert_eq!(items[0].detail, Some("macro".to_string()));
        assert_eq!(items[1].insert_text, Some("print!($0)".to_string()));
    }

    #[test]
    fn parse_completion_list_object() {
        let result = json!({"isIncomplete": false, "items": [{"label": "foo"}]});
        let items = parse_completion(&result);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "foo");
    }

    #[test]
    fn parse_completion_empty() {
        let items = parse_completion(&json!(null));
        assert!(items.is_empty());
    }
}
