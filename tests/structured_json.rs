use kairn::structured::json_doc::JsonDoc;
use kairn::structured::{NodeKind, ScalarType, StructuredDoc};

#[test]
fn parse_simple_object() {
    let doc = JsonDoc::parse(r#"{"a":1,"b":"hello"}"#).unwrap();
    assert_eq!(doc.node_kind(doc.root()), NodeKind::Dict);
    let children = doc.children(doc.root());
    assert_eq!(children.len(), 2);
    assert_eq!(doc.key(children[0]), Some("a"));
    assert_eq!(doc.value_display(children[0]), "1");
    assert_eq!(doc.scalar_type(children[0]), ScalarType::Number);
    assert_eq!(doc.key(children[1]), Some("b"));
    assert_eq!(doc.value_display(children[1]), "hello");
    assert_eq!(doc.scalar_type(children[1]), ScalarType::String);
}

#[test]
fn parse_nested() {
    let doc = JsonDoc::parse(r#"{"obj":{"x":1},"arr":[2,3]}"#).unwrap();
    let children = doc.children(doc.root());
    assert_eq!(children.len(), 2);
    // serde_json uses BTreeMap, so keys are sorted: "arr" < "obj"
    let arr_node = children.iter().find(|&&c| doc.key(c) == Some("arr")).unwrap();
    let obj_node = children.iter().find(|&&c| doc.key(c) == Some("obj")).unwrap();
    assert_eq!(doc.node_kind(*obj_node), NodeKind::Dict);
    assert_eq!(doc.node_kind(*arr_node), NodeKind::Array);
    assert_eq!(doc.children(*obj_node).len(), 1);
    assert_eq!(doc.children(*arr_node).len(), 2);
}

#[test]
fn parse_array() {
    let doc = JsonDoc::parse("[1,2,3]").unwrap();
    assert_eq!(doc.node_kind(doc.root()), NodeKind::Array);
    let children = doc.children(doc.root());
    assert_eq!(children.len(), 3);
    for &c in children {
        assert_eq!(doc.node_kind(c), NodeKind::Scalar);
        assert_eq!(doc.scalar_type(c), ScalarType::Number);
    }
}

#[test]
fn edit_value() {
    let mut doc = JsonDoc::parse(r#"{"a":1}"#).unwrap();
    let child = doc.children(doc.root())[0];
    doc.set_value(child, "42").unwrap();
    assert_eq!(doc.value_display(child), "42");
}

#[test]
fn edit_key() {
    let mut doc = JsonDoc::parse(r#"{"a":1}"#).unwrap();
    let child = doc.children(doc.root())[0];
    doc.set_key(child, "renamed").unwrap();
    assert_eq!(doc.key(child), Some("renamed"));
}

#[test]
fn add_sibling_to_dict() {
    let mut doc = JsonDoc::parse(r#"{"a":1}"#).unwrap();
    let child = doc.children(doc.root())[0];
    let new_id = doc.add_sibling(child).unwrap();
    assert_eq!(doc.children(doc.root()).len(), 2);
    assert_eq!(doc.node_kind(new_id), NodeKind::Scalar);
    assert_eq!(doc.value_display(new_id), "null");
}

#[test]
fn add_sibling_to_array() {
    let mut doc = JsonDoc::parse("[1,2]").unwrap();
    let child = doc.children(doc.root())[0];
    let new_id = doc.add_sibling(child).unwrap();
    assert_eq!(doc.children(doc.root()).len(), 3);
    assert_eq!(doc.value_display(new_id), "null");
}

#[test]
fn add_child() {
    let mut doc = JsonDoc::parse(r#"{"a":1}"#).unwrap();
    let new_id = doc.add_child(doc.root()).unwrap();
    assert_eq!(doc.children(doc.root()).len(), 2);
    assert_eq!(doc.node_kind(new_id), NodeKind::Scalar);
    // Cannot add child to scalar
    let child = doc.children(doc.root())[0];
    assert!(doc.add_child(child).is_err());
}

#[test]
fn remove_node() {
    let mut doc = JsonDoc::parse(r#"{"a":1,"b":2}"#).unwrap();
    let child = doc.children(doc.root())[0];
    doc.remove(child).unwrap();
    assert_eq!(doc.children(doc.root()).len(), 1);
}

#[test]
fn clone_in_array() {
    let mut doc = JsonDoc::parse("[1,2,3]").unwrap();
    let child = doc.children(doc.root())[0];
    let cloned = doc.clone_node(child).unwrap();
    assert_eq!(doc.children(doc.root()).len(), 4);
    assert_eq!(doc.value_display(cloned), "1");
}

#[test]
fn swap_up_down() {
    let mut doc = JsonDoc::parse("[1,2,3]").unwrap();
    let children = doc.children(doc.root()).to_vec();
    // Swap second element up
    doc.swap_up(children[1]).unwrap();
    let new_children = doc.children(doc.root());
    assert_eq!(doc.value_display(new_children[0]), "2");
    assert_eq!(doc.value_display(new_children[1]), "1");
    // Swap it back down
    doc.swap_down(new_children[0]).unwrap();
    let final_children = doc.children(doc.root());
    assert_eq!(doc.value_display(final_children[0]), "1");
    assert_eq!(doc.value_display(final_children[1]), "2");
}

#[test]
fn cycle_type() {
    let mut doc = JsonDoc::parse(r#"{"a":"hello"}"#).unwrap();
    let child = doc.children(doc.root())[0];
    assert_eq!(doc.scalar_type(child), ScalarType::String);
    doc.cycle_type(child);
    assert_eq!(doc.scalar_type(child), ScalarType::Number);
    doc.cycle_type(child);
    assert_eq!(doc.scalar_type(child), ScalarType::Bool);
    doc.cycle_type(child);
    assert_eq!(doc.scalar_type(child), ScalarType::Null);
    doc.cycle_type(child);
    assert_eq!(doc.scalar_type(child), ScalarType::String);
}

#[test]
fn convert_container() {
    let mut doc = JsonDoc::parse(r#"{"a":1,"b":2}"#).unwrap();
    assert_eq!(doc.node_kind(doc.root()), NodeKind::Dict);
    doc.convert_container(doc.root());
    assert_eq!(doc.node_kind(doc.root()), NodeKind::Array);
    // Keys should be stripped
    let children = doc.children(doc.root());
    assert_eq!(doc.key(children[0]), None);
    // Convert back
    doc.convert_container(doc.root());
    assert_eq!(doc.node_kind(doc.root()), NodeKind::Dict);
    let children = doc.children(doc.root());
    assert!(doc.key(children[0]).is_some());
}

#[test]
fn serialize_roundtrip() {
    let input = r#"{"name":"test","count":42,"active":true,"data":null,"items":[1,2,3]}"#;
    let doc = JsonDoc::parse(input).unwrap();
    let output = doc.serialize();
    let doc2 = JsonDoc::parse(&output).unwrap();
    // Verify structure is equivalent
    assert_eq!(doc.children(doc.root()).len(), doc2.children(doc2.root()).len());
    assert_eq!(doc.node_kind(doc.root()), doc2.node_kind(doc2.root()));
}

#[test]
fn inline_hint() {
    let mut doc = JsonDoc::parse("[1,2,3]").unwrap();
    assert!(!doc.is_inline(doc.root()));
    doc.toggle_inline(doc.root());
    assert!(doc.is_inline(doc.root()));
    let output = doc.serialize();
    // Inline array should be on one line (no newlines inside brackets)
    assert!(output.trim().starts_with('['));
    assert!(!output.trim().contains('\n'));
}
