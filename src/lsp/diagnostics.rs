//! LSP diagnostics — storage and parsing.

use serde_json::Value;

/// Severity level from LSP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// A single diagnostic (error/warning) at a location.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub(crate) line: usize,
    pub(crate) col_start: usize,
    pub(crate) col_end: usize,
    pub(crate) severity: Severity,
    pub(crate) message: String,
}

impl Diagnostic {
    pub fn new(line: usize, col_start: usize, col_end: usize, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            line,
            col_start,
            col_end,
            severity,
            message: message.into(),
        }
    }
}

pub use super::diagnostic_store::DiagnosticStore;

/// Parse a `textDocument/publishDiagnostics` notification params.
pub fn parse_publish_diagnostics(params: &Value) -> Option<(String, Vec<Diagnostic>)> {
    let uri_val = params.get("uri")?;
    let raw_uri = uri_val.as_str()?;
    let uri = super::uri::uri_to_path(raw_uri);
    let diags_raw = params.get("diagnostics")?;
    let diags_val = diags_raw.as_array()?;
    let diagnostics = diags_val.iter().filter_map(parse_one_diagnostic).collect();
    Some((uri, diagnostics))
}

fn parse_one_diagnostic(val: &Value) -> Option<Diagnostic> {
    let range = val.get("range")?;
    let start = range.get("start")?;
    let end = range.get("end")?;
    let line_val = start.get("line")?;
    let line = line_val.as_u64()? as usize;
    let col_start_val = start.get("character")?;
    let col_start = col_start_val.as_u64()? as usize;
    let col_end_val = end.get("character")?;
    let col_end = col_end_val.as_u64().unwrap_or(col_start as u64 + 1) as usize;
    let severity = match val.get("severity").and_then(|v| v.as_u64()) {
        Some(1) => Severity::Error,
        Some(2) => Severity::Warning,
        Some(3) => Severity::Info,
        Some(4) => Severity::Hint,
        _ => Severity::Error,
    };
    let msg_val = val.get("message")?;
    let msg_str = msg_val.as_str()?;
    let message = msg_str.to_string();
    Some(Diagnostic {
        line,
        col_start,
        col_end,
        severity,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_publish_diagnostics_basic() {
        let params = json!({
            "uri": "file:///src/main.rs",
            "diagnostics": [{
                "range": {
                    "start": {"line": 5, "character": 10},
                    "end": {"line": 5, "character": 15}
                },
                "severity": 1,
                "message": "expected `;`"
            }]
        });
        let (uri, diags) = parse_publish_diagnostics(&params).unwrap();
        assert_eq!(uri, "/src/main.rs");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].line, 5);
        assert_eq!(diags[0].col_start, 10);
        assert_eq!(diags[0].col_end, 15);
        assert_eq!(diags[0].severity, Severity::Error);
        assert_eq!(diags[0].message, "expected `;`");
    }

    #[test]
    fn parse_empty_diagnostics() {
        let params = json!({"uri": "file:///a.rs", "diagnostics": []});
        let (_, diags) = parse_publish_diagnostics(&params).unwrap();
        assert!(diags.is_empty());
    }

    #[test]
    fn parse_diagnostics_decodes_percent_encoded_uri() {
        let params = json!({
            "uri": "file:///home/user/kairn%2B%2B/src/main.cpp",
            "diagnostics": [{
                "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}},
                "severity": 1,
                "message": "err"
            }]
        });
        let (uri, _) = parse_publish_diagnostics(&params).unwrap();
        assert_eq!(uri, "/home/user/kairn++/src/main.cpp");
    }

    #[test]
    fn store_set_and_get() {
        let mut store = DiagnosticStore::new();
        let d = Diagnostic {
            line: 3,
            col_start: 0,
            col_end: 5,
            severity: Severity::Warning,
            message: "unused".into(),
        };
        store.set("file:///a.rs", vec![d]);
        assert_eq!(store.get("file:///a.rs").len(), 1);
        assert_eq!(store.get("file:///b.rs").len(), 0);
    }

    #[test]
    fn store_at_line() {
        let mut store = DiagnosticStore::new();
        store.set(
            "file:///a.rs",
            vec![
                Diagnostic {
                    line: 1,
                    col_start: 0,
                    col_end: 3,
                    severity: Severity::Error,
                    message: "err".into(),
                },
                Diagnostic {
                    line: 5,
                    col_start: 0,
                    col_end: 2,
                    severity: Severity::Warning,
                    message: "warn".into(),
                },
            ],
        );
        assert_eq!(store.at_line("file:///a.rs", 1).unwrap().message, "err");
        assert_eq!(store.at_line("file:///a.rs", 5).unwrap().message, "warn");
        assert!(store.at_line("file:///a.rs", 3).is_none());
    }

    #[test]
    fn parse_warning_severity() {
        let params = json!({
            "uri": "file:///x.rs",
            "diagnostics": [{
                "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}},
                "severity": 2,
                "message": "unused variable"
            }]
        });
        let (_, diags) = parse_publish_diagnostics(&params).unwrap();
        assert_eq!(diags[0].severity, Severity::Warning);
    }
}
