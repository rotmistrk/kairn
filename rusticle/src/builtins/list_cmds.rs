//! List commands: list, lindex, llength, lappend, lrange, lsearch, lsort,
//! lset, join, split, lmap, lfilter, lreduce, range.

use crate::error::{ErrorCode, TclError};
use crate::interpreter::Interpreter;
use crate::value::TclValue;

/// Register list commands.
pub fn register(interp: &mut Interpreter) {
    interp.register_fn("list", cmd_list);
    interp.register_fn("lindex", cmd_lindex);
    interp.register_fn("llength", cmd_llength);
    interp.register_fn("lappend", cmd_lappend);
    interp.register_fn("lrange", cmd_lrange);
    interp.register_fn("lsearch", cmd_lsearch);
    interp.register_fn("lsort", cmd_lsort);
    interp.register_fn("lset", cmd_lset);
    interp.register_fn("join", cmd_join);
    interp.register_fn("split", cmd_split);
    interp.register_fn("lmap", cmd_lmap);
    interp.register_fn("lfilter", cmd_lfilter);
    interp.register_fn("lreduce", cmd_lreduce);
    interp.register_fn("range", cmd_range);
}

/// `list args...` — create a list.
fn cmd_list(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    Ok(TclValue::List(args.to_vec()))
}

/// `lindex list index`
fn cmd_lindex(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new(
            "wrong # args: should be \"lindex list index\"",
        ));
    }
    let list = args[0].as_list()?;
    let idx = args[1].as_int()?;
    let idx = resolve_index(idx, list.len());
    Ok(list
        .get(idx)
        .cloned()
        .unwrap_or(TclValue::Str(String::new())))
}

/// `llength list`
fn cmd_llength(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"llength list\""));
    }
    let list = args[0].as_list()?;
    Ok(TclValue::Int(list.len() as i64))
}

/// `lappend var element...`
fn cmd_lappend(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"lappend varName ?value ...?\"",
        ));
    }
    let name = args[0].as_str().to_string();
    let mut list = interp
        .get_var(&name)
        .map(|v| v.as_list())
        .unwrap_or(Ok(Vec::new()))?;
    for arg in &args[1..] {
        list.push(arg.clone());
    }
    let val = TclValue::List(list);
    interp.set_var(&name, val.clone());
    Ok(val)
}

/// `lrange list first last`
fn cmd_lrange(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 3 {
        return Err(TclError::new(
            "wrong # args: should be \"lrange list first last\"",
        ));
    }
    let list = args[0].as_list()?;
    let first = resolve_index(args[1].as_int()?, list.len());
    let last_raw = args[2].as_int()?;
    let last = resolve_index(last_raw, list.len());
    if first > last || first >= list.len() {
        return Ok(TclValue::List(Vec::new()));
    }
    let end = (last + 1).min(list.len());
    Ok(TclValue::List(list[first..end].to_vec()))
}

/// `lsearch list pattern`
fn cmd_lsearch(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new(
            "wrong # args: should be \"lsearch list pattern\"",
        ));
    }
    let list = args[0].as_list()?;
    let pattern = args[1].as_str().to_string();
    for (i, item) in list.iter().enumerate() {
        if item.as_str() == pattern {
            return Ok(TclValue::Int(i as i64));
        }
    }
    Ok(TclValue::Int(-1))
}

/// `lsort list`
fn cmd_lsort(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"lsort list\""));
    }
    let mut list = args[0].as_list()?;
    list.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    Ok(TclValue::List(list))
}

/// `lset var index value`
fn cmd_lset(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 3 {
        return Err(TclError::new(
            "wrong # args: should be \"lset varName index value\"",
        ));
    }
    let name = args[0].as_str().to_string();
    let idx = args[1].as_int()?;
    let value = args[2].clone();
    let mut list = interp
        .get_var(&name)
        .map(|v| v.as_list())
        .unwrap_or(Ok(Vec::new()))?;
    let idx = resolve_index(idx, list.len());
    if idx < list.len() {
        list[idx] = value;
    }
    let val = TclValue::List(list);
    interp.set_var(&name, val.clone());
    Ok(val)
}

/// `join list ?sep?`
fn cmd_join(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"join list ?joinString?\"",
        ));
    }
    let list = args[0].as_list()?;
    let sep = if args.len() > 1 {
        args[1].as_str().to_string()
    } else {
        " ".to_string()
    };
    let joined = list
        .iter()
        .map(|v| v.as_str().to_string())
        .collect::<Vec<_>>()
        .join(&sep);
    Ok(TclValue::Str(joined))
}

/// `split str ?sep?`
fn cmd_split(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"split string ?splitChars?\"",
        ));
    }
    let s = args[0].as_str().to_string();
    let parts = if args.len() > 1 {
        let sep = args[1].as_str().to_string();
        if sep.is_empty() {
            // Split into individual characters
            s.chars().map(|c| TclValue::Str(c.to_string())).collect()
        } else {
            s.split(&sep)
                .map(|p| TclValue::Str(p.to_string()))
                .collect()
        }
    } else {
        // Default: split on whitespace
        s.split_whitespace()
            .map(|p| TclValue::Str(p.to_string()))
            .collect()
    };
    Ok(TclValue::List(parts))
}

/// `lmap list lambda` — map a lambda over a list.
fn cmd_lmap(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new(
            "wrong # args: should be \"lmap list lambda\"",
        ));
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
        return Err(TclError::new(
            "wrong # args: should be \"lfilter list lambda\"",
        ));
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
        return Err(TclError::new(
            "wrong # args: should be \"lreduce list init lambda\"",
        ));
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
        return Err(TclError::new(
            "wrong # args: should be \"range start end ?step?\"",
        ));
    }
    let start = args[0].as_int()?;
    let end = args[1].as_int()?;
    let step = if args.len() > 2 { args[2].as_int()? } else { 1 };
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
fn resolve_index(idx: i64, len: usize) -> usize {
    if idx < 0 {
        let adjusted = len as i64 + idx;
        if adjusted < 0 {
            0
        } else {
            adjusted as usize
        }
    } else {
        idx as usize
    }
}

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
        assert_eq!(
            interp.eval("lsearch [list a b c] x").unwrap().as_str(),
            "-1"
        );
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
        assert_eq!(
            interp.eval("join [list a b c] ,").unwrap().as_str(),
            "a,b,c"
        );
        assert_eq!(
            interp.eval("llength [split \"a,b,c\" ,]").unwrap().as_str(),
            "3"
        );
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
        let result = interp
            .eval("lmap [list 1 2 3] {x { expr {$x * 10} }}")
            .unwrap();
        assert_eq!(result.as_str(), "10 20 30");
    }

    #[test]
    fn lfilter_basic() {
        let mut interp = Interpreter::new();
        let result = interp
            .eval("lfilter [list 1 2 3 4 5] {x { expr {$x > 3} }}")
            .unwrap();
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
