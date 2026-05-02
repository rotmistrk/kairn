use rusticle::interpreter::Interpreter;

#[test]
fn dict_literal() {
    let mut interp = Interpreter::new();
    interp.eval(r#"set d %{ name: "alice", age: 30 }"#).unwrap();
    let result = interp.eval("dict get $d name").unwrap();
    assert_eq!(result.as_str(), "alice");
}

#[test]
fn list_literal() {
    let mut interp = Interpreter::new();
    interp.eval("set l %[ 1, 2, 3 ]").unwrap();
    let result = interp.eval("llength $l").unwrap();
    assert_eq!(result.as_int().unwrap(), 3);
}

#[test]
fn nested_literals() {
    let mut interp = Interpreter::new();
    interp.eval(r#"set d %{ items: %[ "a", "b" ] }"#).unwrap();
    let result = interp.eval(r#"return $d("items")(1)"#).unwrap();
    assert_eq!(result.as_str(), "b");
}

#[test]
fn accessor_list_index() {
    let mut interp = Interpreter::new();
    let result = interp.eval("set l [list a b c]; return $l(1)").unwrap();
    assert_eq!(result.as_str(), "b");
}

#[test]
fn accessor_dict_key() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval(r#"set d [dict create name alice]; return $d(name)"#)
        .unwrap();
    assert_eq!(result.as_str(), "alice");
}

#[test]
fn accessor_dot_len() {
    let mut interp = Interpreter::new();
    let result = interp.eval("set l [list 1 2 3]; return $l.len").unwrap();
    assert_eq!(result.as_int().unwrap(), 3);
}

#[test]
fn pipe_operator() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval(r#""  hello  " | string trim | string toupper"#)
        .unwrap();
    assert_eq!(result.as_str(), "HELLO");
}

#[test]
fn optional_chain_missing() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval(r#"set d %{ name: "alice" }; return [$d(missing)? ?? "default"]"#)
        .unwrap();
    assert_eq!(result.as_str(), "default");
}

#[test]
fn heredoc_with_subst() {
    let mut interp = Interpreter::new();
    interp.eval("set name world").unwrap();
    let result = interp
        .eval("set msg <<END\nhello $name\nEND\nreturn $msg")
        .unwrap();
    assert_eq!(result.as_str().trim(), "hello world");
}

#[test]
fn heredoc_raw_no_subst() {
    let mut interp = Interpreter::new();
    interp.eval("set name world").unwrap();
    let result = interp
        .eval("set msg <<'END'\nhello $name\nEND\nreturn $msg")
        .unwrap();
    assert_eq!(result.as_str().trim(), "hello $name");
}

#[test]
fn comment_ignored() {
    let mut interp = Interpreter::new();
    interp.eval("# this is a comment\nset x 42").unwrap();
    let result = interp.eval("return $x").unwrap();
    assert_eq!(result.as_int().unwrap(), 42);
}

#[test]
fn semicolon_separator() {
    let mut interp = Interpreter::new();
    interp.eval("set x 1; set y 2").unwrap();
    assert_eq!(interp.eval("return $x").unwrap().as_int().unwrap(), 1);
    assert_eq!(interp.eval("return $y").unwrap().as_int().unwrap(), 2);
}

#[test]
fn trailing_comma_dict() {
    let mut interp = Interpreter::new();
    interp.eval("set d %{ a: 1, b: 2, }").unwrap();
    let result = interp.eval("dict get $d b").unwrap();
    assert_eq!(result.as_str(), "2");
}

#[test]
fn trailing_comma_list() {
    let mut interp = Interpreter::new();
    interp.eval("set l %[ 1, 2, 3, ]").unwrap();
    let result = interp.eval("llength $l").unwrap();
    assert_eq!(result.as_int().unwrap(), 3);
}

#[test]
fn accessor_dot_keys() {
    let mut interp = Interpreter::new();
    interp.eval("set d [dict create a 1 b 2]").unwrap();
    let result = interp.eval("return $d.keys").unwrap();
    let keys = result.as_str();
    assert!(keys.contains('a'));
    assert!(keys.contains('b'));
}

#[test]
fn accessor_dot_type() {
    let mut interp = Interpreter::new();
    let result = interp.eval("set x 42; return $x.type").unwrap();
    let ty = result.as_str();
    assert!(!ty.is_empty());
}
