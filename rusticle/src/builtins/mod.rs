//! Built-in commands for the rusticle interpreter.

mod control;
mod dict_cmds;
mod expr_cmd;
mod expr_eval;
mod info_cmd;
mod io_cmd;
mod list_cmds;
mod procs;
mod string_cmds;
mod var_cmds;

use crate::interpreter::Interpreter;

/// Register all built-in commands with the interpreter.
pub fn register_all(interp: &mut Interpreter) {
    var_cmds::register(interp);
    control::register(interp);
    procs::register(interp);
    expr_cmd::register(interp);
    string_cmds::register(interp);
    list_cmds::register(interp);
    dict_cmds::register(interp);
    io_cmd::register(interp);
    info_cmd::register(interp);
}
