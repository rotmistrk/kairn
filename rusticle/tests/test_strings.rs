use rusticle::interpreter::Interpreter;

#[test]
fn string_length() {
    let mut interp = Interpreter::new();
    let r = interp.eval("string length hello").unwrap();
    assert_eq!(r.as_int().unwrap(), 5);
}

#[test]
fn string_length_empty() {
    let mut interp = Interpreter::new();
    let r = interp.eval(r#"string length """#).unwrap();
    assert_eq!(r.as_int().unwrap(), 0);
}

#[test]
fn string_range() {
    let mut interp = Interpreter::new();
    let r = interp.eval("string range hello 1 3").unwrap();
    assert_eq!(r.as_str(), "ell");
}

#[test]
fn string_match_exact() {
    let mut interp = Interpreter::new();
    let r = interp.eval("string match hello hello").unwrap();
    assert_eq!(r.as_int().unwrap(), 1);
}

#[test]
fn string_match_glob_star() {
    let mut interp = Interpreter::new();
    let r = interp.eval(r#"string match "h*" hello"#).unwrap();
    assert_eq!(r.as_int().unwrap(), 1);
}

#[test]
fn string_match_no_match() {
    let mut interp = Interpreter::new();
    let r = interp.eval(r#"string match "x*" hello"#).unwrap();
    assert_eq!(r.as_int().unwrap(), 0);
}

#[test]
fn string_map_replace() {
    let mut interp = Interpreter::new();
    let r = interp
        .eval(r#"string map {hello goodbye} "hello world""#)
        .unwrap();
    assert_eq!(r.as_str(), "goodbye world");
}

#[test]
fn string_trim() {
    let mut interp = Interpreter::new();
    let r = interp.eval(r#"string trim "  hello  ""#).unwrap();
    assert_eq!(r.as_str(), "hello");
}

#[test]
fn string_tolower() {
    let mut interp = Interpreter::new();
    let r = interp.eval("string tolower HELLO").unwrap();
    assert_eq!(r.as_str(), "hello");
}

#[test]
fn string_toupper() {
    let mut interp = Interpreter::new();
    let r = interp.eval("string toupper hello").unwrap();
    assert_eq!(r.as_str(), "HELLO");
}

#[test]
fn string_first_found() {
    let mut interp = Interpreter::new();
    let r = interp.eval("string first ll hello").unwrap();
    assert_eq!(r.as_int().unwrap(), 2);
}

#[test]
fn string_first_not_found() {
    let mut interp = Interpreter::new();
    let r = interp.eval("string first xx hello").unwrap();
    assert_eq!(r.as_int().unwrap(), -1);
}

#[test]
fn format_string() {
    let mut interp = Interpreter::new();
    let r = interp.eval(r#"format "%s is %d" hello 42"#).unwrap();
    assert_eq!(r.as_str(), "hello is 42");
}

#[test]
fn split_by_comma() {
    let mut interp = Interpreter::new();
    interp.eval(r#"set parts [split "a,b,c" ","]"#).unwrap();
    let r = interp.eval("llength $parts").unwrap();
    assert_eq!(r.as_int().unwrap(), 3);
}

#[test]
fn join_with_separator() {
    let mut interp = Interpreter::new();
    let r = interp.eval(r#"join [list a b c] ", ""#).unwrap();
    assert_eq!(r.as_str(), "a, b, c");
}

#[test]
fn split_and_join_roundtrip() {
    let mut interp = Interpreter::new();
    let r = interp.eval(r#"join [split "a,b,c" ","] ",""#).unwrap();
    assert_eq!(r.as_str(), "a,b,c");
}
