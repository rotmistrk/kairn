//! Completion response parsing — split from response_parse.rs for file length.

use serde_json::Value;

use super::response_parse::{CompletionItem, CompletionKind};
use super::text_edit::TextEdit;

pub fn parse_completion(result: &Value) -> Vec<CompletionItem> {
    let items = if let Some(arr) = result.as_array() {
        arr
    } else if let Some(arr) = result.get("items").and_then(|v| v.as_array()) {
        arr
    } else {
        return Vec::new();
    };
    let mut parsed: Vec<_> = items.iter().filter_map(parse_one_completion).collect();
    parsed.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
    parsed
}

fn parse_one_completion(val: &Value) -> Option<CompletionItem> {
    let label_val = val.get("label")?;
    let label_str = label_val.as_str()?;
    let label = label_str.to_string();
    let detail = val.get("detail").and_then(|v| v.as_str()).map(|s| s.to_string());
    let insert_text = val.get("insertText").and_then(|v| v.as_str()).map(|s| s.to_string());
    let kind = match val.get("kind").and_then(|v| v.as_u64()) {
        Some(2) => CompletionKind::Method,
        Some(3) => CompletionKind::Function,
        _ => CompletionKind::Other,
    };
    let additional_edits = parse_additional_edits(val);
    let sort_key = val
        .get("sortText")
        .and_then(|v| v.as_str())
        .unwrap_or(&label)
        .to_string();
    Some(CompletionItem {
        label,
        detail,
        insert_text,
        kind,
        additional_edits,
        sort_key,
    })
}

fn parse_additional_edits(val: &Value) -> Vec<TextEdit> {
    let Some(arr) = val.get("additionalTextEdits").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter().filter_map(parse_one_edit).collect()
}

fn parse_one_edit(e: &Value) -> Option<TextEdit> {
    let range = e.get("range")?;
    let start = range.get("start")?;
    let end = range.get("end")?;
    let sl = start.get("line")?;
    let start_line = sl.as_u64()? as u32;
    let sc = start.get("character")?;
    let start_col = sc.as_u64()? as u32;
    let el = end.get("line")?;
    let end_line = el.as_u64()? as u32;
    let ec = end.get("character")?;
    let end_col = ec.as_u64()? as u32;
    let nt = e.get("newText")?;
    let new_text_str = nt.as_str()?;
    let new_text = new_text_str.to_string();
    Some(TextEdit {
        start_line,
        start_col,
        end_line,
        end_col,
        new_text,
    })
}
