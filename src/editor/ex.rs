/// Ex-command parser for vim `:` commands.
///
/// Parses command strings like `:w`, `:q!`, `:e file.rs`, `:42`,
/// `:set tabstop=4`, etc. into [`Command`] values.
use super::command::Command;

/// Parse an ex-command string into a [`Command`].
///
/// The input should not include the leading `:`.
pub fn parse_ex(input: &str) -> Command {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Command::Noop;
    }
    // Line number: `:42` → GotoLine
    if let Ok(n) = trimmed.parse::<usize>() {
        return Command::GotoLine(n.saturating_sub(1));
    }
    let (cmd, arg) = split_cmd_arg(trimmed);
    match cmd {
        "w" if arg.is_empty() => Command::Save,
        "w" => Command::SaveAs(arg.to_string()),
        "wa" => Command::SaveAll,
        "q" => Command::CloseBuffer,
        "q!" => Command::ForceCloseBuffer,
        "wq" | "x" => Command::Save, // editor chains close after save
        "wq!" | "x!" => Command::Save,
        "qa" | "qa!" => Command::Quit,
        "e" if !arg.is_empty() => Command::OpenFile(arg.to_string()),
        "noh" | "nohlsearch" => Command::ClearSearchHighlight,
        _ => Command::Noop,
    }
}

/// Split "cmd arg" into ("cmd", "arg"). Handles `!` suffix.
fn split_cmd_arg(input: &str) -> (&str, &str) {
    // Find the boundary between command and argument
    let cmd_end = input
        .find(|c: char| c.is_whitespace())
        .unwrap_or(input.len());
    let cmd = &input[..cmd_end];
    let arg = input[cmd_end..].trim_start();
    (cmd, arg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_write() {
        assert_eq!(parse_ex("w"), Command::Save);
    }

    #[test]
    fn parse_write_as() {
        assert_eq!(
            parse_ex("w output.txt"),
            Command::SaveAs("output.txt".into())
        );
    }

    #[test]
    fn parse_quit() {
        assert_eq!(parse_ex("q"), Command::CloseBuffer);
    }

    #[test]
    fn parse_force_quit() {
        assert_eq!(parse_ex("q!"), Command::ForceCloseBuffer);
    }

    #[test]
    fn parse_wq() {
        assert_eq!(parse_ex("wq"), Command::Save);
    }

    #[test]
    fn parse_edit() {
        assert_eq!(
            parse_ex("e src/main.rs"),
            Command::OpenFile("src/main.rs".into())
        );
    }

    #[test]
    fn parse_goto_line() {
        assert_eq!(parse_ex("42"), Command::GotoLine(41));
    }

    #[test]
    fn parse_nohlsearch() {
        assert_eq!(parse_ex("noh"), Command::ClearSearchHighlight);
    }

    #[test]
    fn parse_empty() {
        assert_eq!(parse_ex(""), Command::Noop);
    }

    #[test]
    fn parse_unknown() {
        assert_eq!(parse_ex("foobar"), Command::Noop);
    }

    #[test]
    fn parse_write_all() {
        assert_eq!(parse_ex("wa"), Command::SaveAll);
    }
}
