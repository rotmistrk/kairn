//! Help topic: M-x commands reference (generated from dispatch table).

use crate::handler_exec::dispatch_table;

pub(crate) fn help_commands() -> String {
    let mut s = String::from("─── M-x Commands ──────────────────\n\n");
    let mut entries: Vec<_> = dispatch_table().collect();
    entries.sort_by_key(|e| e.names[0]);
    for entry in entries {
        let name = entry.names[0];
        let aliases = if entry.names.len() > 1 {
            format!(" ({})", entry.names[1..].join(", "))
        } else {
            String::new()
        };
        s.push_str(&format!("  :{name}{aliases}\n"));
        if !entry.description.is_empty() {
            s.push_str(&format!("      {}\n", entry.description));
        }
    }
    s.push_str("\n  Anything not recognized is evaluated as Tcl.\n");
    s.push_str("\nSee also:\n");
    s.push_str("  → :help tcl         Tcl scripting reference\n");
    s.push_str("  → :help             Back to overview\n");
    s
}
