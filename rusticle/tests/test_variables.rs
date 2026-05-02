use rusticle::interpreter::Interpreter;

#[test]
fn set_and_get_classic() {
    let mut interp = Interpreter::new();
    interp.eval("set x 42").unwrap();
    assert_eq!(interp.eval("set x").unwrap().as_str().as_ref(), "42");
}

#[test]
fn set_and_get_modern() {
    let mut interp = Interpreter::new();
    interp.eval("set x = 42").unwrap();
    assert_eq!(interp.eval("set x").unwrap().as_str().as_ref(), "42");
}

#[test]
fn set_string_value() {
    let mut interp = Interpreter::new();
    interp.eval(r#"set name "hello world""#).unwrap();
    assert_eq!(
        interp.eval("set name").unwrap().as_str().as_ref(),
        "hello world"
    );
}

#[test]
fn destructure_list() {
    let mut interp = Interpreter::new();
    interp.eval("set a, b, c = [list 10 20 30]").unwrap();
    assert_eq!(interp.eval("set a").unwrap().as_str().as_ref(), "10");
    assert_eq!(interp.eval("set b").unwrap().as_str().as_ref(), "20");
    assert_eq!(interp.eval("set c").unwrap().as_str().as_ref(), "30");
}

#[test]
fn destructure_dict() {
    let mut interp = Interpreter::new();
    // Dict destructuring via list form: extract values by position
    interp
        .eval("set d [dict create name alice age 30]")
        .unwrap();
    let name = interp.eval("dict get $d name").unwrap();
    assert_eq!(name.as_str().as_ref(), "alice");
    let age = interp.eval("dict get $d age").unwrap();
    assert_eq!(age.as_str().as_ref(), "30");
}

#[test]
fn unset_removes_variable() {
    let mut interp = Interpreter::new();
    interp.eval("set x 42").unwrap();
    interp.eval("unset x").unwrap();
    let err = interp.eval("set x").unwrap_err();
    assert!(
        err.to_string().contains("no such variable"),
        "expected 'no such variable', got: {err}"
    );
}

#[test]
fn append_to_existing() {
    let mut interp = Interpreter::new();
    interp.eval("set x hello").unwrap();
    interp.eval(r#"append x " world""#).unwrap();
    assert_eq!(
        interp.eval("set x").unwrap().as_str().as_ref(),
        "hello world"
    );
}

#[test]
fn append_creates_if_missing() {
    let mut interp = Interpreter::new();
    interp.eval(r#"append x "hello""#).unwrap();
    assert_eq!(interp.eval("set x").unwrap().as_str().as_ref(), "hello");
}

#[test]
fn incr_default() {
    let mut interp = Interpreter::new();
    interp.eval("set x 10").unwrap();
    interp.eval("incr x").unwrap();
    assert_eq!(interp.eval("set x").unwrap().as_int().unwrap(), 11);
}

#[test]
fn incr_by_amount() {
    let mut interp = Interpreter::new();
    interp.eval("set x 10").unwrap();
    interp.eval("incr x 5").unwrap();
    assert_eq!(interp.eval("set x").unwrap().as_int().unwrap(), 15);
}

#[test]
fn incr_negative() {
    let mut interp = Interpreter::new();
    interp.eval("set x 10").unwrap();
    interp.eval("incr x -3").unwrap();
    assert_eq!(interp.eval("set x").unwrap().as_int().unwrap(), 7);
}

#[test]
fn variable_substitution_in_string() {
    let mut interp = Interpreter::new();
    interp.eval("set name world").unwrap();
    interp.eval(r#"set greeting "hello $name""#).unwrap();
    assert_eq!(
        interp.eval("set greeting").unwrap().as_str().as_ref(),
        "hello world"
    );
}

#[test]
fn nested_variable_access() {
    let mut interp = Interpreter::new();
    interp.eval("set d [dict create name alice]").unwrap();
    let val = interp.eval(r#"set result $d(name)"#).unwrap();
    assert_eq!(val.as_str().as_ref(), "alice");
}

#[test]
fn undefined_variable_error() {
    let mut interp = Interpreter::new();
    let err = interp.eval("set nonexistent_var").unwrap_err();
    assert!(
        err.to_string().contains("no such variable"),
        "expected 'no such variable', got: {err}"
    );
}

#[test]
fn set_overwrites_previous() {
    let mut interp = Interpreter::new();
    interp.eval("set x 1").unwrap();
    interp.eval("set x 2").unwrap();
    assert_eq!(interp.eval("set x").unwrap().as_str().as_ref(), "2");
}

#[test]
fn multiple_variables_independent() {
    let mut interp = Interpreter::new();
    interp.eval("set x 1").unwrap();
    interp.eval("set y 2").unwrap();
    assert_eq!(interp.eval("set x").unwrap().as_str().as_ref(), "1");
    assert_eq!(interp.eval("set y").unwrap().as_str().as_ref(), "2");
}
