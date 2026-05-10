//! List commands: list, lindex, llength, lappend, lrange, lsearch, lsort,
//! lset, join, split, lmap, lfilter, lreduce, range.

use crate::error::TclError;
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
}

/// `list args...` — create a list.
fn cmd_list(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    Ok(TclValue::List(args.to_vec()))
}

/// `lindex list index`
fn cmd_lindex(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new("wrong # args: should be \"lindex list index\""));
    }
    let list = args[0].as_list()?;
    let idx = args[1].as_int()?;
    let idx = resolve_index(idx, list.len());
    Ok(list.get(idx).cloned().unwrap_or(TclValue::Str(String::new())))
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
        return Err(TclError::new("wrong # args: should be \"lappend varName ?value ...?\""));
    }
    let name = args[0].as_str().to_string();
    let mut list = interp.get_var(&name).map(|v| v.as_list()).unwrap_or(Ok(Vec::new()))?;
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
        return Err(TclError::new("wrong # args: should be \"lrange list first last\""));
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
        return Err(TclError::new("wrong # args: should be \"lsearch list pattern\""));
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
        return Err(TclError::new("wrong # args: should be \"lset varName index value\""));
    }
    let name = args[0].as_str().to_string();
    let idx = args[1].as_int()?;
    let value = args[2].clone();
    let mut list = interp.get_var(&name).map(|v| v.as_list()).unwrap_or(Ok(Vec::new()))?;
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
        return Err(TclError::new("wrong # args: should be \"join list ?joinString?\""));
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
        return Err(TclError::new("wrong # args: should be \"split string ?splitChars?\""));
    }
    let s = args[0].as_str().to_string();
    let parts = if args.len() > 1 {
        let sep = args[1].as_str().to_string();
        if sep.is_empty() {
            // Split into individual characters
            s.chars().map(|c| TclValue::Str(c.to_string())).collect()
        } else {
            s.split(&sep).map(|p| TclValue::Str(p.to_string())).collect()
        }
    } else {
        // Default: split on whitespace
        s.split_whitespace().map(|p| TclValue::Str(p.to_string())).collect()
    };
    Ok(TclValue::List(parts))
}

pub(super) fn resolve_index(idx: i64, len: usize) -> usize {
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
