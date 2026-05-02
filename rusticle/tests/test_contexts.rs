use rusticle::interpreter::Interpreter;

#[test]
fn context_creation() {
    let mut interp = Interpreter::new();
    interp.eval(r#"context cfg { set name "kairn" }"#).unwrap();
    let result = interp.eval("return $cfg::name").unwrap();
    assert_eq!(result.as_str(), "kairn");
}

#[test]
fn context_multiple_vars() {
    let mut interp = Interpreter::new();
    interp
        .eval(r#"context app { set name "kairn"; set version 2 }"#)
        .unwrap();
    let name = interp.eval("return $app::name").unwrap();
    let ver = interp.eval("return $app::version").unwrap();
    assert_eq!(name.as_str(), "kairn");
    assert_eq!(ver.as_str(), "2");
}

#[test]
fn context_var_modification() {
    let mut interp = Interpreter::new();
    interp.eval(r#"context cfg { set val "old" }"#).unwrap();
    interp.eval(r#"set cfg::val "new""#).unwrap();
    let result = interp.eval("return $cfg::val").unwrap();
    assert_eq!(result.as_str(), "new");
}

#[test]
fn declare_enum_valid() {
    let mut interp = Interpreter::new();
    interp
        .eval("context st { declare color : enum {red green blue} }")
        .unwrap();
    interp.eval("set st::color red").unwrap();
    let result = interp.eval("return $st::color").unwrap();
    assert_eq!(result.as_str(), "red");
}

#[test]
fn declare_enum_invalid() {
    let mut interp = Interpreter::new();
    interp
        .eval("context st { declare color : enum {red green blue} }")
        .unwrap();
    assert!(interp.eval("set st::color yellow").is_err());
}

#[test]
fn declare_int_valid() {
    let mut interp = Interpreter::new();
    interp.eval("context nums { declare count : int }").unwrap();
    interp.eval("set nums::count 42").unwrap();
    let result = interp.eval("return $nums::count").unwrap();
    assert_eq!(result.as_str(), "42");
}

#[test]
fn declare_int_invalid() {
    let mut interp = Interpreter::new();
    interp.eval("context nums { declare count : int }").unwrap();
    assert!(interp.eval("set nums::count abc").is_err());
}

#[test]
fn declare_bool_valid() {
    let mut interp = Interpreter::new();
    interp
        .eval("context flags { declare active : bool }")
        .unwrap();
    interp.eval("set flags::active true").unwrap();
    let result = interp.eval("return $flags::active").unwrap();
    assert_eq!(result.as_str(), "true");
}

#[test]
fn declare_bool_invalid() {
    let mut interp = Interpreter::new();
    interp
        .eval("context flags { declare active : bool }")
        .unwrap();
    assert!(interp.eval("set flags::active maybe").is_err());
}
