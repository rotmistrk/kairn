//! I/O commands: puts, source.

use crate::error::TclError;
use crate::interpreter::Interpreter;
use crate::value::TclValue;

/// Register I/O commands.
pub fn register(interp: &mut Interpreter) {
    interp.register_fn("puts", cmd_puts);
    interp.register_fn("source", cmd_source);
}

/// `puts ?-nonewline? string` — output a string.
fn cmd_puts(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"puts ?-nonewline? string\"",
        ));
    }
    let (no_newline, text) = if args[0].as_str() == "-nonewline" {
        if args.len() < 2 {
            return Err(TclError::new(
                "wrong # args: should be \"puts ?-nonewline? string\"",
            ));
        }
        (true, args[1].as_str().to_string())
    } else {
        (false, args[0].as_str().to_string())
    };
    if no_newline {
        interp.output.push(text);
    } else {
        interp.output.push(format!("{text}\n"));
    }
    Ok(TclValue::Str(String::new()))
}

/// `source script` — evaluate a script string (in this implementation).
fn cmd_source(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"source script\""));
    }
    let script = args[0].as_str().to_string();
    interp.eval(&script)
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;

    #[test]
    fn puts_captures_output() {
        let mut interp = Interpreter::new();
        interp.eval("puts hello").unwrap();
        assert_eq!(interp.get_output(), &["hello\n"]);
    }

    #[test]
    fn puts_nonewline() {
        let mut interp = Interpreter::new();
        interp.eval("puts -nonewline hello").unwrap();
        assert_eq!(interp.get_output(), &["hello"]);
    }
}
