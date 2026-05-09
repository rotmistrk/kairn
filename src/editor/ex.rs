//! Ex command parser — handles :w, :q, :wq, :N, :s/pat/rep/, :d, :y, :!cmd.

use super::command::Command;

/// Parsed ex-command with resolved line range.
#[derive(Debug, PartialEq, Eq)]
pub enum ExCommand {
    Save,
    Quit,
    SaveQuit,
    GotoLine(usize),
    Delete { start: usize, end: usize },
    Yank { start: usize, end: usize },
    Substitute { start: usize, end: usize, pattern: String, replacement: String, global: bool },
    Shell { start: usize, end: usize, command: String },
    Set(String),
}

/// Parse an ex command string into a Command (for simple commands) or ExCommand.
pub fn parse_ex(input: &str) -> Command {
    let trimmed = input.trim();
    match trimmed {
        "w" => Command::Save,
        "q" | "q!" => Command::CloseBuffer,
        "wq" | "x" => Command::Save,
        _ => {
            if let Ok(n) = trimmed.parse::<usize>() {
                Command::GotoLine(n)
            } else {
                Command::Noop
            }
        }
    }
}

/// Parse a full ex command with range support. Returns ExCommand for complex ops.
pub fn parse_ex_full(cmd: &str, cursor_row: usize, total_lines: usize) -> Option<ExCommand> {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return None;
    }

    // Simple commands first
    match cmd {
        "w" => return Some(ExCommand::Save),
        "q" | "q!" => return Some(ExCommand::Quit),
        "wq" | "x" => return Some(ExCommand::SaveQuit),
        _ => {}
    }

    // Check for :set
    if let Some(rest) = cmd.strip_prefix("set ") {
        return Some(ExCommand::Set(rest.trim().to_string()));
    }

    // Goto line number
    if let Ok(n) = cmd.parse::<usize>() {
        return Some(ExCommand::GotoLine(n));
    }

    // Find where the command letter starts
    let cmd_start = cmd
        .find(|c: char| c.is_ascii_alphabetic() || c == '!' || c == 's')
        .unwrap_or(cmd.len());

    let range_str = cmd[..cmd_start].trim();
    let cmd_part = &cmd[cmd_start..];

    let (start, end) = parse_range(range_str, cursor_row, total_lines)?;

    if cmd_part.starts_with('y') {
        return Some(ExCommand::Yank { start, end });
    }
    if cmd_part.starts_with('d') {
        return Some(ExCommand::Delete { start, end });
    }
    if let Some(rest) = cmd_part.strip_prefix('s') {
        let (pattern, replacement, flags) = parse_substitute(rest)?;
        return Some(ExCommand::Substitute {
            start,
            end,
            pattern,
            replacement,
            global: flags.contains('g'),
        });
    }
    if let Some(rest) = cmd_part.strip_prefix('!') {
        return Some(ExCommand::Shell {
            start,
            end,
            command: rest.to_string(),
        });
    }
    None
}

fn parse_range(range: &str, cursor: usize, total: usize) -> Option<(usize, usize)> {
    if range.is_empty() {
        return Some((cursor, cursor));
    }
    if range == "%" {
        return Some((0, total.saturating_sub(1)));
    }
    let parts: Vec<&str> = range.splitn(2, ',').collect();
    match parts.len() {
        1 => {
            let addr = parse_address(parts[0].trim(), cursor, total)?;
            Some((addr, addr))
        }
        2 => {
            let s = parse_address(parts[0].trim(), cursor, total)?;
            let e = parse_address(parts[1].trim(), cursor, total)?;
            Some((s, e))
        }
        _ => None,
    }
}

fn parse_address(addr: &str, cursor: usize, total: usize) -> Option<usize> {
    match addr {
        "." => Some(cursor),
        "$" => Some(total.saturating_sub(1)),
        _ => {
            // Check relative offsets BEFORE plain number ("+2".parse::<usize>() succeeds in Rust!)
            if let Some(rest) = addr.strip_prefix(".+") {
                let offset: usize = rest.parse().ok()?;
                return Some((cursor + offset).min(total.saturating_sub(1)));
            }
            if let Some(rest) = addr.strip_prefix(".-") {
                let offset: usize = rest.parse().ok()?;
                return Some(cursor.saturating_sub(offset));
            }
            if let Some(rest) = addr.strip_prefix('+') {
                let offset: usize = rest.parse().ok()?;
                return Some((cursor + offset).min(total.saturating_sub(1)));
            }
            if let Some(rest) = addr.strip_prefix('-') {
                let offset: usize = rest.parse().ok()?;
                return Some(cursor.saturating_sub(offset));
            }
            // Plain line number
            if let Ok(n) = addr.parse::<usize>() {
                return Some(n.saturating_sub(1));
            }
            None
        }
    }
}

fn parse_substitute(s: &str) -> Option<(String, String, String)> {
    if s.is_empty() {
        return None;
    }
    let delim = s.chars().next()?;
    let rest = &s[delim.len_utf8()..];
    let parts: Vec<&str> = rest.splitn(3, delim).collect();
    if parts.len() < 2 {
        return None;
    }
    let pattern = parts[0].to_string();
    let replacement = parts[1].to_string();
    let flags = parts.get(2).unwrap_or(&"").to_string();
    Some((pattern, replacement, flags))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        assert_eq!(parse_ex("w"), Command::Save);
        assert_eq!(parse_ex("q"), Command::CloseBuffer);
        assert_eq!(parse_ex("5"), Command::GotoLine(5));
    }

    #[test]
    fn test_parse_substitute() {
        let result = parse_ex_full("%s/foo/bar/g", 0, 10);
        assert_eq!(
            result,
            Some(ExCommand::Substitute {
                start: 0,
                end: 9,
                pattern: "foo".to_string(),
                replacement: "bar".to_string(),
                global: true,
            })
        );
    }

    #[test]
    fn test_parse_delete_range() {
        let result = parse_ex_full("1,3d", 0, 10);
        assert_eq!(result, Some(ExCommand::Delete { start: 0, end: 2 }));
    }

    #[test]
    fn test_parse_yank() {
        let result = parse_ex_full("%y", 0, 5);
        assert_eq!(result, Some(ExCommand::Yank { start: 0, end: 4 }));
    }
}

    #[test]
    fn test_parse_relative_range() {
        let result = parse_ex_full(".,+2d", 1, 5);
        assert_eq!(result, Some(ExCommand::Delete { start: 1, end: 3 }));
        let result = parse_ex_full(".,+2y", 1, 5);
        assert_eq!(result, Some(ExCommand::Yank { start: 1, end: 3 }));
    }
