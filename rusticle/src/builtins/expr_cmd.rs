//! Expression evaluator: `expr {expression}`.

use crate::error::TclError;
use crate::interpreter::Interpreter;
use crate::value::TclValue;

use super::expr_eval::eval_expression;

/// Register expression commands.
pub fn register(interp: &mut Interpreter) {
    interp.register_fn("expr", cmd_expr);
}

/// `expr {expression}` — evaluate a mathematical/logical expression.
fn cmd_expr(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"expr expression\""));
    }
    // Concatenate all args (Tcl allows `expr 1 + 2`)
    let expr_str = args
        .iter()
        .map(|a| a.as_str().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    // Substitute variables and commands first
    let substituted = crate::interpreter::eval::substitute(interp, &expr_str)?;
    let input = substituted.as_str().to_string();
    eval_expression(&input)
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;

    #[test]
    fn basic_arithmetic() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval("expr {2 + 3}").unwrap().as_str(), "5");
        assert_eq!(interp.eval("expr {10 - 3}").unwrap().as_str(), "7");
        assert_eq!(interp.eval("expr {4 * 5}").unwrap().as_str(), "20");
        assert_eq!(interp.eval("expr {10 / 3}").unwrap().as_str(), "3");
        assert_eq!(interp.eval("expr {10 % 3}").unwrap().as_str(), "1");
    }

    #[test]
    fn comparison() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval("expr {3 == 3}").unwrap().as_str(), "1");
        assert_eq!(interp.eval("expr {3 != 4}").unwrap().as_str(), "1");
        assert_eq!(interp.eval("expr {3 < 4}").unwrap().as_str(), "1");
        assert_eq!(interp.eval("expr {5 > 4}").unwrap().as_str(), "1");
    }

    #[test]
    fn logical_operators() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval("expr {1 && 1}").unwrap().as_str(), "1");
        assert_eq!(interp.eval("expr {1 && 0}").unwrap().as_str(), "0");
        assert_eq!(interp.eval("expr {0 || 1}").unwrap().as_str(), "1");
        assert_eq!(interp.eval("expr {!0}").unwrap().as_str(), "1");
    }

    #[test]
    fn variable_in_expr() {
        let mut interp = Interpreter::new();
        interp.eval("set x 10").unwrap();
        assert_eq!(interp.eval("expr {$x + 5}").unwrap().as_str(), "15");
    }

    #[test]
    fn float_arithmetic() {
        let mut interp = Interpreter::new();
        let result = interp.eval("expr {3.14 * 2}").unwrap();
        let f = result.as_float().unwrap();
        assert!((f - 6.28).abs() < 0.001);
    }

    #[test]
    fn parenthesized() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval("expr {(2 + 3) * 4}").unwrap().as_str(), "20");
    }

    #[test]
    fn divide_by_zero() {
        let mut interp = Interpreter::new();
        assert!(interp.eval("expr {1 / 0}").is_err());
    }
}
