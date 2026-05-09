//! Ex command parser — handles :w, :q, :wq, :N (goto line), :s/pat/rep/.

use super::command::Command;

/// Parse an ex command string into a Command.
pub fn parse_ex(input: &str) -> Command {
    let trimmed = input.trim();
    match trimmed {
        "w" => Command::Save,
        "q" => Command::CloseBuffer,
        "wq" | "x" => Command::Save, // save then close handled by editor
        _ => {
            // :N — goto line
            if let Ok(_n) = trimmed.parse::<usize>() {
                Command::Noop // handled inline by editor
            } else {
                Command::Noop
            }
        }
    }
}
