//! Expression parser: tokenization and recursive-descent evaluation.

use crate::error::TclError;
use super::expr_ops::{eval_arithmetic, eval_comparison};
use crate::value::TclValue;

/// Evaluate an expression string.
pub fn eval_expression(input: &str) -> Result<TclValue, TclError> {
    let tokens = tokenize(input.trim())?;
    let mut pos = 0;
    parse_or(&tokens, &mut pos)
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
    matches!(c, '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|')
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
            Token::Op(op) if matches!(op.as_str(), "==" | "!=" | "<" | ">" | "<=" | ">=") => op.clone(),
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

