use rusticle::interpreter::Interpreter;

fn eval(script: &str) -> String {
    let mut interp = Interpreter::new();
    interp.eval(script).unwrap().as_str().into_owned()
}

fn eval_var(script: &str, var: &str) -> String {
    let mut interp = Interpreter::new();
    interp.eval(script).unwrap();
    interp
        .eval(&format!("set {var}"))
        .unwrap()
        .as_str()
        .into_owned()
}

#[test]
fn if_true_branch() {
    assert_eq!(eval_var("if {1} {set r yes}", "r"), "yes");
}

#[test]
fn if_false_branch() {
    let mut interp = Interpreter::new();
    interp.eval("if {0} {set r yes}").unwrap();
    assert!(interp.eval("set r").is_err());
}

#[test]
fn if_else() {
    assert_eq!(eval_var("if {0} {set r yes} else {set r no}", "r"), "no");
}

#[test]
fn if_elseif() {
    assert_eq!(
        eval_var("if {0} {set r a} elseif {1} {set r b} else {set r c}", "r"),
        "b"
    );
}

#[test]
fn while_loop_counts() {
    assert_eq!(eval_var("set x 0; while {$x < 5} {incr x}", "x"), "5");
}

#[test]
fn while_with_break() {
    assert_eq!(
        eval_var("set x 0; while {1} {incr x; if {$x == 3} {break}}", "x"),
        "3"
    );
}

#[test]
fn while_with_continue() {
    assert_eq!(
        eval_var(
            "set sum 0; set i 0; while {$i < 5} {incr i; if {$i == 3} {continue}; incr sum $i}",
            "sum"
        ),
        "12"
    );
}

#[test]
fn foreach_over_list() {
    assert_eq!(
        eval_var("set sum 0; foreach i [list 1 2 3] {incr sum $i}", "sum"),
        "6"
    );
}

#[test]
fn foreach_destructuring() {
    assert_eq!(
        eval_var(
            r#"foreach {k v} [list a 1 b 2] {append result "$k=$v "}"#,
            "result"
        ),
        "a=1 b=2 "
    );
}

#[test]
fn for_loop() {
    assert_eq!(
        eval_var(
            "set sum 0; for {set i 0} {$i < 5} {incr i} {incr sum $i}",
            "sum"
        ),
        "10"
    );
}

#[test]
fn for_with_break() {
    assert_eq!(
        eval_var(
            "set x 0; for {set i 0} {$i < 10} {incr i} {if {$i == 3} {break}; incr x}",
            "x"
        ),
        "3"
    );
}

#[test]
fn switch_matches() {
    assert_eq!(
        eval_var("switch hello {hello {set r hi} world {set r bye}}", "r"),
        "hi"
    );
}

#[test]
fn switch_default() {
    assert_eq!(
        eval_var("switch unknown {a {set r 1} default {set r 0}}", "r"),
        "0"
    );
}

#[test]
fn match_literal() {
    assert_eq!(
        eval_var(r#"match ok {"ok" {set r success} _ {set r unknown}}"#, "r"),
        "success"
    );
}

#[test]
fn match_wildcard() {
    assert_eq!(eval_var("match anything {_ {set r caught}}", "r"), "caught");
}

#[test]
fn try_on_error() {
    assert_eq!(
        eval_var("try {error boom} on error {msg} {set r $msg}", "r"),
        "boom"
    );
}

#[test]
fn try_finally() {
    assert_eq!(
        eval_var(
            r#"set log ""; try {set log a} finally {append log b}"#,
            "log"
        ),
        "ab"
    );
}

#[test]
fn try_error_and_finally() {
    assert_eq!(
        eval_var(
            r#"set log ""; try {error x} on error {e} {append log caught} finally {append log ,done}"#,
            "log"
        ),
        "caught,done"
    );
}

#[test]
fn catch_returns_zero_on_success() {
    assert_eq!(eval("catch {set x 1}"), "0");
}

#[test]
fn catch_returns_one_on_error() {
    let mut interp = Interpreter::new();
    let code = interp
        .eval("catch {error oops} msg")
        .unwrap()
        .as_str()
        .into_owned();
    assert_eq!(code, "1");
    let msg = interp.eval("set msg").unwrap().as_str().into_owned();
    assert_eq!(msg, "oops");
}

#[test]
fn error_raises() {
    let mut interp = Interpreter::new();
    let err = interp.eval(r#"error "boom""#).unwrap_err();
    assert!(
        format!("{err:?}").contains("boom"),
        "expected error containing 'boom', got: {err:?}"
    );
}

#[test]
fn return_from_proc() {
    assert_eq!(eval("proc foo {} {return 42}; foo"), "42");
}

#[test]
fn nested_loops_with_break() {
    assert_eq!(
        eval_var(
            "set r 0; foreach i [list 1 2 3] {foreach j [list 10 20] {if {$j == 20} {break}; incr r $j}}",
            "r"
        ),
        "30"
    );
}
