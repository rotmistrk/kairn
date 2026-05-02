use rusticle::interpreter::Interpreter;

#[test]
fn proc_basic() {
    let mut interp = Interpreter::new();
    interp
        .eval("proc double {x} {return [expr {$x * 2}]}")
        .unwrap();
    let result = interp.eval("double 5").unwrap();
    assert_eq!(result.as_int().unwrap(), 10);
}

#[test]
fn proc_default_arg() {
    let mut interp = Interpreter::new();
    interp
        .eval(r#"proc greet {{name world}} {return "hello $name"}"#)
        .unwrap();
    assert_eq!(interp.eval("greet").unwrap().as_str(), "hello world");
    assert_eq!(interp.eval("greet rust").unwrap().as_str(), "hello rust");
}

#[test]
fn proc_recursive_factorial() {
    let mut interp = Interpreter::new();
    interp
        .eval(
            r#"proc factorial {n} {
                set result 1
                while {$n > 1} {
                    set result [expr {$result * $n}]
                    set n [expr {$n - 1}]
                }
                return $result
            }"#,
        )
        .unwrap();
    let result = interp.eval("factorial 5").unwrap();
    assert_eq!(result.as_int().unwrap(), 120);
}

#[test]
fn proc_calling_proc() {
    let mut interp = Interpreter::new();
    interp
        .eval(
            r#"
            proc add {a b} {return [expr {$a + $b}]}
            proc multiply {a b} {
                set result 0
                foreach i [range 0 $b] {
                    set result [add $result $a]
                }
                return $result
            }
        "#,
        )
        .unwrap();
    let result = interp.eval("multiply 7 3").unwrap();
    assert_eq!(result.as_int().unwrap(), 21);
}

#[test]
fn proc_accesses_outer_scope() {
    let mut interp = Interpreter::new();
    interp.eval("set x 10").unwrap();
    interp.eval("proc foo {} { return $x }").unwrap();
    let result = interp.eval("foo").unwrap();
    assert_eq!(result.as_int().unwrap(), 10);
}

#[test]
fn proc_outer_set() {
    let mut interp = Interpreter::new();
    interp
        .eval("proc setit {} { outer set result 42 }")
        .unwrap();
    interp.eval("setit").unwrap();
    let result = interp.eval("return $result").unwrap();
    assert_eq!(result.as_int().unwrap(), 42);
}

#[test]
fn proc_return_value() {
    let mut interp = Interpreter::new();
    interp
        .eval(
            r#"proc compute {a b} {
                set sum [expr {$a + $b}]
                return [expr {$sum * 2}]
            }"#,
        )
        .unwrap();
    let result = interp.eval("compute 3 4").unwrap();
    assert_eq!(result.as_int().unwrap(), 14);
}

#[test]
fn proc_wrong_arity() {
    let mut interp = Interpreter::new();
    interp.eval("proc takes_two {a b} {return ok}").unwrap();
    let err = interp.eval("takes_two 1").unwrap_err();
    assert!(err.message.contains("wrong"));
}

#[test]
fn lambda_in_lmap() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval("lmap [list 1 2 3] {x { expr {$x * 10} }}")
        .unwrap();
    assert_eq!(result.as_str(), "10 20 30");
}

#[test]
fn proc_with_expr_inside() {
    let mut interp = Interpreter::new();
    interp
        .eval(
            r#"proc combo {x} {
                set n [expr {$x + 1}]
                set s [string toupper "hello"]
                set d [dict create key $n]
                return [dict get $d key]
            }"#,
        )
        .unwrap();
    let result = interp.eval("combo 9").unwrap();
    assert_eq!(result.as_int().unwrap(), 10);
}

#[test]
fn proc_with_foreach_inside() {
    let mut interp = Interpreter::new();
    interp
        .eval(
            r#"proc sum_list {lst} {
                set total 0
                foreach item $lst {
                    set total [expr {$total + $item}]
                }
                return $total
            }"#,
        )
        .unwrap();
    let result = interp.eval("sum_list [list 1 2 3 4]").unwrap();
    assert_eq!(result.as_int().unwrap(), 10);
}

#[test]
fn nested_proc_calls() {
    let mut interp = Interpreter::new();
    interp
        .eval(
            r#"
            proc c {x} {return [expr {$x + 1}]}
            proc b {x} {return [c [expr {$x * 2}]]}
            proc a {x} {return [b [expr {$x + 3}]]}
        "#,
        )
        .unwrap();
    // a(1) -> b(4) -> c(8) -> 9
    let result = interp.eval("a 1").unwrap();
    assert_eq!(result.as_int().unwrap(), 9);
}
