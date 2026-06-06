//! Vim-style ex command table with unambiguous prefix matching.
//!
//! Each command has a full name and a minimum abbreviation. A user input
//! matches if it is a prefix of the full name AND at least as long as the
//! minimum abbreviation.

/// Known ex command identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExCmdId {
    Delete,
    Diff,
    Edit,
    Format,
    NoBlame,
    NoDiff,
    NoHighlight,
    Only,
    Revert,
    Quit,
    QuitForce,
    Set,
    SetGlobal,
    Split,
    Substitute,
    Vsplit,
    Write,
    WriteQuit,
    Exit,
    Yank,
}

struct CmdEntry {
    full: &'static str,
    min_abbrev: usize, // minimum chars required
    id: ExCmdId,
}

/// The command table. Ordered by full name for clarity.
const CMD_TABLE: &[CmdEntry] = &[
    CmdEntry {
        full: "delete",
        min_abbrev: 1,
        id: ExCmdId::Delete,
    },
    CmdEntry {
        full: "diff",
        min_abbrev: 3,
        id: ExCmdId::Diff,
    },
    CmdEntry {
        full: "edit",
        min_abbrev: 1,
        id: ExCmdId::Edit,
    },
    CmdEntry {
        full: "fmt",
        min_abbrev: 3,
        id: ExCmdId::Format,
    },
    CmdEntry {
        full: "noblame",
        min_abbrev: 3,
        id: ExCmdId::NoBlame,
    },
    CmdEntry {
        full: "nodiff",
        min_abbrev: 3,
        id: ExCmdId::NoDiff,
    },
    CmdEntry {
        full: "nohlsearch",
        min_abbrev: 3,
        id: ExCmdId::NoHighlight,
    },
    CmdEntry {
        full: "only",
        min_abbrev: 2,
        id: ExCmdId::Only,
    },
    CmdEntry {
        full: "quit",
        min_abbrev: 1,
        id: ExCmdId::Quit,
    },
    CmdEntry {
        full: "revert",
        min_abbrev: 3,
        id: ExCmdId::Revert,
    },
    CmdEntry {
        full: "set",
        min_abbrev: 2,
        id: ExCmdId::Set,
    },
    CmdEntry {
        full: "setglobal",
        min_abbrev: 4,
        id: ExCmdId::SetGlobal,
    },
    CmdEntry {
        full: "split",
        min_abbrev: 2,
        id: ExCmdId::Split,
    },
    CmdEntry {
        full: "substitute",
        min_abbrev: 1,
        id: ExCmdId::Substitute,
    },
    CmdEntry {
        full: "vsplit",
        min_abbrev: 2,
        id: ExCmdId::Vsplit,
    },
    CmdEntry {
        full: "write",
        min_abbrev: 1,
        id: ExCmdId::Write,
    },
    CmdEntry {
        full: "wq",
        min_abbrev: 2,
        id: ExCmdId::WriteQuit,
    },
    CmdEntry {
        full: "x",
        min_abbrev: 1,
        id: ExCmdId::Exit,
    },
    CmdEntry {
        full: "yank",
        min_abbrev: 1,
        id: ExCmdId::Yank,
    },
];

/// Full command names for Tab completion in ex mode.
pub const CMD_TABLE_NAMES: &[&str] = &[
    "delete",
    "diff",
    "edit",
    "fmt",
    "nodiff",
    "noblame",
    "nohlsearch",
    "only",
    "quit",
    "set",
    "setglobal",
    "split",
    "substitute",
    "vsplit",
    "write",
    "wq",
    "x",
    "yank",
];

/// Look up a command name by unambiguous prefix.
/// Returns None if no match or ambiguous.
pub fn lookup_command(input: &str) -> Option<ExCmdId> {
    if input.is_empty() {
        return None;
    }
    // Special cases that aren't alpha-only
    if input == "q!" {
        return Some(ExCmdId::QuitForce);
    }
    let mut found: Option<ExCmdId> = None;
    for entry in CMD_TABLE {
        if input.len() >= entry.min_abbrev && input.len() <= entry.full.len() && entry.full.starts_with(input) {
            if found.is_some() {
                return None; // ambiguous
            }
            found = Some(entry.id);
        }
    }
    found
}

/// Extract the command word (leading alphabetic chars) from an ex command string.
/// Returns (command_word, rest) where rest is everything after the command word.
pub fn split_cmd_word(input: &str) -> (&str, &str) {
    let end = input.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(input.len());
    (&input[..end], &input[end..])
}

#[cfg(test)]
#[path = "ex_commands_tests.rs"]
mod tests;
