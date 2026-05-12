use kairn::structured::json_doc::JsonDoc;
use kairn::structured::{NodeKind, StructuredDoc};

#[test]
fn parse_jsonc_line_comments() {
    let input = r#"{
  // This is the name
  "name": "test",
  // This is the count
  "count": 42
}"#;
    let doc = JsonDoc::parse_jsonc(input).unwrap();
    assert_eq!(doc.node_kind(doc.root()), NodeKind::Dict);
    let children = doc.children(doc.root());
    assert_eq!(children.len(), 2);
    // Comments should be attached to the following nodes as meta
    let has_comment = children.iter().any(|&c| doc.meta(c).contains("This is the name"));
    assert!(
        has_comment,
        "Expected comment in meta, got: {:?}",
        children.iter().map(|&c| doc.meta(c)).collect::<Vec<_>>()
    );
}

#[test]
fn parse_jsonc_block_comments() {
    let input = r#"{
  /* server config */
  "host": "localhost",
  /* port number */
  "port": 8080
}"#;
    let doc = JsonDoc::parse_jsonc(input).unwrap();
    let children = doc.children(doc.root());
    assert_eq!(children.len(), 2);
    let has_comment = children.iter().any(|&c| doc.meta(c).contains("server config"));
    assert!(has_comment, "Expected block comment in meta");
}

#[test]
fn parse_jsonc_comment_in_string_ignored() {
    let input = r#"{
  "url": "http://example.com",
  "note": "use // for comments"
}"#;
    let doc = JsonDoc::parse_jsonc(input).unwrap();
    let children = doc.children(doc.root());
    // "// for comments" inside a string should NOT be stripped
    let note_node = children.iter().find(|&&c| doc.key(c) == Some("note")).unwrap();
    assert_eq!(doc.value_display(*note_node), "use // for comments");
}

#[test]
fn serialize_jsonc_preserves_comments() {
    let input = r#"{
  // greeting
  "hello": "world",
  // number
  "n": 1
}"#;
    let mut doc = JsonDoc::parse_jsonc(input).unwrap();
    // Edit a value
    let children = doc.children(doc.root()).to_vec();
    let hello_node = children.iter().find(|&&c| doc.key(c) == Some("hello")).unwrap();
    doc.set_value(*hello_node, "earth").unwrap();
    let output = doc.serialize();
    // Comments should still be present
    assert!(output.contains("// greeting"), "Output:\n{output}");
    assert!(output.contains("// number"), "Output:\n{output}");
    // Edited value should be there
    assert!(output.contains("\"earth\""), "Output:\n{output}");
}

#[test]
fn parse_jsonc_no_comments() {
    // Plain JSON should work fine through parse_jsonc
    let input = r#"{"a": 1, "b": 2}"#;
    let doc = JsonDoc::parse_jsonc(input).unwrap();
    assert_eq!(doc.children(doc.root()).len(), 2);
}
