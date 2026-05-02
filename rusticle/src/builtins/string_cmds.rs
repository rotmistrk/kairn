//! String commands: string length/range/match/map/trim/tolower/toupper/first, format.

use crate::error::TclError;
use crate::interpreter::Interpreter;
use crate::value::TclValue;

/// Register string commands.
pub fn register(interp: &mut Interpreter) {
    interp.register_fn("string", cmd_string);
    interp.register_fn("format", cmd_format);
}

/// `string subcommand args...`
fn cmd_string(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"string subcommand ...\"",
        ));
    }
    let subcmd = args[0].as_str().to_string();
    let rest = &args[1..];
    match subcmd.as_str() {
        "length" => string_length(rest),
        "range" => string_range(rest),
        "match" => string_match(rest),
        "map" => string_map(interp, rest),
        "trim" => string_trim(rest),
        "tolower" => string_tolower(rest),
        "toupper" => string_toupper(rest),
        "first" => string_first(rest),
        _ => Err(TclError::new(format!(
            "unknown string subcommand \"{subcmd}\""
        ))),
    }
}

/// `string length str`
fn string_length(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"string length string\"",
        ));
    }
    Ok(TclValue::Int(args[0].as_str().len() as i64))
}

/// `string range str first last`
fn string_range(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 3 {
        return Err(TclError::new(
            "wrong # args: should be \"string range string first last\"",
        ));
    }
    let s = args[0].as_str().to_string();
    let first = resolve_index(args[1].as_int()?, s.len());
    let last = resolve_index(args[2].as_int()?, s.len());
    if first > last || first >= s.len() {
        return Ok(TclValue::Str(String::new()));
    }
    let end = (last + 1).min(s.len());
    Ok(TclValue::Str(s[first..end].to_string()))
}

/// `string match pattern str`
fn string_match(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new(
            "wrong # args: should be \"string match pattern string\"",
        ));
    }
    let pattern = args[0].as_str().to_string();
    let s = args[1].as_str().to_string();
    Ok(TclValue::Bool(glob_match(&pattern, &s)))
}

/// `string map {old new ...} str`
fn string_map(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new(
            "wrong # args: should be \"string map mapping string\"",
        ));
    }
    let mapping = args[0].as_list()?;
    let mut s = args[1].as_str().to_string();
    let mut i = 0;
    while i + 1 < mapping.len() {
        let old = mapping[i].as_str().to_string();
        let new = mapping[i + 1].as_str().to_string();
        s = s.replace(&old, &new);
        i += 2;
    }
    Ok(TclValue::Str(s))
}

/// `string trim str ?chars?`
fn string_trim(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"string trim string ?chars?\"",
        ));
    }
    let s = args[0].as_str().to_string();
    if args.len() > 1 {
        let chars: Vec<char> = args[1].as_str().chars().collect();
        let trimmed = s.trim_matches(|c| chars.contains(&c)).to_string();
        Ok(TclValue::Str(trimmed))
    } else {
        Ok(TclValue::Str(s.trim().to_string()))
    }
}

/// `string tolower str`
fn string_tolower(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"string tolower string\"",
        ));
    }
    Ok(TclValue::Str(args[0].as_str().to_lowercase()))
}

/// `string toupper str`
fn string_toupper(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"string toupper string\"",
        ));
    }
    Ok(TclValue::Str(args[0].as_str().to_uppercase()))
}

/// `string first needle haystack`
fn string_first(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new(
            "wrong # args: should be \"string first needle haystack\"",
        ));
    }
    let needle = args[0].as_str().to_string();
    let haystack = args[1].as_str().to_string();
    let pos = haystack.find(&needle).map(|p| p as i64).unwrap_or(-1);
    Ok(TclValue::Int(pos))
}

/// `format fmt args...` — simple printf-style formatting.
fn cmd_format(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"format formatString ?arg ...?\"",
        ));
    }
    let fmt = args[0].as_str().to_string();
    let mut result = String::new();
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;
    let mut arg_idx = 1;
    while i < chars.len() {
        if chars[i] == '%' && i + 1 < chars.len() {
            i += 1;
            match chars[i] {
                's' => {
                    if arg_idx < args.len() {
                        result.push_str(&args[arg_idx].as_str());
                        arg_idx += 1;
                    }
                }
                'd' => {
                    if arg_idx < args.len() {
                        let n = args[arg_idx].as_int()?;
                        result.push_str(&n.to_string());
                        arg_idx += 1;
                    }
                }
                'f' => {
                    if arg_idx < args.len() {
                        let f = args[arg_idx].as_float()?;
                        result.push_str(&format!("{f:.6}"));
                        arg_idx += 1;
                    }
                }
                '%' => result.push('%'),
                _ => {
                    result.push('%');
                    result.push(chars[i]);
                }
            }
            i += 1;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    Ok(TclValue::Str(result))
}

/// Resolve a string index (handling negative indices).
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

/// Simple glob matching (* and ?).
fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    glob_match_impl(&p, 0, &t, 0)
}

/// Recursive glob matching.
fn glob_match_impl(p: &[char], pi: usize, t: &[char], ti: usize) -> bool {
    if pi == p.len() {
        return ti == t.len();
    }
    if p[pi] == '*' {
        // Try matching * with 0..n characters
        for skip in 0..=(t.len() - ti) {
            if glob_match_impl(p, pi + 1, t, ti + skip) {
                return true;
            }
        }
        return false;
    }
    if ti >= t.len() {
        return false;
    }
    if p[pi] == '?' || p[pi] == t[ti] {
        return glob_match_impl(p, pi + 1, t, ti + 1);
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;

    #[test]
    fn string_length() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval("string length hello").unwrap().as_str(), "5");
    }

    #[test]
    fn string_range() {
        let mut interp = Interpreter::new();
        assert_eq!(
            interp.eval("string range hello 1 3").unwrap().as_str(),
            "ell"
        );
    }

    #[test]
    fn string_match() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval("string match h* hello").unwrap().as_str(), "1");
        assert_eq!(interp.eval("string match x* hello").unwrap().as_str(), "0");
    }

    #[test]
    fn string_trim() {
        let mut interp = Interpreter::new();
        assert_eq!(
            interp.eval("string trim \"  hello  \"").unwrap().as_str(),
            "hello"
        );
    }

    #[test]
    fn string_case() {
        let mut interp = Interpreter::new();
        assert_eq!(
            interp.eval("string toupper hello").unwrap().as_str(),
            "HELLO"
        );
        assert_eq!(
            interp.eval("string tolower HELLO").unwrap().as_str(),
            "hello"
        );
    }

    #[test]
    fn string_first() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval("string first ll hello").unwrap().as_str(), "2");
        assert_eq!(interp.eval("string first xx hello").unwrap().as_str(), "-1");
    }

    #[test]
    fn format_basic() {
        let mut interp = Interpreter::new();
        assert_eq!(
            interp
                .eval("format \"%s is %d\" hello 42")
                .unwrap()
                .as_str(),
            "hello is 42"
        );
    }

    #[test]
    fn string_map() {
        let mut interp = Interpreter::new();
        let result = interp
            .eval("string map {foo bar baz qux} \"foo and baz\"")
            .unwrap();
        assert_eq!(result.as_str(), "bar and qux");
    }
}
