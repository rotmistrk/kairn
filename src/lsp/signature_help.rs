//! LSP signatureHelp request and response parsing.

use serde_json::{json, Value};

use super::client::LspClient;

/// Send a textDocument/signatureHelp request.
pub fn signature_help(client: &mut LspClient, uri: &str, line: u32, character: u32) -> u64 {
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": character }
    });
    client.send_request("textDocument/signatureHelp", params)
}

/// Parsed signature help result.
#[derive(Debug, Clone)]
pub struct SignatureHelp {
    pub(crate) label: String,
    pub(crate) active_param: Option<usize>,
    pub(crate) params: Vec<(usize, usize)>,
    pub(crate) documentation: Option<String>,
}

/// Parse a signatureHelp response.
pub fn parse_signature_help(result: &Value) -> Option<SignatureHelp> {
    let sigs = result.get("signatures")?.as_array()?;
    let active_sig = result.get("activeSignature").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let sig = sigs.get(active_sig)?;
    let label = sig.get("label")?.as_str()?.to_string();
    let active_param = result
        .get("activeParameter")
        .and_then(|v| v.as_u64())
        .or_else(|| sig.get("activeParameter").and_then(|v| v.as_u64()))
        .map(|v| v as usize);
    let params = sig
        .get("parameters")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let lbl = p.get("label")?;
                    if let Some(arr) = lbl.as_array() {
                        let start = arr.first()?.as_u64()? as usize;
                        let end = arr.get(1)?.as_u64()? as usize;
                        Some((start, end))
                    } else if let Some(s) = lbl.as_str() {
                        let start = label.find(s)?;
                        Some((start, start + s.len()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let documentation = sig.get("documentation").and_then(|d| {
        d.as_str()
            .map(|s| s.to_string())
            .or_else(|| d.get("value").and_then(|v| v.as_str()).map(|s| s.to_string()))
    });
    Some(SignatureHelp {
        label,
        active_param,
        params,
        documentation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_signature() {
        let result = json!({
            "signatures": [{
                "label": "fn insert(key: String, value: usize)",
                "parameters": [
                    {"label": [10, 22]},
                    {"label": [24, 37]}
                ]
            }],
            "activeSignature": 0,
            "activeParameter": 0
        });
        let sig = parse_signature_help(&result).unwrap();
        assert_eq!(sig.label, "fn insert(key: String, value: usize)");
        assert_eq!(sig.active_param, Some(0));
        assert_eq!(sig.params.len(), 2);
        assert_eq!(sig.params[0], (10, 22));
    }

    #[test]
    fn parse_string_param_labels() {
        let result = json!({
            "signatures": [{
                "label": "fn foo(a: i32, b: bool)",
                "parameters": [
                    {"label": "a: i32"},
                    {"label": "b: bool"}
                ]
            }],
            "activeParameter": 1
        });
        let sig = parse_signature_help(&result).unwrap();
        assert_eq!(sig.active_param, Some(1));
        assert_eq!(sig.params[1], (15, 22));
    }

    #[test]
    fn parse_null_returns_none() {
        assert!(parse_signature_help(&json!(null)).is_none());
    }
}
