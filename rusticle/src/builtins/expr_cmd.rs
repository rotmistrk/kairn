//! Expression evaluator: `expr {expression}`.

use crate::error::TclError;
use crate::interpreter::Interpreter;
use crate::value::TclValue;

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

/// Evaluate an expression string.
fn eval_expression(input: &str) -> Result<TclValue, TclError> {
    let tokens = tokenize(input.trim())?;
    let mut pos = 0;
    let result = parse_or(&tokens, &mut pos)?;
    Ok(result)
}

/// Token types for the expression parser.
#[derive(Clone, Debug)]
enum Token {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Op(String),
    LParen,
    RParen,
}

/// Tokenize an expression string.
fn tokenize(input: &str) -> Result<Vec<Token>, TclError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        if chars[i] == '(' {
            tokens.push(Token::LParen);
            i += 1;
        } else if chars[i] == ')' {
            tokens.push(Token::RParen);
            i += 1;
        } else if chars[i] == '"' {
            let (s, next) = read_string(&chars, i);
            tokens.push(Token::Str(s));
            i = next;
        } else if is_op_start(chars[i]) {
            let (op, next) = read_op(&chars, i);
            tokens.push(Token::Op(op));
            i = next;
        } else if chars[i].is_ascii_digit() || (chars[i] == '-' && is_unary_context(&tokens)) {
            let (num, next) = read_number(&chars, i);
            tokens.push(num);
            i = next;
        } else if chars[i].is_alphabetic() {
            let (word, next) = read_word(&chars, i);
            match word.as_str() {
                "true" => tokens.push(Token::Bool(true)),
                "false" => tokens.push(Token::Bool(false)),
                _ => tokens.push(Token::Str(word)),
            }
            i = next;
        } else {
            i += 1;
        }
    }
    Ok(tokens)
}

/// Check if `-` should be treated as unary minus.
fn is_unary_context(tokens: &[Token]) -> bool {
    tokens.is_empty() || matches!(tokens.last(), Some(Token::Op(_)) | Some(Token::LParen))
}

/// Read a quoted string.
fn read_string(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start + 1;
    let mut s = String::new();
    while i < chars.len() && chars[i] != '"' {
        if chars[i] == '\\' && i + 1 < chars.len() {
            s.push(chars[i + 1]);
            i += 2;
        } else {
            s.push(chars[i]);
            i += 1;
        }
    }
    if i < chars.len() {
        i += 1;
    }
    (s, i)
}

/// Read an operator.
fn read_op(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start;
    let mut op = String::new();
    op.push(chars[i]);
    i += 1;
    // Two-character operators
    if i < chars.len() {
        let two = format!("{}{}", chars[start], chars[i]);
        if matches!(two.as_str(), "==" | "!=" | "<=" | ">=" | "&&" | "||") {
            op = two;
            i += 1;
        }
    }
    (op, i)
}

/// Check if a character starts an operator.
fn is_op_start(c: char) -> bool {
    matches!(
        c,
        '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|'
    )
}

/// Read a number (integer or float).
fn read_number(chars: &[char], start: usize) -> (Token, usize) {
    let mut i = start;
    let mut s = String::new();
    if i < chars.len() && chars[i] == '-' {
        s.push('-');
        i += 1;
    }
    let mut has_dot = false;
    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
        if chars[i] == '.' {
            has_dot = true;
        }
        s.push(chars[i]);
        i += 1;
    }
    if has_dot {
        if let Ok(f) = s.parse::<f64>() {
            return (Token::Float(f), i);
        }
    }
    if let Ok(n) = s.parse::<i64>() {
        return (Token::Int(n), i);
    }
    (Token::Str(s), i)
}

/// Read an alphabetic word.
fn read_word(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start;
    let mut s = String::new();
    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
        s.push(chars[i]);
        i += 1;
    }
    (s, i)
}

/// Parse || (logical or).
fn parse_or(tokens: &[Token], pos: &mut usize) -> Result<TclValue, TclError> {
    let mut left = parse_and(tokens, pos)?;
    while *pos < tokens.len() {
        if matches!(&tokens[*pos], Token::Op(op) if op == "||") {
            *pos += 1;
            let right = parse_and(tokens, pos)?;
            let lb = left.as_bool()?;
            let rb = right.as_bool()?;
            left = TclValue::Bool(lb || rb);
        } else {
            break;
        }
    }
    Ok(left)
}

/// Parse && (logical and).
fn parse_and(tokens: &[Token], pos: &mut usize) -> Result<TclValue, TclError> {
    let mut left = parse_comparison(tokens, pos)?;
    while *pos < tokens.len() {
        if matches!(&tokens[*pos], Token::Op(op) if op == "&&") {
            *pos += 1;
            let right = parse_comparison(tokens, pos)?;
            let lb = left.as_bool()?;
            let rb = right.as_bool()?;
            left = TclValue::Bool(lb && rb);
        } else {
            break;
        }
    }
    Ok(left)
}

/// Parse comparison operators.
fn parse_comparison(tokens: &[Token], pos: &mut usize) -> Result<TclValue, TclError> {
    let mut left = parse_additive(tokens, pos)?;
    while *pos < tokens.len() {
        let op = match &tokens[*pos] {
            Token::Op(op) if matches!(op.as_str(), "==" | "!=" | "<" | ">" | "<=" | ">=") => {
                op.clone()
            }
            _ => break,
        };
        *pos += 1;
        let right = parse_additive(tokens, pos)?;
        left = eval_comparison(&left, &op, &right)?;
    }
    Ok(left)
}

/// Parse + and -.
fn parse_additive(tokens: &[Token], pos: &mut usize) -> Result<TclValue, TclError> {
    let mut left = parse_multiplicative(tokens, pos)?;
    while *pos < tokens.len() {
        let op = match &tokens[*pos] {
            Token::Op(op) if op == "+" || op == "-" => op.clone(),
            _ => break,
        };
        *pos += 1;
        let right = parse_multiplicative(tokens, pos)?;
        left = eval_arithmetic(&left, &op, &right)?;
    }
    Ok(left)
}

/// Parse *, /, %.
fn parse_multiplicative(tokens: &[Token], pos: &mut usize) -> Result<TclValue, TclError> {
    let mut left = parse_unary(tokens, pos)?;
    while *pos < tokens.len() {
        let op = match &tokens[*pos] {
            Token::Op(op) if op == "*" || op == "/" || op == "%" => op.clone(),
            _ => break,
        };
        *pos += 1;
        let right = parse_unary(tokens, pos)?;
        left = eval_arithmetic(&left, &op, &right)?;
    }
    Ok(left)
}

/// Parse unary operators (!, -).
fn parse_unary(tokens: &[Token], pos: &mut usize) -> Result<TclValue, TclError> {
    if *pos < tokens.len() && matches!(&tokens[*pos], Token::Op(op) if op == "!") {
        *pos += 1;
        let val = parse_primary(tokens, pos)?;
        let b = val.as_bool()?;
        return Ok(TclValue::Bool(!b));
    }
    parse_primary(tokens, pos)
}

/// Parse primary expressions (numbers, strings, parenthesized).
fn parse_primary(tokens: &[Token], pos: &mut usize) -> Result<TclValue, TclError> {
    if *pos >= tokens.len() {
        return Err(TclError::new("unexpected end of expression"));
    }
    let token = tokens[*pos].clone();
    *pos += 1;
    match token {
        Token::Int(n) => Ok(TclValue::Int(n)),
        Token::Float(f) => Ok(TclValue::Float(f)),
        Token::Bool(b) => Ok(TclValue::Bool(b)),
        Token::Str(s) => {
            // Try to parse as number
            if let Ok(n) = s.parse::<i64>() {
                Ok(TclValue::Int(n))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(TclValue::Float(f))
            } else {
                Ok(TclValue::Str(s))
            }
        }
        Token::LParen => {
            let val = parse_or(tokens, pos)?;
            if *pos < tokens.len() && matches!(&tokens[*pos], Token::RParen) {
                *pos += 1;
            }
            Ok(val)
        }
        _ => Err(TclError::new("unexpected token in expression")),
    }
}

/// Evaluate an arithmetic operation.
fn eval_arithmetic(left: &TclValue, op: &str, right: &TclValue) -> Result<TclValue, TclError> {
    // If either is float, use float arithmetic
    if matches!(left, TclValue::Float(_)) || matches!(right, TclValue::Float(_)) {
        let l = left.as_float()?;
        let r = right.as_float()?;
        return Ok(TclValue::Float(match op {
            "+" => l + r,
            "-" => l - r,
            "*" => l * r,
            "/" => {
                if r == 0.0 {
                    return Err(TclError::new("divide by zero"));
                }
                l / r
            }
            "%" => {
                if r == 0.0 {
                    return Err(TclError::new("divide by zero"));
                }
                l % r
            }
            _ => return Err(TclError::new(format!("unknown operator: {op}"))),
        }));
    }
    let l = left.as_int()?;
    let r = right.as_int()?;
    Ok(TclValue::Int(match op {
        "+" => l + r,
        "-" => l - r,
        "*" => l * r,
        "/" => {
            if r == 0 {
                return Err(TclError::new("divide by zero"));
            }
            l / r
        }
        "%" => {
            if r == 0 {
                return Err(TclError::new("divide by zero"));
            }
            l % r
        }
        _ => return Err(TclError::new(format!("unknown operator: {op}"))),
    }))
}

/// Evaluate a comparison operation.
fn eval_comparison(left: &TclValue, op: &str, right: &TclValue) -> Result<TclValue, TclError> {
    // Try numeric comparison first
    if let (Ok(l), Ok(r)) = (left.as_float(), right.as_float()) {
        return Ok(TclValue::Bool(match op {
            "==" => (l - r).abs() < f64::EPSILON,
            "!=" => (l - r).abs() >= f64::EPSILON,
            "<" => l < r,
            ">" => l > r,
            "<=" => l <= r,
            ">=" => l >= r,
            _ => return Err(TclError::new(format!("unknown operator: {op}"))),
        }));
    }
    // Fall back to string comparison
    let l = left.as_str();
    let r = right.as_str();
    Ok(TclValue::Bool(match op {
        "==" => l == r,
        "!=" => l != r,
        "<" => l < r,
        ">" => l > r,
        "<=" => l <= r,
        ">=" => l >= r,
        _ => return Err(TclError::new(format!("unknown operator: {op}"))),
    }))
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
