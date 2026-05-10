//! Functional list commands: lmap, lfilter, lreduce, range.

use crate::error::{ErrorCode, TclError};
use crate::interpreter::Interpreter;
use crate::value::TclValue;

pub fn register(interp: &mut Interpreter) {
    interp.register_fn("lmap", cmd_lmap);
    interp.register_fn("lfilter", cmd_lfilter);
    interp.register_fn("lreduce", cmd_lreduce);
    interp.register_fn("range", cmd_range);
}

fn cmd_lmap(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new("wrong # args: should be \"lmap list lambda\""));
    }
    let list = args[0].as_list()?;
    let lambda = args[1].as_str().to_string();
    let (param, body) = parse_lambda(&lambda)?;
    let mut result = Vec::new();
    for item in &list {
        interp.push_scope();
        interp.set_var(&param, item.clone());
        let val = match interp.eval(&body) {
            Ok(v) => v,
            Err(e) if e.code == ErrorCode::Break => {
                interp.pop_scope();
                break;
            }
            Err(e) if e.code == ErrorCode::Continue => {
                interp.pop_scope();
                continue;
            }
            Err(e) => {
                interp.pop_scope();
                return Err(e);
            }
        };
        interp.pop_scope();
        result.push(val);
    }
    Ok(TclValue::List(result))
}

/// `lfilter list lambda` — filter a list.
fn cmd_lfilter(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new("wrong # args: should be \"lfilter list lambda\""));
    }
    let list = args[0].as_list()?;
    let lambda = args[1].as_str().to_string();
    let (param, body) = parse_lambda(&lambda)?;
    let mut result = Vec::new();
    for item in &list {
        interp.push_scope();
        interp.set_var(&param, item.clone());
        let val = interp.eval(&body);
        interp.pop_scope();
        if val?.as_bool()? {
            result.push(item.clone());
        }
    }
    Ok(TclValue::List(result))
}

/// `lreduce list init lambda` — fold a list.
fn cmd_lreduce(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 3 {
        return Err(TclError::new("wrong # args: should be \"lreduce list init lambda\""));
    }
    let list = args[0].as_list()?;
    let mut acc = args[1].clone();
    let lambda = args[2].as_str().to_string();
    let (params, body) = parse_lambda2(&lambda)?;
    for item in &list {
        interp.push_scope();
        interp.set_var(&params.0, acc.clone());
        interp.set_var(&params.1, item.clone());
        acc = interp.eval(&body)?;
        interp.pop_scope();
    }
    Ok(acc)
}

/// `range start end ?step?` — generate an integer list.
fn cmd_range(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new("wrong # args: should be \"range start end ?step?\""));
    }
    let start = args[0].as_int()?;
    let end = args[1].as_int()?;
    let step = if args.len() > 2 {
        args[2].as_int()?
    } else {
        1
    };
    if step == 0 {
        return Err(TclError::new("range: step cannot be zero"));
    }
    let mut result = Vec::new();
    let mut i = start;
    if step > 0 {
        while i < end {
            result.push(TclValue::Int(i));
            i += step;
        }
    } else {
        while i > end {
            result.push(TclValue::Int(i));
            i += step;
        }
    }
    Ok(TclValue::List(result))
}

/// Parse a single-parameter lambda `{param { body }}`.
fn parse_lambda(s: &str) -> Result<(String, String), TclError> {
    let trimmed = s.trim();
    // Find the first { that starts the body
    let parts: Vec<&str> = trimmed.splitn(2, '{').collect();
    if parts.len() < 2 {
        return Err(TclError::new("invalid lambda: expected {param { body }}"));
    }
    let param = parts[0].trim().to_string();
    let body = parts[1].trim_end_matches('}').trim().to_string();
    if param.is_empty() {
        return Err(TclError::new("invalid lambda: missing parameter name"));
    }
    Ok((param, body))
}

/// Parse a two-parameter lambda `{acc x { body }}`.
fn parse_lambda2(s: &str) -> Result<((String, String), String), TclError> {
    let trimmed = s.trim();
    let parts: Vec<&str> = trimmed.splitn(3, |c: char| c.is_whitespace()).collect();
    if parts.len() < 2 {
        return Err(TclError::new("invalid lambda: expected {p1 p2 { body }}"));
    }
    let p1 = parts[0].trim().to_string();
    // The rest contains p2 and { body }
    let rest = parts[1..].join(" ");
    let rest_parts: Vec<&str> = rest.splitn(2, '{').collect();
    if rest_parts.len() < 2 {
        return Err(TclError::new("invalid lambda: expected {p1 p2 { body }}"));
    }
    let p2 = rest_parts[0].trim().to_string();
    let body = rest_parts[1].trim_end_matches('}').trim().to_string();
    Ok(((p1, p2), body))
}

/// Resolve a list index (handling negative indices).
#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;

    #[test]
    fn list_create() {
        let mut interp = Interpreter::new();
        let result = interp.eval("list a b c").unwrap();
        assert_eq!(result.as_str(), "a b c");
    }

    #[test]
    fn lindex() {
        let mut interp = Interpreter::new();
        let result = interp.eval("lindex [list a b c] 1").unwrap();
        assert_eq!(result.as_str(), "b");
    }

    #[test]
    fn llength() {
        let mut interp = Interpreter::new();
        let result = interp.eval("llength [list a b c]").unwrap();
        assert_eq!(result.as_str(), "3");
    }

    #[test]
    fn lappend() {
        let mut interp = Interpreter::new();
        interp.eval("set x [list a b]").unwrap();
        interp.eval("lappend x c").unwrap();
        assert_eq!(interp.eval("llength $x").unwrap().as_str(), "3");
    }

    #[test]
    fn lrange() {
        let mut interp = Interpreter::new();
        let result = interp.eval("lrange [list a b c d e] 1 3").unwrap();
        assert_eq!(result.as_str(), "b c d");
    }

    #[test]
    fn lsearch() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval("lsearch [list a b c] b").unwrap().as_str(), "1");
        assert_eq!(interp.eval("lsearch [list a b c] x").unwrap().as_str(), "-1");
    }

    #[test]
    fn lsort() {
        let mut interp = Interpreter::new();
        let result = interp.eval("lsort [list c a b]").unwrap();
        assert_eq!(result.as_str(), "a b c");
    }

    #[test]
    fn join_and_split() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval("join [list a b c] ,").unwrap().as_str(), "a,b,c");
        assert_eq!(interp.eval("llength [split \"a,b,c\" ,]").unwrap().as_str(), "3");
    }

    #[test]
    fn range_basic() {
        let mut interp = Interpreter::new();
        let result = interp.eval("range 1 5").unwrap();
        assert_eq!(result.as_str(), "1 2 3 4");
    }

    #[test]
    fn range_with_step() {
        let mut interp = Interpreter::new();
        let result = interp.eval("range 0 10 3").unwrap();
        assert_eq!(result.as_str(), "0 3 6 9");
    }

    #[test]
    fn lmap_basic() {
        let mut interp = Interpreter::new();
        let result = interp.eval("lmap [list 1 2 3] {x { expr {$x * 10} }}").unwrap();
        assert_eq!(result.as_str(), "10 20 30");
    }

    #[test]
    fn lfilter_basic() {
        let mut interp = Interpreter::new();
        let result = interp.eval("lfilter [list 1 2 3 4 5] {x { expr {$x > 3} }}").unwrap();
        assert_eq!(result.as_str(), "4 5");
    }

    #[test]
    fn lreduce_sum() {
        let mut interp = Interpreter::new();
        let result = interp
            .eval("lreduce [list 1 2 3 4] 0 {acc x { expr {$acc + $x} }}")
            .unwrap();
        assert_eq!(result.as_str(), "10");
    }

    #[test]
    fn lset_basic() {
        let mut interp = Interpreter::new();
        interp.eval("set x [list a b c]").unwrap();
        interp.eval("lset x 1 z").unwrap();
        assert_eq!(interp.eval("lindex $x 1").unwrap().as_str(), "z");
    }
}
