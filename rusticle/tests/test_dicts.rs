use rusticle::interpreter::Interpreter;

#[test]
fn dict_create_and_get() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval("dict get [dict create name alice age 30] name")
        .unwrap();
    assert_eq!(result.as_str().as_ref(), "alice");
}

#[test]
fn dict_set_modifies() {
    let mut interp = Interpreter::new();
    interp.eval("dict set d name bob").unwrap();
    let result = interp.eval("dict get $d name").unwrap();
    assert_eq!(result.as_str().as_ref(), "bob");
}

#[test]
fn dict_exists_true() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval("dict exists [dict create name alice] name")
        .unwrap();
    assert_eq!(result.as_int().unwrap(), 1);
}

#[test]
fn dict_exists_false() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval("dict exists [dict create name alice] missing")
        .unwrap();
    assert_eq!(result.as_int().unwrap(), 0);
}

#[test]
fn dict_keys() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval("dict keys [dict create name alice age 30]")
        .unwrap();
    assert_eq!(result.as_str().as_ref(), "name age");
}

#[test]
fn dict_values() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval("dict values [dict create name alice age 30]")
        .unwrap();
    assert_eq!(result.as_str().as_ref(), "alice 30");
}

#[test]
fn dict_size() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval("dict size [dict create name alice age 30]")
        .unwrap();
    assert_eq!(result.as_int().unwrap(), 2);
}

#[test]
fn dict_for_iterates() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval(
            r#"
            set result ""
            dict for {k v} [dict create a 1 b 2] {
                append result "$k=$v "
            }
            set result
            "#,
        )
        .unwrap();
    assert_eq!(result.as_str().as_ref(), "a=1 b=2 ");
}

#[test]
fn nested_dicts() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval(
            r#"
            set inner [dict create city paris]
            set outer [dict create addr $inner]
            dict get [dict get $outer addr] city
            "#,
        )
        .unwrap();
    assert_eq!(result.as_str().as_ref(), "paris");
}

#[test]
fn dict_get_missing_key_error() {
    let mut interp = Interpreter::new();
    let err = interp
        .eval("dict get [dict create name alice] missing")
        .unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("not known"),
        "expected 'not known' in error, got: {msg}"
    );
}
