//! Apply LSP workspace edits to files on disk.

use serde_json::Value;

use super::uri;

/// Apply a workspace edit from a rename response. Returns number of files changed.
pub fn apply_workspace_edit(result: &Value) -> usize {
    let mut files_changed = 0;
    if let Some(map) = result.get("changes").and_then(|v| v.as_object()) {
        for (uri, edits) in map {
            let path = uri_to_path(uri);
            if apply_text_edits(&path, edits) {
                files_changed += 1;
            }
        }
    } else if let Some(arr) = result.get("documentChanges").and_then(|v| v.as_array()) {
        for doc_change in arr {
            if apply_doc_change(doc_change) {
                files_changed += 1;
            }
        }
    }
    files_changed
}

fn apply_doc_change(doc_change: &Value) -> bool {
    if let Some(kind) = doc_change.get("kind").and_then(|k| k.as_str()) {
        return super::resource_ops::apply_resource_op(kind, doc_change);
    }
    let uri = doc_change
        .get("textDocument")
        .and_then(|td| td.get("uri"))
        .and_then(|u| u.as_str())
        .unwrap_or("");
    let path = uri_to_path(uri);
    let edits = doc_change.get("edits").unwrap_or(&Value::Null);
    apply_text_edits(&path, edits)
}

fn uri_to_path(u: &str) -> String {
    uri::uri_to_path(u)
}

/// Apply text edits to a single file. Returns true on success.
fn apply_text_edits(path: &str, edits_val: &Value) -> bool {
    let Some(edits) = edits_val.as_array() else {
        return false;
    };
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let lines: Vec<&str> = content.lines().collect();

    // Parse edits and sort in reverse order (apply from bottom to top)
    let mut parsed: Vec<(usize, usize, usize, usize, &str)> = edits
        .iter()
        .filter_map(|e| {
            let range = e.get("range")?;
            let start = range.get("start")?;
            let end = range.get("end")?;
            let sl = start.get("line")?.as_u64()? as usize;
            let sc = start.get("character")?.as_u64()? as usize;
            let el = end.get("line")?.as_u64()? as usize;
            let ec = end.get("character")?.as_u64()? as usize;
            let new_text = e.get("newText")?.as_str()?;
            Some((sl, sc, el, ec, new_text))
        })
        .collect();
    parsed.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));

    // Convert to byte offsets and apply
    let mut result = content.clone();
    for (sl, sc, el, ec, new_text) in &parsed {
        let start_byte = line_col_to_byte(&lines, *sl, *sc);
        let end_byte = line_col_to_byte(&lines, *el, *ec);
        if start_byte <= end_byte && end_byte <= result.len() {
            result.replace_range(start_byte..end_byte, new_text);
        }
    }
    if let Err(e) = std::fs::write(path, &result) {
        log::error!("LSP rename: failed to write {path}: {e}");
        return false;
    }
    true
}

/// Convert line/col (0-indexed) to byte offset.
fn line_col_to_byte(lines: &[&str], line: usize, col: usize) -> usize {
    let mut offset = 0;
    for (i, l) in lines.iter().enumerate() {
        if i == line {
            return offset + col.min(l.len());
        }
        offset += l.len() + 1; // +1 for newline
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;

    #[test]
    fn apply_simple_rename() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("test.rs");
        {
            let mut f = std::fs::File::create(&file).expect("create");
            writeln!(f, "fn foo() {{}}").expect("write");
            writeln!(f, "fn main() {{ foo(); }}").expect("write");
        }
        let uri = format!("file://{}", file.display());
        let edit = json!({
            "changes": {
                uri: [
                    {"range": {"start": {"line": 0, "character": 3},
                        "end": {"line": 0, "character": 6}}, "newText": "bar"},
                    {"range": {"start": {"line": 1, "character": 12},
                        "end": {"line": 1, "character": 15}}, "newText": "bar"}
                ]
            }
        });
        let count = apply_workspace_edit(&edit);
        assert_eq!(count, 1);
        let content = std::fs::read_to_string(&file).expect("read");
        assert!(content.contains("fn bar()"));
        assert!(content.contains("bar();"));
    }

    #[test]
    fn apply_document_changes_format() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("lib.rs");
        std::fs::write(&file, "let x = 1;\n").expect("write");
        let uri = format!("file://{}", file.display());
        let edit = json!({
            "documentChanges": [{
                "textDocument": {"uri": uri, "version": 1},
                "edits": [
                    {"range": {"start": {"line": 0, "character": 4},
                        "end": {"line": 0, "character": 5}}, "newText": "y"}
                ]
            }]
        });
        let count = apply_workspace_edit(&edit);
        assert_eq!(count, 1);
        let content = std::fs::read_to_string(&file).expect("read");
        assert!(content.contains("let y = 1;"));
    }

    #[test]
    fn empty_edit_returns_zero() {
        let count = apply_workspace_edit(&json!({}));
        assert_eq!(count, 0);
    }

    #[test]
    fn rename_file_operation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let old_file = dir.path().join("Old.java");
        std::fs::write(&old_file, "class Old {}").expect("write");
        let old_uri = format!("file://{}", old_file.display());
        let new_file = dir.path().join("New.java");
        let new_uri = format!("file://{}", new_file.display());
        let edit = json!({
            "documentChanges": [
                {"kind": "rename", "oldUri": old_uri, "newUri": new_uri}
            ]
        });
        let count = apply_workspace_edit(&edit);
        assert_eq!(count, 1);
        assert!(!old_file.exists(), "old file should be gone");
        assert!(new_file.exists(), "new file should exist");
        let content = std::fs::read_to_string(&new_file).expect("read");
        assert_eq!(content, "class Old {}");
    }

    #[test]
    fn create_file_operation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let new_file = dir.path().join("sub").join("Created.java");
        let uri = format!("file://{}", new_file.display());
        let edit = json!({
            "documentChanges": [
                {"kind": "create", "uri": uri}
            ]
        });
        let count = apply_workspace_edit(&edit);
        assert_eq!(count, 1);
        assert!(new_file.exists(), "file should be created");
    }

    #[test]
    fn delete_file_operation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("doomed.txt");
        std::fs::write(&file, "bye").expect("write");
        let uri = format!("file://{}", file.display());
        let edit = json!({
            "documentChanges": [
                {"kind": "delete", "uri": uri}
            ]
        });
        let count = apply_workspace_edit(&edit);
        assert_eq!(count, 1);
        assert!(!file.exists(), "file should be deleted");
    }

    #[test]
    fn mixed_text_edits_and_rename() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("Foo.java");
        std::fs::write(&file, "class Foo {}\n").expect("write");
        let uri = format!("file://{}", file.display());
        let new_file = dir.path().join("Bar.java");
        let new_uri = format!("file://{}", new_file.display());
        let edit = json!({
            "documentChanges": [
                {
                    "textDocument": {"uri": uri, "version": 1},
                    "edits": [
                        {"range": {"start": {"line": 0, "character": 6},
                            "end": {"line": 0, "character": 9}}, "newText": "Bar"}
                    ]
                },
                {"kind": "rename", "oldUri": uri, "newUri": new_uri}
            ]
        });
        let count = apply_workspace_edit(&edit);
        assert_eq!(count, 2);
        assert!(!file.exists(), "old file should be renamed");
        assert!(new_file.exists(), "new file should exist");
        let content = std::fs::read_to_string(&new_file).expect("read");
        assert!(content.contains("class Bar {}"));
    }
}
