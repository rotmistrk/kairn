//! Ex command parser — handles :w, :q, :wq, :N, :s/pat/rep/, :d, :y, :!cmd.

/// Parsed ex-command with resolved line range.
#[derive(Debug, PartialEq, Eq)]
pub enum ExCommand {
    Save,
    Quit,
    QuitForce,
    SaveQuit,
    GotoLine(usize),
    Delete {
        start: usize,
        end: usize,
    },
    Yank {
        start: usize,
        end: usize,
    },
    Substitute {
        start: usize,
        end: usize,
        pattern: String,
        replacement: String,
        global: bool,
    },
    Shell {
        start: usize,
        end: usize,
        command: String,
    },
    Set(String),
    SetGlobal(String),
    Edit(String),
    Diff(String),
    NoDiff,
    NoHighlight,
    Split(String),
    Vsplit(String),
    Only,
    Revert,
}

/// Parse a full ex command with range support. Returns ExCommand for complex ops.
pub fn parse_ex_full(cmd: &str, cursor_row: usize, total_lines: usize) -> Option<ExCommand> {
    use super::ex_commands::{lookup_command, split_cmd_word, ExCmdId};

    let cmd = cmd.trim();
    if cmd.is_empty() {
        return None;
    }

    // Goto line number (pure digits)
    if let Ok(n) = cmd.parse::<usize>() {
        return Some(ExCommand::GotoLine(n));
    }

    // Find where the command letter starts (skip range prefix)
    let cmd_start = cmd
        .find(|c: char| c.is_ascii_alphabetic() || c == '!' || c == 's')
        .unwrap_or(cmd.len());

    let range_str = cmd[..cmd_start].trim();
    let cmd_part = &cmd[cmd_start..];

    // Shell filter: range!command
    if let Some(rest) = cmd_part.strip_prefix('!') {
        let (start, end) = parse_range(range_str, cursor_row, total_lines)?;
        return Some(ExCommand::Shell {
            start,
            end,
            command: rest.to_string(),
        });
    }

    // Extract command word and look it up
    let (cmd_word, rest) = split_cmd_word(cmd_part);

    // Handle q! as a special case (! modifies the command)
    let (cmd_id, rest) = if cmd_word == "q" && rest.starts_with('!') {
        (lookup_command("q!")?, &rest[1..])
    } else {
        (lookup_command(cmd_word)?, rest)
    };

    match cmd_id {
        ExCmdId::Write => Some(ExCommand::Save),
        ExCmdId::Quit => Some(ExCommand::Quit),
        ExCmdId::QuitForce => Some(ExCommand::QuitForce),
        ExCmdId::WriteQuit | ExCmdId::Exit => Some(ExCommand::SaveQuit),
        ExCmdId::Set => Some(ExCommand::Set(rest.trim().to_string())),
        ExCmdId::SetGlobal => Some(ExCommand::SetGlobal(rest.trim().to_string())),
        ExCmdId::Edit => Some(ExCommand::Edit(rest.trim().to_string())),
        ExCmdId::Diff => Some(ExCommand::Diff(rest.trim().to_string())),
        ExCmdId::NoDiff => Some(ExCommand::NoDiff),
        ExCmdId::NoHighlight => Some(ExCommand::NoHighlight),
        ExCmdId::Split => Some(ExCommand::Split(rest.trim().to_string())),
        ExCmdId::Vsplit => Some(ExCommand::Vsplit(rest.trim().to_string())),
        ExCmdId::Only => Some(ExCommand::Only),
        ExCmdId::Revert => Some(ExCommand::Revert),
        ExCmdId::Delete => {
            let (start, end) = parse_range(range_str, cursor_row, total_lines)?;
            Some(ExCommand::Delete { start, end })
        }
        ExCmdId::Yank => {
            let (start, end) = parse_range(range_str, cursor_row, total_lines)?;
            Some(ExCommand::Yank { start, end })
        }
        ExCmdId::Substitute => {
            let (start, end) = parse_range(range_str, cursor_row, total_lines)?;
            let (pattern, replacement, flags) = parse_substitute(rest)?;
            Some(ExCommand::Substitute {
                start,
                end,
                pattern,
                replacement,
                global: flags.contains('g'),
            })
        }
    }
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
#[path = "ex_tests.rs"]
mod tests;
