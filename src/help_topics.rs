//! Help topic registry and content generation.

use txv_core::key_help::KeyHelpEntry;

use crate::help_topic_commands::help_commands;
use crate::help_topic_hooks::help_hooks;
use crate::help_topic_keys::help_keys_from_bindings;
use crate::help_topic_mcp::help_mcp;
use crate::help_topic_tcl::help_tcl;
use crate::help_topic_views::{help_csv, help_editor, help_struct, help_todo, help_tree};

/// All available help topic names.
pub fn topic_names() -> &'static [&'static str] {
    &[
        "commands", "csv", "editor", "hooks", "keys", "mcp", "struct", "tcl", "todo", "tree",
    ]
}

/// Generate content for a help topic. Empty string means overview.
pub fn generate_topic(topic: &str, bindings: &[KeyHelpEntry]) -> String {
    match topic {
        "" => overview(),
        "keys" => help_keys_from_bindings(bindings),
        "tcl" => help_tcl(),
        "mcp" => help_mcp(),
        "hooks" => help_hooks(),
        "commands" => help_commands(),
        "editor" => help_editor(),
        "tree" => help_tree(),
        "csv" => help_csv(),
        "struct" => help_struct(),
        "todo" => help_todo(),
        _ => unknown_topic(topic),
    }
}

fn overview() -> String {
    "\
╦╔═╔═╗╦╦═╗╔╗╔  Help
╠╩╗╠═╣║╠╦╝║║║
╩ ╩╩ ╩╩╩╚═╝╚╝

Topics:
  → :help keys        Key bindings reference
  → :help commands    M-x commands
  → :help tcl         Tcl scripting
  → :help mcp         MCP tool permissions
  → :help hooks       Event hooks
  → :help editor      Editor (vim) keys
  → :help tree        File tree keys
  → :help csv         CSV view keys
  → :help struct      Structured view keys
  → :help todo        Todo tree keys

Quick start:
  M-x (or Alt-x)  Command palette
  F2/F3/F4         Focus tree/editor/tools
  F1               This help
  Ctrl-Q           Quit

Navigation:
  j/k              Scroll down/up
  g/G              Jump to top/bottom
  Enter            Follow → cross-reference
  /                Search
"
    .to_string()
}

fn unknown_topic(topic: &str) -> String {
    let mut s = format!("Unknown help topic: {topic}\n\nAvailable topics:\n");
    for name in topic_names() {
        s.push_str(&format!("  → :help {name}\n"));
    }
    s
}
