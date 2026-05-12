use kairn::structured::jsonl_doc::JsonlDoc;
use kairn::structured::{NodeKind, ScalarType, StructuredDoc};

#[test]
fn parse_jsonl() {
    let input = r#"{"name":"alice","age":30}
{"name":"bob","age":25}
{"name":"carol","age":35}
"#;
    let doc = JsonlDoc::parse(input).unwrap();
    assert_eq!(doc.node_kind(doc.root()), NodeKind::Array);
    let children = doc.children(doc.root());
    assert_eq!(children.len(), 3);
    for &child in children {
        assert_eq!(doc.node_kind(child), NodeKind::Dict);
    }
}

#[test]
fn parse_jsonl_skips_empty_lines() {
    let input = r#"{"a":1}

{"b":2}

"#;
    let doc = JsonlDoc::parse(input).unwrap();
    assert_eq!(doc.children(doc.root()).len(), 2);
}

#[test]
fn serialize_jsonl() {
    let input = r#"{"name":"alice","age":30}
{"name":"bob","age":25}
"#;
    let doc = JsonlDoc::parse(input).unwrap();
    let output = doc.serialize();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2);
    // Each line should be valid compact JSON
    for line in &lines {
        let _: serde_json::Value = serde_json::from_str(line).unwrap();
    }
    // Verify no pretty-printing (no leading spaces)
    for line in &lines {
        assert!(!line.starts_with(' '));
    }
}

#[test]
fn edit_jsonl() {
    let input = r#"{"x":1}
{"x":2}
"#;
    let mut doc = JsonlDoc::parse(input).unwrap();
    let children = doc.children(doc.root()).to_vec();
    // First object's first child is "x": 1
    let first_obj_children = doc.children(children[0]).to_vec();
    let x_node = first_obj_children[0];
    assert_eq!(doc.value_display(x_node), "1");
    assert_eq!(doc.scalar_type(x_node), ScalarType::Number);
    doc.set_value(x_node, "99").unwrap();
    let output = doc.serialize();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("99"));
    assert!(lines[1].contains("2"));
}

#[test]
fn jsonl_scalar_lines() {
    let input = "1\n\"hello\"\ntrue\nnull\n";
    let doc = JsonlDoc::parse(input).unwrap();
    let children = doc.children(doc.root());
    assert_eq!(children.len(), 4);
    assert_eq!(doc.scalar_type(children[0]), ScalarType::Number);
    assert_eq!(doc.scalar_type(children[1]), ScalarType::String);
    assert_eq!(doc.scalar_type(children[2]), ScalarType::Bool);
    assert_eq!(doc.scalar_type(children[3]), ScalarType::Null);
}
