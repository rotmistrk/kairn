//! :help keys — generated from live status bar bindings and dispatch table.

use txv_core::key_help::KeyHelpEntry;

use crate::handler_exec::dispatch_table;

/// Generate help text from live binding entries + dispatch table.
pub fn help_keys_from_bindings(bindings: &[KeyHelpEntry]) -> String {
    let mut s = String::new();
    let mut last_group = String::new();
    for entry in bindings {
        if entry.group() != last_group {
            if !last_group.is_empty() {
                s.push('\n');
            }
            let g = entry.group();
            s.push_str(&format!("─── {g} ────────────────────\n"));
            last_group = entry.group().to_string();
        }
        let k = entry.key();
        let a = entry.action();
        s.push_str(&format!("  {k:<16}{a}\n"));
    }
    s.push_str("\n─── M-x Commands ──────────────────────\n");
    for entry in dispatch_table() {
        let name = entry.names[0];
        let aliases = if entry.names.len() > 1 {
            format!(" ({})", entry.names[1..].join(", "))
        } else {
            String::new()
        };
        s.push_str(&format!("  :{name}{aliases}\n"));
    }
    s.push_str("\n─── Per-View Keys ─────────────────────\n");
    s.push_str("  → :help editor    Editor (vim) keys\n");
    s.push_str("  → :help tree      File tree keys\n");
    s.push_str("  → :help csv       CSV view keys\n");
    s.push_str("  → :help struct    Structured view keys\n");
    s.push_str("  → :help todo      Todo tree keys\n");
    s
}
