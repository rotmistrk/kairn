use rusticle::interpreter::Interpreter;

fn eval_ok(script: &str) -> String {
    let mut interp = Interpreter::new();
    interp.eval(script).unwrap().as_str().into_owned()
}

fn eval_int(script: &str) -> i64 {
    let mut interp = Interpreter::new();
    interp.eval(script).unwrap().as_int().unwrap()
}

fn eval_err(script: &str) {
    let mut interp = Interpreter::new();
    assert!(interp.eval(script).is_err());
}

#[test]
fn add_integers() {
    assert_eq!(eval_int("expr {2 + 3}"), 5);
}

#[test]
fn subtract() {
    assert_eq!(eval_int("expr {10 - 3}"), 7);
}

#[test]
fn multiply() {
    assert_eq!(eval_int("expr {4 * 5}"), 20);
}

#[test]
fn divide_integers() {
    assert_eq!(eval_int("expr {10 / 3}"), 3);
}

#[test]
fn modulo() {
    assert_eq!(eval_int("expr {10 % 3}"), 1);
}

#[test]
fn float_arithmetic() {
    let result = eval_ok("expr {3.14 * 2}");
    assert!(result.starts_with("6.28"), "expected ~6.28, got {result}");
}

#[test]
fn comparison_equal() {
    assert_eq!(eval_int("expr {5 == 5}"), 1);
}

#[test]
fn comparison_not_equal() {
    assert_eq!(eval_int("expr {5 != 3}"), 1);
}

#[test]
fn comparison_less() {
    assert_eq!(eval_int("expr {3 < 5}"), 1);
}

#[test]
fn comparison_greater() {
    assert_eq!(eval_int("expr {5 > 3}"), 1);
}

#[test]
fn comparison_lte() {
    assert_eq!(eval_int("expr {5 <= 5}"), 1);
}

#[test]
fn comparison_gte() {
    assert_eq!(eval_int("expr {5 >= 6}"), 0);
}

#[test]
fn logical_and() {
    assert_eq!(eval_int("expr {1 && 1}"), 1);
}

#[test]
fn logical_or() {
    assert_eq!(eval_int("expr {0 || 1}"), 1);
}

#[test]
fn logical_not() {
    assert_eq!(eval_int("expr {!0}"), 1);
}

#[test]
fn parentheses() {
    assert_eq!(eval_int("expr {(2 + 3) * 4}"), 20);
}

#[test]
fn nested_parens() {
    assert_eq!(eval_int("expr {((1 + 2) * (3 + 4))}"), 21);
}

#[test]
fn variable_in_expr() {
    let mut interp = Interpreter::new();
    interp.eval("set x 10").unwrap();
    let result = interp.eval("expr {$x + 5}").unwrap();
    assert_eq!(result.as_int().unwrap(), 15);
}

#[test]
fn division_by_zero() {
    eval_err("expr {1 / 0}");
}

#[test]
fn complex_expression() {
    assert_eq!(eval_int("expr {(10 + 5) * 2 - 3}"), 27);
}
