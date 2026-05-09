//! CommandCompleter — provides tab-completion for kairn command mode.

use txv_core::complete::{Completer, Completion};

/// Known commands for the M-x prompt.
const COMMANDS: &[&str] = &[
    "help", "quit", "open", "save", "close", "shell",
];

/// Completer for kairn application commands.
pub struct CommandCompleter;

impl Completer for CommandCompleter {
    fn complete(&self, input: &str, _cursor: usize) -> Vec<Completion> {
        let trimmed = input.trim();
        COMMANDS
            .iter()
            .filter(|cmd| cmd.starts_with(trimmed))
            .map(|cmd| Completion {
                text: cmd.to_string(),
                display: cmd.to_string(),
                kind: "command",
            })
            .collect()
    }
}
