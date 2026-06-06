//! LSP response parsing — completion, locations, hover, code actions.

use serde_json::Value;

use super::text_edit::TextEdit;

/// A completion item from the server.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub(crate) label: String,
    pub(crate) detail: Option<String>,
    pub(crate) insert_text: Option<String>,
    pub(crate) kind: CompletionKind,
    pub(crate) additional_edits: Vec<TextEdit>,
}

impl CompletionItem {
    pub fn new(
        label: impl Into<String>,
        detail: Option<String>,
        insert_text: Option<String>,
        kind: CompletionKind,
    ) -> Self {
        Self {
            label: label.into(),
            detail,
            insert_text,
            kind,
            additional_edits: Vec::new(),
        }
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
    pub fn insert_text(&self) -> Option<&str> {
        self.insert_text.as_deref()
    }
    pub fn kind(&self) -> CompletionKind {
        self.kind
    }
    pub fn additional_edits(&self) -> &[TextEdit] {
        &self.additional_edits
    }
}

/// LSP completion item kind (subset we care about).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Function,
    Method,
    Other,
}

pub use super::location::Location;

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
    let kind = match val.get("kind").and_then(|v| v.as_u64()) {
        Some(2) => CompletionKind::Method,
        Some(3) => CompletionKind::Function,
        _ => CompletionKind::Other,
    };
    let additional_edits = val
        .get("additionalTextEdits")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    let range = e.get("range")?;
                    let start = range.get("start")?;
                    let end = range.get("end")?;
                    Some(TextEdit {
                        start_line: start.get("line")?.as_u64()? as u32,
                        start_col: start.get("character")?.as_u64()? as u32,
                        end_line: end.get("line")?.as_u64()? as u32,
                        end_col: end.get("character")?.as_u64()? as u32,
                        new_text: e.get("newText")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Some(CompletionItem {
        label,
        detail,
        insert_text,
        kind,
        additional_edits,
    })
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
        let _ = obj;
        return contents.get("value").and_then(|v| v.as_str()).map(|s| s.to_string());
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

/// Parse code actions response into action titles.
pub fn parse_code_actions(result: &Value) -> Vec<String> {
    match result.as_array() {
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
            {"uri": "file:///a.rs", "range": {
                "start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 5}
            }},
            {"uri": "file:///b.rs", "range": {
                "start": {"line": 2, "character": 3}, "end": {"line": 2, "character": 7}
            }}
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
