//! Help topic: M-x commands reference (generated from dispatch table).

use crate::handler_exec::dispatch_table;

pub(crate) fn help_commands() -> String {
    let mut s = String::from("─── M-x Commands ──────────────────\n\n");
    let mut names: Vec<&str> = dispatch_table().map(|e| e.names[0]).collect();
    names.sort_unstable();
    for name in names {
        s.push_str(&format!("  {name}\n"));
    }
    s.push_str("\n  Anything not recognized is evaluated as Tcl.\n");
    s.push_str("\nSee also:\n");
    s.push_str("  → :help tcl         Tcl scripting reference\n");
    s
}
