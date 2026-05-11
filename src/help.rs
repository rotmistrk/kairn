//! Help text generator for kairn.

use crate::help_editor::help_editor;
use crate::help_global::help_global;

/// Generate the full help text listing all key bindings.
pub fn help_text() -> String {
    format!(
        "\
╦╔═╔═╗╦╦═╗╔╗╔  Help
╠╩╗╠═╣║╠╦╝║║║
╩ ╩╩ ╩╩╩╚═╝╚╝

{}\
{}",
        help_global(),
        help_editor()
    )
}
