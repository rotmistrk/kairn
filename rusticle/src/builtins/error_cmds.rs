//! Error handling commands: try, catch, error.

use crate::error::{ErrorCode, TclError};
use crate::interpreter::Interpreter;
use crate::value::TclValue;

pub fn register(interp: &mut Interpreter) {
    interp.register_fn("try", cmd_try);
    interp.register_fn("catch", cmd_catch);
    interp.register_fn("error", cmd_error);
}

fn cmd_try(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"try body ...\""));
    }
    let body = args[0].as_str().to_string();
    let result = interp.eval(&body);

    let mut finally_body: Option<String> = None;
    let mut handled = false;
    let mut final_result = result;

    let mut i = 1;
    while i < args.len() {
        let keyword = args[i].as_str().to_string();
        i += 1;
        if keyword == "on" {
            if i + 2 >= args.len() {
                return Err(TclError::new("wrong # args in try/on"));
            }
            let _error_type = args[i].as_str().to_string();
            i += 1;
            let var_spec = args[i].as_str().to_string();
            i += 1;
            let handler = args[i].as_str().to_string();
            i += 1;
            if let Err(ref e) = final_result {
                if e.code == ErrorCode::Error && !handled {
                    let var = var_spec.trim_matches(|c| c == '{' || c == '}').to_string();
                    interp.set_var(&var, TclValue::Str(e.message.clone()))?;
                    final_result = interp.eval(&handler);
                    handled = true;
                }
            }
        } else if keyword == "finally" && i < args.len() {
            finally_body = Some(args[i].as_str().to_string());
            i += 1;
        }
    }

    if let Some(fb) = finally_body {
        interp.eval(&fb)?;
    }

    final_result
}

/// `catch script ?resultVar?`
fn cmd_catch(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"catch script ?resultVar?\""));
    }
    let script = args[0].as_str().to_string();
    let result = interp.eval(&script);
    let (code, value) = match result {
        Ok(v) => (0, v),
        Err(e) => (1, TclValue::Str(e.message)),
    };
    if args.len() > 1 {
        let var = args[1].as_str().to_string();
        interp.set_var(&var, value)?;
    }
    Ok(TclValue::Int(code))
}

/// `error message`
fn cmd_error(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"error message\""));
    }
    Err(TclError::new(args[0].as_str().to_string()))
}
